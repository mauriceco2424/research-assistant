//! Chat command bridge for AI profile operations.
//!
//! Converts chat-friendly requests into service calls and user-facing summaries.

use std::{fmt::Write as _, path::PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use crate::{
    bases::{Base, BaseManager},
    orchestration::profiles::{
        governance::{ProfileAuditLog, ProfileDeleteResult, ProfileExportResult, ProfileGovernance},
        model::{ProfileScopeMode, ProfileScopeSetting},
        regenerate::{ProfileRegenerateOutcome, ProfileRegenerator},
        service::{
            ProfileFieldChange, ProfileInterviewOptions, ProfileInterviewOutcome,
            ProfileInterviewStatus, ProfileService, ProfileShowOutput, ProfileUpdateOutput,
        },
    },
};

pub struct ProfileCommandBridge<'a> {
    manager: &'a BaseManager,
}

impl<'a> ProfileCommandBridge<'a> {
    pub fn new(manager: &'a BaseManager) -> Self {
        Self { manager }
    }

    pub fn show(&self, base: &Base, request: ProfileShowRequest) -> Result<String> {
        let service = ProfileService::new(self.manager, base);
        let profile_type = service.parse_type(&request.profile_type)?;
        let output = service.show(profile_type)?;
        Ok(format_show_response(&output, request.include_history))
    }

    pub fn update(&self, base: &Base, request: ProfileUpdateRequest) -> Result<String> {
        let service = ProfileService::new(self.manager, base);
        let profile_type = service.parse_type(&request.profile_type)?;
        let changes = parse_field_changes(&request.field_changes)?;
        let output = service.update(profile_type, &changes, request.confirm)?;
        Ok(format_update_response(&output))
    }

    pub fn interview(&self, base: &Base, request: ProfileInterviewRequest) -> Result<String> {
        if !request.confirm {
            bail!("profile interview requires --confirm to proceed.");
        }
        let service = ProfileService::new(self.manager, base);
        let profile_type = service.parse_type(&request.profile_type)?;
        let answers = parse_field_changes(&request.answers)?;
        let options = ProfileInterviewOptions {
            profile_type,
            answers,
            requires_remote: request.requires_remote,
            remote_prompt_hint: request.remote_prompt_hint.clone(),
            approve_remote: request.approve_remote,
            confirm: request.confirm,
        };
        let outcome = service.interview(options)?;
        Ok(format_interview_response(&outcome))
    }

    pub fn run(&self, base: &Base, request: ProfileRunRequest) -> Result<String> {
        if !request.run_kind.eq_ignore_ascii_case("writing-style") {
            bail!("Unsupported profile run '{}'. Only 'writing-style' is available.", request.run_kind);
        }
        let mut interview_request = ProfileInterviewRequest::default();
        interview_request.profile_type = if request.profile_type.is_empty() {
            "writing".into()
        } else {
            request.profile_type.clone()
        };
        interview_request.requires_remote = request.requires_remote.unwrap_or(true);
        interview_request.remote_prompt_hint = request.remote_prompt_hint.clone();
        interview_request.answers = request.answers.clone();
        interview_request.confirm = request.confirm;
        interview_request.approve_remote = request.approve_remote.unwrap_or(true);
        self.interview(base, interview_request)
    }

    pub fn audit(&self, base: &Base, request: ProfileAuditRequest) -> Result<String> {
        let service = ProfileService::new(self.manager, base);
        let profile_type = service.parse_type(&request.profile_type)?;
        let governance = ProfileGovernance::new(self.manager, base);
        let log = governance.audit(profile_type)?;
        Ok(format_audit_response(&log, request.include_undo_instructions))
    }

    pub fn export(&self, base: &Base, request: ProfileExportRequest) -> Result<String> {
        let service = ProfileService::new(self.manager, base);
        let profile_type = service.parse_type(&request.profile_type)?;
        let governance = ProfileGovernance::new(self.manager, base);
        let result = governance.export(
            profile_type,
            request.destination.clone(),
            request.include_history,
        )?;
        Ok(format_export_response(&result, request.include_history))
    }

    pub fn delete(&self, base: &Base, request: ProfileDeleteRequest) -> Result<String> {
        let service = ProfileService::new(self.manager, base);
        let profile_type = service.parse_type(&request.profile_type)?;
        let confirm = request
            .confirm_phrase
            .as_deref()
            .context("profile delete requires --confirm DELETE <type>.")?;
        let governance = ProfileGovernance::new(self.manager, base);
        let result = governance.delete(profile_type, confirm)?;
        Ok(format_delete_response(&result))
    }

