use crate::acquisition::figure_store::{FigureAssetRecord, FigureStore};
use crate::bases::{Base, BaseManager, LibraryEntry};
use crate::orchestration::{log_event, EventType};
use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

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
        html.push_str(&format!("<h2>{}</h2><ul>", cat.name));
        for entry in &cat.entries {
            html.push_str(&format!("<li><strong>{}</strong>", entry.title));
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
    let categories = generate_category_reports(entries);
    let figure_map = load_figures_grouped(base)?;
    let cat_path = write_category_report(base, &categories, &figure_map)?;
    let global_path = write_global_report(base, entries, &figure_map)?;
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
