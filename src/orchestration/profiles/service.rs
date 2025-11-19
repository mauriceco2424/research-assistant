use std::path::PathBuf;

use anyhow::{bail, Result};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::bases::{Base, BaseManager, ProfileLayout};
use crate::orchestration::{
    log_profile_event,
    profiles::{
        defaults::{
            default_knowledge_profile, default_user_profile, default_work_profile,
            default_writing_profile,
        },
        model::{
            HistoryRef, KnowledgeProfile, ProfileMetadata, ProfileType, RemoteInferenceStatus,
            UserProfile, WorkProfile, WritingProfile,
        },
        render::build_profile_html,
        scope::ProfileScopeStore,
        storage::{read_profile, write_profile, write_profile_html},
        summarize::{summarize_knowledge, summarize_user, summarize_work, summarize_writing},
    },
    ProfileEventDetails,
};

use super::summarize::ProfileSummary;

#[derive(Debug, Clone)]
pub struct ProfileFieldChange {
    pub field: String,
    pub value: String,
}

impl ProfileFieldChange {
    pub fn new(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileShowOutput {
    pub profile_type: ProfileType,
    pub metadata: ProfileMetadata,
    pub summary: ProfileSummary,
    pub history_preview: Option<HistoryRef>,
    pub json_path: PathBuf,
    pub html_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProfileUpdateOutput {
    pub profile_type: ProfileType,
    pub event_id: Uuid,
    pub diff_summary: Vec<String>,
    pub json_path: PathBuf,
    pub html_path: PathBuf,
    pub hash_after: String,
}

pub struct ProfileService<'a> {
    manager: &'a BaseManager,
    base: Base,
    layout: ProfileLayout,
    scopes: ProfileScopeStore<'a>,
}

impl<'a> ProfileService<'a> {
    pub fn new(manager: &'a BaseManager, base: &'a Base) -> Self {
        Self {
            manager,
            base: base.clone(),
            layout: ProfileLayout::new(base),
            scopes: ProfileScopeStore::new(base),
        }
    }

    pub fn parse_type(&self, raw: &str) -> Result<ProfileType> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "user" => Ok(ProfileType::User),
            "work" => Ok(ProfileType::Work),
            "writing" => Ok(ProfileType::Writing),
            "knowledge" => Ok(ProfileType::Knowledge),
            other => bail!("Unknown profile type '{other}'. Expected user/work/writing/knowledge."),
        }
    }

    pub fn show(&self, profile_type: ProfileType) -> Result<ProfileShowOutput> {
        self.scopes.enforce_local_read(profile_type)?;
        match profile_type {
            ProfileType::User => {
                let profile = self.load_user_profile()?;
                self.build_show_output(profile_type, profile, summarize_user)
            }
            ProfileType::Work => {
                let profile = self.load_work_profile()?;
                self.build_show_output(profile_type, profile, summarize_work)
            }
            ProfileType::Writing => {
                let profile = self.load_writing_profile()?;
                self.build_show_output(profile_type, profile, summarize_writing)
            }
            ProfileType::Knowledge => {
                let profile = self.load_knowledge_profile()?;
                self.build_show_output(profile_type, profile, summarize_knowledge)
            }
        }
    }

    fn build_show_output<P>(
        &self,
        profile_type: ProfileType,
        profile: P,
        summarizer: fn(&P) -> ProfileSummary,
    ) -> Result<ProfileShowOutput>
    where
        P: Serialize + Clone + HasProfileMetadata + HasHistory,
    {
        let summary = summarizer(&profile);
        let html = build_profile_html(
            profile_type,
            profile.metadata(),
            &summary.highlights,
            &summary.fields,
        );
        let html_path = write_profile_html(self.layout.profile_html(slug(profile_type)), &html)?;
        Ok(ProfileShowOutput {
            profile_type,
            metadata: profile.metadata().clone(),
            summary,
            history_preview: profile.history().last().cloned(),
            json_path: self.layout.profile_json(slug(profile_type)),
            html_path,
        })
    }

