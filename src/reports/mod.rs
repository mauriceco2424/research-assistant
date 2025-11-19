pub mod build_service;
pub mod categorization;
pub mod config_store;
pub mod consent_registry;
pub mod figure_gallery;
pub mod html_renderer;
pub mod manifest;
pub mod manifest_writer;
pub mod share_builder;
pub mod share_manifest;
pub mod share_service;
pub mod visualizations;

use crate::acquisition::figure_store::{FigureAssetRecord, FigureStore};
use crate::bases::{Base, BaseManager, CategoryAssignmentsIndex, CategoryStore, LibraryEntry};
use crate::orchestration::{
    log_event, EventType, MetricRecord, OrchestrationLog, ReportMetricsRecord, REPORT_SLA_SECS,
};
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

/// Category report representation.
#[derive(Debug, Clone)]
pub struct CategoryReport {
    pub name: String,
    pub description: Option<String>,
    pub narrative_summary: Option<String>,
    pub entries: Vec<LibraryEntry>,
    pub pinned_entries: Vec<LibraryEntry>,
    pub figure_gallery_enabled: bool,
}

fn fallback_category_reports(entries: &[LibraryEntry]) -> Vec<CategoryReport> {
    let mut buckets: BTreeMap<String, Vec<LibraryEntry>> = BTreeMap::new();
    for entry in entries {
        let key = entry
            .venue
            .clone()
            .unwrap_or_else(|| entry.title.chars().next().unwrap_or('U').to_string());
        buckets.entry(key).or_default().push(entry.clone());
    }
    buckets
        .into_iter()
        .map(|(name, mut entries)| {
            entries.sort_by(|a, b| a.title.cmp(&b.title));
            CategoryReport {
                name,
                description: None,
                narrative_summary: None,
                entries,
                pinned_entries: Vec::new(),
                figure_gallery_enabled: false,
            }
        })
        .collect()
}

fn load_category_reports(base: &Base, entries: &[LibraryEntry]) -> Result<Vec<CategoryReport>> {
    let store = CategoryStore::new(base)?;
    let records = store.list()?;
    if records.is_empty() {
        return Ok(fallback_category_reports(entries));
    }
    let assignments_index = CategoryAssignmentsIndex::new(base)?;
    let assignments = assignments_index.list_all()?;
    let entries_by_id: HashMap<Uuid, LibraryEntry> = entries
        .iter()
        .map(|entry| (entry.entry_id, entry.clone()))
        .collect();
    let mut grouped: HashMap<Uuid, Vec<LibraryEntry>> = HashMap::new();
    for assignment in assignments {
        if let Some(entry) = entries_by_id.get(&assignment.paper_id) {
            grouped
                .entry(assignment.category_id)
                .or_default()
                .push(entry.clone());
        }
    }
    let mut reports = Vec::new();
    for record in records {
        let mut assigned = grouped
            .remove(&record.definition.category_id)
            .unwrap_or_default();
        assigned.sort_by(|a, b| a.title.cmp(&b.title));
        let mut pinned = Vec::new();
        if !record.definition.pinned_papers.is_empty() {
            for pinned_id in &record.definition.pinned_papers {
                if let Some(pos) = assigned.iter().position(|e| &e.entry_id == pinned_id) {
                    pinned.push(assigned.remove(pos));
                } else if let Some(entry) = entries_by_id.get(pinned_id) {
                    pinned.push(entry.clone());
                }
            }
        }
        reports.push(CategoryReport {
            name: record.definition.name.clone(),
            description: Some(record.definition.description.clone()),
            narrative_summary: Some(record.narrative.summary.clone()),
            entries: assigned,
            pinned_entries: pinned,
            figure_gallery_enabled: record.definition.figure_gallery_enabled,
        });
    }
    Ok(reports)
}

