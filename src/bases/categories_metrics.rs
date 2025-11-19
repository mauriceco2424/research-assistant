use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use super::Base;

/// Metric record describing category/backlog health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStatusMetric {
    pub metric_id: Uuid,
    pub category_id: Option<Uuid>,
    pub paper_count: u32,
    pub uncategorized_estimate: u32,
    pub staleness_days: u32,
    pub overload_ratio: f32,
    pub generated_at: DateTime<Utc>,
}

/// Persists category status metrics per Base for historical comparisons.
pub struct CategoryMetricsStore {
    path: PathBuf,
}

impl CategoryMetricsStore {
    pub fn new(base: &Base) -> Self {
        let path = base.ai_layer_path.join("categories_metrics.json");
        Self { path }
    }

    pub fn save(&self, metrics: &[CategoryStatusMetric]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_vec_pretty(metrics)?;
        fs::write(&self.path, data)?;
        Ok(())
    }

    pub fn load(&self) -> Result<Vec<CategoryStatusMetric>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read(&self.path)?;
        let metrics = serde_json::from_slice(&data)?;
        Ok(metrics)
    }
}
