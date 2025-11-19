use crate::bases::{
    category_slug, CategoryDefinition, CategoryNarrative, CategoryRecord, LibraryEntry,
};
use chrono::Utc;
use uuid::Uuid;

pub struct SplitChild {
    pub record: CategoryRecord,
    pub paper_ids: Vec<Uuid>,
}

pub struct SplitResult {
    pub parent_name: String,
    pub children: Vec<SplitChild>,
}

/// Generates two child categories by partitioning papers using a simple rule (year or author parity).
pub fn suggest_split(parent: &CategoryRecord, entries: &[LibraryEntry], rule: &str) -> SplitResult {
    let mut left_entries = Vec::new();
    let mut right_entries = Vec::new();
    for entry in entries {
        if matches_rule(entry, rule) {
            left_entries.push(entry.clone());
        } else {
            right_entries.push(entry.clone());
        }
    }
    let left_record = make_child_record(
        parent,
        &format!("{} - A", parent.definition.name),
        &left_entries,
    );
    let right_record = make_child_record(
        parent,
        &format!("{} - B", parent.definition.name),
        &right_entries,
    );
    SplitResult {
        parent_name: parent.definition.name.clone(),
        children: vec![
            SplitChild {
                record: left_record,
                paper_ids: left_entries.iter().map(|e| e.entry_id).collect(),
            },
            SplitChild {
                record: right_record,
                paper_ids: right_entries.iter().map(|e| e.entry_id).collect(),
            },
        ],
    }
}

fn matches_rule(entry: &LibraryEntry, rule: &str) -> bool {
    match rule.to_ascii_lowercase().as_str() {
        r if r.contains("year") => entry.year.unwrap_or(0) % 2 == 0,
        r if r.contains("author") => entry
            .authors
            .first()
            .map(|author| {
                author
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_uppercase())
                    .unwrap_or(false)
            })
            .unwrap_or(false),
        _ => entry.title.len() % 2 == 0,
    }
}

fn make_child_record(
    parent: &CategoryRecord,
    name: &str,
    entries: &[LibraryEntry],
) -> CategoryRecord {
    let category_id = Uuid::new_v4();
    let summary = format!("{} child capturing {} papers.", name, entries.len());
    let definition = CategoryDefinition {
        category_id,
        base_id: parent.definition.base_id,
        name: name.to_string(),
        slug: category_slug(name),
        description: summary.clone(),
        confidence: None,
        representative_papers: entries.iter().map(|entry| entry.entry_id).take(5).collect(),
        pinned_papers: Vec::new(),
        figure_gallery_enabled: parent.definition.figure_gallery_enabled,
        origin: parent.definition.origin,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let narrative = CategoryNarrative {
        narrative_id: Uuid::new_v4(),
        category_id,
        summary,
        learning_prompts: Vec::new(),
        notes: Vec::new(),
        references: entries.iter().map(|entry| entry.entry_id).take(3).collect(),
        ai_assisted: false,
        last_updated_at: Utc::now(),
    };
    CategoryRecord::new(definition, narrative)
}