    pub fn update(
        &self,
        profile_type: ProfileType,
        changes: &[ProfileFieldChange],
        confirm: bool,
    ) -> Result<ProfileUpdateOutput> {
        if changes.is_empty() {
            bail!("Provide at least one field change (key=value) to update a profile.");
        }
        if !confirm {
            bail!(
                "Profile updates require confirmation. Pass confirm=true after reviewing changes."
            );
        }
        self.scopes.enforce_local_read(profile_type)?;
        match profile_type {
            ProfileType::User => {
                let mut profile = self.load_user_profile()?;
                let hash_before = canonical_snapshot(&profile);
                let diff = apply_user_changes(&mut profile, changes)?;
                self.persist_update(profile_type, &mut profile, diff, hash_before, summarize_user)
            }
            ProfileType::Work => {
                let mut profile = self.load_work_profile()?;
                let hash_before = canonical_snapshot(&profile);
                let diff = apply_work_changes(&mut profile, changes)?;
                self.persist_update(profile_type, &mut profile, diff, hash_before, summarize_work)
            }
            ProfileType::Writing => {
                let mut profile = self.load_writing_profile()?;
                let hash_before = canonical_snapshot(&profile);
                let diff = apply_writing_changes(&mut profile, changes)?;
                self.persist_update(
                    profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_writing,
                )
            }
            ProfileType::Knowledge => {
                let mut profile = self.load_knowledge_profile()?;
                let hash_before = canonical_snapshot(&profile);
                let diff = apply_knowledge_changes(&mut profile, changes)?;
                self.persist_update(
                    profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_knowledge,
                )
            }
        }
    }

    fn persist_update<P>(
        &self,
        profile_type: ProfileType,
        profile: &mut P,
        diff_summary: Vec<String>,
        hash_before: Option<String>,
        summarizer: fn(&P) -> ProfileSummary,
    ) -> Result<ProfileUpdateOutput>
    where
        P: Serialize + Clone + HasProfileMetadata + HasHistory,
    {
        if diff_summary.is_empty() {
            bail!("No changes detected for {profile_type:?} profile.");
        }
        profile.metadata_mut().last_updated = Utc::now();
        let summary = summarizer(profile);
        let hash_after = canonical_snapshot(profile);
        let json_path = self.layout.profile_json(slug(profile_type));
        let _ = write_profile(&json_path, profile)?;
        let html = build_profile_html(
            profile_type,
            profile.metadata(),
            &summary.highlights,
            &summary.fields,
        );
        let html_path = write_profile_html(self.layout.profile_html(slug(profile_type)), &html)?;
        let event_id = log_profile_event(
            self.manager,
            &self.base,
            ProfileEventDetails {
                profile_type,
                change_kind: super::model::ProfileChangeKind::ManualEdit,
                diff_summary: diff_summary.clone(),
                hash_before,
                hash_after: hash_after.clone(),
                undo_token: None,
                payload: json!({
                    "path": json_path,
                }),
            },
        )?;
        profile.history_mut().push(HistoryRef {
            event_id,
            timestamp: Utc::now(),
            hash_after: hash_after.clone().unwrap_or_default(),
        });
        let _ = write_profile(&json_path, profile)?;
        Ok(ProfileUpdateOutput {
            profile_type,
            event_id,
            diff_summary,
            json_path,
            html_path,
            hash_after: hash_after.unwrap_or_default(),
        })
    }

    pub fn load_user_profile(&self) -> Result<UserProfile> {
        self.load_or_seed("user", default_user_profile)
    }

    pub fn load_work_profile(&self) -> Result<WorkProfile> {
        self.load_or_seed("work", default_work_profile)
    }

    pub fn load_writing_profile(&self) -> Result<WritingProfile> {
        self.load_or_seed("writing", default_writing_profile)
    }

    pub fn load_knowledge_profile(&self) -> Result<KnowledgeProfile> {
        self.load_or_seed("knowledge", default_knowledge_profile)
    }

    fn load_or_seed<P, F>(&self, slug: &str, factory: F) -> Result<P>
    where
        P: Clone + Serialize + for<'de> serde::Deserialize<'de>,
        F: Fn() -> P,
    {
        let path = self.layout.profile_json(slug);
        if let Some(profile) = read_profile(&path)? {
            return Ok(profile);
        }
        let profile = factory();
        let _ = write_profile(&path, &profile)?;
        Ok(profile)
    }
}

fn apply_user_changes(profile: &mut UserProfile, changes: &[ProfileFieldChange]) -> Result<Vec<String>> {
    let mut diff = Vec::new();
    for change in changes {
        match change.field.to_ascii_lowercase().as_str() {
            "name" => {
                profile.fields.name = change.value.clone();
                diff.push("Updated name".into());
            }
            "affiliations" => {
                profile.fields.affiliations = split_list(&change.value);
                diff.push("Updated affiliations".into());
            }
            "communication_style" => {
                profile.fields.communication_style = split_list(&change.value);
                diff.push("Updated communication style".into());
            }
            "availability" => {
                profile.fields.availability = Some(change.value.clone());
                diff.push("Updated availability".into());
            }
            other => bail!("Unsupported user profile field '{other}'"),
        }
    }
    Ok(diff)
}

