use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ensure_category_dirs, Base, CategoryPaths};

/// Full category record on disk (definition + narrative).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRecord {
    pub definition: CategoryDefinition,
    pub narrative: CategoryNarrative,
}

impl CategoryRecord {
    pub fn new(definition: CategoryDefinition, narrative: CategoryNarrative) -> Self {
        Self {
            definition,
            narrative,
        }
    }
}

/// Structured category definition metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDefinition {
    pub category_id: Uuid,
    pub base_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub confidence: Option<f32>,
    #[serde(default)]
    pub representative_papers: Vec<Uuid>,
    #[serde(default)]
    pub pinned_papers: Vec<Uuid>,
    #[serde(default)]
    pub figure_gallery_enabled: bool,
    pub origin: CategoryOrigin,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Narrative + learning metadata for a category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryNarrative {
    pub narrative_id: Uuid,
    pub category_id: Uuid,
    pub summary: String,
    #[serde(default)]
    pub learning_prompts: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub references: Vec<Uuid>,
    pub ai_assisted: bool,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategoryOrigin {
    Proposed,
    Manual,
}

/// On-disk store for category definition + narrative pairs.
pub struct CategoryStore {
    base_id: Uuid,
    paths: CategoryPaths,
}

impl CategoryStore {
    pub fn new(base: &Base) -> Result<Self> {
        let paths = ensure_category_dirs(base)?;
        Ok(Self {
            base_id: base.id,
            paths,
        })
    }

    fn definition_path(&self, category_id: &Uuid) -> PathBuf {
        self.paths
            .definitions_dir
            .join(format!("{category_id}.json"))
    }

    pub fn list(&self) -> Result<Vec<CategoryRecord>> {
        let mut records = Vec::new();
        if !self.paths.definitions_dir.exists() {
            return Ok(records);
        }
        for entry in fs::read_dir(&self.paths.definitions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let data = fs::read(entry.path())
                .with_context(|| format!("Failed to read category file {:?}", entry.path()))?;
            let record: CategoryRecord = serde_json::from_slice(&data)
                .with_context(|| format!("Failed to parse category file {:?}", entry.path()))?;
            records.push(record);
        }
        records.sort_by(|a, b| a.definition.name.cmp(&b.definition.name));
        Ok(records)
    }

    pub fn get(&self, category_id: &Uuid) -> Result<Option<CategoryRecord>> {
        let path = self.definition_path(category_id);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read(&path).with_context(|| format!("Failed to read {:?}", path))?;
        let record =
            serde_json::from_slice(&data).with_context(|| format!("Failed to parse {:?}", path))?;
        Ok(Some(record))
    }

    pub fn save(&self, record: &CategoryRecord) -> Result<()> {
        let mut to_write = record.clone();
        to_write.definition.base_id = self.base_id;
        to_write.definition.updated_at = Utc::now();
        to_write.narrative.category_id = to_write.definition.category_id;
        to_write.narrative.last_updated_at = Utc::now();
        let path = self.definition_path(&to_write.definition.category_id);
        fs::create_dir_all(&self.paths.definitions_dir)?;
        let data = serde_json::to_vec_pretty(&to_write)?;
        fs::write(&path, data).with_context(|| format!("Failed to write {:?}", path))?;
        Ok(())
    }

    pub fn delete(&self, category_id: &Uuid) -> Result<()> {
        let path = self.definition_path(category_id);
        if path.exists() {
            fs::remove_file(&path).with_context(|| format!("Failed to delete {:?}", path))?;
        }
        Ok(())
    }
}

/// Assignment row representing the relationship between a paper and category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAssignment {
    pub assignment_id: Uuid,
    pub category_id: Uuid,
    pub paper_id: Uuid,
    #[serde(default)]
    pub source: AssignmentSource,
    pub confidence: f32,
    #[serde(default)]
    pub status: AssignmentStatus,
    pub last_reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentSource {
    Auto,
    Manual,
}

impl Default for AssignmentSource {
    fn default() -> Self {
        AssignmentSource::Auto
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStatus {
    Active,
    PendingReview,
}

impl Default for AssignmentStatus {
    fn default() -> Self {
        AssignmentStatus::Active
    }
}

/// Filesystem-backed assignment index storing per-category JSON files.
pub struct CategoryAssignmentsIndex {
    dir: PathBuf,
}

impl CategoryAssignmentsIndex {
    pub fn new(base: &Base) -> Result<Self> {
        let paths = ensure_category_dirs(base)?;
        Ok(Self {
            dir: paths.assignments_dir,
        })
    }

    fn file_path(&self, category_id: &Uuid) -> PathBuf {
        self.dir.join(format!("{category_id}.json"))
    }

    /// Returns assignments for a single category.
    pub fn list_for_category(&self, category_id: &Uuid) -> Result<Vec<CategoryAssignment>> {
        let path = self.file_path(category_id);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read(&path).with_context(|| format!("Failed to read {:?}", path))?;
        let items: Vec<CategoryAssignment> = serde_json::from_slice(&data)
            .with_context(|| format!("Failed to parse assignments {:?}", path))?;
        Ok(items)
    }

    /// Replaces the assignment list for a category.
    pub fn replace_category(
        &self,
        category_id: &Uuid,
        assignments: &[CategoryAssignment],
    ) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        let path = self.file_path(category_id);
        let data = serde_json::to_vec_pretty(assignments)?;
        fs::write(&path, data).with_context(|| format!("Failed to write {:?}", path))?;
        Ok(())
    }

    /// Upserts a single assignment while maintaining uniqueness by paper_id.
    pub fn upsert(&self, assignment: CategoryAssignment) -> Result<()> {
        let category_id = assignment.category_id;
        let mut assignments = self.list_for_category(&assignment.category_id)?;
        if let Some(existing) = assignments
            .iter_mut()
            .find(|a| a.paper_id == assignment.paper_id)
        {
            *existing = assignment;
        } else {
            assignments.push(assignment);
        }
        self.replace_category(&category_id, &assignments)
    }

    /// Removes the assignment for a given paper, if present.
    pub fn remove(&self, category_id: &Uuid, paper_id: &Uuid) -> Result<()> {
        let mut assignments = self.list_for_category(category_id)?;
        let len_before = assignments.len();
        assignments.retain(|a| &a.paper_id != paper_id);
        if assignments.is_empty() {
            let path = self.file_path(category_id);
            if path.exists() {
                fs::remove_file(&path).with_context(|| {
                    format!("Failed to delete empty assignment file {:?}", path)
                })?;
            }
        } else if len_before != assignments.len() {
            self.replace_category(category_id, &assignments)?;
        }
        Ok(())
    }

    /// Returns all assignments across every category.
    pub fn list_all(&self) -> Result<Vec<CategoryAssignment>> {
        let mut all = Vec::new();
        if !self.dir.exists() {
            return Ok(all);
        }
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let data = fs::read(entry.path())
                .with_context(|| format!("Failed to read assignment file {:?}", entry.path()))?;
            let mut items: Vec<CategoryAssignment> = serde_json::from_slice(&data)
                .with_context(|| format!("Failed to parse assignment file {:?}", entry.path()))?;
            all.append(&mut items);
        }
        Ok(all)
    }
}

/// Slugify helper specific to category names.
pub fn category_slug(name: &str) -> String {
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

/// Preview entry generated by proposal worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryProposalPreview {
    pub proposal_id: Uuid,
    pub definition: CategoryDefinition,
    pub narrative: CategoryNarrative,
    pub member_entry_ids: Vec<Uuid>,
    pub generated_at: DateTime<Utc>,
}

/// Proposal batch persisted for preview/application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryProposalBatch {
    pub batch_id: Uuid,
    pub base_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub duration_ms: Option<i64>,
    pub proposals: Vec<CategoryProposalPreview>,
}

