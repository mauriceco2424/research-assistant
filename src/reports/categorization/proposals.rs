use super::features::{FeatureVector, FeatureVectorBuilder};
use crate::bases::{
    category_slug, Base, CategoryDefinition, CategoryNarrative, CategoryOrigin,
    CategoryProposalPreview, LibraryEntry,
};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Local clustering worker that generates category proposals.
pub struct CategoryProposalWorker {
    pub max_clusters: usize,
    pub min_cluster_size: usize,
    pub max_iterations: usize,
    pub timeout: Duration,
}

impl CategoryProposalWorker {
    pub fn new(max_clusters: usize, timeout_ms: u64) -> Self {
        Self {
            max_clusters: max_clusters.max(1),
            min_cluster_size: 2,
            max_iterations: 25,
            timeout: Duration::from_millis(timeout_ms.max(1)),
        }
    }

    pub fn generate(
        &self,
        base: &Base,
        entries: &[LibraryEntry],
    ) -> Result<Vec<CategoryProposalPreview>> {
        if entries.len() < 2 {
            return Ok(Vec::new());
        }
        let builder = FeatureVectorBuilder::default();
        let vectors = builder.build(entries);
        if vectors.len() < 2 {
            return Ok(Vec::new());
        }
        let target_k = self.target_cluster_count(vectors.len());
        let dense: Vec<Vec<f32>> = vectors.iter().map(|v| v.embedding.clone()).collect();
        let started = Instant::now();
        let (assignments, centroids) = self.kmeans(&dense, target_k)?;
        if started.elapsed() > self.timeout {
            return Ok(Vec::new());
        }
        let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
        for (idx, cluster_id) in assignments.iter().enumerate() {
            clusters.entry(*cluster_id).or_default().push(idx);
        }
        let mut proposals = Vec::new();
        for (cluster_id, members) in clusters {
            if members.len() < self.min_cluster_size {
                continue;
            }
            let centroid = &centroids[cluster_id];
            let (keywords, description) = summarize_cluster(&vectors, &members);
            let member_entry_ids: Vec<Uuid> =
                members.iter().map(|idx| vectors[*idx].entry_id).collect();
            let representative = member_entry_ids.iter().take(5).cloned().collect();
            let name = if keywords.is_empty() {
                format!("Cluster {}", proposals.len() + 1)
            } else {
                keywords.join(" / ")
            };
            let confidence = score_cluster(&vectors, &members, centroid);
            let category_id = Uuid::new_v4();
            let definition = CategoryDefinition {
                category_id,
                base_id: base.id,
                name: name.clone(),
                slug: category_slug(&name),
                description: description.clone(),
                confidence: Some(confidence),
                representative_papers: representative,
                pinned_papers: Vec::new(),
                figure_gallery_enabled: false,
                origin: CategoryOrigin::Proposed,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let narrative = CategoryNarrative {
                narrative_id: Uuid::new_v4(),
                category_id,
                summary: format!(
                    "Auto-proposed grouping with {} papers driven by {:?}.",
                    member_entry_ids.len(),
                    keywords
                ),
                learning_prompts: Vec::new(),
                notes: Vec::new(),
                references: member_entry_ids.iter().take(3).cloned().collect(),
                ai_assisted: false,
                last_updated_at: Utc::now(),
            };
            proposals.push(CategoryProposalPreview {
                proposal_id: Uuid::new_v4(),
                definition,
                narrative,
                member_entry_ids,
                generated_at: Utc::now(),
            });
        }
        proposals.sort_by(|a, b| {
            b.definition
                .confidence
                .partial_cmp(&a.definition.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(proposals)
    }

    fn target_cluster_count(&self, vector_count: usize) -> usize {
        vector_count
            .min(self.max_clusters.max(1))
            .max(1)
            .min(vector_count)
    }

    fn kmeans(&self, vectors: &[Vec<f32>], k: usize) -> Result<(Vec<usize>, Vec<Vec<f32>>)> {
        let dims = vectors.first().map(|v| v.len()).unwrap_or(0);
        if dims == 0 || k == 0 {
            return Ok((Vec::new(), Vec::new()));
        }
        let mut centroids = vectors.iter().take(k).cloned().collect::<Vec<Vec<f32>>>();
        while centroids.len() < k {
            centroids.push(vec![0.0; dims]);
        }
        let mut assignments = vec![0usize; vectors.len()];
        for _ in 0..self.max_iterations {
            let mut changed = false;
            for (idx, vector) in vectors.iter().enumerate() {
                let mut best = 0;
                let mut best_dist = f32::MAX;
                for (centroid_idx, centroid) in centroids.iter().enumerate() {
                    let dist = squared_distance(vector, centroid);
                    if dist < best_dist {
                        best_dist = dist;
                        best = centroid_idx;
                    }
                }
                if assignments[idx] != best {
                    assignments[idx] = best;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
            let mut accum = vec![vec![0.0f32; dims]; k];
            let mut counts = vec![0usize; k];
            for (idx, vector) in vectors.iter().enumerate() {
                let cluster_id = assignments[idx];
                counts[cluster_id] += 1;
                for (dim, value) in vector.iter().enumerate() {
                    accum[cluster_id][dim] += value;
                }
            }
            for (i, centroid) in centroids.iter_mut().enumerate() {
                if counts[i] == 0 {
                    continue;
                }
                for dim in 0..dims {
                    centroid[dim] = accum[i][dim] / counts[i] as f32;
                }
            }
        }
        Ok((assignments, centroids))
    }
}

fn summarize_cluster(vectors: &[FeatureVector], members: &[usize]) -> (Vec<String>, String) {
    let mut weights: HashMap<String, f32> = HashMap::new();
    for idx in members {
        if let Some(vector) = vectors.get(*idx) {
            for (token, weight) in &vector.terms {
                *weights.entry(token.clone()).or_insert(0.0) += *weight;
            }
        }
    }
    let mut pairs: Vec<(String, f32)> = weights.into_iter().collect();
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let keywords: Vec<String> = pairs
        .iter()
        .take(3)
        .map(|(token, _)| token.clone())
        .collect();
    let description = if keywords.is_empty() {
        format!("Contains {} papers.", members.len())
    } else {
        format!(
            "Contains {} papers emphasizing {}.",
            members.len(),
            keywords.join(", ")
        )
    };
    (keywords, description)
}

fn score_cluster(vectors: &[FeatureVector], members: &[usize], centroid: &[f32]) -> f32 {
    if members.is_empty() {
        return 0.0;
    }
    let mut total = 0.0f32;
    for idx in members {
        if let Some(vector) = vectors.get(*idx) {
            total += squared_distance(&vector.embedding, centroid).sqrt();
        }
    }
    let avg = total / members.len() as f32;
    1.0 / (1.0 + avg)
}

fn squared_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum()
}