    pub fn regenerate(&self, base: &Base, request: ProfileRegenerateRequest) -> Result<String> {
        let service = ProfileService::new(self.manager, base);
        let profile_type = service.parse_type(&request.profile_type)?;
        let regenerator = ProfileRegenerator::new(self.manager, base);
        let outcome = match &request.source {
            ProfileRegenerateSource::History => regenerator.from_history(profile_type)?,
            ProfileRegenerateSource::Archive(path) => {
                regenerator.from_archive(profile_type, path.as_path())?
            }
        };
        Ok(format_regenerate_response(&outcome, &request.source))
    }

    pub fn scope(&self, base: &Base, request: ProfileScopeRequest) -> Result<String> {
        let service = ProfileService::new(self.manager, base);
        let profile_type = service.parse_type(&request.profile_type)?;
        let governance = ProfileGovernance::new(self.manager, base);
        if let Some(mode_raw) = &request.scope_mode {
            let mode = parse_scope_mode(mode_raw)?;
            let status =
                governance.update_scope(profile_type, mode, request.allowed_bases.clone())?;
            Ok(format_scope_response(&status.setting, status.event_id))
        } else {
            let setting = governance.scope_status(profile_type)?;
            Ok(format_scope_response(&setting, None))
        }
    }
}

fn format_show_response(output: &ProfileShowOutput, include_history: bool) -> String {
    let mut response = String::new();
    response.push_str(&format!(
        "Profile: {:?}\nLast updated: {}\nScope: {:?}\n",
        output.profile_type,
        output.metadata.last_updated.to_rfc3339(),
        output.metadata.scope
    ));
    if !output.summary.highlights.is_empty() {
        response.push_str("\nSummary:\n");
        for line in &output.summary.highlights {
            response.push_str(&format!("- {line}\n"));
        }
    }
    if !output.summary.fields.is_empty() {
        response.push_str("\nDetails:\n");
        for (label, value) in &output.summary.fields {
            response.push_str(&format!("• {label}: {value}\n"));
        }
    }
    if include_history {
        if let Some(history) = &output.history_preview {
            response.push_str(&format!(
                "\nLatest event: {} @ {}",
                history.event_id,
                history.timestamp.to_rfc3339()
            ));
        } else {
            response.push_str("\nNo profile history recorded yet.");
        }
    } else {
        response.push_str("\nHistory suppressed (set include_history=true to show more).");
    }
    response.push_str(&format!(
        "\nJSON: {}\nHTML: {}",
        output.json_path.display(),
        output.html_path.display()
    ));
    response
}

fn format_update_response(output: &ProfileUpdateOutput) -> String {
    let mut response = String::new();
    response.push_str(&format!(
        "profile update {:?} recorded event {}.\n",
        output.profile_type, output.event_id
    ));
    if output.diff_summary.is_empty() {
        response.push_str("No diff summary available.\n");
    } else {
        response.push_str("Changes:\n");
        for entry in &output.diff_summary {
            response.push_str(&format!("- {entry}\n"));
        }
    }
    response.push_str(&format!(
        "Artifact hash: {}\nJSON: {}\nHTML: {}",
        output.hash_after,
        output.json_path.display(),
        output.html_path.display()
    ));
    response
}

fn format_interview_response(outcome: &ProfileInterviewOutcome) -> String {
    let mut response = String::new();
    match outcome.status {
        ProfileInterviewStatus::Completed => {
            response.push_str("profile interview completed.\n");
        }
        ProfileInterviewStatus::PendingRemote => {
            response.push_str("profile interview recorded but needs remote approval.\n");
        }
    }
    if let Some(event_id) = outcome.event_id {
        response.push_str(&format!("Event ID: {event_id}\n"));
    }
    if let Some(manifest_id) = outcome.manifest_id {
        response.push_str(&format!("Consent manifest: {manifest_id}\n"));
    }
    response
}

fn parse_field_changes(raw: &[String]) -> Result<Vec<ProfileFieldChange>> {
    let mut changes = Vec::new();
    for entry in raw {
        let mut parts = entry.splitn(2, '=');
        let field = parts
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("Invalid field change '{entry}'. Use key=value syntax."))?;
        let value = parts
            .next()
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .ok_or_else(|| anyhow!("Invalid field change '{entry}'. Use key=value syntax."))?;
        changes.push(ProfileFieldChange::new(field, value));
    }
    if changes.is_empty() {
        bail!("No field changes provided. Use key=value syntax.");
    }
    Ok(changes)
}

fn format_audit_response(log: &ProfileAuditLog, include_undo: bool) -> String {
    let mut response = String::new();
    let _ = writeln!(
        &mut response,
        "Profile audit for {:?} ({} entries)",
        log.profile_type,
        log.entries.len()
    );
    let _ = writeln!(
        &mut response,
        "Generated at {}",
        log.generated_at.to_rfc3339()
    );
    if log.entries.is_empty() {
        response.push_str("No profile events recorded yet.");
        return response;
    }
    for entry in &log.entries {
        let _ = writeln!(
            &mut response,
            "- {} {:?} (event {})",
            entry.timestamp.to_rfc3339(),
            entry.change_kind,
            entry.event_id
        );
        for diff in &entry.diff_summary {
            let _ = writeln!(&mut response, "  * {diff}");
        }
        if let Some(hash) = &entry.hash_after {
            let _ = writeln!(&mut response, "  hash: {hash}");
        }
        if include_undo {
            if let Some(undo) = &entry.undo_token {
                let _ = writeln!(&mut response, "  undo: {undo}");
            }
        }
    }
    response.trim().to_string()
}

