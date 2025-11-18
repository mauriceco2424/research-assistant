use crate::bases::{Base, BaseManager, MetadataRecord};
use anyhow::{Context, Result};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub doi: String,
    pub record_ids: Vec<Uuid>,
    pub titles: Vec<String>,
}

pub fn detect_duplicate_groups(records: &[MetadataRecord]) -> Vec<DuplicateGroup> {
    let mut groups: HashMap<String, DuplicateGroup> = HashMap::new();
    for record in records {
        if let Some(doi) = &record.doi {
            if doi.trim().is_empty() {
                continue;
            }
            let entry = groups.entry(doi.clone()).or_insert_with(|| DuplicateGroup {
                doi: doi.clone(),
                record_ids: Vec::new(),
                titles: Vec::new(),
            });
            entry.record_ids.push(record.record_id);
            entry.titles.push(record.title.clone());
        }
    }
    groups
        .into_values()
        .filter(|group| group.record_ids.len() > 1)
        .collect()
}

pub fn merge_duplicate_group(
    manager: &BaseManager,
    base: &Base,
    doi: &str,
    keep_record_id: Uuid,
) -> Result<usize> {
    let records = manager.load_metadata_records(base)?;
    let keep_record = records
        .iter()
        .find(|record| record.record_id == keep_record_id)
        .context("Keep record not found")?;
    if keep_record.doi.as_deref() != Some(doi) {
        anyhow::bail!("Keep record does not match the requested DOI");
    }
    let to_remove: Vec<Uuid> = records
        .iter()
        .filter(|record| record.doi.as_deref() == Some(doi) && record.record_id != keep_record_id)
        .map(|record| record.record_id)
        .collect();
    if to_remove.is_empty() {
        return Ok(0);
    }
    manager.remove_metadata_records(base, &to_remove)?;
    Ok(to_remove.len())
}

pub fn format_duplicate_group(group: &DuplicateGroup) -> String {
    let titles = group.titles.join("; ");
    format!(
        "DOI {} has {} records: {}",
        group.doi,
        group.record_ids.len(),
        titles
    )
}
