//! Evidence linking helpers for knowledge profiles.

use crate::bases::Base;
use std::path::{Path, PathBuf};

use super::model::{EvidenceKind, KnowledgeEntry, KnowledgeProfile, VerificationStatus};

#[derive(Debug, Clone)]
pub struct StaleEvidenceRef {
    pub concept: String,
    pub missing_reference: String,
}

/// Refreshes verification statuses based on evidence availability.
pub fn refresh_evidence_links(base: &Base, profile: &mut KnowledgeProfile) -> Vec<String> {
    let mut updates = Vec::new();
    for entry in &mut profile.entries {
        let missing = missing_references(base, entry);
        let new_status = if entry.evidence_refs.is_empty() {
            VerificationStatus::Unverified
        } else if missing.is_empty() {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Stale
        };
        if entry.verification_status != new_status {
            entry.verification_status = new_status;
            updates.push(format!("Updated verification for {}", entry.concept));
        }
    }
    updates
}

/// Collects missing references for summary APIs.
pub fn detect_missing_references(
    base: &Base,
    profile: &KnowledgeProfile,
) -> Vec<StaleEvidenceRef> {
    let mut notices = Vec::new();
    for entry in &profile.entries {
        for missing in missing_references(base, entry) {
            notices.push(StaleEvidenceRef {
                concept: entry.concept.clone(),
                missing_reference: missing,
            });
        }
    }
    notices
}

fn missing_references(base: &Base, entry: &KnowledgeEntry) -> Vec<String> {
    entry
        .evidence_refs
        .iter()
        .filter(|reference| !evidence_exists(base, reference))
        .map(|reference| format_reference(reference))
        .collect()
}

fn evidence_exists(base: &Base, evidence: &super::model::EvidenceRef) -> bool {
    match evidence.kind {
        EvidenceKind::Paper => {
            let dir = base.user_layer_path.join("papers");
            path_candidates(&dir, &evidence.identifier, ".pdf")
                .into_iter()
                .any(|path| path.exists())
        }
        EvidenceKind::Note => {
            let dir = base.ai_layer_path.join("notes");
            path_candidates(&dir, &evidence.identifier, ".md")
                .into_iter()
                .any(|path| path.exists())
        }
        EvidenceKind::Manual => true,
    }
}

fn path_candidates(root: &Path, identifier: &str, fallback_ext: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut direct = root.join(identifier);
    if direct.exists() {
        paths.push(direct);
        return paths;
    }
    if identifier.contains('/') || identifier.contains('\\') {
        direct = root.join(PathBuf::from(identifier));
    }
    paths.push(direct.clone());
    if Path::new(identifier).extension().is_none() {
        let mut with_ext = direct.clone();
        with_ext.set_extension(fallback_ext.trim_start_matches('.'));
        paths.push(with_ext);
    }
    paths
}

fn format_reference(evidence: &super::model::EvidenceRef) -> String {
    format!(
        "{}:{}",
        match evidence.kind {
            EvidenceKind::Paper => "paper",
            EvidenceKind::Note => "note",
            EvidenceKind::Manual => "manual",
        },
        evidence.identifier
    )
}
