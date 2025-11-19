use crate::bases::Base;
use crate::reports::consent_registry::ConsentRegistry;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const VISUALIZATION_DIR: &str = "reports/visualizations";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationKind {
    ConceptMap,
    Timeline,
    CitationGraph,
    BacklogChart,
}

impl VisualizationKind {
    pub fn from_label(label: &str) -> Result<Self> {
        match label.to_ascii_lowercase().as_str() {
            "concept_map" | "concept-map" => Ok(Self::ConceptMap),
            "timeline" => Ok(Self::Timeline),
            "citation_graph" | "citation-graph" => Ok(Self::CitationGraph),
            "backlog_chart" | "backlog-chart" => Ok(Self::BacklogChart),
            other => anyhow::bail!(
                "Unknown visualization type '{other}'. Supported: concept_map, timeline, citation_graph, backlog_chart."
            ),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::ConceptMap => "concept_map",
            Self::Timeline => "timeline",
            Self::CitationGraph => "citation_graph",
            Self::BacklogChart => "backlog_chart",
        }
    }

    pub fn requires_remote_layout(&self) -> bool {
        matches!(self, Self::ConceptMap)
    }
}

#[derive(Debug, Clone)]
pub struct VisualizationRenderEntry {
    pub kind: VisualizationKind,
    pub data_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct VisualizationDataset {
    pub dataset_id: Uuid,
    pub kind: VisualizationKind,
    pub data_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct VisualizationSelection {
    pub datasets: Vec<VisualizationDataset>,
    pub consent_tokens: Vec<Uuid>,
    pub render_entries: Vec<VisualizationRenderEntry>,
    pub dataset_ids: Vec<Uuid>,
}

pub struct VisualizationSelector<'a> {
    base: &'a Base,
}

impl<'a> VisualizationSelector<'a> {
    pub fn new(base: &'a Base) -> Self {
        Self { base }
    }

    pub fn select(
        &self,
        labels: &[String],
        registry: &ConsentRegistry<'a>,
    ) -> Result<VisualizationSelection> {
        if labels.is_empty() {
            return Ok(VisualizationSelection::default());
        }
        let mut selection = VisualizationSelection::default();
        for label in labels {
            let kind = VisualizationKind::from_label(label)?;
            if kind.requires_remote_layout() {
                let manifest =
                    registry.ensure_active(&format!("visualization:{}", kind.label()))?;
                selection.consent_tokens.push(manifest.consent_id);
            }
            let dataset = self.ensure_dataset(&kind)?;
            selection.dataset_ids.push(dataset.dataset_id);
            selection.render_entries.push(VisualizationRenderEntry {
                kind: kind.clone(),
                data_path: Some(dataset.data_path.clone()),
            });
            selection.datasets.push(dataset);
        }
        Ok(selection)
    }

    fn ensure_dataset(&self, kind: &VisualizationKind) -> Result<VisualizationDataset> {
        let data_dir = self.base.user_layer_path.join(VISUALIZATION_DIR);
        fs::create_dir_all(&data_dir)?;
        let file_path = data_dir.join(format!("{}.json", kind.label()));
        if !file_path.exists() {
            fs::write(&file_path, "[]")
                .with_context(|| format!("Failed to create {}", file_path.display()))?;
        }
        let dataset_id = deterministic_dataset_id(&file_path);
        Ok(VisualizationDataset {
            dataset_id,
            kind: kind.clone(),
            data_path: file_path,
        })
    }
}

fn deterministic_dataset_id(path: &PathBuf) -> Uuid {
    let digest = Sha256::digest(path.to_string_lossy().as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}
