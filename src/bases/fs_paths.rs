//! Filesystem helpers for AI-layer category storage.
//!
//! Categories live under each Base's AI directory so edits remain local-first
//! and regenerable. This module centralizes the directory layout so callers
//! can consistently locate category definitions, assignments, snapshots, and
//! proposal previews.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use super::Base;

/// Paths to all category storage locations for a single Base.
#[derive(Debug, Clone)]
pub struct CategoryPaths {
    pub root: PathBuf,
    pub definitions_dir: PathBuf,
    pub assignments_dir: PathBuf,
    pub snapshots_dir: PathBuf,
    pub proposals_dir: PathBuf,
}

impl CategoryPaths {
    pub fn new(root: PathBuf) -> Self {
        Self {
            definitions_dir: root.join("definitions"),
            assignments_dir: root.join("assignments"),
            snapshots_dir: root.join("snapshots"),
            proposals_dir: root.join("proposals"),
            root,
        }
    }
}

/// Ensures the AI-layer category directories exist and returns their paths.
pub fn ensure_category_dirs(base: &Base) -> Result<CategoryPaths> {
    let root = base.ai_layer_path.join("categories");
    let paths = CategoryPaths::new(root);
    for dir in [
        &paths.root,
        &paths.definitions_dir,
        &paths.assignments_dir,
        &paths.snapshots_dir,
        &paths.proposals_dir,
    ] {
        fs::create_dir_all(dir)?;
    }
    Ok(paths)
}
