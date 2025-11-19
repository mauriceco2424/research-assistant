use crate::bases::{Base, ProfileLayout};
use crate::orchestration::profiles::defaults::{
    default_knowledge_profile, default_user_profile, default_work_profile, default_writing_profile,
};
use crate::orchestration::profiles::storage::write_profile;
use anyhow::Result;
use std::fs;

pub fn ensure_profile_shells(base: &Base) -> Result<()> {
    let layout = ProfileLayout::new(base);
    fs::create_dir_all(&layout.ai_profiles_dir)?;
    fs::create_dir_all(&layout.user_profiles_dir)?;
    seed_profile(&layout, "user", default_user_profile)?;
    seed_profile(&layout, "work", default_work_profile)?;
    seed_profile(&layout, "writing", default_writing_profile)?;
    seed_profile(&layout, "knowledge", default_knowledge_profile)?;
    Ok(())
}

fn seed_profile<F, P>(layout: &ProfileLayout, profile_type: &str, factory: F) -> Result<()>
where
    F: Fn() -> P,
    P: serde::Serialize,
{
    let path = layout.profile_json(profile_type);
    if path.exists() {
        return Ok(());
    }
    let profile = factory();
    let _ = write_profile(path, &profile)?;
    Ok(())
}
