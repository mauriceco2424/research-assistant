use std::collections::{HashMap, HashSet};

use chrono::Utc;
use uuid::Uuid;

use crate::bases::{CategoryAssignment, CategoryRecord, CategoryStatusMetric, LibraryEntry};

pub struct BacklogSegment {
    pub label: String,
    pub count: usize,
    pub representative_entry_ids: Vec<Uuid>,
}

pub struct CategoryStatusSummary {
    pub metrics: Vec<CategoryStatusMetric>,
    pub backlog_segments: Vec<BacklogSegment>,
}

pub struct CategoryMetricsCollector;

impl CategoryMetricsCollector {
    pub fn collect(
        entries: &[LibraryEntry],
        categories: &[CategoryRecord],
        assignments: &[CategoryAssignment],
        include_backlog: bool,
    ) -> CategoryStatusSummary {
        let total_entries = entries.len().max(1) as f32;
        let now = Utc::now();
        let mut assignment_map: HashMap<Uuid, Vec<&CategoryAssignment>> = HashMap::new();
        for assignment in assignments {
            assignment_map
                .entry(assignment.category_id)
                .or_default()
                .push(assignment);
        }
        let mut metrics = Vec::new();
        for record in categories {
            let assigned = assignment_map
                .get(&record.definition.category_id)
                .cloned()
                .unwrap_or_default();
            let paper_count = assigned.len() as u32;
            let staleness_days = assigned
                .iter()
                .filter_map(|assignment| assignment.last_reviewed_at)
                .map(|ts| (now - ts).num_days() as u32)
                .min()
                .unwrap_or_else(|| (now - record.definition.updated_at).num_days().max(0) as u32);
            let overload_ratio = paper_count as f32 / total_entries;
            metrics.push(CategoryStatusMetric {
                metric_id: Uuid::new_v4(),
                category_id: Some(record.definition.category_id),
                paper_count,
                uncategorized_estimate: 0,
                staleness_days,
                overload_ratio,
                generated_at: now,
            });
        }

        let mut backlog_segments = Vec::new();
        if include_backlog {
            let assigned_ids: HashSet<Uuid> = assignments
                .iter()
                .map(|assignment| assignment.paper_id)
                .collect();
            let mut backlog_map: HashMap<String, Vec<&LibraryEntry>> = HashMap::new();
            for entry in entries {
                if assigned_ids.contains(&entry.entry_id) {
                    continue;
                }
                let label = backlog_label(entry);
                backlog_map.entry(label).or_default().push(entry);
            }
            for (label, group) in backlog_map {
                let count = group.len();
                let representative = group
                    .iter()
                    .take(3)
                    .map(|entry| entry.entry_id)
                    .collect::<Vec<_>>();
                backlog_segments.push(BacklogSegment {
                    label: label.clone(),
                    count,
                    representative_entry_ids: representative.clone(),
                });
                metrics.push(CategoryStatusMetric {
                    metric_id: Uuid::new_v4(),
                    category_id: None,
                    paper_count: 0,
                    uncategorized_estimate: count as u32,
                    staleness_days: 0,
                    overload_ratio: count as f32 / total_entries,
                    generated_at: now,
                });
            }
        }

        CategoryStatusSummary {
            metrics,
            backlog_segments,
        }
    }
}

fn backlog_label(entry: &LibraryEntry) -> String {
    if let Some(venue) = &entry.venue {
        format!("Venue: {}", venue)
    } else if let Some(first_author) = entry.authors.first() {
        format!(
            "Author cluster: {}",
            first_author
                .chars()
                .take(1)
                .collect::<String>()
                .to_uppercase()
        )
    } else {
        entry
            .title
            .split_whitespace()
            .next()
            .map(|token| format!("Topic: {}", token))
            .unwrap_or_else(|| "Topic: Misc".into())
    }
}
