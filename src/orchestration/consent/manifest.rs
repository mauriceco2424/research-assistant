use crate::{
    bases::{Base, BaseManager},
    orchestration::profiles::model::ProfileType,
};
use anyhow::Result;
use serde_json::json;

use super::{require_remote_operation_consent, ConsentManifest, ConsentOperation, ConsentScope};

/// Builds a prompt manifest describing the remote inference request.
pub fn build_profile_interview_prompt(
    profile_type: ProfileType,
    prompt_hint: Option<&str>,
) -> serde_json::Value {
    json!({
        "operation": "profile_interview",
        "profile_type": format!("{profile_type:?}").to_ascii_lowercase(),
        "prompt_hint": prompt_hint.unwrap_or("profile interview remote inference"),
    })
}

/// Requests consent for a profile interview remote inference.
pub fn request_profile_interview_consent(
    manager: &BaseManager,
    base: &Base,
    profile_type: ProfileType,
    prompt_hint: Option<&str>,
) -> Result<ConsentManifest> {
    let manifest = build_profile_interview_prompt(profile_type, prompt_hint);
    let approval_text = prompt_hint.unwrap_or("profile interview remote inference");
    require_remote_operation_consent(
        manager,
        base,
        ConsentOperation::ProfileInterviewRemote,
        approval_text,
        ConsentScope::default(),
        manifest,
    )
}
