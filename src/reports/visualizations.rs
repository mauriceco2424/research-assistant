use crate::bases::Base;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationKind {
    ConceptMap,
    Timeline,
    CitationGraph,
    BacklogChart,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationDataset {
    pub dataset_id: Uuid,
    pub kind: VisualizationKind,
    pub source_scope: String,
    pub data_path: Option<String>,
    pub status: VisualizationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VisualizationStatus {
    Current,
    Stale,
    Pending,
}

impl Default for VisualizationStatus {
    fn default() -> Self {
        VisualizationStatus::Pending
    }
}

pub struct VisualizationRegistry {
    base: Base,
    datasets: HashMap<Uuid, VisualizationDataset>,
}

impl VisualizationRegistry {
    pub fn new(base: &Base) -> Self {
        Self {
            base: base.clone(),
            datasets: HashMap::new(),
        }
    }

    pub fn list(&self) -> Vec<VisualizationDataset> {
        self.datasets.values().cloned().collect()
    }

    pub fn register(&mut self, dataset: VisualizationDataset) {
        self.datasets.insert(dataset.dataset_id, dataset);
    }

    pub fn mark_stale(&mut self, dataset_id: &Uuid) {
        if let Some(entry) = self.datasets.get_mut(dataset_id) {
            entry.status = VisualizationStatus::Stale;
        }
    }

    pub fn base(&self) -> &Base {
        &self.base
    }
}
