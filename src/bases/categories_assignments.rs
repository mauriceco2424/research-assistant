use std::collections::HashSet;

use anyhow::Result;
use uuid::Uuid;

use super::categories::{
    AssignmentSource, AssignmentStatus, CategoryAssignment, CategoryAssignmentsIndex,
};

/// Moves or assigns papers to a target category, optionally removing them from previous categories.
pub fn move_papers(
    index: &CategoryAssignmentsIndex,
    target_category_id: &Uuid,
    paper_ids: &[Uuid],
    remove_from_previous: bool,
) -> Result<()> {
    if paper_ids.is_empty() {
        return Ok(());
    }
    let mut paper_set: HashSet<Uuid> = paper_ids.iter().cloned().collect();
    if remove_from_previous {
        remove_papers_from_all(index, &paper_set)?;
    }
    let mut assignments = index.list_for_category(target_category_id)?;
    let existing: HashSet<Uuid> = assignments.iter().map(|a| a.paper_id).collect();
    for paper_id in paper_set.drain() {
        if existing.contains(&paper_id) {
            continue;
        }
        assignments.push(CategoryAssignment {
            assignment_id: Uuid::new_v4(),
            category_id: *target_category_id,
            paper_id,
            source: AssignmentSource::Manual,
            confidence: 1.0,
            status: AssignmentStatus::Active,
            last_reviewed_at: None,
        });
    }
    index.replace_category(target_category_id, &assignments)?;
    Ok(())
}

fn remove_papers_from_all(
    index: &CategoryAssignmentsIndex,
    paper_ids: &HashSet<Uuid>,
) -> Result<()> {
    let all_assignments = index.list_all()?;
    let mut affected: HashSet<Uuid> = HashSet::new();
    for assignment in &all_assignments {
        if paper_ids.contains(&assignment.paper_id) {
            affected.insert(assignment.category_id);
        }
    }
    for category_id in affected {
        let mut assignments = index.list_for_category(&category_id)?;
        assignments.retain(|assignment| !paper_ids.contains(&assignment.paper_id));
        index.replace_category(&category_id, &assignments)?;
    }
    Ok(())
}
