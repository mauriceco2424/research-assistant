use std::fs;

use anyhow::Result;
use researchbase::{
    bases::ProfileLayout,
    chat::commands::profile::{
        ProfileAuditRequest, ProfileDeleteRequest, ProfileExportRequest, ProfileRegenerateRequest,
        ProfileRegenerateSource, ProfileScopeRequest, ProfileShowRequest, ProfileUpdateRequest,
    },
    chat::ChatSession,
};

use crate::support::profile_base::ProfileBaseFixture;

#[test]
fn profile_governance_end_to_end() -> Result<()> {
    let fixture = ProfileBaseFixture::new("profile-governance");
    let mut chat = fixture.chat_session()?;

    seed_work_profile(&mut chat)?;
    let layout = ProfileLayout::new(&fixture.base);
    let work_json = layout.profile_json("work");
    let work_html = layout.profile_html("work");

    let audit_output = chat.profile_audit(ProfileAuditRequest {
        profile_type: "work".into(),
        include_undo_instructions: true,
    })?;
    assert!(
        audit_output.contains("Profile audit for Work"),
        "Audit output missing summary: {audit_output}"
    );
    assert!(
        audit_output.contains("Updated focus"),
        "Expected diff summary from profile update: {audit_output}"
    );

    let exports_dir = layout.user_exports_dir.clone();
    let export_output = chat.profile_export(ProfileExportRequest {
        profile_type: "work".into(),
        destination: None,
        include_history: true,
    })?;
    assert!(
        export_output.contains("Exported Work profile"),
        "Unexpected export output: {export_output}"
    );
    let archives: Vec<_> = fs::read_dir(&exports_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "zip").unwrap_or(false))
        .collect();
    assert!(
        !archives.is_empty(),
        "Expected a ZIP export under {}",
        exports_dir.display()
    );

    // Simulate concurrent export by holding the lock file.
    let lock_path = exports_dir.join(".profile_export.lock");
    fs::create_dir_all(&exports_dir)?;
    fs::write(&lock_path, b"lock")?;
    let err = chat
        .profile_export(ProfileExportRequest {
            profile_type: "work".into(),
            destination: None,
            include_history: false,
        })
        .expect_err("Expected EXPORT_IN_PROGRESS error");
    assert!(
        err.to_string().contains("EXPORT_IN_PROGRESS"),
        "Unexpected error message: {err:?}"
    );
    fs::remove_file(lock_path)?;

    let delete_output = chat.profile_delete(ProfileDeleteRequest {
        profile_type: "work".into(),
        confirm_phrase: Some("DELETE work".into()),
    })?;
    assert!(
        delete_output.contains("Deleted Work profile artifacts"),
        "Unexpected delete output: {delete_output}"
    );
    assert!(
        !work_json.exists() && !work_html.exists(),
        "Expected artifacts removed"
    );

    let regen_output = chat.profile_regenerate(ProfileRegenerateRequest {
        profile_type: "work".into(),
        source: ProfileRegenerateSource::History,
    })?;
    assert!(
        regen_output.contains("Regenerated Work profile"),
        "Unexpected regenerate output: {regen_output}"
    );
    assert!(work_json.exists(), "Regeneration should recreate JSON");
    let json_data = fs::read_to_string(work_json)?;
    assert!(
        json_data.contains("Governance flows verified"),
        "Regenerated profile missing expected content"
    );

    Ok(())
}

#[test]
fn profile_scope_roundtrip() -> Result<()> {
    let fixture = ProfileBaseFixture::new("profile-scope");
    let mut chat = fixture.chat_session()?;

    let list_output = chat.profile_scope(ProfileScopeRequest {
        profile_type: "writing".into(),
        scope_mode: None,
        allowed_bases: vec![],
    })?;
    assert!(
        list_output.contains("Scope for Writing"),
        "Scope listing missing profile context: {list_output}"
    );

    let update_output = chat.profile_scope(ProfileScopeRequest {
        profile_type: "writing".into(),
        scope_mode: Some("shared".into()),
        allowed_bases: vec!["demo-base".into()],
    })?;
    assert!(
        update_output.contains("shared"),
        "Scope update should mention shared mode: {update_output}"
    );
    assert!(
        update_output.contains("demo-base"),
        "Scope update should list allowed bases: {update_output}"
    );

    Ok(())
}

fn seed_work_profile(chat: &mut ChatSession) -> Result<()> {
    let mut show_request = ProfileShowRequest::default();
    show_request.profile_type = "work".into();
    chat.profile_show(show_request)?;

    let mut update_request = ProfileUpdateRequest::default();
    update_request.profile_type = "work".into();
    update_request.field_changes =
        vec!["focus=Governance flows verified".into(), "risks=timelines".into()];
    update_request.confirm = true;
    let update_output = chat.profile_update(update_request)?;
    assert!(
        update_output.contains("Updated focus statement"),
        "Expected update diff summary: {update_output}"
    );
    Ok(())
}
