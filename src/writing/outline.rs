use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

use crate::bases::{Base, BaseManager, LibraryEntry};
use crate::orchestration::{
    MetricRecord, OrchestrationLog, WritingMetricKind, WritingMetricRecord,
};
use crate::storage::ai_layer::WritingAiStore;
use crate::writing::citations;

use super::WritingResult;

const OUTLINE_VERSION: &str = "1.0.0";
const DEFAULT_SECTIONS: [&str; 5] = [
    "Introduction",
    "Background and Related Work",
    "Methodology",
    "Experiments and Results",
    "Discussion and Future Work",
];

/// Status values for outline nodes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutlineStatus {
    Proposed,
    Accepted,
    Rejected,
    Drafted,
}

impl Default for OutlineStatus {
    fn default() -> Self {
        OutlineStatus::Proposed
    }
}

/// Node representation stored in the AI layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineNode {
    pub id: Uuid,
    pub project_slug: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub status: OutlineStatus,
    #[serde(default)]
    pub order: usize,
    #[serde(default)]
    pub last_edited_event_id: Option<Uuid>,
    #[serde(default)]
    pub revision_history: Vec<String>,
}

/// Persisted outline document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineDocument {
    #[serde(default = "outline_version")]
    pub version: String,
    pub project_slug: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub nodes: Vec<OutlineNode>,
}

/// Request payload for generating outline proposals.
#[derive(Debug, Clone, Default)]
pub struct OutlineProposalRequest {
    pub goal: Option<String>,
    pub references: Vec<String>,
    pub max_nodes: Option<usize>,
}

/// Returned after proposals are generated and persisted.
#[derive(Debug, Clone)]
pub struct OutlineProposal {
    pub outline: OutlineDocument,
    pub added_nodes: Vec<OutlineNode>,
    pub warnings: Vec<String>,
    pub outline_path: PathBuf,
}

/// Stored when outline updates should be undoable.
#[derive(Debug, Clone)]
pub struct OutlineCheckpoint {
    pub checkpoint_path: PathBuf,
    pub outline: OutlineDocument,
}

pub struct OutlineStore<'a> {
    ai: WritingAiStore<'a>,
}

impl<'a> OutlineStore<'a> {
    pub fn new(base: &'a Base) -> Self {
        Self {
            ai: WritingAiStore::new(base),
        }
    }

    /// Loads the persisted outline or returns an empty document.
    pub fn load(&self, slug: &str) -> WritingResult<OutlineDocument> {
        if let Some(existing) = self.ai.load_outline::<OutlineDocument>(slug)? {
            return Ok(existing);
        }
        Ok(OutlineDocument {
            version: outline_version(),
            project_slug: slug.to_string(),
            summary: None,
            updated_at: Utc::now(),
            nodes: Vec::new(),
        })
    }

    /// Saves the outline into the AI layer.
    pub fn save(&self, slug: &str, outline: &OutlineDocument) -> WritingResult<PathBuf> {
        self.ai.save_outline(slug, outline)?;
        Ok(self.ai.project_root(slug).join("outline.json"))
    }

    /// Persists an undo payload for the current outline.
    pub fn checkpoint(
        &self,
        slug: &str,
        event_id: Uuid,
        outline: &OutlineDocument,
    ) -> WritingResult<OutlineCheckpoint> {
        let path = self
            .ai
            .save_undo_payload(slug, &event_id.to_string(), outline)?;
        Ok(OutlineCheckpoint {
            checkpoint_path: path,
            outline: outline.clone(),
        })
    }
}

