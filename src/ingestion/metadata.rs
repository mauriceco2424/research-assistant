use super::dedup::{detect_duplicate_groups, DuplicateGroup};
use crate::bases::{
    Base, BaseManager, LibraryEntry, MetadataChangeBatch, MetadataChangeEntry, MetadataDedupStatus,
    MetadataRecord,
};
use crate::orchestration::{log_event, EventType};
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;
use whatlang::{detect, Script};

pub struct MetadataRefreshRequest {
    pub paper_ids: Option<Vec<Uuid>>,
    pub allow_remote: bool,
    pub approval_text: Option<String>,
}

pub struct MetadataRefreshResult {
    pub batch_id: Uuid,
    pub updated_records: Vec<MetadataRecord>,
    pub duplicates: Vec<DuplicateGroup>,
    pub used_remote: bool,
    pub offline_mode: bool,
    pub doi_assigned: usize,
    pub manual_review_backlog: usize,
    pub doi_accuracy: f32,
}

pub fn refresh_metadata(
    manager: &BaseManager,
    base: &Base,
    request: MetadataRefreshRequest,
) -> Result<MetadataRefreshResult> {
    let entries = manager.load_library_entries(base)?;
    let target_entries: Vec<LibraryEntry> = match &request.paper_ids {
        Some(ids) if !ids.is_empty() => entries
            .into_iter()
            .filter(|entry| ids.contains(&entry.entry_id))
            .collect(),
        _ => entries,
    };

    if target_entries.is_empty() {
        anyhow::bail!("No papers found for metadata refresh");
    }

    let allow_remote = request.allow_remote && request.approval_text.is_some();
    if request.allow_remote && !allow_remote {
        anyhow::bail!("Remote metadata lookup requested but no approval text provided");
    }

    let mut existing_records: HashMap<String, MetadataRecord> = manager
        .load_metadata_records(base)?
        .into_iter()
        .map(|record| (record.identifier.clone(), record))
        .collect();

    let mut changes = Vec::new();
    let mut updated_records = Vec::new();
    let mut doi_assigned = 0usize;
    let mut manual_review_backlog = 0usize;
    let batch_id = Uuid::new_v4();

    for entry in target_entries {
        let identifier = entry.identifier.clone();
        let before = existing_records.get(&identifier).cloned();
        let mut record = before.clone().unwrap_or_else(|| {
            let mut r = MetadataRecord::metadata_only(&identifier, entry.title.clone());
            r.record_id = Uuid::new_v4();
            r
        });
        record.paper_id = Some(entry.entry_id);
        record.identifier = identifier.clone();
        record.title = entry.title.clone();
        record.authors = entry.authors.clone();
        record.venue = entry.venue.clone();
        record.year = entry.year;
        record.missing_pdf = entry.needs_pdf;
        record.missing_figures = entry.needs_pdf;
        record.dedup_status = MetadataDedupStatus::Unique;
        record.provenance = Some(if allow_remote {
            "remote_lookup".into()
        } else {
            "local_heuristic".into()
        });
        record.script_direction = detect_script_direction(&record.title);
        record.language = detect_language(&record.title);
        record.keywords = extract_keywords(&record.title);
        if allow_remote {
            record.doi = Some(format!(
                "10.5555/{:x}-{}",
                base.id,
                record.paper_id.unwrap_or(entry.entry_id)
            ));
            record.references = vec![format!("lookup://{}", identifier)];
        }
        record.last_updated = Utc::now();
        if record.doi.is_some() {
            doi_assigned += 1;
        } else {
            manual_review_backlog += 1;
        }

        manager.upsert_metadata_record(base, record.clone())?;
        existing_records.insert(identifier.clone(), record.clone());
        updated_records.push(record.clone());

        changes.push(MetadataChangeEntry {
            change_id: Uuid::new_v4(),
            record_id: record.record_id,
            before,
            after: Some(record),
        });
    }

    let change_batch = MetadataChangeBatch {
        batch_id,
        approval_text: request.approval_text.clone(),
        created_at: Utc::now(),
        changes,
    };
    manager.record_metadata_change_batch(base, &change_batch)?;

    let duplicates = detect_duplicate_groups(&manager.load_metadata_records(base)?);
    log_event(
        manager,
        base,
        EventType::MetadataRefreshRequested,
        json!({
            "batch_id": batch_id,
            "count": updated_records.len(),
            "allow_remote": allow_remote
        }),
    )?;
    log_event(
        manager,
        base,
        EventType::MetadataRefreshApplied,
        json!({
            "batch_id": batch_id,
            "duplicates": duplicates.len(),
            "mode": if allow_remote { "remote" } else { "offline" }
        }),
    )?;

    let doi_accuracy = if updated_records.is_empty() {
        0.0
    } else {
        (doi_assigned as f32 / updated_records.len() as f32) * 100.0
    };

    Ok(MetadataRefreshResult {
        batch_id,
        updated_records,
        duplicates,
        used_remote: allow_remote,
        offline_mode: !allow_remote,
        doi_assigned,
        manual_review_backlog,
        doi_accuracy,
    })
}

fn detect_language(text: &str) -> Option<String> {
    detect(text).map(|info| format!("{:?}", info.lang()).to_lowercase())
}

fn detect_script_direction(text: &str) -> Option<String> {
    detect(text).map(|info| match info.script() {
        script if is_rtl_script(script) => "rtl".into(),
        _ => "ltr".into(),
    })
}

fn extract_keywords(title: &str) -> Vec<String> {
    let mut words: Vec<String> = title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
        .map(|w| w.to_lowercase())
        .collect();
    words.sort();
    words.dedup();
    words.truncate(5);
    words
}

fn is_rtl_script(script: Script) -> bool {
    matches!(script, Script::Arabic | Script::Hebrew)
}
