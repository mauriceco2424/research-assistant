use anyhow::Result;
use chrono::{Duration, Utc};
use researchbase::bases::LibraryEntry;
use researchbase::chat::ChatSession;
use uuid::Uuid;

use crate::support::profile_base::ProfileBaseFixture;

#[test]
fn router_executes_multi_intent_sequence() -> Result<()> {
    let fixture = ProfileBaseFixture::new("intent-router-execute");
    let entries = vec![
        sample_entry("Paper Alpha", -4),
        sample_entry("Paper Beta", -3),
        sample_entry("Paper Gamma", -2),
        sample_entry("Paper Delta", -1),
    ];
    fixture
        .manager
        .save_library_entries(&fixture.base, &entries)?;

    let mut chat = fixture.chat_session()?;
    let responses = chat.handle_message(
        "Summarize the last 3 papers and show my writing profile",
    )?;
    assert_eq!(
        responses.len(),
        2,
        "Expected two responses for multi-intent flow: {responses:?}"
    );
    assert!(
        responses[0].contains("Recent 3 papers"),
        "Summary response missing details: {}",
        responses[0]
    );
    assert!(
        responses[1].to_ascii_lowercase().contains("profile"),
        "Profile response missing: {}",
        responses[1]
    );
    Ok(())
}

fn sample_entry(title: &str, minutes_offset: i64) -> LibraryEntry {
    LibraryEntry {
        entry_id: Uuid::new_v4(),
        title: title.to_string(),
        authors: vec!["Test Author".into()],
        venue: None,
        year: Some(2024),
        identifier: format!("doi:{title}"),
        pdf_paths: Vec::new(),
        needs_pdf: true,
        notes: None,
        created_at: Utc::now() + Duration::minutes(minutes_offset),
        updated_at: Utc::now(),
    }
}
