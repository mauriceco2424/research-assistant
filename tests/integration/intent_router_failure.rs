use anyhow::Result;

use crate::support::profile_base::ProfileBaseFixture;

#[test]
fn failure_short_circuits_downstream_intents() -> Result<()> {
    let fixture = ProfileBaseFixture::new("intent-router-failure");
    let mut chat = fixture.chat_session()?;
    let responses = chat.handle_message(
        "Summarize the last 3 papers and show my writing profile",
    )?;
    assert_eq!(
        responses.len(),
        3,
        "Expected failure, manual hint, and cancellation responses: {responses:?}"
    );
    assert!(
        responses[0].contains("failed"),
        "First response should report failure: {}",
        responses[0]
    );
    assert!(
        responses[1].contains("help commands"),
        "Manual hint missing: {}",
        responses[1]
    );
    assert!(
        responses[2].contains("Cancelled 1"),
        "Third response should report cancellation: {}",
        responses[2]
    );
    Ok(())
}