fn format_export_response(result: &ProfileExportResult, include_history: bool) -> String {
    let mut response = String::new();
    let _ = writeln!(
        &mut response,
        "Exported {:?} profile to {}",
        result.profile_type,
        result.archive_path.display()
    );
    let _ = writeln!(
        &mut response,
        "Archive hash: {}\nEvent ID: {}",
        result.hash,
        result.event_id
    );
    if include_history {
        response.push_str("Included audit log in archive.");
    }
    response.trim().to_string()
}

fn format_delete_response(result: &ProfileDeleteResult) -> String {
    let mut response = String::new();
    let _ = writeln!(
        &mut response,
        "Deleted {:?} profile artifacts.",
        result.profile_type
    );
    if result.files_removed.is_empty() {
        response.push_str("No files were removed.");
    } else {
        response.push_str("Removed files:");
        for path in &result.files_removed {
            let _ = writeln!(&mut response, "\n- {}", path.display());
        }
    }
    let _ = writeln!(&mut response, "\nEvent ID: {}", result.event_id);
    response.trim().to_string()
}

fn format_regenerate_response(
    outcome: &ProfileRegenerateOutcome,
    source: &ProfileRegenerateSource,
) -> String {
    let mut response = String::new();
    let from = match source {
        ProfileRegenerateSource::History => "history replay".to_string(),
        ProfileRegenerateSource::Archive(path) => {
            format!("archive {}", path.display())
        }
    };
    let _ = writeln!(
        &mut response,
        "Regenerated {:?} profile from {from}.",
        outcome.profile_type
    );
    let _ = writeln!(
        &mut response,
        "Replayed events: {}\nFinal hash: {}\nEvent ID: {}",
        outcome.replayed_events,
        outcome.hash_after,
        outcome.event_id
    );
    response.trim().to_string()
}

fn format_scope_response(setting: &ProfileScopeSetting, event_id: Option<uuid::Uuid>) -> String {
    let mut response = String::new();
    let _ = writeln!(
        &mut response,
        "Scope for {:?}: {:?}",
        setting.profile_type, setting.scope_mode
    );
    if setting.allowed_bases.is_empty() {
        response.push_str("Allowed bases: (none)\n");
    } else {
        let _ = writeln!(
            &mut response,
            "Allowed bases: {}",
            setting.allowed_bases.join(", ")
        );
    }
    let _ = writeln!(
        &mut response,
        "Last updated: {}",
        setting.updated_at.to_rfc3339()
    );
    if let Some(event_id) = event_id {
        let _ = writeln!(&mut response, "Event ID: {event_id}");
    }
    response.trim().to_string()
}

fn parse_scope_mode(raw: &str) -> Result<ProfileScopeMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "this-base" | "this_base" | "thisbase" | "local" => Ok(ProfileScopeMode::ThisBase),
        "shared" => Ok(ProfileScopeMode::Shared),
        "disabled" => Ok(ProfileScopeMode::Disabled),
        other => bail!("Unknown scope mode '{other}'. Use this-base|shared|disabled."),
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfileShowRequest {
    pub profile_type: String,
    pub include_history: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileUpdateRequest {
    pub profile_type: String,
    pub field_changes: Vec<String>,
    pub confirm: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileInterviewRequest {
    pub profile_type: String,
    pub requires_remote: bool,
    pub remote_prompt_hint: Option<String>,
    pub answers: Vec<String>,
    pub confirm: bool,
    pub approve_remote: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileRunRequest {
    pub profile_type: String,
    pub run_kind: String,
    pub requires_remote: Option<bool>,
    pub remote_prompt_hint: Option<String>,
    pub answers: Vec<String>,
    pub confirm: bool,
    pub approve_remote: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileAuditRequest {
    pub profile_type: String,
    pub include_undo_instructions: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileExportRequest {
    pub profile_type: String,
    pub destination: Option<PathBuf>,
    pub include_history: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileDeleteRequest {
    pub profile_type: String,
    pub confirm_phrase: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ProfileRegenerateSource {
    History,
    Archive(PathBuf),
}

impl Default for ProfileRegenerateSource {
    fn default() -> Self {
        Self::History
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfileRegenerateRequest {
    pub profile_type: String,
    pub source: ProfileRegenerateSource,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileScopeRequest {
    pub profile_type: String,
    pub scope_mode: Option<String>,
    pub allowed_bases: Vec<String>,
}
