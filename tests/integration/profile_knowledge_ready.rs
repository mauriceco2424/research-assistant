use std::fs;

use anyhow::Result;
use chrono::Utc;
use researchbase::chat::commands::profile::ProfileUpdateRequest;
use researchbase::orchestration::profiles::{
    api::get_knowledge_summary,
    model::{EvidenceKind, EvidenceRef, KnowledgeEntry, MasteryLevel, VerificationStatus},
    service::ProfileService,
    storage::write_profile,
};

use crate::support::profile_base::ProfileBaseFixture;

#[test]
fn knowledge_summary_and_strength_toggles() -> Result<()> {
    let fixture = ProfileBaseFixture::new("knowledge-ready");
    let mut service = ProfileService::new(&fixture.manager, &fixture.base);
    let mut profile = service.load_knowledge_profile()?;

    profile.entries.push(KnowledgeEntry {
        concept: "Graph Neural Networks".into(),
        mastery_level: MasteryLevel::Developing,
        evidence_refs: vec![EvidenceRef {
            kind: EvidenceKind::Paper,
            identifier: "gnn.pdf".into(),
            confidence: 0.9,
        }],
        weakness_flags: Vec::new(),
        learning_links: Vec::new(),
        last_reviewed: Utc::now(),
        verification_status: VerificationStatus::Unverified,
    });

    profile.entries.push(KnowledgeEntry {
        concept: "Bayesian EU".into(),
        mastery_level: MasteryLevel::Developing,
        evidence_refs: vec![EvidenceRef {
            kind: EvidenceKind::Note,
            identifier: "notes/bayes-eu.md".into(),
            confidence: 0.5,
        }],
        weakness_flags: vec!["Needs refresher".into()],
        learning_links: Vec::new(),
        last_reviewed: Utc::now(),
        verification_status: VerificationStatus::Unverified,
    });

    write_profile(fixture.profile_json_path("knowledge"), &profile)?;

    // Seed evidence file for the first entry.
    let papers_dir = fixture.base.user_layer_path.join("papers");
    fs::create_dir_all(&papers_dir)?;
    fs::write(papers_dir.join("gnn.pdf"), b"test-pdf")?;

    let mut chat = fixture.chat_session()?;
    let mut update_request = ProfileUpdateRequest::default();
    update_request.profile_type = "knowledge".into();
    update_request.field_changes = vec![
        "Graph Neural Networks.mastery_level=expert".into(),
        "Bayesian EU.weakness_flags=Needs refresher, practice derivations".into(),
    ];
    update_request.confirm = true;
    chat.profile_update(update_request)?;

    let summary = get_knowledge_summary(&fixture.manager, &fixture.base)?;
    assert_eq!(summary.counts.strengths, 1);
    assert_eq!(summary.counts.weaknesses, 1);
    assert_eq!(summary.counts.unverified, 0);
    assert!(
        summary
            .stale_evidence_refs
            .iter()
            .any(|notice| notice.missing_reference.contains("bayes-eu")),
        "Expected stale note reference in summary: {:?}",
        summary.stale_evidence_refs
    );

    let gnn_entry = summary
        .entries
        .iter()
        .find(|entry| entry.concept == "Graph Neural Networks")
        .expect("graph entry");
    assert_eq!(gnn_entry.mastery_level, MasteryLevel::Expert);
    assert_eq!(gnn_entry.verification_status, VerificationStatus::Verified);

    Ok(())
}
