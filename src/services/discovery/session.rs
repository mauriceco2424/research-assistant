use crate::models::discovery::DiscoveryCandidate;
use anyhow::Result;
use uuid::Uuid;

/// Placeholder: follow-up discovery suggestions using prior session context.
pub fn generate_session_followups(session_id: Uuid) -> Result<Vec<DiscoveryCandidate>> {
    let candidate = DiscoveryCandidate {
        id: Uuid::new_v4(),
        title: format!("Follow-up paper for session {}", session_id),
        authors: vec!["SessionBot".into()],
        venue: Some("ResearchBase Session".into()),
        year: Some(2024),
        source_link: None,
        rationale: Some(format!("Session follow-up {}", session_id)),
        identifiers: Default::default(),
        duplicate_match: None,
    };
    Ok(vec![candidate])
}
