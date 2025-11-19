use anyhow::Result;
use researchbase::chat::commands::profile::{ProfileShowRequest, ProfileUpdateRequest};
use researchbase::chat::ChatSession;

use crate::support::profile_base::ProfileBaseFixture;

#[test]
fn profile_show_update_roundtrip() -> Result<()> {
    let fixture = ProfileBaseFixture::new("profile-show-update");
    let mut chat = fixture.chat_session()?;

    // Initial show should reference seeded defaults.
    let mut show_request = ProfileShowRequest::default();
    show_request.profile_type = "work".into();
    show_request.include_history = true;
    let initial = chat.profile_show(show_request.clone())?;
    assert!(
        initial.contains("Profile: Work"),
        "Expected default work profile summary"
    );

    // Apply focus statement update via chat.
    let mut update_request = ProfileUpdateRequest::default();
    update_request.profile_type = "work".into();
    update_request.field_changes = vec!["focus=Submit CHI draft".into()];
    update_request.confirm = true;
    let update_output = chat.profile_update(update_request)?;
    assert!(
        update_output.contains("Updated focus statement"),
        "Expected diff summary in update output: {update_output}"
    );

    // Show again and ensure the new focus is rendered.
    let follow_up = chat.profile_show(show_request)?;
    assert!(
        follow_up.contains("Submit CHI draft"),
        "Updated focus statement missing from profile show output: {follow_up}"
    );

    // Confirm the JSON artifact was updated on disk.
    let json_path = fixture.profile_json_path("work");
    let json_data = std::fs::read_to_string(json_path)?;
    assert!(
        json_data.contains("Submit CHI draft"),
        "Updated JSON did not include focus statement"
    );

    Ok(())
}
