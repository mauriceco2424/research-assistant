use std::collections::HashSet;

use anyhow::{Context, Result};
use chrono::Utc;
use uuid::Uuid;

use super::categories::{
    category_slug, CategoryAssignment, CategoryAssignmentsIndex, CategoryDefinition,
    CategoryNarrative, CategoryOrigin, CategoryRecord, CategoryStore,
};

pub struct MergeOptions {
    pub source_ids: Vec<Uuid>,
    pub target_name: String,
    pub keep_pinned: bool,
}

pub struct MergeOutcome {
    pub merged_category: CategoryRecord,
    pub merged_ids: Vec<Uuid>,
}

pub fn merge_categories(
    store: &CategoryStore,
    assignments: &CategoryAssignmentsIndex,
    options: MergeOptions,
) -> Result<MergeOutcome> {
    if options.source_ids.len() < 2 {
        anyhow::bail!("Provide at least two categories to merge.");
    }
    let mut merged_ids = Vec::new();
    let mut representatives = Vec::new();
    let mut pinned = Vec::new();
    let mut summary_notes = Vec::new();
    let mut member_assignments: Vec<CategoryAssignment> = Vec::new();
    let mut reference_ids: HashSet<Uuid> = HashSet::new();

    for category_id in &options.source_ids {
        let record = store
            .get(category_id)?
            .with_context(|| format!("Category {:?} not found", category_id))?;
        merged_ids.push(*category_id);
        representatives.extend(record.definition.representative_papers.clone());
        if options.keep_pinned {
            pinned.extend(record.definition.pinned_papers.clone());
        }
        summary_notes.push(record.narrative.summary.clone());
        for reference in &record.narrative.references {
            reference_ids.insert(*reference);
        }
        let assignments_for_category = assignments.list_for_category(category_id)?;
        member_assignments.extend(assignments_for_category);
    }

    let target_id = Uuid::new_v4();
    for assignment in &mut member_assignments {
        assignment.category_id = target_id;
    }
    let mut dedup_pinned: Vec<Uuid> = Vec::new();
    for paper_id in pinned {
        if !dedup_pinned.contains(&paper_id) {
            dedup_pinned.push(paper_id);
        }
    }

    let definition = CategoryDefinition {
        category_id: target_id,
        base_id: store.base_id(),
        name: options.target_name.clone(),
        slug: category_slug(&options.target_name),
        description: format!("Merged category from {} groups.", options.source_ids.len()),
        confidence: None,
        representative_papers: representatives.into_iter().take(5).collect(),
        pinned_papers: dedup_pinned,
        figure_gallery_enabled: false,
        origin: CategoryOrigin::Manual,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let narrative = CategoryNarrative {
        narrative_id: Uuid::new_v4(),
        category_id: target_id,
        summary: summary_notes.join("\n\n"),
        learning_prompts: Vec::new(),
        notes: Vec::new(),
        references: reference_ids.into_iter().collect(),
        ai_assisted: false,
        last_updated_at: Utc::now(),
    };
    let merged_record = CategoryRecord::new(definition, narrative);
    store.save(&merged_record)?;
    assignments.replace_category(&target_id, &member_assignments)?;
    for source_id in &options.source_ids {
        store.delete(source_id)?;
    }
    Ok(MergeOutcome {
        merged_category: merged_record,
        merged_ids,
    })
}
