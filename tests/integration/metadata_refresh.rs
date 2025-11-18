use super::IntegrationHarness;
use anyhow::Result;
use researchbase::bases::{BaseManager, LibraryEntry};
use researchbase::ingestion::{refresh_metadata, MetadataRefreshRequest};
use std::path::PathBuf;

#[test]
fn metadata_refresh_updates_records_with_consent() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "metadata-refresh-happy");
    seed_entry(&manager, &base, "Efficient Structures", "doi:seed");

    let outcome = refresh_metadata(
        &manager,
        &base,
        MetadataRefreshRequest {
            paper_ids: None,
            allow_remote: true,
            approval_text: Some("Approved in test".into()),
        },
    )?;
    assert!(!outcome.offline_mode);
    assert!(outcome.used_remote);
    assert_eq!(outcome.updated_records.len(), 1);
    let record = &outcome.updated_records[0];
    assert!(record.doi.as_ref().unwrap().starts_with("10.5555/"));
    Ok(())
}

fn seed_entry(manager: &BaseManager, base: &researchbase::bases::Base, title: &str, identifier: &str) {
    let entry = LibraryEntry {
        entry_id: uuid::Uuid::new_v4(),
        title: title.into(),
        authors: vec!["Unit Tester".into()],
        venue: Some("DemoConf".into()),
        year: Some(2024),
        identifier: identifier.into(),
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