/// Generates outline proposals and persists them to the AI layer.
pub fn generate_outline_proposals(
    manager: &BaseManager,
    base: &Base,
    slug: &str,
    request: OutlineProposalRequest,
) -> WritingResult<OutlineProposal> {
    let started = std::time::Instant::now();
    let store = OutlineStore::new(base);
    let mut outline = store.load(slug)?;
    let mut warnings = Vec::new();

    let mut existing_titles: HashSet<String> = outline
        .nodes
        .iter()
        .map(|n| n.title.to_ascii_lowercase())
        .collect();
    let mut nodes = Vec::new();

    // Always seed with canonical writing sections to keep structure predictable.
    let offset = outline.nodes.len();
    for (idx, section) in DEFAULT_SECTIONS.iter().enumerate() {
        if existing_titles.contains(&section.to_ascii_lowercase()) {
            continue;
        }
        let node = OutlineNode {
            id: Uuid::new_v4(),
            project_slug: slug.to_string(),
            parent_id: None,
            title: section.to_string(),
            summary: Some(format!("Placeholder section for {section}")),
            references: Vec::new(),
            status: OutlineStatus::Proposed,
            order: offset + idx,
            last_edited_event_id: None,
            revision_history: Vec::new(),
        };
        existing_titles.insert(section.to_ascii_lowercase());
        nodes.push(node);
    }

    let library_index = index_library_entries(manager, base)?;
    let max_nodes = request.max_nodes.unwrap_or(150);
    for (idx, reference) in request.references.iter().take(max_nodes).enumerate() {
        let key = reference.trim();
        if key.is_empty() {
            continue;
        }
        if existing_titles.contains(&key.to_ascii_lowercase()) {
            continue;
        }
        let (title, summary) = library_index
            .get(key)
            .cloned()
            .unwrap_or_else(|| (format!("Reference {key}"), None));
        let node = OutlineNode {
            id: Uuid::new_v4(),
            project_slug: slug.to_string(),
            parent_id: None,
            title: title.clone(),
            summary: summary.clone(),
            references: vec![key.to_string()],
            status: OutlineStatus::Proposed,
            order: offset + DEFAULT_SECTIONS.len() + idx,
            last_edited_event_id: None,
            revision_history: Vec::new(),
        };
        existing_titles.insert(title.to_ascii_lowercase());
        nodes.push(node);
        if summary.is_none() {
            warnings.push(format!(
                "Reference '{}' not found in the Base library; added as placeholder.",
                key
            ));
        }
    }

    if nodes.is_empty() {
        // No new nodes to add; return current outline.
        let outline_path = store.save(slug, &outline)?;
        return Ok(OutlineProposal {
            outline,
            added_nodes: Vec::new(),
            warnings,
            outline_path,
        });
    }

    outline.nodes.extend(nodes.clone());
    outline.updated_at = Utc::now();
    outline.summary = outline.summary.or(request.goal);
    let outline_path = store.save(slug, &outline)?;
    record_outline_metric(
        base,
        started.elapsed(),
        true,
        serde_json::json!({
            "added": nodes.len(),
            "total": outline.nodes.len(),
            "warnings": warnings.len(),
        }),
    );

    Ok(OutlineProposal {
        outline,
        added_nodes: nodes,
        warnings,
        outline_path,
    })
}

/// Updates node status and records an undo checkpoint when an event id is provided.
pub fn update_outline_status(
    base: &Base,
    slug: &str,
    node_id: Uuid,
    status: OutlineStatus,
    event_id: Option<Uuid>,
) -> WritingResult<(OutlineDocument, Option<OutlineCheckpoint>)> {
    let started = std::time::Instant::now();
    let store = OutlineStore::new(base);
    let mut outline = store.load(slug)?;
    let mut checkpoint = None;
    if let Some(event_id) = event_id {
        checkpoint = Some(store.checkpoint(slug, event_id, &outline)?);
    }

    let mut changed = false;
    let mut accepted_references: Vec<String> = Vec::new();
    for node in &mut outline.nodes {
        if node.id == node_id {
            node.status = status;
            node.last_edited_event_id = event_id;
            node.revision_history
                .push(format!("{} -> {:?}", Utc::now().to_rfc3339(), status));
            if status == OutlineStatus::Accepted && !node.references.is_empty() {
                accepted_references = node.references.clone();
            }
            changed = true;
            break;
        }
    }

    if !changed {
        anyhow::bail!("Outline node {} not found for project {}", node_id, slug);
    }

    outline.updated_at = Utc::now();
    let _ = store.save(slug, &outline)?;
    if !accepted_references.is_empty() {
        citations::sync_references_for_outline(base, slug, &accepted_references)?;
    }
    record_outline_metric(
        base,
        started.elapsed(),
        true,
        serde_json::json!({
            "status": format!("{:?}", status),
            "node": node_id,
            "accepted_count": outline.nodes.iter().filter(|n| n.status == OutlineStatus::Accepted).count(),
        }),
    );
    Ok((outline, checkpoint))
}

fn outline_version() -> String {
    OUTLINE_VERSION.to_string()
}

fn index_library_entries(
    manager: &BaseManager,
    base: &Base,
) -> WritingResult<HashMap<String, (String, Option<String>)>> {
    let mut map = HashMap::new();
    let entries: Vec<LibraryEntry> = manager.load_library_entries(base)?;
    for entry in entries {
        let summary = build_entry_summary(&entry);
        map.insert(
            entry.entry_id.to_string(),
            (entry.title.clone(), summary.clone()),
        );
        map.insert(
            entry.identifier.to_string(),
            (entry.title.clone(), summary.clone()),
        );
        map.insert(
            entry.title.to_ascii_lowercase(),
            (entry.title.clone(), summary.clone()),
        );
    }
    Ok(map)
}

fn build_entry_summary(entry: &LibraryEntry) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(venue) = &entry.venue {
        parts.push(venue.clone());
    }
    if let Some(year) = entry.year {
        parts.push(year.to_string());
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("Reference context: {}.", parts.join(", ")))
    }
}

fn record_outline_metric(
    base: &Base,
    duration: std::time::Duration,
    success: bool,
    details: serde_json::Value,
) {
    let log = OrchestrationLog::for_base(base);
    let _ = log.record_metric(&MetricRecord::Writing(WritingMetricRecord {
        kind: WritingMetricKind::OutlineSync,
        duration_ms: duration.as_millis() as i64,
        success,
        details,
    }));
}
