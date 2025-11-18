use super::IntegrationHarness;
use anyhow::Result;
use researchbase::bases::{BaseManager, LibraryEntry};
use researchbase::ingestion::{refresh_metadata, MetadataRefreshRequest};
use std::path::PathBuf;

#[test]
fn metadata_refresh_requires_consent_for_remote() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "metadata-consent");
    seed_library(&manager, &base, "consent-paper");

    let err = refresh_metadata(
        &manager,
        &base,
        MetadataRefreshRequest {
            paper_ids: None,
            allow_remote: true,
            approval_text: None,
        },
    )
    .expect_err("remote metadata refresh should require approval text");
    assert!(
        err.to_string()
            .contains("Remote metadata lookup requires explicit approval text"),
        "unexpected error {err:?}"
    );
    Ok(())
}

fn seed_library(manager: &BaseManager, base: &researchbase::bases::Base, title: &str) {
    let entry = LibraryEntry {
        entry_id: uuid::Uuid::new_v4(),
        title: title.into(),
        authors: vec!["Author One".into()],
        venue: Some("TestConf".into()),
        year: Some(2020),
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
