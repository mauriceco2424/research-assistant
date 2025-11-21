use anyhow::{bail, Result};

use crate::bases::Base;
use crate::writing::compile::{compile_project, CompileOptions};

/// Runs a compile for the given project slug.
pub fn run_compile(
    base: &Base,
    slug: &str,
    compiler: Option<String>,
    clean: bool,
) -> Result<String> {
    let session = compile_project(base, slug, CompileOptions { compiler, clean })?;
    if session.status == crate::writing::compile::BuildStatus::Skipped {
        bail!("Compile skipped: {:?}", session.error_summary);
    }
    Ok(format!(
        "[OK] Compile {} with {} -> status {:?}. Log: {}, PDF: {}",
        session.id,
        session.compiler,
        session.status,
        session.log_path.display(),
        session
            .pdf_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "n/a".to_string())
    ))
}
