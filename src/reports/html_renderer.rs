use crate::bases::{category_slug, Base, CategoryRecord, LibraryEntry};
use crate::reports::figure_gallery::GalleryAsset;
use crate::reports::visualizations::VisualizationRenderEntry;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const REPORT_DIR: &str = "reports";
const CATEGORY_DIR: &str = "categories";

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub include_figures: bool,
    pub include_visualizations: Vec<String>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            include_figures: false,
            include_visualizations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderedFile {
    pub scope: String,
    pub staged_path: PathBuf,
    pub final_path: PathBuf,
}

impl RenderedFile {
    pub fn commit(&self) -> Result<()> {
        if let Some(parent) = self.final_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        fs::copy(&self.staged_path, &self.final_path).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                self.staged_path.display(),
                self.final_path.display()
            )
        })?;
        Ok(())
    }
}

pub struct HtmlRenderer<'a> {
    base: &'a Base,
    staging_dir: PathBuf,
    render_config: RenderConfig,
    viz_entries: Vec<VisualizationRenderEntry>,
}

impl<'a> HtmlRenderer<'a> {
    pub fn new(
        base: &'a Base,
        request_id: Uuid,
        render_config: RenderConfig,
        viz_entries: Vec<VisualizationRenderEntry>,
    ) -> Result<Self> {
        let staging_dir = base
            .user_layer_path
            .join(REPORT_DIR)
            .join(".staging")
            .join(request_id.to_string());
        fs::create_dir_all(&staging_dir)?;
        Ok(Self {
            base,
            staging_dir,
            render_config,
            viz_entries,
        })
    }

    pub fn staging_dir(&self) -> &Path {
        &self.staging_dir
    }

    pub fn render_global(
        &self,
        entries: &[LibraryEntry],
        figure_map: &HashMap<Uuid, Vec<GalleryAsset>>,
    ) -> Result<RenderedFile> {
        let mut sorted = entries.to_vec();
        sorted.sort_by(|a, b| a.title.cmp(&b.title));
        let staged_path = self.staging_dir.join("global.html");
        self.write_global_report(&staged_path, &sorted, figure_map)?;
        let final_path = self
            .base
            .user_layer_path
            .join(REPORT_DIR)
            .join("global.html");
        Ok(RenderedFile {
            scope: "global".into(),
            staged_path,
            final_path,
        })
    }

    pub fn render_category(
        &self,
        record: &CategoryRecord,
        entries: &[LibraryEntry],
        pinned: &[LibraryEntry],
        figure_map: &HashMap<Uuid, Vec<GalleryAsset>>,
    ) -> Result<RenderedFile> {
        let slug = category_slug(&record.definition.name);
        let staged_path = self
            .staging_dir
            .join(CATEGORY_DIR)
            .join(format!("{slug}.html"));
        if let Some(parent) = staged_path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.write_category_report(&staged_path, record, entries, pinned, figure_map)?;
        let final_path = self
            .base
            .user_layer_path
            .join(REPORT_DIR)
            .join(CATEGORY_DIR)
            .join(format!("{slug}.html"));
        Ok(RenderedFile {
            scope: format!("category:{}", record.definition.name),
            staged_path,
            final_path,
        })
    }