fn apply_work_changes(profile: &mut WorkProfile, changes: &[ProfileFieldChange]) -> Result<Vec<String>> {
    let mut diff = Vec::new();
    for change in changes {
        match change.field.to_ascii_lowercase().as_str() {
            "focus_statement" | "focus" => {
                profile.fields.focus_statement = Some(change.value.clone());
                diff.push("Updated focus statement".into());
            }
            "preferred_tools" => {
                profile.fields.preferred_tools = split_list(&change.value);
                diff.push("Updated preferred tools".into());
            }
            "risks" => {
                profile.fields.risks = split_list(&change.value);
                diff.push("Updated risk notes".into());
            }
            other => bail!("Unsupported work profile field '{other}'"),
        }
    }
    Ok(diff)
}

fn apply_writing_changes(
    profile: &mut WritingProfile,
    changes: &[ProfileFieldChange],
) -> Result<Vec<String>> {
    let mut diff = Vec::new();
    for change in changes {
        match change.field.to_ascii_lowercase().as_str() {
            "tone_descriptors" => {
                profile.fields.tone_descriptors = split_list(&change.value);
                diff.push("Updated tone descriptors".into());
            }
            "structure_preferences" => {
                profile.fields.structure_preferences = split_list(&change.value);
                diff.push("Updated structure preferences".into());
            }
            "remote_status" => {
                profile.fields.remote_inference_metadata.status =
                    match change.value.to_ascii_lowercase().as_str() {
                        "approved" => RemoteInferenceStatus::Approved,
                        "rejected" => RemoteInferenceStatus::Rejected,
                        _ => RemoteInferenceStatus::Pending,
                    };
                diff.push("Updated remote inference status".into());
            }
            other => bail!("Unsupported writing profile field '{other}'"),
        }
    }
    Ok(diff)
}

fn apply_knowledge_changes(
    profile: &mut KnowledgeProfile,
    changes: &[ProfileFieldChange],
) -> Result<Vec<String>> {
    let mut diff = Vec::new();
    for change in changes {
        match change.field.to_ascii_lowercase().as_str() {
            "summary" => {
                profile.summary = split_list(&change.value);
                diff.push("Updated summary".into());
            }
            other => bail!("Unsupported knowledge profile field '{other}'"),
        }
    }
    Ok(diff)
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn slug(profile_type: ProfileType) -> &'static str {
    match profile_type {
        ProfileType::User => "user",
        ProfileType::Work => "work",
        ProfileType::Writing => "writing",
        ProfileType::Knowledge => "knowledge",
    }
}

fn canonical_snapshot<P>(profile: &P) -> Option<String>
where
    P: Serialize + Clone + HasHistory,
{
    let mut clone = profile.clone();
    clone.history_mut().clear();
    serde_json::to_vec(&clone)
        .ok()
        .map(|bytes| crate::orchestration::profiles::storage::compute_hash(&bytes))
}

trait HasProfileMetadata {
    fn metadata(&self) -> &ProfileMetadata;
    fn metadata_mut(&mut self) -> &mut ProfileMetadata;
}

trait HasHistory {
    fn history(&self) -> &Vec<HistoryRef>;
    fn history_mut(&mut self) -> &mut Vec<HistoryRef>;
}

impl HasProfileMetadata for UserProfile {
    fn metadata(&self) -> &ProfileMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ProfileMetadata {
        &mut self.metadata
    }
}

impl HasHistory for UserProfile {
    fn history(&self) -> &Vec<HistoryRef> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<HistoryRef> {
        &mut self.history
    }
}

impl HasProfileMetadata for WorkProfile {
    fn metadata(&self) -> &ProfileMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ProfileMetadata {
        &mut self.metadata
    }
}

impl HasHistory for WorkProfile {
    fn history(&self) -> &Vec<HistoryRef> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<HistoryRef> {
        &mut self.history
    }
}

impl HasProfileMetadata for WritingProfile {
    fn metadata(&self) -> &ProfileMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ProfileMetadata {
        &mut self.metadata
    }
}

impl HasHistory for WritingProfile {
    fn history(&self) -> &Vec<HistoryRef> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<HistoryRef> {
        &mut self.history
    }
}

impl HasProfileMetadata for KnowledgeProfile {
    fn metadata(&self) -> &ProfileMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ProfileMetadata {
        &mut self.metadata
    }
}

impl HasHistory for KnowledgeProfile {
    fn history(&self) -> &Vec<HistoryRef> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<HistoryRef> {
        &mut self.history
    }
}
