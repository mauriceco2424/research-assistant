use super::IntegrationHarness;
use anyhow::Result;
use researchbase::acquisition::{run_figure_extraction, FigureStore};
use researchbase::bases::{BaseManager, LibraryEntry};
use std::path::PathBuf;

#[test]
fn figure_extraction_creates_assets_with_consent() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    manager.config.acquisition.remote_allowed = true;
    let base = harness.create_base(&mut manager, "figure-extraction");
    seed_entry(&manager, &base, "Paper With Figures");

    let outcome =
        run_figure_extraction(&manager, &base, None, "Approved figure extraction test")?;
    assert_eq!(outcome.records.len(), 1);
    let store = FigureStore::new(&base);
    let records = store.load_records()?;
    assert_eq!(records.len(), 1);
    assert!(records[0].image_path.exists());
    Ok(())
}

fn seed_entry(manager: &BaseManager, base: &researchbase::bases::Base, title: &str) {
    let entry = LibraryEntry {
        entry_id: uuid::Uuid::new_v4(),
        title: title.into(),
        authors: vec!["FigureBot".into()],
        venue: Some("FigConf".into()),
        year: Some(2023),
        identifier: format!("fig:{title}"),
        pdf_paths: vec![PathBuf::from("dummy.pdf")],
        needs_pdf: false,
        notes: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    manager
        .save_library_entries(base, &[entry])
        .expect("failed to seed entry");
}
