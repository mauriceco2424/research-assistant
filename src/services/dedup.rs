use crate::bases::{Base, BaseManager, LibraryEntry};
use crate::models::discovery::{
    DiscoveryCandidate, DiscoveryIdentifiers, DuplicateMatch, DuplicateMethod,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use uuid::Uuid;

fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn load_entries_by_identifier(entries: &[LibraryEntry]) -> HashMap<String, Uuid> {
    let mut map = HashMap::new();
    for entry in entries {
        map.insert(entry.identifier.to_lowercase(), entry.entry_id);
    }
    map
}

pub fn detect_duplicate(
    manager: &BaseManager,
    base: &Base,
    candidate: &DiscoveryCandidate,
) -> Result<Option<DuplicateMatch>> {
    let entries = manager
        .load_library_entries(base)
        .with_context(|| "Failed to load library entries for deduplication")?;
    let identifiers = load_entries_by_identifier(&entries);

    // Prefer DOI then arXiv/eprint matches.
    if let Some(doi) = candidate.identifiers.doi.as_ref() {
        if let Some(id) = identifiers.get(&doi.to_lowercase()) {
            return Ok(Some(DuplicateMatch {
                matched_record_id: *id,
                method: DuplicateMethod::Doi,
            }));
        }
    }
    if let Some(arxiv) = candidate.identifiers.arxiv.as_ref() {
        if let Some(id) = identifiers.get(&arxiv.to_lowercase()) {
            return Ok(Some(DuplicateMatch {
                matched_record_id: *id,
                method: DuplicateMethod::Arxiv,
            }));
        }
    }

    // Fallback to normalized title + first author + year heuristic.
    let normalized_title = normalize_title(&candidate.title);
    let first_author = candidate
        .authors
        .get(0)
        .map(|a| a.to_lowercase())
        .unwrap_or_default();
    let year = candidate.year.unwrap_or_default();
    for entry in entries {
        let entry_title = normalize_title(&entry.title);
        let entry_first_author = entry
            .authors
            .get(0)
            .map(|a| a.to_lowercase())
            .unwrap_or_default();
        let entry_year = entry.year.unwrap_or_default();
        if entry_title == normalized_title
            && entry_first_author == first_author
            && entry_year == year
        {
            return Ok(Some(DuplicateMatch {
                matched_record_id: entry.entry_id,
                method: DuplicateMethod::TitleAuthorYear,
            }));
        }
    }
    Ok(None)
}

pub fn enrich_with_identifiers(
    existing: &DiscoveryIdentifiers,
    candidate_identifier: &str,
) -> DiscoveryIdentifiers {
    let mut ids = existing.clone();
    // Lightweight heuristic: populate DOI/arXiv if identifier looks like one.
    if candidate_identifier.starts_with("doi:") && ids.doi.is_none() {
        ids.doi = Some(candidate_identifier.to_string());
    } else if candidate_identifier.contains("arxiv") && ids.arxiv.is_none() {
        ids.arxiv = Some(candidate_identifier.to_string());
    }
    ids
}