/// Filesystem-backed persistence for proposal batches.
pub struct CategoryProposalStore {
    dir: PathBuf,
    base_id: Uuid,
}

impl CategoryProposalStore {
    pub fn new(base: &Base) -> Result<Self> {
        let paths = ensure_category_dirs(base)?;
        Ok(Self {
            dir: paths.proposals_dir,
            base_id: base.id,
        })
    }

    pub fn save_batch(
        &self,
        proposals: Vec<CategoryProposalPreview>,
        duration_ms: Option<i64>,
    ) -> Result<CategoryProposalBatch> {
        fs::create_dir_all(&self.dir)?;
        let batch = CategoryProposalBatch {
            batch_id: Uuid::new_v4(),
            base_id: self.base_id,
            generated_at: Utc::now(),
            duration_ms,
            proposals,
        };
        let path = self.path_for(&batch.batch_id, batch.generated_at);
        let data = serde_json::to_vec_pretty(&batch)?;
        fs::write(&path, data).with_context(|| format!("Failed to write {:?}", path))?;
        Ok(batch)
    }

    pub fn latest_batch(&self) -> Result<Option<CategoryProposalBatch>> {
        if !self.dir.exists() {
            return Ok(None);
        }
        let mut entries = fs::read_dir(&self.dir)?
            .filter_map(|res| res.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.metadata().and_then(|m| m.modified()).ok());
        if let Some(entry) = entries.pop() {
            let data = fs::read(entry.path())
                .with_context(|| format!("Failed to read {:?}", entry.path()))?;
            let batch: CategoryProposalBatch = serde_json::from_slice(&data)?;
            return Ok(Some(batch));
        }
        Ok(None)
    }

    pub fn load_batch(&self, batch_id: &Uuid) -> Result<Option<CategoryProposalBatch>> {
        if !self.dir.exists() {
            return Ok(None);
        }
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            if !entry
                .file_name()
                .to_string_lossy()
                .contains(&batch_id.to_string())
            {
                continue;
            }
            let data = fs::read(entry.path())
                .with_context(|| format!("Failed to read {:?}", entry.path()))?;
            let batch: CategoryProposalBatch = serde_json::from_slice(&data)?;
            return Ok(Some(batch));
        }
        Ok(None)
    }

    fn path_for(&self, batch_id: &Uuid, timestamp: DateTime<Utc>) -> PathBuf {
        let filename = format!("{}_{}.json", timestamp.format("%Y%m%d%H%M%S"), batch_id);
        self.dir.join(filename)
    }
}
