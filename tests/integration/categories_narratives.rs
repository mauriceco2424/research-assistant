use super::IntegrationHarness;
use anyhow::Result;
use chrono::Utc;
use researchbase::bases::{
    category_slug, AssignmentSource, AssignmentStatus, BaseManager, CategoryAssignment,
    CategoryAssignmentsIndex, CategoryDefinition, CategoryNarrative, CategoryOrigin, CategoryRecord,
    CategoryStore, LibraryEntry,
};
use researchbase::chat::ChatSession;
use uuid::Uuid;

#[test]
fn edit_narrative_and_pins_updates_reports() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "categories-narratives");
    let paper_ids = seed_entries(&manager, &base)?;
    let category_id = seed_category(&base, &paper_ids)?;

    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;
    chat.category_narrative(
        "Narratives",
        Some("Updated summary".into()),
        Some(vec!["Prompt 1".into(), "Prompt 2".into()]),
        Some(vec!["Note A".into()]),
        Some(vec![paper_ids[0], paper_ids[1]]),
        Some(true),
        None,
    )?;
    chat.category_pin("Narratives", paper_ids[2], true)?;

    let store = CategoryStore::new(&base)?;
    let updated = store.get(&category_id)?.expect("category exists");
    assert_eq!(updated.narrative.summary, "Updated summary");
    assert_eq!(updated.narrative.learning_prompts.len(), 2);
    assert!(updated.definition.figure_gallery_enabled);
    assert_eq!(updated.definition.pinned_papers.len(), 3);

    let report_path = base.user_layer_path.join("reports/category_report.html");
    assert!(
        report_path.exists(),
        "Report should regenerate after narrative edits"
    );
    Ok(())
}

fn seed_entries(
    manager: &BaseManager,
    base: &researchbase::bases::Base,
) -> Result<Vec<Uuid>> {
    let mut ids = Vec::new();
    let mut entries = Vec::new();
    for idx in 0..4 {
        let entry_id = Uuid::new_v4();
        ids.push(entry_id);
        entries.push(LibraryEntry {
            entry_id,
            title: format!("Narrative Paper {}", idx),
            authors: vec![format!("Author {}", idx)],
            venue: Some("Narrative".into()),
            year: Some(2021),
            identifier: format!("narrative:{}", idx),
            pdf_paths: Vec::new(),
            needs_pdf: true,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
    }
    manager.save_library_entries(base, &entries)?;
    Ok(ids)
}

fn seed_category(
    base: &researchbase::bases::Base,
    paper_ids: &[Uuid],
) -> Result<Uuid> {
    let store = CategoryStore::new(base)?;
    let assignments = CategoryAssignmentsIndex::new(base)?;
    let category_id = Uuid::new_v4();
    let record = CategoryRecord::new(
        CategoryDefinition {
            category_id,
            base_id: base.id,
            name: "Narratives".into(),
            slug: category_slug("Narratives"),
            description: "Seed category".into(),
            confidence: None,
            representative_papers: Vec::new(),
            pinned_papers: Vec::new(),
            figure_gallery_enabled: false,
            origin: CategoryOrigin::Manual,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        CategoryNarrative {
            narrative_id: Uuid::new_v4(),
            category_id,
            summary: "Initial summary".into(),
            learning_prompts: Vec::new(),
            notes: Vec::new(),
            references: Vec::new(),
            ai_assisted: false,
            last_updated_at: Utc::now(),
        },
    );
    store.save(&record)?;
    let assignments_list: Vec<CategoryAssignment> = paper_ids
        .iter()
        .map(|paper_id| CategoryAssignment {
            assignment_id: Uuid::new_v4(),
            category_id,
            paper_id: *paper_id,
            source: AssignmentSource::Manual,
            confidence: 1.0,
            status: AssignmentStatus::Active,
            last_reviewed_at: Some(Utc::now()),
        })
        .collect();
    assignments.replace_category(&category_id, &assignments_list)?;
    Ok(category_id)
}
