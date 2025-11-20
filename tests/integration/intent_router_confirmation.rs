use anyhow::Result;
use researchbase::chat::ChatSession;

use crate::support::profile_base::ProfileBaseFixture;
use crate::IntegrationHarness;

#[test]
fn destructive_action_requires_confirmation() -> Result<()> {
    let fixture = ProfileBaseFixture::new("intent-router-confirm");
    let mut chat = fixture.chat_session()?;
    let responses = chat.handle_message("Delete the writing profile")?;
    assert_eq!(
        responses.len(),
        1,
        "Expected confirmation prompt message: {responses:?}"
    );
    assert!(
        responses[0].contains("DELETE writing"),
        "Confirm phrase missing from response: {}",
        responses[0]
    );
    let confirmations_dir = fixture.base.ai_layer_path.join("intents/confirmations");
    let files: Vec<_> = std::fs::read_dir(&confirmations_dir)?.collect();
    assert!(
        !files.is_empty(),
        "Expected confirmation ticket to be written at {:?}",
        confirmations_dir
    );
    Ok(())
}

#[test]
fn missing_base_prompts_selection() -> Result<()> {
    let _harness = IntegrationHarness::new();
    let mut chat = ChatSession::new()?;
    let responses = chat.handle_message("Summarize the last 3 papers")?;
    assert_eq!(responses.len(), 1);
    assert!(
        responses[0]
            .to_ascii_lowercase()
            .contains("select or create a base"),
        "Missing base guidance should mention selection: {}",
        responses[0]
    );
    Ok(())
}
