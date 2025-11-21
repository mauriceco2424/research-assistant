use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::bases::{Base, BaseManager};
use crate::storage::ai_layer::WritingAiStore;
use crate::writing::citations::{
    capture_citations, detect_bibliography_drift, sync_bibliography, verify_citations,
};
use crate::writing::outline::OutlineDocument;
use crate::writing::project::ProjectPaths;

use super::WritingResult;

/// Metadata persisted for each draft section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftSectionMetadata {
    pub section_id: Uuid,
    pub file_path: String,
    pub last_generated_at: String,
    #[serde(default)]
    pub last_edited_event_id: Option<Uuid>,
    #[serde(default)]
    pub citations: Vec<String>,
    #[serde(default)]
    pub undo_chain: Vec<String>,
    #[serde(default)]
    pub hash: String,
}

/// Request to produce a draft for an outline node.
#[derive(Debug, Clone, Default)]
pub struct DraftRequest {
    pub slug: String,
    pub node_id: Uuid,
    pub instructions: Option<String>,
    pub cite_sources: Vec<String>,
}

/// Outcome of a draft generation.
#[derive(Debug, Clone)]
pub struct DraftOutcome {
    pub path: PathBuf,
    pub metadata: DraftSectionMetadata,
    pub content: String,
    pub drift_warnings: Vec<String>,
    pub bibliography_path: PathBuf,
}

/// Generates a draft file plus AI-layer metadata for an outline node.
pub fn generate_draft(
    manager: &BaseManager,
    base: &Base,
    request: DraftRequest,
    outline: Option<&OutlineDocument>,
    event_id: Option<Uuid>,
) -> WritingResult<DraftOutcome> {
    let paths = ProjectPaths::new(base, &request.slug);
    if !paths.project_root.exists() {
        bail!(
            "Writing project '{}' is missing; run /writing start first.",
            request.slug
        );
    }
    fs::create_dir_all(&paths.sections_dir)?;
    let file_path = paths
        .sections_dir
        .join(format!("{}.tex", request.node_id.to_string()));

    let outline_title = outline
        .and_then(|doc| doc.nodes.iter().find(|n| n.id == request.node_id))
        .map(|node| node.title.clone())
        .unwrap_or_else(|| "Draft Section".to_string());

    let instructions = request.instructions.clone().unwrap_or_default();
    let content = render_draft_content(&outline_title, &instructions, &request.cite_sources);
    fs::write(&file_path, &content)
        .with_context(|| format!("Failed to write draft section {}", file_path.display()))?;

    let citations = capture_citations(&content);
    let citation_links = verify_citations(manager, base, &citations)?;
    let bibliography_path = sync_bibliography(base, &request.slug, &citation_links)?;
    let drift_warnings = detect_bibliography_drift(base, &request.slug, &citations)?;

    let metadata = DraftSectionMetadata {
        section_id: request.node_id,
        file_path: file_path.display().to_string(),
        last_generated_at: Utc::now().to_rfc3339(),
        last_edited_event_id: event_id,
        citations,
        undo_chain: Vec::new(),
        hash: content_hash(&content),
    };

    let ai_store = WritingAiStore::new(base);
    ai_store.save_draft_section(&request.slug, &request.node_id.to_string(), &metadata)?;

    Ok(DraftOutcome {
        path: file_path,
        metadata,
        content,
        drift_warnings,
        bibliography_path,
    })
}

/// Applies structured edits to an existing draft while preserving metadata.
pub fn apply_edit(
    base: &Base,
    slug: &str,
    node_id: Uuid,
    updated_content: &str,
    event_id: Option<Uuid>,
) -> WritingResult<DraftOutcome> {
    let paths = ProjectPaths::new(base, slug);
    let file_path = paths.sections_dir.join(format!("{node_id}.tex"));
    if !file_path.exists() {
        bail!("Draft section {} not found for project {}", node_id, slug);
    }
    fs::write(&file_path, updated_content)
        .with_context(|| format!("Failed to update draft {}", file_path.display()))?;

    let citations = capture_citations(updated_content);
    let ai_store = WritingAiStore::new(base);
    let mut metadata: DraftSectionMetadata = ai_store
        .load_draft_section(slug, &node_id.to_string())?
        .unwrap_or(DraftSectionMetadata {
            section_id: node_id,
            file_path: file_path.display().to_string(),
            last_generated_at: Utc::now().to_rfc3339(),
            last_edited_event_id: None,
            citations: Vec::new(),
            undo_chain: Vec::new(),
            hash: String::new(),
        });
    metadata.last_edited_event_id = event_id;
    metadata.citations = citations.clone();
    metadata.hash = content_hash(updated_content);
    ai_store.save_draft_section(slug, &node_id.to_string(), &metadata)?;

    Ok(DraftOutcome {
        path: file_path,
        metadata,
        content: updated_content.to_string(),
        drift_warnings: detect_bibliography_drift(base, slug, &citations)?,
        bibliography_path: ProjectPaths::new(base, slug).bibliography_path,
    })
}

fn render_draft_content(title: &str, instructions: &str, cite_sources: &[String]) -> String {
    let mut body = String::new();
    body.push_str(&format!("% Auto-generated draft for {title}\n\n"));
    if !instructions.is_empty() {
        body.push_str(&format!("% Instructions: {instructions}\n\n"));
    }
    body.push_str(&format!("\\section{{{title}}}\n"));
    body.push_str("This section was generated by the Writing Assistant. ");
    if !cite_sources.is_empty() {
        let cites: Vec<String> = cite_sources
            .iter()
            .map(|key| format!("\\cite{{{key}}}"))
            .collect();
        body.push_str(&format!(
            "Citations included for requested sources: {}.\n\n",
            cites.join(", ")
        ));
    } else {
        body.push_str("\n\n");
    }
    body.push_str(
        "Fill in detailed arguments, evidence, and transitions. Replace placeholders before publishing.\n",
    );
    body
}

fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
