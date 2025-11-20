pub mod categories;
pub mod categories_assignments;
pub mod categories_merge;
pub mod categories_metrics;
pub mod categories_narrative;
pub mod categories_snapshot;
mod config;
mod fs_paths;
pub mod layout;
mod migrations;

pub use categories::{
    category_slug, AssignmentSource, AssignmentStatus, CategoryAssignment,
    CategoryAssignmentsIndex, CategoryDefinition, CategoryNarrative, CategoryOrigin,
    CategoryProposalBatch, CategoryProposalPreview, CategoryProposalStore, CategoryRecord,
    CategoryStore,
};
pub use categories_assignments::move_papers;
pub use categories_merge::{merge_categories, MergeOptions, MergeOutcome};
pub use categories_metrics::{CategoryMetricsStore, CategoryStatusMetric};
pub use categories_narrative::{apply_narrative_update, NarrativeUpdate};
pub use categories_snapshot::{CategorySnapshot, CategorySnapshotStore};
pub use config::{
    ensure_workspace_structure, workspace_root, AcquisitionSettings, AppConfig, IngestionSettings,
    WorkspacePaths,
};
pub use fs_paths::{ensure_category_dirs, CategoryPaths};
pub use layout::{
    ai_profiles_dir, ensure_intents_dir, profile_json_path, user_profiles_dir, ProfileLayout,
    AI_PROFILES_SUBDIR, CONSENT_MANIFESTS_SUBDIR, INTENTS_SUBDIR, PROFILE_EXPORTS_SUBDIR,
    USER_PROFILES_SUBDIR,
};

use crate::orchestration::{log_event, EventType, OrchestrationEvent};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

/// Represents a Paper Base with its metadata and filesystem locations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub user_layer_path: PathBuf,
    pub ai_layer_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub last_active_at: Option<DateTime<Utc>>,
}

/// Library entry stored in a Base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub entry_id: Uuid,
    pub title: String,
    pub authors: Vec<String>,
    pub venue: Option<String>,
    pub year: Option<i32>,
    pub identifier: String,
    pub pdf_paths: Vec<PathBuf>,
    pub needs_pdf: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LibraryEntry {
    pub fn mark_pdf_attached(&mut self, path: PathBuf) {
        self.pdf_paths.push(path);
        self.needs_pdf = self.pdf_paths.is_empty();
        self.updated_at = Utc::now();
    }
}

/// Manages Bases, configuration, and storage.
pub struct BaseManager {
    pub config: AppConfig,
    pub paths: WorkspacePaths,
    pub config_path: PathBuf,
}

impl BaseManager {
    pub fn new() -> Result<Self> {
        let paths = ensure_workspace_structure()?;
        let mut config = config::load_or_default()?;
        let config_path = config::config_file_path()?;

        // If no last active base, try to pick the first existing base.
        if config.last_active_base_id.is_none() {
            if let Some(first_base) = Self::discover_bases(&paths)?.first() {
                config.last_active_base_id = Some(first_base.id.to_string());
                config::save(&config)?;
            }
        }

        Ok(Self {
            config,
            paths,
            config_path,
        })
    }

