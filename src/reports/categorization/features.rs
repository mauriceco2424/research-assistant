use crate::bases::LibraryEntry;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

const DEFAULT_EMBEDDING_DIMS: usize = 32;

/// Dense representation of a paper for clustering.
#[derive(Debug, Clone)]
pub struct FeatureVector {
    pub entry_id: Uuid,
    pub terms: HashMap<String, f32>,
    pub embedding: Vec<f32>,
}

impl FeatureVector {
    pub fn new(entry_id: Uuid, terms: HashMap<String, f32>, embedding: Vec<f32>) -> Self {
        Self {
            entry_id,
            terms,
            embedding,
        }
    }
}

/// Builds TF-IDF + hashed embedding vectors for clustering.
#[derive(Debug, Clone)]
pub struct FeatureVectorBuilder {
    stop_words: HashSet<String>,
    embedding_dims: usize,
}

impl Default for FeatureVectorBuilder {
    fn default() -> Self {
        Self::new(DEFAULT_EMBEDDING_DIMS)
    }
}

impl FeatureVectorBuilder {
    pub fn new(embedding_dims: usize) -> Self {
        let stop_words = vec![
            "a", "an", "and", "or", "to", "of", "in", "on", "for", "with", "the", "this", "that",
            "by", "from", "via", "study", "analysis",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        Self {
            stop_words,
            embedding_dims: embedding_dims.max(8),
        }
    }

    /// Converts a library slice into feature vectors for clustering.
    pub fn build(&self, entries: &[LibraryEntry]) -> Vec<FeatureVector> {
        if entries.is_empty() {
            return Vec::new();
        }
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        let mut tokenized: Vec<Vec<String>> = Vec::new();
        for entry in entries {
            let tokens = self.tokenize(entry);
            let mut seen = HashSet::new();
            for token in &tokens {
                if seen.insert(token.clone()) {
                    *doc_freq.entry(token.clone()).or_insert(0) += 1;
                }
            }
            tokenized.push(tokens);
        }
        let doc_count = entries.len() as f32;
        entries
            .iter()
            .zip(tokenized.iter())
            .map(|(entry, tokens)| {
                let mut tf: HashMap<String, usize> = HashMap::new();
                for token in tokens {
                    *tf.entry(token.clone()).or_insert(0) += 1;
                }
                let mut terms = HashMap::new();
                let total = tokens.len().max(1) as f32;
                for (token, count) in tf {
                    let df = doc_freq.get(&token).copied().unwrap_or(1) as f32;
                    let tf_weight = count as f32 / total;
                    let idf = ((doc_count + 1.0) / (df + 1.0)).ln() + 1.0;
                    terms.insert(token.clone(), tf_weight * idf);
                }
                let embedding = self.embed(&terms);
                FeatureVector::new(entry.entry_id, terms, embedding)
            })
            .collect()
    }

    fn tokenize(&self, entry: &LibraryEntry) -> Vec<String> {
        let mut buf = Vec::new();
        self.push_text_tokens(&entry.title, &mut buf);
        if let Some(venue) = &entry.venue {
            self.push_text_tokens(venue, &mut buf);
        }
        for author in &entry.authors {
            self.push_text_tokens(author, &mut buf);
        }
        buf
    }

    fn push_text_tokens(&self, text: &str, buf: &mut Vec<String>) {
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            let token = word.to_ascii_lowercase();
            if token.len() > 2 && !self.stop_words.contains(&token) {
                buf.push(token);
            }
        }
    }

    fn embed(&self, terms: &HashMap<String, f32>) -> Vec<f32> {
        let mut embedding = vec![0.0f32; self.embedding_dims];
        for (token, weight) in terms {
            let idx = self.slot_for_token(token);
            embedding[idx] += *weight;
        }
        embedding
    }

    fn slot_for_token(&self, token: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        (hasher.finish() as usize) % self.embedding_dims
    }
}
