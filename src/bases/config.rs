//! Configuration primitives for ResearchBase Paper Bases.
//!
//! Stored in a machine-readable TOML file located at:
//!   %APPDATA%/ResearchBase/config.toml on Windows
//!   $XDG_CONFIG_HOME/researchbase/config.toml on Linux
//!   ~/Library/Application Support/ResearchBase/config.toml on macOS
//!
//! The config tracks the last active Base and per-install acquisition
//! preferences. This module defines the structs and helper functions the
//! rest of the application will use once the Tauri app is initialized.

use serde::{Deserialize, Serialize};

/// Root configuration persisted per installation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Identifier of the Paper Base that was active when the app last shut down.
    pub last_active_base_id: Option<String>,
    /// Per-install acquisition options (network permissions, batch limits, etc.).
    #[serde(default)]
    pub acquisition: AcquisitionSettings,
    /// Ingestion defaults (checkpoint cadence, remote lookup toggles).
    #[serde(default)]
    pub ingestion: IngestionSettings,
    /// Categorization proposal knobs (cluster limits, worker timeout).
    #[serde(default)]
    pub categorization: CategorizationSettings,
    /// Writing Assistant specific defaults (compiler preferences, etc.).
    #[serde(default)]
    pub writing: WritingSettings,
}

/// Acquisition-related preferences tied to the local install.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionSettings {
    /// Whether remote metadata/PDF lookups are enabled for this install.
    #[serde(default = "default_remote_allowed")]
    pub remote_allowed: bool,
    /// Maximum number of papers allowed in a single approved batch.
    #[serde(default = "default_batch_limit")]
    pub max_batch_size: u32,
}

impl Default for AcquisitionSettings {
    fn default() -> Self {
        Self {
            remote_allowed: default_remote_allowed(),
            max_batch_size: default_batch_limit(),
        }
    }
}

const fn default_remote_allowed() -> bool {
    false
}

const fn default_batch_limit() -> u32 {
    100
}

/// Ingestion-related defaults that affect chat commands and batch runners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionSettings {
    /// Number of files processed between checkpoints/chunks to support resume.
    #[serde(default = "default_checkpoint_interval_files")]
    pub checkpoint_interval_files: u32,
    /// Maximum number of files copied concurrently by the ingestion runner.
    #[serde(default = "default_max_parallel_file_copies")]
    pub max_parallel_file_copies: u32,
    /// Whether remote metadata lookups are allowed during ingestion/enrichment.
    #[serde(default = "default_remote_metadata_allowed")]
    pub remote_metadata_allowed: bool,
}

impl Default for IngestionSettings {
    fn default() -> Self {
        Self {
            checkpoint_interval_files: default_checkpoint_interval_files(),
            max_parallel_file_copies: default_max_parallel_file_copies(),
            remote_metadata_allowed: default_remote_metadata_allowed(),
        }
    }
}

const fn default_checkpoint_interval_files() -> u32 {
    25
}

const fn default_max_parallel_file_copies() -> u32 {
    4
}

const fn default_remote_metadata_allowed() -> bool {
    false
}

/// Categorization proposal tuning parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationSettings {
    /// Maximum number of category proposals returned per run.
    #[serde(default = "default_max_proposals")]
    pub max_proposals: u32,
    /// Wall-clock timeout (ms) for the proposal worker.
    #[serde(default = "default_proposal_timeout_ms")]
    pub timeout_ms: u64,
}

impl Default for CategorizationSettings {
    fn default() -> Self {
        Self {
            max_proposals: default_max_proposals(),
            timeout_ms: default_proposal_timeout_ms(),
        }
    }
}

const fn default_max_proposals() -> u32 {
    5
}

const fn default_proposal_timeout_ms() -> u64 {
    120_000
}

/// Writing Assistant compiler defaults and overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingSettings {
    /// Preferred compiler command (tectonic by default).
    #[serde(default = "default_primary_compiler")]
    pub primary_compiler: CompilerBinary,
    /// Fallback compiler command (pdflatex by default).
    #[serde(default = "default_fallback_compiler")]
    pub fallback_compiler: CompilerBinary,
}

impl Default for WritingSettings {
    fn default() -> Self {
        Self {
            primary_compiler: default_primary_compiler(),
            fallback_compiler: default_fallback_compiler(),
        }
    }
}

/// Represents an invocable compiler + optional args.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerBinary {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl CompilerBinary {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
        }
    }
}

fn default_primary_compiler() -> CompilerBinary {
    CompilerBinary::new("tectonic")
}

fn default_fallback_compiler() -> CompilerBinary {
    CompilerBinary::new("pdflatex")
}

/// Standard relative path to the config file (resolved per OS at runtime).
pub const CONFIG_FILE_NAME: &str = "config.toml";

use anyhow::{Context, Result};
use directories::BaseDirs;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Returns the root directory where ResearchBase stores data.
///
/// Order of precedence:
/// 1. `RESEARCHBASE_HOME` environment variable.
/// 2. OS-specific data directory via `directories::BaseDirs`.
pub fn workspace_root() -> Result<PathBuf> {
    if let Ok(path) = env::var("RESEARCHBASE_HOME") {
        return Ok(PathBuf::from(path));
    }
    let base_dirs = BaseDirs::new().context("Unable to determine OS data directory")?;
    Ok(base_dirs.data_dir().join("ResearchBase"))
}

/// Returns the config directory (same as workspace root for now).
pub fn config_dir() -> Result<PathBuf> {
    let root = workspace_root()?;
    Ok(root.join("config"))
}

/// Path to the config file.
pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE_NAME))
}

/// Loads the configuration from disk or returns defaults.
pub fn load_or_default() -> Result<AppConfig> {
    let path = config_file_path()?;
    if path.exists() {
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file {:?}", path))?;
        let cfg: AppConfig = toml::from_str(&data)
            .with_context(|| format!("Failed to parse config file {:?}", path))?;
        Ok(cfg)
    } else {
        Ok(AppConfig::default())
    }
}

/// Persists the configuration to disk.
pub fn save(config: &AppConfig) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;
    let path = config_file_path()?;
    let data = toml::to_string_pretty(config)?;
    fs::write(&path, data)?;
    Ok(())
}

/// Ensures the workspace structure exists (User/ and AI/ directories).
pub fn ensure_workspace_structure() -> Result<WorkspacePaths> {
    let root = workspace_root()?;
    let user_dir = root.join("User");
    let ai_dir = root.join("AI");
    fs::create_dir_all(&user_dir)?;
    fs::create_dir_all(&ai_dir)?;
    Ok(WorkspacePaths {
        root,
        user_dir,
        ai_dir,
    })
}

/// Convenience struct exposing important workspace paths.
#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub root: PathBuf,
    pub user_dir: PathBuf,
    pub ai_dir: PathBuf,
}

impl WorkspacePaths {
    pub fn base_user_layer(&self, slug: &str) -> PathBuf {
        self.user_dir.join(slug)
    }

    pub fn base_ai_layer(&self, base_id: &str) -> PathBuf {
        self.ai_dir.join(base_id)
    }
}
