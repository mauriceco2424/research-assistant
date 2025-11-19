use super::IntegrationHarness;
use anyhow::Result;
use chrono::{Duration, Utc};
use researchbase::bases::{
    category_slug, AssignmentSource, AssignmentStatus, BaseManager, CategoryAssignment,
    CategoryAssignmentsIndex, CategoryDefinition, CategoryNarrative, CategoryOrigin, CategoryRecord,
    CategoryStore, LibraryEntry,
};
use researchbase::chat::ChatSession;
use uuid::Uuid;

#[test]
fn categories_status_reports_alerts_and_backlog() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "categories-status");
    seed_entries(&manager, &base)?;
    seed_categories(&base)?;

    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;
    let status = chat.categories_status(true)?;
    assert!(
        status.contains("Alerts"),
        "Status response should include alert summary:\n{status}"
    );
    assert!(
        status.contains("Backlog"),
        "Status response should include backlog segments:\n{status}"
    );
    Ok(())
}

fn seed_entries(manager: &BaseManager, base: &researchbase::bases::Base) -> Result<()> {
    let mut entries = Vec::new();
    for idx in 0..6 {
        entries.push(LibraryEntry {
            entry_id: Uuid::new_v4(),
            title: format!("Status Paper {}", idx),
            authors: vec![format!("Researcher {}", idx)],
            venue: Some(if idx < 3 { "Unsorted" } else { "Applied" }.into()),
            year: Some(2018 + idx as i32),
            identifier: format!("status:{}", idx),
            pdf_paths: Vec::new(),
            needs_pdf: true,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
    }
    manager.save_library_entries(base, &entries)?;
    Ok(())
}

fn seed_categories(base: &researchbase::bases::Base) -> Result<()> {
    let store = CategoryStore::new(base)?;
    let assignments = CategoryAssignmentsIndex::new(base)?;
    let mut category_ids = Vec::new();
    for name in ["Legacy", "Applied"] {
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
                created_at: Utc::now() - Duration::days(40),
                updated_at: Utc::now() - Duration::days(40),
            },
            CategoryNarrative {
                narrative_id: Uuid::new_v4(),
                category_id,
                summary: format!("Narrative for {}", name),
                learning_prompts: Vec::new(),
                notes: Vec::new(),
                references: Vec::new(),
                ai_assisted: false,
                last_updated_at: Utc::now() - Duration::days(40),
            },
        );
        store.save(&record)?;
        category_ids.push(category_id);
    }

    let old_assignment = CategoryAssignment {
        assignment_id: Uuid::new_v4(),
        category_id: category_ids[0],
        paper_id: Uuid::new_v4(),
        source: AssignmentSource::Manual,
        confidence: 1.0,
        status: AssignmentStatus::PendingReview,
        last_reviewed_at: Some(Utc::now() - Duration::days(45)),
    };
    assignments.replace_category(&category_ids[0], &[old_assignment])?;

    let mut applied_assignments = Vec::new();
    for _ in 0..2 {
        applied_assignments.push(CategoryAssignment {
            assignment_id: Uuid::new_v4(),
            category_id: category_ids[1],
            paper_id: Uuid::new_v4(),
            source: AssignmentSource::Manual,
            confidence: 1.0,
            status: AssignmentStatus::Active,
            last_reviewed_at: Some(Utc::now()),
        });
    }
    assignments.replace_category(&category_ids[1], &applied_assignments)?;
    Ok(())
}
