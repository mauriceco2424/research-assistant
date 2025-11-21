use crate::acquisition::discover_candidates;
use crate::models::discovery::{DiscoveryCandidate, DiscoveryIdentifiers};
use crate::services::dedup::{detect_duplicate, enrich_with_identifiers};
use crate::{Base, BaseManager};
use anyhow::Result;
use uuid::Uuid;

/// Build discovery candidates for a topic prompt, enriching with dedup info.
pub fn generate_topic_candidates(
    manager: &BaseManager,
    base: &Base,
    topic: &str,
    count: usize,
) -> Result<Vec<DiscoveryCandidate>> {
    let generated = discover_candidates(topic, count);
    let mut candidates = Vec::new();
    for c in generated {
        let mut identifiers = DiscoveryIdentifiers::default();
        identifiers = enrich_with_identifiers(&identifiers, &c.identifier);
        let mut candidate = DiscoveryCandidate {
            id: Uuid::new_v4(),
            title: c.title.clone(),
            authors: c.authors.clone(),
            venue: c.venue.clone(),
            year: c.year,
            source_link: Some(format!("id:{}", c.identifier)),
            rationale: Some(format!("topic: {topic}")),
            identifiers,
            duplicate_match: None,
        };
        candidate.duplicate_match = detect_duplicate(manager, base, &candidate)?;
        candidates.push(candidate);
    }
    Ok(candidates)
}
