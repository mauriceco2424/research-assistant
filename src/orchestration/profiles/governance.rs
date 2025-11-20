//! Governance utilities for audit/export/delete/regenerate flows.

use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use zip::write::FileOptions;

use crate::{
    bases::{Base, BaseManager, ProfileLayout},
    orchestration::{
        log_profile_event,
        profiles::{
            model::{ProfileChangeKind, ProfileScopeMode, ProfileScopeSetting, ProfileType},
            scope::ProfileScopeStore,
            storage::compute_hash,
        },
        EventType, OrchestrationLog, ProfileEventDetails,
    },
};

pub struct ProfileGovernance<'a> {
    manager: &'a BaseManager,
    base: Base,
    layout: ProfileLayout,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileAuditLog {
    pub profile_type: ProfileType,
    pub generated_at: DateTime<Utc>,
    pub entries: Vec<ProfileAuditEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileAuditEntry {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub change_kind: ProfileChangeKind,
    #[serde(default)]
    pub diff_summary: Vec<String>,
    pub hash_after: Option<String>,
    pub undo_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProfileExportResult {
    pub profile_type: ProfileType,
    pub archive_path: PathBuf,
    pub hash: String,
    pub event_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ProfileDeleteResult {
    pub profile_type: ProfileType,
    pub files_removed: Vec<PathBuf>,
    pub event_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ProfileScopeStatus {
    pub setting: ProfileScopeSetting,
    pub event_id: Option<Uuid>,
}

impl<'a> ProfileGovernance<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        Self {
            manager,
            base: base.clone(),
            layout: ProfileLayout::new(base),
        }
    }

    pub fn audit(&self, profile_type: ProfileType) -> Result<ProfileAuditLog> {
        let log = OrchestrationLog::for_base(&self.base);
        let mut entries = Vec::new();
        for event in log.load_events()? {
            if event.event_type != EventType::ProfileChange {
                continue;
            }
            let details: ProfileEventDetails = serde_json::from_value(event.details)?;
            if details.profile_type != profile_type {
                continue;
            }
            entries.push(ProfileAuditEntry {
                event_id: event.event_id,
                timestamp: event.timestamp,
                change_kind: details.change_kind,
                diff_summary: details.diff_summary,
                hash_after: details.hash_after,
                undo_token: details.undo_token,
            });
        }
        Ok(ProfileAuditLog {
            profile_type,
            generated_at: Utc::now(),
            entries,
        })
    }

    pub fn export(
        &self,
        profile_type: ProfileType,
        destination: Option<PathBuf>,
        include_history: bool,
    ) -> Result<ProfileExportResult> {
        let _lock = self.acquire_export_lock()?;
        let (json_path, html_path) = self.profile_paths(profile_type);
        self.ensure_profile_exists(&json_path)?;
        let archive_path = self.resolve_archive_path(destination, profile_type.slug())?;
        let mut zip_writer = zip::ZipWriter::new(
            File::create(&archive_path)
                .with_context(|| format!("Failed to create {}", archive_path.display()))?,
        );
        let options = FileOptions::default();
        zip_writer.start_file(format!("{}.json", profile_type.slug()), options)?;
        zip_writer.write_all(&fs::read(&json_path)?)?;
        if html_path.exists() {
            zip_writer.start_file(format!("{}.html", profile_type.slug()), options)?;
            zip_writer.write_all(&fs::read(&html_path)?)?;
        }
        if include_history {
            let audit_blob = serde_json::to_vec_pretty(&self.audit(profile_type)?)?;
            zip_writer.start_file("audit.json", options)?;
            zip_writer.write_all(&audit_blob)?;
        }
        zip_writer.finish()?;
        let archive_bytes = fs::read(&archive_path)
            .with_context(|| format!("Failed to read {}", archive_path.display()))?;
        let archive_hash = compute_hash(&archive_bytes);
        let payload = json!({
            "archive_path": archive_path,
            "include_history": include_history,
        });
        let diff_summary = vec![format!(
            "Exported {profile_type:?} profile to {}",
            archive_path.display()
        )];
        let event_id = log_profile_event(
            self.manager,
            &self.base,
            ProfileEventDetails {
                profile_type,
                change_kind: ProfileChangeKind::Export,
                diff_summary,
                hash_before: None,
                hash_after: None,
                undo_token: None,
                payload,
            },
        )?;
        Ok(ProfileExportResult {
            profile_type,
            archive_path,
            hash: archive_hash,
            event_id,
        })
    }

    pub fn delete(
        &self,
        profile_type: ProfileType,
        confirm_phrase: &str,
    ) -> Result<ProfileDeleteResult> {
        let expected = format!("DELETE {}", profile_type.slug());
        if !confirm_phrase.eq_ignore_ascii_case(&expected) {
            bail!("profile delete requires confirm phrase '{expected}'.");
        }
        let (json_path, html_path) = self.profile_paths(profile_type);
        if !json_path.exists() && !html_path.exists() {
            bail!("{profile_type:?} profile artifacts do not exist.");
        }
        let hash_before = self.read_canonical_hash(&json_path)?;
        let mut removed = Vec::new();
        if json_path.exists() {
            fs::remove_file(&json_path)?;
            removed.push(json_path.clone());
        }
        if html_path.exists() {
            fs::remove_file(&html_path)?;
            removed.push(html_path.clone());
        }
        let payload = json!({ "deleted": removed });
        let event_id = log_profile_event(
            self.manager,
            &self.base,
            ProfileEventDetails {
                profile_type,
                change_kind: ProfileChangeKind::Delete,
                diff_summary: vec![format!(
                    "Deleted {} profile artifacts ({} files)",
                    profile_type.slug(),
                    removed.len()
                )],
                hash_before,
                hash_after: None,
                undo_token: None,
                payload,
            },
        )?;
        Ok(ProfileDeleteResult {
            profile_type,
            files_removed: removed,
            event_id,
        })
    }

    pub fn scope_status(&self, profile_type: ProfileType) -> Result<ProfileScopeSetting> {
        let store = ProfileScopeStore::new(&self.base);
        store.get(profile_type)
    }

    pub fn update_scope(
        &self,
        profile_type: ProfileType,
        mode: ProfileScopeMode,
        mut allowed_bases: Vec<String>,
    ) -> Result<ProfileScopeStatus> {
        if mode != ProfileScopeMode::Shared {
            allowed_bases.clear();
        } else {
            normalize_bases(&mut allowed_bases);
        }
        let store = ProfileScopeStore::new(&self.base);
        let setting = store.set(profile_type, mode, allowed_bases.clone())?;
        let payload = json!({
            "scope_mode": mode,
            "allowed_bases": allowed_bases,
        });
        let diff_summary = vec![format!(
            "Scope set to {:?}{}",
            mode,
            if setting.allowed_bases.is_empty() {
                String::new()
            } else {
                format!(" ({})", setting.allowed_bases.join(", "))
            }
        )];
        let event_id = log_profile_event(
            self.manager,
            &self.base,
            ProfileEventDetails {
                profile_type,
                change_kind: ProfileChangeKind::ScopeChange,
                diff_summary,
                hash_before: None,
                hash_after: None,
                undo_token: None,
                payload,
            },
        )?;
        Ok(ProfileScopeStatus {
            setting,
            event_id: Some(event_id),
        })
    }

    fn profile_paths(&self, profile_type: ProfileType) -> (PathBuf, PathBuf) {
        (
            self.layout.profile_json(profile_type.slug()),
            self.layout.profile_html(profile_type.slug()),
        )
    }

    fn ensure_profile_exists(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            bail!(
                "Profile artifact {} not found. Run `profile interview {}` first.",
                path.display(),
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or_default()
            );
        }
        Ok(())
    }

    fn resolve_archive_path(&self, destination: Option<PathBuf>, slug: &str) -> Result<PathBuf> {
        let default_name = format!("{slug}-{}.zip", Utc::now().format("%Y%m%dT%H%M%SZ"));
        match destination {
            Some(path)
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("zip"))
                    .unwrap_or(false) =>
            {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                Ok(path)
            }
            Some(dir) => {
                fs::create_dir_all(&dir)?;
                Ok(dir.join(default_name))
            }
            None => {
                fs::create_dir_all(&self.layout.user_exports_dir)?;
                Ok(self.layout.user_exports_dir.join(default_name))
            }
        }
    }

    fn read_canonical_hash(&self, path: &Path) -> Result<Option<String>> {
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read(path)?;
        let mut value: serde_json::Value = serde_json::from_slice(&data)?;
        if let serde_json::Value::Object(ref mut map) = value {
            map.remove("history");
        }
        let canonical_bytes = serde_json::to_vec(&value)?;
        Ok(Some(compute_hash(&canonical_bytes)))
    }

    fn acquire_export_lock(&self) -> Result<ExportLock> {
        let lock_path = self.layout.user_exports_dir.join(".profile_export.lock");
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => Ok(ExportLock { path: lock_path }),
            Err(_) => bail!("EXPORT_IN_PROGRESS: another export is currently running."),
        }
    }
}

struct ExportLock {
    path: PathBuf,
}

impl Drop for ExportLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn normalize_bases(slugs: &mut Vec<String>) {
    let mut seen = HashSet::new();
    slugs.retain(|slug| seen.insert(slug.to_ascii_lowercase()));
    slugs.sort_unstable_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));
}
