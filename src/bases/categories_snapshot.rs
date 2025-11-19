use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ensure_category_dirs, CategoryAssignment, CategoryAssignmentsIndex, CategoryRecord,
    CategoryStore,
};
use crate::bases::Base;

/// Snapshot metadata stored on disk for undo/redo flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySnapshot {
    pub snapshot_id: Uuid,
    pub base_id: Uuid,
    pub taken_at: DateTime<Utc>,
    pub reason: String,
    pub files: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct SnapshotPayload {
    snapshot: CategorySnapshot,
    records: Vec<CategoryRecord>,
    assignments: Vec<CategoryAssignment>,
}

/// Handles capture/restore of category snapshots for undo workflows.
pub struct CategorySnapshotStore {
    base_id: Uuid,
    snapshots_dir: PathBuf,
    definitions_dir: PathBuf,
    assignments_dir: PathBuf,
}

impl CategorySnapshotStore {
    pub fn new(base: &Base) -> Result<Self> {
        let paths = ensure_category_dirs(base)?;
        Ok(Self {
            base_id: base.id,
            snapshots_dir: paths.snapshots_dir,
            definitions_dir: paths.definitions_dir,
            assignments_dir: paths.assignments_dir,
        })
    }

    fn snapshot_path(&self, snapshot_id: &Uuid) -> PathBuf {
        self.snapshots_dir
            .join(format!("{}.json", snapshot_id.to_string()))
    }

    pub fn list(&self) -> Result<Vec<CategorySnapshot>> {
        let mut snapshots = Vec::new();
        if !self.snapshots_dir.exists() {
            return Ok(snapshots);
        }
        for entry in fs::read_dir(&self.snapshots_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let data = fs::read(entry.path())
                .with_context(|| format!("Failed to read snapshot {:?}", entry.path()))?;
            let payload: SnapshotPayload = serde_json::from_slice(&data)
                .with_context(|| format!("Failed to parse snapshot {:?}", entry.path()))?;
            snapshots.push(payload.snapshot);
        }
        snapshots.sort_by(|a, b| b.taken_at.cmp(&a.taken_at));
        Ok(snapshots)
    }

    pub fn capture(
        &self,
        store: &CategoryStore,
        assignments: &CategoryAssignmentsIndex,
        reason: &str,
    ) -> Result<CategorySnapshot> {
        fs::create_dir_all(&self.snapshots_dir)?;
        let snapshot_id = Uuid::new_v4();
        let snapshot = CategorySnapshot {
            snapshot_id,
            base_id: self.base_id,
            taken_at: Utc::now(),
            reason: reason.to_string(),
            files: self.list_tracked_files()?,
        };
        let payload = SnapshotPayload {
            snapshot: snapshot.clone(),
            records: store.list()?,
            assignments: assignments.list_all()?,
        };
        let path = self.snapshot_path(&snapshot_id);
        let data = serde_json::to_vec_pretty(&payload)?;
        fs::write(&path, data).with_context(|| format!("Failed to write {:?}", path))?;
        Ok(snapshot)
    }

    pub fn restore(
        &self,
        snapshot_id: &Uuid,
        store: &CategoryStore,
        assignments: &CategoryAssignmentsIndex,
    ) -> Result<()> {
        let path = self.snapshot_path(snapshot_id);
        if !path.exists() {
            bail!("Snapshot {:?} not found", snapshot_id);
        }
        let data =
            fs::read(&path).with_context(|| format!("Failed to read snapshot {:?}", path))?;
        let payload: SnapshotPayload =
            serde_json::from_slice(&data).with_context(|| "Failed to parse snapshot payload")?;

        self.clear_dir(&self.definitions_dir)?;
        for record in payload.records {
            store.save(&record)?;
        }

        self.clear_dir(&self.assignments_dir)?;
        let mut grouped: HashMap<Uuid, Vec<CategoryAssignment>> = HashMap::new();
        for assignment in payload.assignments {
            grouped
                .entry(assignment.category_id)
                .or_default()
                .push(assignment);
        }
        for (category_id, list) in grouped {
            assignments.replace_category(&category_id, &list)?;
        }

        Ok(())
    }

    fn list_tracked_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        files.extend(self.collect_relative(&self.definitions_dir, "definitions")?);
        files.extend(self.collect_relative(&self.assignments_dir, "assignments")?);
        Ok(files)
    }

    fn collect_relative(&self, dir: &Path, label: &str) -> Result<Vec<String>> {
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut items = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let filename = entry.file_name().to_string_lossy().to_string();
                items.push(format!("{label}/{filename}"));
            }
        }
        Ok(items)
    }

    fn clear_dir(&self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                fs::remove_file(entry.path())
                    .with_context(|| format!("Failed to remove {:?}", entry.path()))?;
            }
        }
        Ok(())
    }
}
