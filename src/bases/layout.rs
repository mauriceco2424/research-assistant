//! Shared filesystem layout helpers for profile storage.
//!
//! All profile artifacts must live under the Base-specific AI and User layer
//! directories. Centralizing the sub-directory logic here avoids duplicating
//! string constants across orchestration modules.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use super::Base;

/// Name of the AI-layer subdirectory storing profile JSON artifacts.
pub const AI_PROFILES_SUBDIR: &str = "profiles";
/// Name of the User-layer subdirectory storing profile HTML exports.
pub const USER_PROFILES_SUBDIR: &str = "profiles";
/// Name of the exports directory (nested under the User-layer profiles dir).
pub const PROFILE_EXPORTS_SUBDIR: &str = "exports";
/// Relative path for consent manifest storage inside the AI layer.
pub const CONSENT_MANIFESTS_SUBDIR: &str = "consent/manifests";
/// Relative path storing AI-layer intent payloads and logs.
pub const INTENTS_SUBDIR: &str = "intents";

/// Convenience wrapper for locating all profile-related paths for a Base.
#[derive(Debug, Clone)]
pub struct ProfileLayout {
    pub ai_profiles_dir: PathBuf,
    pub user_profiles_dir: PathBuf,
    pub user_exports_dir: PathBuf,
    pub consent_manifests_dir: PathBuf,
}

impl ProfileLayout {
    /// Constructs a new layout reference for the provided Base.
    pub fn new(base: &Base) -> Self {
        let ai_profiles_dir = base.ai_layer_path.join(AI_PROFILES_SUBDIR);
        let user_profiles_dir = base.user_layer_path.join(USER_PROFILES_SUBDIR);
        let user_exports_dir = user_profiles_dir.join(PROFILE_EXPORTS_SUBDIR);
        let consent_manifests_dir = base.ai_layer_path.join(CONSENT_MANIFESTS_SUBDIR);
        Self {
            ai_profiles_dir,
            user_profiles_dir,
            user_exports_dir,
            consent_manifests_dir,
        }
    }

    /// Path to the JSON artifact for the requested profile type.
    pub fn profile_json(&self, profile_type: &str) -> PathBuf {
        self.ai_profiles_dir.join(format!("{profile_type}.json"))
    }

    /// Path to the HTML summary for the requested profile type.
    pub fn profile_html(&self, profile_type: &str) -> PathBuf {
        self.user_profiles_dir.join(format!("{profile_type}.html"))
    }
}

/// Path builder helper for code that does not need the struct wrapper.
pub fn profile_json_path(base: &Base, profile_type: &str) -> PathBuf {
    ProfileLayout::new(base).profile_json(profile_type)
}

/// Returns the directory storing HTML profile summaries.
pub fn user_profiles_dir(base: &Base) -> PathBuf {
    ProfileLayout::new(base).user_profiles_dir
}

/// Returns the directory storing JSON profile artifacts.
pub fn ai_profiles_dir(base: &Base) -> PathBuf {
    ProfileLayout::new(base).ai_profiles_dir
}

/// Ensures the AI-layer `intents` directory exists and returns the path.
pub fn ensure_intents_dir(base: &Base) -> Result<PathBuf> {
    let dir = base.ai_layer_path.join(INTENTS_SUBDIR);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
