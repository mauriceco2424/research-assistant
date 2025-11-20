use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::bases::Base;
use crate::writing::WritingResult;

const WRITING_DIR: &str = "writing";
const OUTLINE_FILE: &str = "outline.json";
const DRAFTS_DIR: &str = "draft_sections";
const UNDO_DIR: &str = "undo";

/// Helper for reading/writing structured payloads in the AI layer.
pub struct WritingAiStore<'a> {
    base: &'a Base,
}

impl<'a> WritingAiStore<'a> {
    pub fn new(base: &'a Base) -> Self {
        Self { base }
    }

    pub fn project_root(&self, slug: &str) -> PathBuf {
        self.base.ai_layer_path.join(WRITING_DIR).join(slug)
    }

    fn draft_dir(&self, slug: &str) -> PathBuf {
        self.project_root(slug).join(DRAFTS_DIR)
    }

    fn undo_dir(&self, slug: &str) -> PathBuf {
        self.project_root(slug).join(UNDO_DIR)
    }

    fn outline_path(&self, slug: &str) -> PathBuf {
        self.project_root(slug).join(OUTLINE_FILE)
    }

    fn ensure_dir(path: &Path) -> WritingResult<()> {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create AI-layer directory {}", path.display()))?;
        Ok(())
    }

    pub fn ensure_project_dirs(&self, slug: &str) -> WritingResult<()> {
        Self::ensure_dir(&self.project_root(slug))?;
        Self::ensure_dir(&self.draft_dir(slug))?;
        Self::ensure_dir(&self.undo_dir(slug))?;
        Ok(())
    }

    pub fn load_outline<T: DeserializeOwned>(&self, slug: &str) -> WritingResult<Option<T>> {
        read_json(&self.outline_path(slug))
    }

    pub fn save_outline<T: Serialize>(&self, slug: &str, payload: &T) -> WritingResult<()> {
        self.ensure_project_dirs(slug)?;
        write_json(&self.outline_path(slug), payload)
    }

    pub fn load_draft_section<T: DeserializeOwned>(
        &self,
        slug: &str,
        section_id: &str,
    ) -> WritingResult<Option<T>> {
        read_json(&self.draft_metadata_path(slug, section_id))
    }

    pub fn save_draft_section<T: Serialize>(
        &self,
        slug: &str,
        section_id: &str,
        payload: &T,
    ) -> WritingResult<PathBuf> {
        self.ensure_project_dirs(slug)?;
        let path = self.draft_metadata_path(slug, section_id);
        write_json(&path, payload)?;
        Ok(path)
    }

    pub fn load_undo_payload<T: DeserializeOwned>(
        &self,
        slug: &str,
        event_id: &str,
    ) -> WritingResult<Option<T>> {
        read_json(&self.undo_payload_path(slug, event_id))
    }

    pub fn save_undo_payload<T: Serialize>(
        &self,
        slug: &str,
        event_id: &str,
        payload: &T,
    ) -> WritingResult<PathBuf> {
        self.ensure_project_dirs(slug)?;
        let path = self.undo_payload_path(slug, event_id);
        write_json(&path, payload)?;
        Ok(path)
    }

    pub fn draft_metadata_path(&self, slug: &str, section_id: &str) -> PathBuf {
        self.draft_dir(slug).join(format!("{section_id}.json"))
    }

    pub fn undo_payload_path(&self, slug: &str, event_id: &str) -> PathBuf {
        self.undo_dir(slug).join(format!("{event_id}.json"))
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> WritingResult<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read AI-layer payload {}", path.display()))?;
    let payload = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse AI-layer payload {}", path.display()))?;
    Ok(Some(payload))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> WritingResult<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory {}", dir.display()))?;
    }
    let data = serde_json::to_string_pretty(value)?;
    fs::write(path, data)
        .with_context(|| format!("Failed to write AI-layer payload {}", path.display()))?;
    Ok(())
}
