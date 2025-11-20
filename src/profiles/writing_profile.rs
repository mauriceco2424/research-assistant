use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::bases::layout::ProfileLayout;
use crate::bases::Base;
use crate::orchestration::profiles::defaults::default_writing_profile;
use crate::orchestration::profiles::model::WritingProfile;
use crate::orchestration::profiles::storage::{read_profile, write_profile};

const WRITING_PROFILE_KEY: &str = "writing";
const STYLE_MODELS_FILE: &str = "writing_style_models.json";

/// Convenience wrapper that exposes writing profile + style model accessors.
pub struct WritingProfileStore<'a> {
    layout: ProfileLayout,
    #[allow(unused)]
    base: &'a Base,
}

impl<'a> WritingProfileStore<'a> {
    pub fn new(base: &'a Base) -> Self {
        Self {
            layout: ProfileLayout::new(base),
            base,
        }
    }

    fn profile_path(&self) -> PathBuf {
        self.layout.profile_json(WRITING_PROFILE_KEY)
    }

    fn style_models_path(&self) -> PathBuf {
        self.layout.ai_profiles_dir.join(STYLE_MODELS_FILE)
    }

    /// Loads the persisted writing profile or returns the default shell.
    pub fn load_profile(&self) -> Result<WritingProfile> {
        if let Some(profile) = read_profile(self.profile_path())? {
            return Ok(profile);
        }
        Ok(default_writing_profile())
    }

    /// Persists the provided writing profile JSON artifact.
    pub fn save_profile(&self, profile: &WritingProfile) -> Result<()> {
        write_profile(self.profile_path(), profile)?;
        Ok(())
    }

    /// Returns all recorded style models for the Base (if any).
    pub fn load_style_models(&self) -> Result<Vec<StyleModelRecord>> {
        let path = self.style_models_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read(&path)
            .with_context(|| format!("Failed to read style models at {}", path.display()))?;
        let models: Vec<StyleModelRecord> = serde_json::from_slice(&data)
            .with_context(|| format!("Failed to parse style models at {}", path.display()))?;
        Ok(models)
    }

    /// Writes the provided style model records atomically.
    pub fn save_style_models(&self, models: &[StyleModelRecord]) -> Result<()> {
        let path = self.style_models_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_vec_pretty(models)
            .with_context(|| format!("Failed to serialize style models to {}", path.display()))?;
        fs::write(&path, data)
            .with_context(|| format!("Failed to write style models to {}", path.display()))?;
        Ok(())
    }

    /// Inserts or updates a style model record.
    pub fn upsert_style_model(&self, record: StyleModelRecord) -> Result<Vec<StyleModelRecord>> {
        let mut models = self.load_style_models()?;
        if let Some(existing) = models.iter_mut().find(|model| model.id == record.id) {
            *existing = record;
        } else {
            models.push(record);
        }
        self.save_style_models(&models)?;
        Ok(models)
    }

    /// Helper to append a new style model derived from the provided metadata.
    pub fn record_style_model(
        &self,
        source_pdf_path: PathBuf,
        analysis_method: StyleAnalysisMethod,
        features: Value,
        consent_token: Option<String>,
        notes: Option<String>,
    ) -> Result<StyleModelRecord> {
        let record = StyleModelRecord {
            id: Uuid::new_v4(),
            source_pdf_path,
            analysis_date: Utc::now(),
            features,
            analysis_method,
            consent_token,
            notes,
        };
        self.upsert_style_model(record.clone())?;
        Ok(record)
    }
}

/// Stored metadata for each ingested style model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleModelRecord {
    pub id: Uuid,
    pub source_pdf_path: PathBuf,
    pub analysis_date: DateTime<Utc>,
    #[serde(default)]
    pub features: Value,
    #[serde(default)]
    pub analysis_method: StyleAnalysisMethod,
    #[serde(default)]
    pub consent_token: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Describes how a style model was generated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleAnalysisMethod {
    Local,
    Remote { provider_id: String },
}

impl Default for StyleAnalysisMethod {
    fn default() -> Self {
        StyleAnalysisMethod::Local
    }
}
