use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bases::{Base, CompilerBinary};

use super::WritingResult;

const PROJECTS_DIR: &str = "WritingProjects";
const MANIFEST_FILE: &str = "project.json";
const MANIFEST_VERSION: &str = "1.0.0";

/// Stable manifest describing a writing project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WritingProjectManifest {
    #[serde(default = "default_manifest_version")]
    pub version: String,
    pub base_id: Uuid,
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owners: Vec<String>,
    pub status: ProjectStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default = "default_manifest_compiler")]
    pub default_compiler: CompilerBinary,
    #[serde(default)]
    pub outline_id: Option<Uuid>,
    #[serde(default)]
    pub active_build_id: Option<Uuid>,
    #[serde(default)]
    pub style_profile_version: Option<String>,
    #[serde(default)]
    pub referenced_paper_ids: Vec<Uuid>,
}

impl WritingProjectManifest {
    /// Creates a new manifest with baseline fields filled in.
    pub fn new(base: &Base, slug: String, title: String) -> Self {
        let timestamp = Utc::now();
        Self {
            version: default_manifest_version(),
            base_id: base.id,
            slug,
            title,
            description: None,
            owners: Vec::new(),
            status: ProjectStatus::Draft,
            created_at: timestamp,
            updated_at: timestamp,
            default_compiler: default_manifest_compiler(),
            outline_id: None,
            active_build_id: None,
            style_profile_version: None,
            referenced_paper_ids: Vec::new(),
        }
    }

    pub fn manifest_path(base: &Base, slug: &str) -> PathBuf {
        ProjectPaths::new(base, slug).manifest_path
    }

    /// Loads an existing manifest from disk.
    pub fn load(base: &Base, slug: &str) -> WritingResult<Self> {
        let paths = ProjectPaths::new(base, slug);
        let data = fs::read_to_string(&paths.manifest_path).with_context(|| {
            format!(
                "Failed to read WritingProject manifest at {}",
                paths.manifest_path.display()
            )
        })?;
        let manifest: Self = serde_json::from_str(&data)
            .with_context(|| format!("Invalid WritingProject manifest for slug {slug}"))?;
        if manifest.slug != slug {
            bail!(
                "Manifest slug mismatch: expected {}, found {}",
                slug,
                manifest.slug
            );
        }
        if manifest.base_id != base.id {
            bail!(
                "Manifest base mismatch: expected {}, found {}",
                base.id,
                manifest.base_id
            );
        }
        Ok(manifest)
    }

    /// Persists the manifest, ensuring the project directory exists.
    pub fn save(&self, base: &Base) -> WritingResult<()> {
        if self.base_id != base.id {
            bail!(
                "Cannot save manifest for base {} into base {}",
                self.base_id,
                base.id
            );
        }
        let paths = ProjectPaths::new(base, &self.slug);
        paths.ensure_dirs()?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&paths.manifest_path, data).with_context(|| {
            format!(
                "Failed to write WritingProject manifest {}",
                paths.manifest_path.display()
            )
        })?;
        Ok(())
    }
}

/// Lifecycle states for a writing project.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ProjectStatus {
    Draft,
    Active,
    Review,
    Archived,
}

impl Default for ProjectStatus {
    fn default() -> Self {
        ProjectStatus::Draft
    }
}

fn default_manifest_version() -> String {
    MANIFEST_VERSION.to_string()
}

fn default_manifest_compiler() -> CompilerBinary {
    CompilerBinary::new("tectonic")
}

/// Convenience wrapper exposing important paths inside a project directory.
#[derive(Debug, Clone)]
pub struct ProjectPaths {
    pub project_root: PathBuf,
    pub manifest_path: PathBuf,
    pub sections_dir: PathBuf,
    pub builds_dir: PathBuf,
    pub bibliography_path: PathBuf,
    pub main_tex_path: PathBuf,
}

impl ProjectPaths {
    pub fn new(base: &Base, slug: &str) -> Self {
        let root = base.user_layer_path.join(PROJECTS_DIR).join(slug);
        Self {
            project_root: root.clone(),
            manifest_path: root.join(MANIFEST_FILE),
            sections_dir: root.join("sections"),
            builds_dir: root.join("builds"),
            bibliography_path: root.join("references.bib"),
            main_tex_path: root.join("main.tex"),
        }
    }

    pub fn ensure_dirs(&self) -> WritingResult<()> {
        fs::create_dir_all(&self.project_root).with_context(|| {
            format!(
                "Failed to create writing project directory {}",
                self.project_root.display()
            )
        })?;
        fs::create_dir_all(&self.sections_dir)?;
        fs::create_dir_all(&self.builds_dir)?;
        Ok(())
    }
}

/// Ensures the `/WritingProjects` tree exists under the provided user-layer root.
pub fn ensure_project_tree<P: AsRef<Path>>(user_layer_root: P) -> WritingResult<()> {
    let projects_root = user_layer_root.as_ref().join(PROJECTS_DIR);
    fs::create_dir_all(&projects_root)?;
    Ok(())
}
