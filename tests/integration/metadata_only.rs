use super::IntegrationHarness;
use researchbase::bases::BaseManager;

#[test]
fn metadata_only_records_persist_and_reload() {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "metadata-only");

    let identifier = "doi:10.1234/example";
    let title = "Example Metadata Placeholder";
    let record = manager
        .ensure_metadata_only_record(&base, identifier, title)
        .expect("metadata-only record should be created");
    assert!(record.missing_pdf);
    assert!(record.missing_figures);
    assert_eq!(record.identifier, identifier);
    assert_eq!(record.title, title);

    // Persistence check: reload manager and ensure record remains.
    let manager = BaseManager::new().expect("manager re-init");
    let reloaded_base = manager
        .active_base()
        .expect("base should exist")
        .expect("active base should resolve");
    let records = manager
        .load_metadata_records(&reloaded_base)
        .expect("load metadata");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].identifier, identifier);
}
