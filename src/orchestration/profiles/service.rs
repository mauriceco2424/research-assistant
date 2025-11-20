use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::bases::{Base, BaseManager, ProfileLayout};
use crate::orchestration::{
    consent::request_profile_interview_consent,
    log_profile_event,
    profiles::{
        defaults::{
            default_knowledge_profile, default_user_profile, default_work_profile,
            default_writing_profile,
        },
        interview::InterviewFlow,
        linking::refresh_evidence_links,
        knowledge::apply_knowledge_mutations,
        model::{
            HistoryRef, KnowledgeProfile, ProfileChangeKind, ProfileMetadata, ProfileType,
            RemoteInferenceStatus, UserProfile, WorkProfile, WritingProfile,
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

#[derive(Debug, Clone)]
pub struct ProfileInterviewOptions {
    pub profile_type: ProfileType,
    pub answers: Vec<ProfileFieldChange>,
    pub requires_remote: bool,
    pub remote_prompt_hint: Option<String>,
    pub approve_remote: bool,
    pub confirm: bool,
}

#[derive(Debug, Clone)]
pub struct ProfileInterviewOutcome {
    pub status: ProfileInterviewStatus,
    pub event_id: Option<Uuid>,
    pub manifest_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileInterviewStatus {
    Completed,
    PendingRemote,
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
                let diff = apply_user_changes(&mut profile, changes)?;
                let hash_before = canonical_snapshot(&profile);
                self.persist_update(
                    profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_user,
                    ProfileChangeKind::ManualEdit,
                    json!({}),
                )
            }
            ProfileType::Work => {
                let mut profile = self.load_work_profile()?;
                let diff = apply_work_changes(&mut profile, changes)?;
                let hash_before = canonical_snapshot(&profile);
                self.persist_update(
                    profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_work,
                    ProfileChangeKind::ManualEdit,
                    json!({}),
                )
            }
            ProfileType::Writing => {
                let mut profile = self.load_writing_profile()?;
                let diff = apply_writing_changes(&mut profile, changes)?;
                let hash_before = canonical_snapshot(&profile);
                self.persist_update(
                    profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_writing,
                    ProfileChangeKind::ManualEdit,
                    json!({}),
                )
            }
            ProfileType::Knowledge => {
                let mut profile = self.load_knowledge_profile()?;
                let diff = apply_knowledge_mutations(&mut profile, changes)?;
                let mut evidence_updates = refresh_evidence_links(&self.base, &mut profile);
                let hash_before = canonical_snapshot(&profile);
                let mut diff = diff;
                diff.append(&mut evidence_updates);
                self.persist_update(
                    profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_knowledge,
                    ProfileChangeKind::ManualEdit,
                    json!({}),
                )
            }
        }
    }

    pub fn interview(&self, options: ProfileInterviewOptions) -> Result<ProfileInterviewOutcome> {
        if !options.confirm {
            bail!("profile interview requires confirm=true");
        }
        self.scopes.enforce_local_read(options.profile_type)?;
        let mut flow = InterviewFlow::new(options.profile_type);
        for answer in options.answers.iter().cloned() {
            flow.record_response(answer);
        }
        let responses = flow.finalize();
        match options.profile_type {
            ProfileType::User => {
                let mut profile = self.load_user_profile()?;
                let mut diff = apply_user_changes(&mut profile, &responses)?;
                if diff.is_empty() {
                    diff.push("Recorded interview without field changes".into());
                }
                let hash_before = canonical_snapshot(&profile);
                let update = self.persist_update(
                    options.profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_user,
                    ProfileChangeKind::Interview,
                    json!({ "interview": { "requires_remote": false } }),
                )?;
                self.record_interview_metric(
                    options.profile_type,
                    ProfileInterviewStatus::Completed,
                    Some(update.event_id),
                )?;
                Ok(ProfileInterviewOutcome {
                    status: ProfileInterviewStatus::Completed,
                    event_id: Some(update.event_id),
                    manifest_id: None,
                })
            }
            ProfileType::Work => {
                let mut profile = self.load_work_profile()?;
                let mut diff = apply_work_changes(&mut profile, &responses)?;
                if diff.is_empty() {
                    diff.push("Recorded work interview without field changes".into());
                }
                let hash_before = canonical_snapshot(&profile);
                let update = self.persist_update(
                    options.profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_work,
                    ProfileChangeKind::Interview,
                    json!({ "interview": { "requires_remote": false } }),
                )?;
                self.record_interview_metric(
                    options.profile_type,
                    ProfileInterviewStatus::Completed,
                    Some(update.event_id),
                )?;
                Ok(ProfileInterviewOutcome {
                    status: ProfileInterviewStatus::Completed,
                    event_id: Some(update.event_id),
                    manifest_id: None,
                })
            }
            ProfileType::Writing => self.handle_writing_interview(options, &responses),
            ProfileType::Knowledge => {
                let mut profile = self.load_knowledge_profile()?;
                let mut diff = apply_knowledge_mutations(&mut profile, &responses)?;
                if diff.is_empty() {
                    diff.push("Recorded knowledge interview without field changes".into());
                }
                let mut evidence_updates = refresh_evidence_links(&self.base, &mut profile);
                diff.append(&mut evidence_updates);
                let hash_before = canonical_snapshot(&profile);
                let update = self.persist_update(
                    options.profile_type,
                    &mut profile,
                    diff,
                    hash_before,
                    summarize_knowledge,
                    ProfileChangeKind::Interview,
                    json!({ "interview": { "requires_remote": false } }),
                )?;
                self.record_interview_metric(
                    options.profile_type,
                    ProfileInterviewStatus::Completed,
                    Some(update.event_id),
                )?;
                Ok(ProfileInterviewOutcome {
                    status: ProfileInterviewStatus::Completed,
                    event_id: Some(update.event_id),
                    manifest_id: None,
                })
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
        change_kind: ProfileChangeKind,
        payload: serde_json::Value,
    ) -> Result<ProfileUpdateOutput>
    where
        P: Serialize + Clone + HasProfileMetadata + HasHistory,
    {
        if diff_summary.is_empty() {
            bail!("No changes detected for {profile_type:?} profile.");
        }
        profile.metadata_mut().last_updated = Utc::now();
        let summary = summarizer(profile);
        let json_path = self.layout.profile_json(slug(profile_type));
        let write_outcome = write_profile(&json_path, profile)?;
        let html = build_profile_html(
            profile_type,
            profile.metadata(),
            &summary.highlights,
            &summary.fields,
        );
        let html_path = write_profile_html(self.layout.profile_html(slug(profile_type)), &html)?;
        let event_payload = wrap_payload_with_path(&json_path, payload);
        let event_id = log_profile_event(
            self.manager,
            &self.base,
            ProfileEventDetails {
                profile_type,
                change_kind,
                diff_summary: diff_summary.clone(),
                hash_before,
                hash_after: Some(write_outcome.hash.clone()),
                undo_token: None,
                payload: event_payload,
            },
        )?;
        profile.history_mut().push(HistoryRef {
            event_id,
            timestamp: Utc::now(),
            hash_after: write_outcome.hash.clone(),
        });
        let _ = write_profile(&json_path, profile)?;
        Ok(ProfileUpdateOutput {
            profile_type,
            event_id,
            diff_summary,
            json_path,
            html_path,
            hash_after: write_outcome.hash,
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

    fn handle_writing_interview(
        &self,
        options: ProfileInterviewOptions,
        responses: &[ProfileFieldChange],
    ) -> Result<ProfileInterviewOutcome> {
        let mut profile = self.load_writing_profile()?;
        let mut diff = apply_writing_changes(&mut profile, responses)?;
        let mut manifest_id = None;
        let mut status = ProfileInterviewStatus::Completed;
        if options.requires_remote {
            if options.approve_remote {
                let manifest = request_profile_interview_consent(
                    self.manager,
                    &self.base,
                    ProfileType::Writing,
                    options.remote_prompt_hint.as_deref(),
                )?;
                profile.fields.remote_inference_metadata.status = RemoteInferenceStatus::Approved;
                profile.fields.remote_inference_metadata.consent_manifest_id =
                    Some(manifest.manifest_id);
                profile.fields.remote_inference_metadata.last_remote_source =
                    options.remote_prompt_hint.clone();
                manifest_id = Some(manifest.manifest_id);
                diff.push("Captured remote interview results (approved)".into());
            } else {
                profile.fields.remote_inference_metadata.status = RemoteInferenceStatus::Pending;
                profile.fields.remote_inference_metadata.consent_manifest_id = None;
                profile.fields.remote_inference_metadata.last_remote_source =
                    Some("needs_remote_approval".into());
                diff.push("Flagged remote interview for approval".into());
                status = ProfileInterviewStatus::PendingRemote;
            }
        }
        if diff.is_empty() {
            diff.push("Recorded writing interview without field changes".into());
        }
        let hash_before = canonical_snapshot(&profile);
        let payload = json!({
            "interview": {
                "requires_remote": options.requires_remote,
                "remote_prompt_hint": options.remote_prompt_hint,
                "manifest_id": manifest_id,
                "status": format!("{status:?}").to_ascii_lowercase(),
            }
        });
        let update = self.persist_update(
            ProfileType::Writing,
            &mut profile,
            diff,
            hash_before,
            summarize_writing,
            ProfileChangeKind::Interview,
            payload,
        )?;
        self.record_interview_metric(ProfileType::Writing, status, Some(update.event_id))?;
        Ok(ProfileInterviewOutcome {
            status,
            event_id: Some(update.event_id),
            manifest_id,
        })
    }

    fn record_interview_metric(
        &self,
        profile_type: ProfileType,
        status: ProfileInterviewStatus,
        event_id: Option<Uuid>,
    ) -> Result<()> {
        let metrics_dir = self.base.ai_layer_path.join("metrics");
        fs::create_dir_all(&metrics_dir)?;
        let path = metrics_dir.join("profile_interviews.jsonl");
        let record = InterviewMetricRecord {
            timestamp: Utc::now(),
            profile_type,
            status,
            event_id,
        };
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        file.write_all(serde_json::to_string(&record)?.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }
}

fn wrap_payload_with_path(path: &PathBuf, payload: Value) -> Value {
    if payload.is_null() {
        json!({ "path": path })
    } else {
        json!({
            "path": path,
            "details": payload
        })
    }
}

#[derive(Serialize)]
struct InterviewMetricRecord {
    timestamp: DateTime<Utc>,
    profile_type: ProfileType,
    status: ProfileInterviewStatus,
    event_id: Option<Uuid>,
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
