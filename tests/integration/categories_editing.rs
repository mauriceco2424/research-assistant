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
fn rename_merge_and_undo_flow() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "categories-editing");
    seed_entries(&manager, &base)?;
    seed_categories(&base)?;

    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;

    chat.category_rename("Methods", "Foundations")?;
    let merge_msg =
        chat.category_merge(vec!["Foundations".into(), "Applications".into()], "Platform", true)?;
    assert!(
        merge_msg.contains("Merged"),
        "Unexpected merge response: {merge_msg}"
    );
    chat.category_undo()?;

    let store = CategoryStore::new(&base)?;
    let names: Vec<String> = store
        .list()?
        .into_iter()
        .map(|record| record.definition.name)
        .collect();
    assert!(names.contains(&"Foundations".to_string()));
    assert!(names.contains(&"Applications".to_string()));
    assert!(
        !names.contains(&"Platform".to_string()),
        "Platform category should have been rolled back"
    );
    Ok(())
}

fn seed_entries(manager: &BaseManager, base: &researchbase::bases::Base) -> Result<()> {
    let entries: Vec<LibraryEntry> = (0..4)
        .map(|idx| LibraryEntry {
            entry_id: Uuid::new_v4(),
            title: format!("Seed Paper {}", idx),
            authors: vec![format!("Author {}", idx)],
            venue: Some(if idx < 2 { "Methods" } else { "Applications" }.into()),
            year: Some(2020 + idx as i32),
            identifier: format!("seed:{}", idx),
            pdf_paths: Vec::new(),
            needs_pdf: true,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .collect();
    manager.save_library_entries(base, &entries)?;
    Ok(())
}

fn seed_categories(base: &researchbase::bases::Base) -> Result<()> {
    let store = CategoryStore::new(base)?;
    let assignments = CategoryAssignmentsIndex::new(base)?;
    for name in ["Methods", "Applications"] {
        let category_id = Uuid::new_v4();
        let record = CategoryRecord::new(
            CategoryDefinition {
                category_id,
                base_id: base.id,
                name: name.to_string(),
                slug: category_slug(name),
                description: format!("{} category", name),
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
                summary: format!("Narrative for {}", name),
                learning_prompts: Vec::new(),
                notes: Vec::new(),
                references: Vec::new(),
                ai_assisted: false,
                last_updated_at: Utc::now(),
            },
        );
        store.save(&record)?;
        let dummy_assignments = vec![CategoryAssignment {
            assignment_id: Uuid::new_v4(),
            category_id,
            paper_id: Uuid::new_v4(),
            source: AssignmentSource::Manual,
            confidence: 1.0,
            status: AssignmentStatus::Active,
            last_reviewed_at: None,
        }];
        assignments.replace_category(&category_id, &dummy_assignments)?;
    }
    Ok(())
}
