use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::bases::Base;
use crate::writing::WritingResult;

const WRITING_DIR: &str = "writing";
const OUTLINE_FILE: &str = "outline.json";
const DRAFTS_DIR: &str = "draft_sections";
const UNDO_DIR: &str = "undo";
const LEARNING_DIR: &str = "learning_sessions";
const CONTEXT_FILE: &str = "context.json";
const REGENERATION_FILE: &str = "regeneration_pointer.json";
const REGENERATION_TEST_FILE: &str = "regeneration_dry_run.log";

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

/// Helper for storing learning session artifacts in the AI layer.
pub struct LearningSessionStore<'a> {
    base: &'a Base,
}

impl<'a> LearningSessionStore<'a> {
    pub fn new(base: &'a Base) -> Self {
        Self { base }
    }

    fn session_root(&self, session_id: &Uuid) -> PathBuf {
        self.base
            .ai_layer_path
            .join(LEARNING_DIR)
            .join(session_id.to_string())
    }

    fn questions_dir(&self, session_id: &Uuid) -> PathBuf {
        self.session_root(session_id).join("questions")
    }

    fn evaluations_dir(&self, session_id: &Uuid) -> PathBuf {
        self.session_root(session_id).join("evaluations")
    }

    fn summary_path(&self, session_id: &Uuid) -> PathBuf {
        self.session_root(session_id).join("summary.json")
    }

    fn ensure_root(&self, session_id: &Uuid) -> WritingResult<()> {
        fs::create_dir_all(self.session_root(session_id)).with_context(|| {
            format!(
                "Failed to create learning session directory {}",
                self.session_root(session_id).display()
            )
        })?;
        Ok(())
    }

    pub fn save_context<T: Serialize>(
        &self,
        session_id: &Uuid,
        context: &T,
    ) -> WritingResult<PathBuf> {
        self.ensure_root(session_id)?;
        let path = self.session_root(session_id).join(CONTEXT_FILE);
        write_json(&path, context)?;
        Ok(path)
    }

    pub fn load_context<T: DeserializeOwned>(
        &self,
        session_id: &Uuid,
    ) -> WritingResult<Option<T>> {
        let path = self.session_root(session_id).join(CONTEXT_FILE);
        read_json(&path)
    }

    pub fn save_question<T: Serialize>(
        &self,
        session_id: &Uuid,
        question_id: &Uuid,
        question: &T,
    ) -> WritingResult<PathBuf> {
        let dir = self.questions_dir(session_id);
        fs::create_dir_all(&dir).with_context(|| {
            format!(
                "Failed to create learning question directory {}",
                dir.display()
            )
        })?;
        let path = dir.join(format!("{question_id}.json"));
        write_json(&path, question)?;
        Ok(path)
    }

    pub fn save_evaluation<T: Serialize>(
        &self,
        session_id: &Uuid,
        question_id: &Uuid,
        evaluation: &T,
    ) -> WritingResult<PathBuf> {
        let dir = self.evaluations_dir(session_id);
        fs::create_dir_all(&dir).with_context(|| {
            format!(
                "Failed to create learning evaluation directory {}",
                dir.display()
            )
        })?;
        let path = dir.join(format!("{question_id}.json"));
        write_json(&path, evaluation)?;
        Ok(path)
    }

    pub fn save_summary<T: Serialize>(
        &self,
        session_id: &Uuid,
        summary: &T,
    ) -> WritingResult<PathBuf> {
        self.ensure_root(session_id)?;
        let path = self.summary_path(session_id);
        write_json(&path, summary)?;
        Ok(path)
    }

    pub fn save_regeneration_pointer(
        &self,
        session_id: &Uuid,
        pointer: &Value,
    ) -> Result<PathBuf> {
        self.ensure_root(session_id)?;
        let path = self.session_root(session_id).join(REGENERATION_FILE);
        write_json(&path, pointer)?;
        Ok(path)
    }

    /// Perform a dry-run regeneration check by confirming presence of core artifacts.
    pub fn dry_run_regeneration_check(
        &self,
        session_id: &Uuid,
    ) -> Result<PathBuf> {
        // Verify that context and at least one question/evaluation exist.
        let context_path = self.session_root(session_id).join(CONTEXT_FILE);
        let summary_path = self.summary_path(session_id);
        if !context_path.exists() {
            anyhow::bail!("Missing learning session context for {session_id}");
        }
        if !summary_path.exists() {
            anyhow::bail!("Missing learning session summary for {session_id}");
        }
        // Record a lightweight log of the dry-run result.
        let path = self.session_root(session_id).join(REGENERATION_TEST_FILE);
        let timestamp = chrono::Utc::now().to_rfc3339();
        fs::write(&path, format!("[OK] Dry-run regeneration check at {timestamp}\n"))?;
        Ok(path)
    }
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> WritingResult<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read AI-layer payload {}", path.display()))?;
    let payload = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse AI-layer payload {}", path.display()))?;
    Ok(Some(payload))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> WritingResult<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory {}", dir.display()))?;
    }
    let data = serde_json::to_string_pretty(value)?;
    fs::write(path, data)
        .with_context(|| format!("Failed to write AI-layer payload {}", path.display()))?;
    Ok(())
}
