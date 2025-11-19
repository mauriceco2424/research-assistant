use super::IntegrationHarness;
use anyhow::Result;
use chrono::Utc;
use researchbase::bases::{
    BaseManager, CategoryAssignmentsIndex, CategoryProposalStore, CategoryStore, LibraryEntry,
};
use researchbase::chat::ChatSession;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn categories_propose_and_apply_flow() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "categorization-flow");
    seed_entries(&manager, &base);

    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;
    let response = chat.categories_propose(None)?;
    assert!(
        response.contains("Generated"),
        "Unexpected response: {response}"
    );

    let store = CategoryProposalStore::new(&base)?;
    let batch = store
        .latest_batch()?
        .expect("proposal batch should exist after running categories_propose");
    assert!(!batch.proposals.is_empty());

    let first_id = batch.proposals[0].proposal_id;
    let mut renames = HashMap::new();
    renames.insert(first_id, "Renamed Cluster".to_string());
    let apply_msg = chat.categories_apply(vec![first_id], renames, Vec::new())?;
    assert!(
        apply_msg.contains("Applied 1 proposals"),
        "Unexpected apply response: {apply_msg}"
    );

    let category_store = CategoryStore::new(&base)?;
    let categories = category_store.list()?;
    assert_eq!(categories.len(), 1);
    assert!(categories[0].definition.name.contains("Renamed"));

    let assignments = CategoryAssignmentsIndex::new(&base)?;
    let assigned = assignments.list_for_category(&categories[0].definition.category_id)?;
    assert!(!assigned.is_empty());

    let report_path = base.user_layer_path.join("reports/category_report.html");
    assert!(
        report_path.exists(),
        "Category report should be regenerated after applying proposals"
    );
    Ok(())
}

fn seed_entries(manager: &BaseManager, base: &researchbase::bases::Base) {
    let mut entries = Vec::new();
    for idx in 0..6 {
        entries.push(LibraryEntry {
            entry_id: Uuid::new_v4(),
            title: format!("Paper {idx}"),
            authors: vec![format!("Author {}", idx % 2)],
            venue: Some(if idx < 3 { "ClusterA" } else { "ClusterB" }.into()),
            year: Some(2020 + idx as i32),
            identifier: format!("paper:{idx}"),
            pdf_paths: vec![PathBuf::from(format!("paper_{idx}.pdf"))],
            needs_pdf: false,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
    }
    manager
        .save_library_entries(base, &entries)
        .expect("failed to seed library entries");
}
