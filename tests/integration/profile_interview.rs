use anyhow::Result;
use researchbase::chat::commands::profile::ProfileInterviewRequest;

use crate::support::profile_base::ProfileBaseFixture;

#[test]
fn interview_remote_approval_and_denial() -> Result<()> {
    let fixture = ProfileBaseFixture::new("profile-interview");
    let mut chat = fixture.chat_session()?;

    // Remote approval path.
    let mut request = ProfileInterviewRequest::default();
    request.profile_type = "writing".into();
    request.requires_remote = true;
    request.remote_prompt_hint = Some("capture writing style summary".into());
    request.answers = vec!["tone_descriptors=confident, precise".into()];
    request.confirm = true;
    request.approve_remote = true;
    let response = chat.profile_interview(request.clone())?;
    assert!(
        response.contains("completed"),
        "Expected approval response: {response}"
    );

    let writing_json_path = fixture.profile_json_path("writing");
    let writing_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&writing_json_path)?)?;
    let remote_meta = &writing_json["fields"]["remote_inference_metadata"];
    assert_eq!(
        remote_meta["status"],
        "approved",
        "Remote metadata should mark approved"
    );
    assert!(
        remote_meta["consent_manifest_id"].as_str().is_some(),
        "Consent manifest should be recorded"
    );

    // Remote denial path.
    request.approve_remote = false;
    request.remote_prompt_hint = Some("fallback writing style".into());
    request.answers = vec!["tone_descriptors=approachable".into()];
    let denial = chat.profile_interview(request)?;
    assert!(
        denial.contains("needs remote approval"),
        "Expected pending approval message: {denial}"
    );
    let updated_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&writing_json_path)?)?;
    let pending_meta = &updated_json["fields"]["remote_inference_metadata"];
    assert_eq!(pending_meta["status"], "pending");
    assert!(pending_meta["consent_manifest_id"].is_null());

    // Metrics file should exist with at least two entries.
    let metrics_path = fixture
        .base
        .ai_layer_path
        .join("metrics/profile_interviews.jsonl");
    let metrics = std::fs::read_to_string(metrics_path)?;
    assert!(
        metrics.lines().count() >= 2,
        "Expected metrics entries for interviews"
    );

    Ok(())
}
