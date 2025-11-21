use anyhow::{Context, Result};

use crate::{
    bases::{Base, BaseManager},
    chat::commands::writing_projects,
    orchestration::events::{
        log_style_model_ingested, log_writing_project_created, WritingEventDetails,
    },
    writing::{
        project::ProjectStatus,
        style::{
            ingest_style_models, run_style_interview, StyleInterviewOutcome,
            StyleModelIngestionResult,
        },
    },
};

#[derive(Debug)]
pub struct WritingStartResponse {
    pub slug: String,
    pub interview: StyleInterviewOutcome,
    pub style_ingestion: StyleModelIngestionResult,
}

/// Orchestrates `/writing start` by slugging, scaffolding, running interview, and logging events.
pub fn start_project(
    manager: &BaseManager,
    base: &Base,
    title: &str,
    preferred_slug: Option<&str>,
) -> Result<WritingStartResponse> {
    let manifest = writing_projects::create_project(base, title, preferred_slug)?;
    let slug = manifest.slug.clone();

    let interview = run_style_interview(base, &[])?;
    let style_ingestion = ingest_style_models(manager, base, &[])?;

    log_writing_project_created(
        base,
        WritingEventDetails::new(&slug).with_files_touched([
            format!("{}/project.json", slug),
            format!("{}/main.tex", slug),
            format!("{}/references.bib", slug),
        ]),
    )?;

    if !style_ingestion.records.is_empty() {
        log_style_model_ingested(
            base,
            WritingEventDetails::new(&slug).with_files_touched([format!(
                "{}/profiles/writing_style_models.json",
                base.ai_layer_path.display()
            )]),
        )?;
    }

    // Ensure manifest status remains draft after scaffolding.
    writing_projects::update_project(base, &slug, Some(ProjectStatus::Draft), None, None)
        .context("Failed to finalize writing project manifest")?;

    Ok(WritingStartResponse {
        slug,
        interview,
        style_ingestion,
    })
}
