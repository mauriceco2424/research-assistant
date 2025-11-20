use anyhow::Result;
use chrono::{Duration, Utc};
use researchbase::bases::{LibraryEntry, ProfileLayout};
use researchbase::chat::ChatSession;
use researchbase::orchestration::consent::{
    ConsentManifest, ConsentOperation, ConsentScope, ConsentStatus, ConsentStore,
};
use researchbase::orchestration::profiles::defaults::default_knowledge_profile;
use researchbase::orchestration::profiles::model::{
    KnowledgeEntry, KnowledgeProfile, MasteryLevel, VerificationStatus,
};
use researchbase::orchestration::profiles::storage::write_profile;
use serde_json::json;
use uuid::Uuid;

use crate::support::profile_base::ProfileBaseFixture;

#[test]
fn suggestions_surface_contextual_actions() -> Result<()> {
    let mut fixture = ProfileBaseFixture::new("intent-router-suggestions");
    seed_backlog(&mut fixture)?;
    seed_expired_consent(&fixture)?;
    mark_stale_knowledge(&fixture)?;

    let mut chat = fixture.chat_session()?;
    let responses = chat.handle_message("What should I do next?")?;
    assert!(
        responses.iter().any(|line| line.contains("consent")),
        "Expected consent reminder in responses: {responses:?}"
    );
    assert!(
        responses.iter().any(|line| line.contains("Knowledge entries need refresh")),
        "Expected knowledge reminder in responses: {responses:?}"
    );
    assert!(
        responses
            .iter()
            .any(|line| line.contains("suggestion.snapshot")),
        "Expected snapshot audit line in responses: {responses:?}"
    );
    Ok(())
}

#[test]
fn fallback_guidance_for_unknown_intent() -> Result<()> {
    let fixture = ProfileBaseFixture::new("intent-router-fallback");
    let mut chat = fixture.chat_session()?;
    let responses = chat.handle_message("Handle that task")?;
    assert_eq!(
        responses.len(),
        2,
        "Expected fallback message + manual hint: {responses:?}"
    );
    assert!(
        responses[0].contains("couldn't route"),
        "First response should explain fallback: {}",
        responses[0]
    );
    assert!(
        responses[1].contains("help commands"),
        "Manual hint missing expected guidance: {}",
        responses[1]
    );
    Ok(())
}

fn seed_backlog(fixture: &mut ProfileBaseFixture) -> Result<()> {
    let entry = LibraryEntry {
        entry_id: Uuid::new_v4(),
        title: "Backlog Paper".into(),
        authors: vec!["Ada Example".into()],
        venue: Some("TestConf".into()),
        year: Some(2024),
        identifier: "urn:backlog:paper".into(),
        pdf_paths: Vec::new(),
        needs_pdf: true,
        notes: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    fixture
        .manager
        .save_library_entries(&fixture.base, &[entry])?;
    Ok(())
}

fn seed_expired_consent(fixture: &ProfileBaseFixture) -> Result<()> {
    let store = ConsentStore::for_base(&fixture.base);
    let manifest = ConsentManifest {
        manifest_id: Uuid::new_v4(),
        base_id: fixture.base.id,
        operation: ConsentOperation::MetadataLookup,
        scope: ConsentScope::default(),
        approval_text: "Intent router suggestion test".into(),
        approved_at: Utc::now() - Duration::days(2),
        prompt_manifest: json!({ "reason": "integration test" }),
        prompt_excerpt: Some("integration test".into()),
        provider: Some("test-suite".into()),
        data_categories: vec!["metadata".into()],
        expires_at: Some(Utc::now() - Duration::hours(1)),
        status: ConsentStatus::Approved,
    };
    store.record(&manifest)?;
    Ok(())
}

fn mark_stale_knowledge(fixture: &ProfileBaseFixture) -> Result<()> {
    let layout = ProfileLayout::new(&fixture.base);
    let path = layout.profile_json("knowledge");
    let mut profile: KnowledgeProfile =
        serde_json::from_slice(&std::fs::read(&path)?).unwrap_or_else(|_| default_knowledge_profile());
    profile.entries.push(KnowledgeEntry {
        concept: "LLM research".into(),
        mastery_level: MasteryLevel::Developing,
        evidence_refs: Vec::new(),
        weakness_flags: vec!["needs_refresh".into()],
        learning_links: Vec::new(),
        last_reviewed: Utc::now() - Duration::days(90),
        verification_status: VerificationStatus::Stale,
    });
    write_profile(path, &profile)?;
    Ok(())
}
