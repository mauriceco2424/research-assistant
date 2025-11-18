use super::IntegrationHarness;
use anyhow::Result;
use researchbase::bases::{BaseManager, LibraryEntry};
use researchbase::ingestion::{refresh_metadata, MetadataRefreshRequest};
use std::path::PathBuf;

#[test]
fn metadata_refresh_falls_back_to_offline() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "metadata-offline");
    seed_entry(&manager, &base, "offline heuristics");

    let outcome = refresh_metadata(
        &manager,
        &base,
        MetadataRefreshRequest {
            paper_ids: None,
            allow_remote: false,
            approval_text: None,
        },
    )?;
    assert!(outcome.offline_mode, "expected offline mode");
    assert_eq!(outcome.updated_records.len(), 1);
    Ok(())
}

fn seed_entry(manager: &BaseManager, base: &researchbase::bases::Base, title: &str) {
    let entry = LibraryEntry {
        entry_id: uuid::Uuid::new_v4(),
        title: title.into(),
        authors: vec![],
        venue: None,
        year: None,
        identifier: format!("local:{title}"),
        pdf_paths: vec![PathBuf::from("dummy.pdf")],
        needs_pdf: false,
        notes: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    manager
        .save_library_entries(base, &[entry])
        .expect("failed to seed library");
}
