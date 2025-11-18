use crate::bases::{Base, BaseManager, LibraryEntry};
use crate::orchestration::{log_event, EventType};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Category report representation.
#[derive(Debug, Clone)]
pub struct CategoryReport {
    pub name: String,
    pub entries: Vec<LibraryEntry>,
}

/// Generates naive categories based on venue or first letter.
pub fn generate_category_reports(entries: &[LibraryEntry]) -> Vec<CategoryReport> {
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
        .map(|(name, entries)| CategoryReport { name, entries })
        .collect()
}

pub fn write_category_report(base: &Base, categories: &[CategoryReport]) -> Result<PathBuf> {
    let report_dir = base.user_layer_path.join("reports");
    fs::create_dir_all(&report_dir)?;
    let path = report_dir.join("category_report.html");
    let mut html = String::new();
    html.push_str("<html><body><h1>Category Report</h1>");
    for cat in categories {
        html.push_str(&format!("<h2>{}</h2><ul>", cat.name));
        for entry in &cat.entries {
            html.push_str(&format!("<li>{}</li>", entry.title));
        }
        html.push_str("</ul>");
    }
    html.push_str("</body></html>");
    fs::write(&path, html)?;
    Ok(path)
}

pub fn write_global_report(base: &Base, entries: &[LibraryEntry]) -> Result<PathBuf> {
    let report_dir = base.user_layer_path.join("reports");
    fs::create_dir_all(&report_dir)?;
    let path = report_dir.join("global_report.html");
    let mut html = String::new();
    html.push_str("<html><body><h1>Global Report</h1><ul>");
    for entry in entries {
        html.push_str(&format!(
            "<li><strong>{}</strong> ({})</li>",
            entry.title,
            entry
                .year
                .map(|y| y.to_string())
                .unwrap_or_else(|| "n.d.".into())
        ));
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
    let categories = generate_category_reports(entries);
    let cat_path = write_category_report(base, &categories)?;
    let global_path = write_global_report(base, entries)?;
    log_event(
        manager,
        base,
        EventType::ReportsGenerated,
        serde_json::json!({
            "category_report": cat_path,
            "global_report": global_path,
        }),
    )?;
    Ok((cat_path, global_path))
}
