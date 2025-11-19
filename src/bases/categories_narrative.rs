use anyhow::{Context, Result};
use uuid::Uuid;

use super::categories::{CategoryRecord, CategoryStore};

#[derive(Default)]
pub struct NarrativeUpdate {
    pub summary: Option<String>,
    pub learning_prompts: Option<Vec<String>>,
    pub notes: Option<Vec<String>>,
    pub pinned_papers: Option<Vec<Uuid>>,
    pub figure_gallery_enabled: Option<bool>,
}

pub fn apply_narrative_update(
    store: &CategoryStore,
    category_id: &Uuid,
    mut update: NarrativeUpdate,
) -> Result<CategoryRecord> {
    let mut record = store
        .get(category_id)?
        .with_context(|| format!("Category {:?} not found", category_id))?;

    if let Some(summary) = update.summary.take() {
        record.narrative.summary = summary;
    }
    if let Some(prompts) = update.learning_prompts.take() {
        record.narrative.learning_prompts = prompts;
    }
    if let Some(notes) = update.notes.take() {
        record.narrative.notes = notes;
    }
    if let Some(pinned) = update.pinned_papers.take() {
        let mut dedup = Vec::new();
        for paper_id in pinned {
            if !dedup.contains(&paper_id) {
                dedup.push(paper_id);
            }
        }
        record.definition.pinned_papers = dedup;
    }
    if let Some(toggle) = update.figure_gallery_enabled {
        record.definition.figure_gallery_enabled = toggle;
    }
    record.narrative.last_updated_at = chrono::Utc::now();
    store.save(&record)?;
    Ok(record)
}
