use std::fs;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde_json::json;

use crate::{
    bases::Base,
    writing::project::{
        generate_project_slug, project_exists, scaffold_user_layer, ProjectStatus,
        WritingProjectManifest,
    },
};

/// List all writing projects under the Base by reading manifest files.
pub fn list_projects(base: &Base) -> Result<Vec<WritingProjectManifest>> {
    let mut manifests = Vec::new();
    let root = crate::writing::project::projects_root(base);
    if !root.exists() {
        return Ok(manifests);
    }
    for entry in fs::read_dir(&root)
        .with_context(|| format!("Failed to read WritingProjects under {}", root.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let slug = entry.file_name().to_string_lossy().to_string();
        let manifest_path = entry.path().join(crate::writing::project::MANIFEST_FILE);
        if !manifest_path.exists() {
            continue;
        }
        let manifest = WritingProjectManifest::load(base, &slug).with_context(|| {
            format!("Failed to load writing project manifest for slug {}", slug)
        })?;
        manifests.push(manifest);
    }
    manifests.sort_by_key(|m| m.created_at);
    Ok(manifests)
}

/// Creates a new project manifest and scaffolds the user-layer files.
pub fn create_project(
    base: &Base,
    title: &str,
    preferred_slug: Option<&str>,
) -> Result<WritingProjectManifest> {
    let slug = generate_project_slug(base, preferred_slug, title)?;
    if project_exists(base, &slug) {
        bail!(
            "Project '{}' already exists; choose another slug or archive it.",
            slug
        );
    }
    let paths = scaffold_user_layer(base, &slug)?;
    let manifest = WritingProjectManifest::new(base, slug.clone(), title.to_string());
    manifest.save(base)?;
    fs::write(
        paths.project_root.join("README.txt"),
        format!("Writing project '{title}' (slug: {slug}). Files managed by Writing Assistant."),
    )
    .ok(); // best-effort helper
    Ok(manifest)
}

/// Updates lifecycle fields on a project manifest.
pub fn update_project(
    base: &Base,
    slug: &str,
    status: Option<ProjectStatus>,
    description: Option<String>,
    default_compiler: Option<String>,
) -> Result<WritingProjectManifest> {
    let mut manifest = WritingProjectManifest::load(base, slug)?;
    if let Some(status) = status {
        manifest.status = status;
    }
    if let Some(desc) = description {
        manifest.description = Some(desc);
    }
    if let Some(compiler) = default_compiler {
        manifest.default_compiler.command = compiler;
        manifest.default_compiler.args.clear();
    }
    manifest.updated_at = Utc::now();
    manifest.save(base)?;
    Ok(manifest)
}

/// Archives a project by updating its status.
pub fn archive_project(base: &Base, slug: &str) -> Result<WritingProjectManifest> {
    update_project(base, slug, Some(ProjectStatus::Archived), None, None)
}

/// Formats a short status line for chat summarization.
pub fn summarize_manifest(manifest: &WritingProjectManifest) -> String {
    json!({
        "slug": manifest.slug,
        "title": manifest.title,
        "status": format!("{:?}", manifest.status),
        "updatedAt": manifest.updated_at,
    })
    .to_string()
}
