use crate::bases::{Base, ProfileLayout};
use crate::orchestration::profiles::model::{ProfileScopeMode, ProfileScopeSetting, ProfileType};
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Persistent store for profile scope settings per Base.
pub struct ProfileScopeStore<'a> {
    base: &'a Base,
    path: PathBuf,
}

impl<'a> ProfileScopeStore<'a> {
    pub fn new(base: &'a Base) -> Self {
        let layout = ProfileLayout::new(base);
        let path = layout.ai_profiles_dir.join("scope.json");
        Self { base, path }
    }

    pub fn load(&self) -> Result<Vec<ProfileScopeSetting>> {
        if !self.path.exists() {
            return Ok(default_settings());
        }
        let data = fs::read(&self.path)?;
        let settings: Vec<ProfileScopeSetting> = serde_json::from_slice(&data)?;
        Ok(settings)
    }

    pub fn get(&self, profile_type: ProfileType) -> Result<ProfileScopeSetting> {
        let mut map: HashMap<ProfileType, ProfileScopeSetting> = self
            .load()?
            .into_iter()
            .map(|setting| (setting.profile_type, setting))
            .collect();
        Ok(map
            .remove(&profile_type)
            .unwrap_or_else(|| default_setting(profile_type)))
    }

    pub fn set(
        &self,
        profile_type: ProfileType,
        scope_mode: ProfileScopeMode,
        allowed_bases: Vec<String>,
    ) -> Result<ProfileScopeSetting> {
        let mut settings = self.load()?;
        let mut found = false;
        for setting in &mut settings {
            if setting.profile_type == profile_type {
                setting.scope_mode = scope_mode;
                setting.allowed_bases = allowed_bases.clone();
                setting.updated_at = Utc::now();
                found = true;
                break;
            }
        }
        if !found {
            let mut setting = default_setting(profile_type);
            setting.scope_mode = scope_mode;
            setting.allowed_bases = allowed_bases;
            setting.updated_at = Utc::now();
            settings.push(setting);
        }
        fs::create_dir_all(
            self.path
                .parent()
                .context("missing parent directory for scope store")?,
        )?;
        fs::write(&self.path, serde_json::to_vec_pretty(&settings)?)?;
        self.get(profile_type)
    }

    pub fn enforce_local_read(&self, profile_type: ProfileType) -> Result<()> {
        let setting = self.get(profile_type)?;
        if setting.scope_mode == ProfileScopeMode::Disabled {
            anyhow::bail!(
                "profile {profile_type:?} is disabled for this Base via `profile scope`."
            );
        }
        Ok(())
    }

    pub fn enforce_share_target(
        &self,
        profile_type: ProfileType,
        target_base_slug: &str,
    ) -> Result<()> {
        let setting = self.get(profile_type)?;
        if !scope_allows_target(&setting, &self.base.slug, target_base_slug) {
            anyhow::bail!("profile {profile_type:?} is not shared with Base '{target_base_slug}'.");
        }
        Ok(())
    }
}

fn default_settings() -> Vec<ProfileScopeSetting> {
    [
        ProfileType::User,
        ProfileType::Work,
        ProfileType::Writing,
        ProfileType::Knowledge,
    ]
    .into_iter()
    .map(default_setting)
    .collect()
}

fn default_setting(profile_type: ProfileType) -> ProfileScopeSetting {
    ProfileScopeSetting {
        profile_type,
        scope_mode: ProfileScopeMode::ThisBase,
        allowed_bases: Vec::new(),
        updated_at: Utc::now(),
    }
}

pub fn scope_allows_target(
    setting: &ProfileScopeSetting,
    owner_slug: &str,
    target_base_slug: &str,
) -> bool {
    match setting.scope_mode {
        ProfileScopeMode::Disabled => false,
        ProfileScopeMode::ThisBase => owner_slug.eq_ignore_ascii_case(target_base_slug),
        ProfileScopeMode::Shared => setting
            .allowed_bases
            .iter()
            .any(|slug| slug.eq_ignore_ascii_case(target_base_slug)),
    }
}