pub fn write_category_report(
    base: &Base,
    categories: &[CategoryReport],
    figure_map: &HashMap<Uuid, Vec<FigureAssetRecord>>,
) -> Result<PathBuf> {
    let report_dir = base.user_layer_path.join("reports");
    fs::create_dir_all(&report_dir)?;
    let path = report_dir.join("category_report.html");
    let mut html = String::new();
    html.push_str("<html><body><h1>Category Report</h1>");
    for cat in categories {
        html.push_str(&format!(
            "<section class=\"category\"><h2>{}</h2>",
            cat.name
        ));
        if let Some(desc) = &cat.description {
            if !desc.is_empty() {
                html.push_str(&format!("<p class=\"description\">{}</p>", desc));
            }
        }
        if let Some(summary) = &cat.narrative_summary {
            if !summary.is_empty() {
                html.push_str(&format!(
                    "<div class=\"narrative\"><strong>Narrative:</strong><p>{}</p></div>",
                    summary
                ));
            }
        }
        if !cat.pinned_entries.is_empty() {
            html.push_str("<div class=\"pinned\"><h3>Pinned Papers</h3>");
            append_entry_list_html(&mut html, &cat.pinned_entries, figure_map);
            html.push_str("</div>");
        }
        append_entry_list_html(&mut html, &cat.entries, figure_map);
        html.push_str("</section>");
    }
    html.push_str("</body></html>");
    fs::write(&path, html)?;
    Ok(path)
}

pub fn write_global_report(
    base: &Base,
    entries: &[LibraryEntry],
    figure_map: &HashMap<Uuid, Vec<FigureAssetRecord>>,
) -> Result<PathBuf> {
    let report_dir = base.user_layer_path.join("reports");
    fs::create_dir_all(&report_dir)?;
    let path = report_dir.join("global_report.html");
    let mut html = String::new();
    html.push_str("<html><body><h1>Global Report</h1><ul>");
    for entry in entries {
        html.push_str(&format!(
            "<li><strong>{}</strong> ({})",
            entry.title,
            entry
                .year
                .map(|y| y.to_string())
                .unwrap_or_else(|| "n.d.".into())
        ));
        if let Some(figures) = figure_map.get(&entry.entry_id) {
            html.push_str("<div class=\"figures\">");
            for figure in figures {
                html.push_str(&render_figure_html(figure));
            }
            html.push_str("</div>");
        }
        html.push_str("</li>");
    }
    html.push_str("</ul></body></html>");
    fs::write(&path, html)?;
    Ok(path)
}

pub fn generate_and_log_reports(
    manager: &BaseManager,
    base: &Base,
    entries: &[LibraryEntry],
) -> Result<(PathBuf, PathBuf)> {
    let started = Instant::now();
    let categories = load_category_reports(base, entries)?;
    let figure_map = load_figures_grouped(base)?;
    let cat_path = write_category_report(base, &categories, &figure_map)?;
    let global_path = write_global_report(base, entries, &figure_map)?;
    let duration_ms = started.elapsed().as_millis() as i64;
    let figure_count: usize = figure_map.values().map(|items| items.len()).sum();
    let sla_breached = duration_ms > REPORT_SLA_SECS * 1000;
    log_event(
        manager,
        base,
        EventType::ReportsGenerated,
        serde_json::json!({
            "category_report": cat_path,
            "global_report": global_path,
            "duration_ms": duration_ms,
            "sla_breached": sla_breached,
        }),
    )?;
    let log = OrchestrationLog::for_base(base);
    log.record_metric(&MetricRecord::Reports(ReportMetricsRecord {
        duration_ms,
        entry_count: entries.len(),
        figure_count,
        sla_breached,
    }))?;
    Ok((cat_path, global_path))
}

fn load_figures_grouped(base: &Base) -> Result<HashMap<Uuid, Vec<FigureAssetRecord>>> {
    let store = FigureStore::new(base);
    let mut map: HashMap<Uuid, Vec<FigureAssetRecord>> = HashMap::new();
    for record in store.load_records()? {
        map.entry(record.paper_id).or_default().push(record);
    }
    Ok(map)
}

fn render_figure_html(figure: &FigureAssetRecord) -> String {
    let path = figure.image_path.to_string_lossy();
    format!(
        "<figure><figcaption>{}</figcaption><a href=\"file://{path}\">{}</a></figure>",
        figure.caption, path
    )
}

fn append_entry_list_html(
    html: &mut String,
    entries: &[LibraryEntry],
    figure_map: &HashMap<Uuid, Vec<FigureAssetRecord>>,
) {
    html.push_str("<ul>");
    for entry in entries {
        html.push_str(&format!("<li><strong>{}</strong>", entry.title));
        if let Some(year) = entry.year {
            html.push_str(&format!(" <em>({})</em>", year));
        }
        if let Some(figures) = figure_map.get(&entry.entry_id) {
            html.push_str("<div class=\"figures\">");
            for figure in figures {
                html.push_str(&render_figure_html(figure));
            }
            html.push_str("</div>");
        }
        html.push_str("</li>");
    }
    html.push_str("</ul>");
}