    fn discover_bases(paths: &WorkspacePaths) -> Result<Vec<Base>> {
        let mut bases = Vec::new();
        if paths.ai_dir.exists() {
            for entry in fs::read_dir(&paths.ai_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let base_metadata = entry.path().join("base.json");
                    if base_metadata.exists() {
                        let base: Base = serde_json::from_slice(&fs::read(&base_metadata)?)?;
                        bases.push(base);
                    }
                }
            }
        }
        bases.sort_by_key(|b| b.created_at);
        Ok(bases)
    }

    pub fn list_bases(&self) -> Result<Vec<Base>> {
        Self::discover_bases(&self.paths)
    }

    pub fn get_base(&self, base_id: &Uuid) -> Result<Option<Base>> {
        Ok(self.list_bases()?.into_iter().find(|b| &b.id == base_id))
    }

    pub fn create_base(&mut self, name: &str) -> Result<Base> {
        let slug = slugify(name);
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let user_path = self.paths.base_user_layer(&slug);
        let ai_path = self.paths.base_ai_layer(&id.to_string());
        fs::create_dir_all(&user_path)?;
        fs::create_dir_all(&ai_path)?;
        let base = Base {
            id,
            name: name.to_string(),
            slug,
            user_layer_path: user_path,
            ai_layer_path: ai_path,
            created_at,
            last_active_at: Some(created_at),
        };
        fs_paths::ensure_category_dirs(&base)?;
        migrations::profile_shells::ensure_profile_shells(&base)?;
        ensure_intents_dir(&base)?;
        self.persist_base(&base)?;
        log_event(
            self,
            &base,
            EventType::BaseCreated,
            serde_json::json!({ "base_id": base.id, "name": base.name }),
        )?;
        self.set_active_base(&base.id)?;
        Ok(base)
    }

    fn persist_base(&self, base: &Base) -> Result<()> {
        let metadata_path = base.ai_layer_path.join("base.json");
        fs::create_dir_all(&base.ai_layer_path)?;
        fs::write(metadata_path, serde_json::to_vec_pretty(base)?)?;
        Ok(())
    }

    pub fn set_active_base(&mut self, base_id: &Uuid) -> Result<()> {
        self.config.last_active_base_id = Some(base_id.to_string());
        // update last_active_at in metadata
        if let Some(mut base) = self.get_base(base_id)? {
            base.last_active_at = Some(Utc::now());
            self.persist_base(&base)?;
            log_event(
                self,
                &base,
                EventType::BaseSelected,
                serde_json::json!({ "base_id": base.id, "name": base.name }),
            )?;
        }
        config::save(&self.config)?;
        Ok(())
    }

    pub fn active_base(&self) -> Result<Option<Base>> {
        match &self.config.last_active_base_id {
            Some(id) => {
                let uuid = Uuid::parse_str(id).context("Invalid last_active_base_id in config")?;
                self.get_base(&uuid)
            }
            None => Ok(None),
        }
    }

    pub fn base_library_path(&self, base: &Base) -> PathBuf {
        base.ai_layer_path.join("library_entries.json")
    }

    fn metadata_store_path(&self, base: &Base) -> PathBuf {
        base.ai_layer_path.join("metadata_records.json")
    }

    fn metadata_changes_path(&self, base: &Base) -> PathBuf {
        base.ai_layer_path.join("metadata_changes.jsonl")
    }

    pub fn load_library_entries(&self, base: &Base) -> Result<Vec<LibraryEntry>> {
        let path = self.base_library_path(base);
        if path.exists() {
            let entries: Vec<LibraryEntry> = serde_json::from_slice(&fs::read(path)?)?;
            Ok(entries)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn load_metadata_records(&self, base: &Base) -> Result<Vec<MetadataRecord>> {
        let path = self.metadata_store_path(base);
        if path.exists() {
            let records: Vec<MetadataRecord> = serde_json::from_slice(&fs::read(path)?)?;
            Ok(records)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn save_library_entries(&self, base: &Base, entries: &[LibraryEntry]) -> Result<()> {
        let path = self.base_library_path(base);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(entries)?)?;
        Ok(())
    }

    pub fn save_metadata_records(&self, base: &Base, records: &[MetadataRecord]) -> Result<()> {
        let path = self.metadata_store_path(base);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(records)?)?;
        Ok(())
    }

    pub fn remove_metadata_records(&self, base: &Base, ids: &[Uuid]) -> Result<()> {
        let mut records = self.load_metadata_records(base)?;
        records.retain(|record| !ids.contains(&record.record_id));
        self.save_metadata_records(base, &records)
    }

    pub fn upsert_metadata_record(
        &self,
        base: &Base,
        record: MetadataRecord,
    ) -> Result<MetadataRecord> {
        let mut records = self.load_metadata_records(base)?;
        if let Some(existing) = records
            .iter_mut()
            .find(|existing| existing.record_id == record.record_id)
        {
            *existing = record.clone();
        } else if let Some(existing) = records
            .iter_mut()
            .find(|existing| existing.identifier == record.identifier)
        {
            existing.merge_from(&record);
        } else {
            records.push(record.clone());
        }
        self.save_metadata_records(base, &records)?;
        Ok(record)
    }

    pub fn ensure_metadata_only_record(
        &self,
        base: &Base,
        identifier: &str,
        title: &str,
    ) -> Result<MetadataRecord> {
        let mut records = self.load_metadata_records(base)?;
        if let Some(existing) = records
            .iter()
            .find(|record| record.identifier == identifier)
        {
            return Ok(existing.clone());
        }
        let record = MetadataRecord::metadata_only(identifier, title.to_string());
        records.push(record.clone());
        self.save_metadata_records(base, &records)?;
        Ok(record)
    }

    pub fn record_metadata_change_batch(
        &self,
        base: &Base,
        batch: &MetadataChangeBatch,
    ) -> Result<()> {
        let path = self.metadata_changes_path(base);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        file.write_all(serde_json::to_string(batch)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn undo_last_metadata_change_batch(
        &self,
        base: &Base,
    ) -> Result<Option<MetadataChangeBatch>> {
        let path = self.metadata_changes_path(base);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&path)?;
        let mut batches = Vec::new();
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            let batch: MetadataChangeBatch = serde_json::from_str(line)?;
            batches.push(batch);
        }
        if let Some(batch) = batches.pop() {
            self.persist_metadata_change_batches(base, &batches)?;
            self.apply_metadata_change_batch(base, &batch, false)?;
            Ok(Some(batch))
        } else {
            Ok(None)
        }
    }

    fn persist_metadata_change_batches(
        &self,
        base: &Base,
        batches: &[MetadataChangeBatch],
    ) -> Result<()> {
        let path = self.metadata_changes_path(base);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&path)?;
        for batch in batches {
            file.write_all(serde_json::to_string(batch)?.as_bytes())?;
            file.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn apply_metadata_change_batch(
        &self,
        base: &Base,
        batch: &MetadataChangeBatch,
        apply_after: bool,
    ) -> Result<()> {
        for entry in &batch.changes {
            match (apply_after, &entry.after, &entry.before) {
                (true, Some(after), _) => {
                    self.upsert_metadata_record(base, after.clone())?;
                }
                (false, _, Some(before)) => {
                    self.upsert_metadata_record(base, before.clone())?;
                }
                (false, _, None) => {
                    self.remove_metadata_records(base, &[entry.record_id])?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn remove_entries_by_ids(&self, base: &Base, ids: &[Uuid]) -> Result<Vec<LibraryEntry>> {
        let mut entries = self.load_library_entries(base)?;
        let mut removed = Vec::new();
        entries.retain(|entry| {
            if ids.contains(&entry.entry_id) {
                removed.push(entry.clone());
                false
            } else {
                true
            }
        });
        self.save_library_entries(base, &entries)?;
        Ok(removed)
    }

    pub fn log_event(&self, base: &Base, event: OrchestrationEvent) -> Result<()> {
        let log_path = base.ai_layer_path.join("events.jsonl");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        file.write_all(serde_json::to_string(&event)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }
}

/// Create a filesystem-safe slug from a base name.
fn slugify(name: &str) -> String {
    let mut slug = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

/// Utility to generate placeholder text for category summaries.
pub fn placeholder_excerpt() -> String {
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.sample(Alphanumeric) as char).collect()
}

/// Normalized metadata representation stored in the AI layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataRecord {
    pub record_id: Uuid,
    pub paper_id: Option<Uuid>,
    pub identifier: String,
    pub doi: Option<String>,
    pub title: String,
    pub authors: Vec<String>,
    pub venue: Option<String>,
    pub year: Option<i32>,
    pub language: Option<String>,
    pub keywords: Vec<String>,
    pub references: Vec<String>,
    pub dedup_status: MetadataDedupStatus,
    pub provenance: Option<String>,
    pub missing_pdf: bool,
    pub missing_figures: bool,
    #[serde(default)]
    pub script_direction: Option<String>,
    pub last_updated: DateTime<Utc>,
}

impl MetadataRecord {
    pub fn metadata_only(identifier: &str, title: String) -> Self {
        Self {
            record_id: Uuid::new_v4(),
            paper_id: None,
            identifier: identifier.to_string(),
            doi: None,
            title,
            authors: Vec::new(),
            venue: None,
            year: None,
            language: None,
            keywords: Vec::new(),
            references: Vec::new(),
            dedup_status: MetadataDedupStatus::MetadataOnly,
            provenance: Some("metadata_only_ingestion".into()),
            missing_pdf: true,
            missing_figures: true,
            script_direction: None,
            last_updated: Utc::now(),
        }
    }

    pub fn merge_from(&mut self, source: &MetadataRecord) {
        self.paper_id = source.paper_id;
        self.identifier = source.identifier.clone();
        self.doi = source.doi.clone();
        self.title = source.title.clone();
        self.authors = source.authors.clone();
        self.venue = source.venue.clone();
        self.year = source.year;
        self.language = source.language.clone();
        self.keywords = source.keywords.clone();
        self.references = source.references.clone();
        self.dedup_status = source.dedup_status;
        self.provenance = source.provenance.clone();
        self.missing_pdf = source.missing_pdf;
        self.missing_figures = source.missing_figures;
        self.script_direction = source.script_direction.clone();
        self.last_updated = Utc::now();
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetadataDedupStatus {
    Unknown,
    Unique,
    Duplicate,
    Merged,
    MetadataOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataChangeEntry {
    pub change_id: Uuid,
    pub record_id: Uuid,
    pub before: Option<MetadataRecord>,
    pub after: Option<MetadataRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataChangeBatch {
    pub batch_id: Uuid,
    pub approval_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub changes: Vec<MetadataChangeEntry>,
}
