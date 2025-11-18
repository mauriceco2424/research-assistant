use super::IntegrationHarness;
use anyhow::Result;
use researchbase::bases::{BaseManager, LibraryEntry};
use researchbase::ingestion::{refresh_metadata, MetadataRefreshRequest};
use std::path::PathBuf;

#[test]
fn metadata_refresh_detects_language_and_rtl() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "metadata-multilang");
    seed_entry(&manager, &base, "Deep Learning Advances");
    seed_entry(&manager, &base, "تحليل البيانات العلمية");

    let _ = refresh_metadata(
        &manager,
        &base,
        MetadataRefreshRequest {
            paper_ids: None,
            allow_remote: false,
            approval_text: None,
        },
    )?;
    let records = manager.load_metadata_records(&base)?;
    assert_eq!(records.len(), 2);
    let arabic = records
        .iter()
        .find(|record| record.title.contains("تحليل"))
        .expect("missing Arabic record");
    assert_eq!(arabic.language.as_deref(), Some("ara"));
    assert_eq!(arabic.script_direction.as_deref(), Some("rtl"));

    let english = records
        .iter()
        .find(|record| record.title.contains("Deep Learning"))
        .expect("missing English record");
    assert_eq!(english.language.as_deref(), Some("eng"));
    assert_eq!(english.script_direction.as_deref(), Some("ltr"));
    Ok(())
}

fn seed_entry(manager: &BaseManager, base: &researchbase::bases::Base, title: &str) {
    let entry = LibraryEntry {
        entry_id: uuid::Uuid::new_v4(),
        title: title.into(),
        authors: vec!["Tester".into()],
        venue: Some("Demo".into()),
        year: Some(2024),
        identifier: format!("id:{}", title),
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
