use super::IntegrationHarness;
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use researchbase::bases::{
    category_slug, Base, BaseManager, CategoryAssignment, CategoryAssignmentsIndex,
    CategoryDefinition, CategoryNarrative, CategoryOrigin, CategoryRecord, CategorySnapshotStore,
    CategoryStore, LibraryEntry,
};
use researchbase::chat::commands::reports::ReportRegenerateOptions;
use researchbase::chat::ChatSession;
use std::fs;
use uuid::Uuid;

#[test]
fn stale_snapshot_blocks_regeneration() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "reports-stale");
    let entries = seed_entries(&manager, &base)?;
    seed_categories(&base, &entries)?;
    capture_snapshot(&base)?;
    age_latest_snapshot(&base, Duration::days(90))?;

    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;
    let outcome = chat.reports_regenerate(ReportRegenerateOptions::default());
    assert!(outcome.is_err(), "Expected stale snapshot error");
    let message = format!("{}", outcome.err().unwrap());
    assert!(
        message.contains("snapshot"),
        "Expected stale snapshot guidance, got {message}"
    );
    Ok(())
}

#[test]
fn fail_fast_restores_previous_outputs() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "reports-fail-fast");
    let entries = seed_entries(&manager, &base)?;
    seed_categories(&base, &entries)?;
    capture_snapshot(&base)?;

    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;
    chat.reports_regenerate(ReportRegenerateOptions::default())?;
    let global_path = base.user_layer_path.join("reports").join("global.html");
    let baseline = fs::read_to_string(&global_path).context("missing baseline report")?;

    std::env::set_var("REPORTS_FORCE_DISK_ERROR", "1");
    let mut options = ReportRegenerateOptions::default();
    options.overwrite_existing = true;
    let outcome = chat.reports_regenerate(options);
    assert!(outcome.is_err(), "Expected simulated disk error");
    let after = fs::read_to_string(&global_path).context("global report missing after error")?;
    assert_eq!(
        baseline, after,
        "Global report should remain untouched after fail-fast rollback"
    );
    std::env::remove_var("REPORTS_FORCE_DISK_ERROR");
    Ok(())
}

#[test]
fn overwrite_confirmation_preserves_outputs() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "reports-overwrite");
    let entries = seed_entries(&manager, &base)?;
    seed_categories(&base, &entries)?;
    capture_snapshot(&base)?;

    let mut chat = ChatSession::new()?;
    chat.select_base(&base.id)?;
    chat.reports_regenerate(ReportRegenerateOptions::default())?;
    let global_path = base.user_layer_path.join("reports").join("global.html");
    let baseline = fs::read_to_string(&global_path).context("baseline missing")?;

    let outcome = chat.reports_regenerate(ReportRegenerateOptions::default());
    assert!(outcome.is_err(), "Expected overwrite confirmation error");
    let after = fs::read_to_string(&global_path).context("global missing after guard")?;
    assert_eq!(
        baseline, after,
        "Global report should remain unchanged when overwrite not confirmed"
    );
    Ok(())
}

fn seed_entries(manager: &BaseManager, base: &Base) -> Result<Vec<LibraryEntry>> {
    let entries: Vec<LibraryEntry> = (0..4)
        .map(|idx| LibraryEntry {
            entry_id: Uuid::new_v4(),
            title: format!("Seed Paper {idx}"),
            authors: vec![format!("Author {idx}")],
            venue: Some(if idx < 2 { "Foundations" } else { "Applications" }.into()),
            year: Some(2020 + idx as i32),
            identifier: format!("seed:{idx}"),
            pdf_paths: Vec::new(),
            needs_pdf: true,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .collect();
    manager.save_library_entries(base, &entries)?;
    Ok(entries)
}

fn seed_categories(base: &Base, entries: &[LibraryEntry]) -> Result<()> {
    let store = CategoryStore::new(base)?;
    let assignments = CategoryAssignmentsIndex::new(base)?;
    for (index, name) in ["Foundations", "Applications"].iter().enumerate() {
        let category_id = Uuid::new_v4();
        let record = CategoryRecord::new(
            CategoryDefinition {
                category_id,
                base_id: base.id,
                name: name.to_string(),
                slug: category_slug(name),
                description: format!("{name} category"),
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
                summary: format!("Narrative for {name}"),
                learning_prompts: Vec::new(),
                notes: Vec::new(),
                references: Vec::new(),
                ai_assisted: false,
                last_updated_at: Utc::now(),
            },
        );
        store.save(&record)?;
        let paper = &entries[index * 2];
        let secondary = &entries[index * 2 + 1];
        let assigned = vec![
            CategoryAssignment {
                assignment_id: Uuid::new_v4(),
                category_id,
                paper_id: paper.entry_id,
                source: researchbase::bases::AssignmentSource::Manual,
                confidence: 1.0,
                status: researchbase::bases::AssignmentStatus::Active,
                last_reviewed_at: None,
            },
            CategoryAssignment {
                assignment_id: Uuid::new_v4(),
                category_id,
                paper_id: secondary.entry_id,
                source: researchbase::bases::AssignmentSource::Manual,
                confidence: 1.0,
                status: researchbase::bases::AssignmentStatus::Active,
                last_reviewed_at: None,
            },
        ];
        assignments.replace_category(&category_id, &assigned)?;
    }
    Ok(())
}

fn capture_snapshot(base: &Base) -> Result<()> {
    let store = CategoryStore::new(base)?;
    let assignments = CategoryAssignmentsIndex::new(base)?;
    let snapshot_store = CategorySnapshotStore::new(base)?;
    snapshot_store.capture(&store, &assignments, "test snapshot")?;
    Ok(())
}

fn age_latest_snapshot(base: &Base, age: Duration) -> Result<()> {
    let snapshots_dir = base
        .ai_layer_path
        .join("categories")
        .join("snapshots");
    let mut entries = fs::read_dir(&snapshots_dir)?
        .filter_map(|item| item.ok())
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    if let Some(entry) = entries.first() {
        let path = entry.path();
        let data = fs::read_to_string(&path)?;
        let mut json: serde_json::Value = serde_json::from_str(&data)?;
        if let Some(snapshot) = json.get_mut("snapshot") {
            snapshot["taken_at"] = serde_json::Value::String(
                (Utc::now() - age).to_rfc3339(),
            );
        }
        fs::write(&path, serde_json::to_string_pretty(&json)?)?;
    }
    Ok(())
}