    fn write_global_report(
        &self,
        path: &Path,
        entries: &[LibraryEntry],
        figure_map: &HashMap<Uuid, Vec<GalleryAsset>>,
    ) -> Result<()> {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\" />");
        html.push_str("<title>ResearchBase – Global Report</title>");
        html.push_str(self.shared_styles());
        html.push_str("</head><body>");
        html.push_str("<header><h1>Global Report</h1>");
        html.push_str("<p class=\"subtitle\">Deterministic HTML generated locally.</p>");
        html.push_str("</header>");
        html.push_str("<section class=\"entries\">");
        for entry in entries {
            render_entry(
                &mut html,
                entry,
                figure_map,
                self.render_config.include_figures,
            );
        }
        html.push_str("</section></body></html>");
        fs::write(path, html).with_context(|| format!("Failed to write {}", path.display()))
    }

    fn write_category_report(
        &self,
        path: &Path,
        record: &CategoryRecord,
        entries: &[LibraryEntry],
        pinned: &[LibraryEntry],
        figure_map: &HashMap<Uuid, Vec<GalleryAsset>>,
    ) -> Result<()> {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\" />");
        html.push_str(&format!(
            "<title>Category – {}</title>",
            escape_html(&record.definition.name)
        ));
        html.push_str(self.shared_styles());
        html.push_str("</head><body>");
        html.push_str("<header class=\"category-header\">");
        html.push_str(&format!(
            "<h1>{}</h1>",
            escape_html(&record.definition.name)
        ));
        if !record.definition.description.is_empty() {
            html.push_str(&format!(
                "<p class=\"description\">{}</p>",
                escape_html(&record.definition.description)
            ));
        }
        if !record.narrative.summary.is_empty() {
            html.push_str(&format!(
                "<div class=\"narrative\"><h2>Narrative</h2><p>{}</p></div>",
                escape_html(&record.narrative.summary)
            ));
        }
        html.push_str("</header>");
        if !pinned.is_empty() {
            html.push_str("<section class=\"pinned\"><h2>Pinned Papers</h2>");
            for entry in pinned {
                render_entry(
                    &mut html,
                    entry,
                    figure_map,
                    self.render_config.include_figures,
                );
            }
            html.push_str("</section>");
        }
        html.push_str("<section class=\"entries\">");
        for entry in entries {
            render_entry(
                &mut html,
                entry,
                figure_map,
                self.render_config.include_figures,
            );
        }
        self.render_visualizations(&mut html)?;
        html.push_str("</section></body></html>");
        fs::write(path, html).with_context(|| format!("Failed to write {}", path.display()))
    }

    fn shared_styles(&self) -> &str {
        "<style>
            body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 2rem; color: #121212; background: #fff;}
            header { border-bottom: 1px solid #e0e0e0; margin-bottom: 1.5rem; padding-bottom: 1rem;}
            .category-header .description { font-size: 1rem; color: #555;}
            .narrative { background: #f8f8f8; padding: 1rem; border-radius: 8px; margin-top: 0.75rem; }
            .entries .entry, .pinned .entry { border-bottom: 1px solid #f0f0f0; padding: 0.75rem 0; }
            .entry h3 { margin: 0; font-size: 1.05rem; }
            .entry .meta { color: #666; font-size: 0.9rem; margin: 0.25rem 0; }
            .figures { display: flex; flex-wrap: wrap; gap: 0.5rem; margin-top: 0.5rem;}
            .figures figure { border: 1px solid #e0e0e0; border-radius: 6px; padding: 0.5rem; max-width: 200px;}
            .figures img { max-width: 100%; border-radius: 4px;}
            .figures figcaption { font-size: 0.85rem; margin-top: 0.25rem; color: #444;}
        </style>"
    }
}

fn render_entry(
    html: &mut String,
    entry: &LibraryEntry,
    figure_map: &HashMap<Uuid, Vec<GalleryAsset>>,
    include_figures: bool,
) {
    html.push_str("<article class=\"entry\">");
    html.push_str(&format!("<h3>{}</h3>", escape_html(&entry.title)));
    let mut meta_parts = Vec::new();
    if !entry.authors.is_empty() {
        meta_parts.push(escape_html(&entry.authors.join(", ")));
    }
    if let Some(venue) = &entry.venue {
        meta_parts.push(escape_html(venue));
    }
    if let Some(year) = entry.year {
        meta_parts.push(year.to_string());
    }
    if !meta_parts.is_empty() {
        html.push_str(&format!("<p class=\"meta\">{}</p>", meta_parts.join(" • ")));
    }
    if include_figures {
        if let Some(figures) = figure_map.get(&entry.entry_id) {
            if !figures.is_empty() {
                html.push_str("<div class=\"figures\">");
                for figure in figures {
                    html.push_str("<figure>");
                    html.push_str(&format!(
                        "<img src=\"file://{}\" alt=\"{}\" />",
                        figure.file_path.to_string_lossy(),
                        escape_html(&figure.caption)
                    ));
                    html.push_str(&format!(
                        "<figcaption>{}</figcaption>",
                        escape_html(&figure.caption)
                    ));
                    html.push_str("</figure>");
                }
                html.push_str("</div>");
            }
        }
    }
    html.push_str("</article>");
}

fn escape_html(input: &str) -> String {
    input
        .chars()
        .map(|ch| match ch {
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '&' => "&amp;".into(),
            '"' => "&quot;".into(),
            '\'' => "&#39;".into(),
            _ => ch.to_string(),
        })
        .collect::<String>()
}

impl<'a> HtmlRenderer<'a> {
    fn render_visualizations(&self, html: &mut String) -> Result<()> {
        if self.viz_entries.is_empty() {
            return Ok(());
        }
        html.push_str("<section class=\"visualizations\"><h2>Visualizations</h2><ul>");
        for entry in &self.viz_entries {
            let status = entry
                .data_path
                .as_ref()
                .map(|path| format!("file://{}", path.display()))
                .unwrap_or_else(|| "pending regeneration".into());
            html.push_str(&format!(
                "<li><strong>{:?}</strong> – {status}</li>",
                entry.kind
            ));
        }
        html.push_str("</ul></section>");
        Ok(())
    }
}
