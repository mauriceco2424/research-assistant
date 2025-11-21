use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bases::{Base, BaseManager, LibraryEntry};
use crate::orchestration::{
    MetricRecord, OrchestrationLog, WritingMetricKind, WritingMetricRecord,
};
use crate::writing::project::ProjectPaths;
use serde_json;

use super::WritingResult;

/// Status for citations once validated against the Paper Base.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CitationStatus {
    Verified,
    NeedsPdf,
    Unverified,
}

impl Default for CitationStatus {
    fn default() -> Self {
        CitationStatus::Unverified
    }
}

/// Link between a cite key and a Paper Base entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationLink {
    pub cite_key: String,
    #[serde(default)]
    pub paper_id: Option<Uuid>,
    #[serde(default)]
    pub status: CitationStatus,
    #[serde(default)]
    pub last_checked_at: String,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Extracts cite keys from LaTeX content.
pub fn capture_citations(content: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for segment in content.split("\\cite{").skip(1) {
        if let Some(rest) = segment.split('}').next() {
            for key in rest.split(',') {
                let trimmed = key.trim();
                if !trimmed.is_empty() {
                    keys.push(trimmed.to_string());
                }
            }
        }
    }
    dedup(keys)
}

/// Validates cite keys against the Base library.
pub fn verify_citations(
    manager: &BaseManager,
    base: &Base,
    cite_keys: &[String],
) -> WritingResult<Vec<CitationLink>> {
    let started = std::time::Instant::now();
    let mut links = Vec::new();
    let index = build_library_index(manager, base)?;
    for key in cite_keys {
        let lowered = key.to_ascii_lowercase();
        let link = if let Some(entry) = index.get(&lowered) {
            let status = if entry.needs_pdf {
                CitationStatus::NeedsPdf
            } else {
                CitationStatus::Verified
            };
            CitationLink {
                cite_key: key.clone(),
                paper_id: Some(entry.entry_id),
                status,
                last_checked_at: Utc::now().to_rfc3339(),
                notes: entry.venue.clone(),
            }
        } else {
            CitationLink {
                cite_key: key.clone(),
                paper_id: None,
                status: CitationStatus::Unverified,
                last_checked_at: Utc::now().to_rfc3339(),
                notes: Some("Missing from Paper Base; add metadata or PDF".into()),
            }
        };
        links.push(link);
    }
    record_citation_metric(base, started.elapsed(), true, &links);
    Ok(links)
}

/// Writes or updates the bibliography file for the given project.
pub fn sync_bibliography(
    base: &Base,
    slug: &str,
    citations: &[CitationLink],
) -> WritingResult<std::path::PathBuf> {
    let paths = ProjectPaths::new(base, slug);
    let bib_path = &paths.bibliography_path;
    let mut existing = if bib_path.exists() {
        fs::read_to_string(bib_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut present_keys = HashSet::new();
    for line in existing.lines() {
        if let Some((key, _)) = parse_bib_key(line) {
            present_keys.insert(key.to_ascii_lowercase());
        }
    }

    let mut additions = String::new();
    for link in citations {
        if present_keys.contains(&link.cite_key.to_ascii_lowercase()) {
            continue;
        }
        additions.push_str(&render_bib_entry(link));
        additions.push('\n');
        present_keys.insert(link.cite_key.to_ascii_lowercase());
    }

    if !additions.is_empty() {
        existing.push_str("\n% Synced by Writing Assistant\n");
        existing.push_str(&additions);
        fs::write(bib_path, existing)
            .with_context(|| format!("Failed to update bibliography {}", bib_path.display()))?;
    }

    Ok(bib_path.clone())
}

/// Syncs bibliography entries for accepted outline references.
pub fn sync_references_for_outline(
    base: &Base,
    slug: &str,
    references: &[String],
) -> WritingResult<Option<PathBuf>> {
    if references.is_empty() {
        return Ok(None);
    }
    let links: Vec<CitationLink> = references
        .iter()
        .map(|cite_key| CitationLink {
            cite_key: cite_key.clone(),
            paper_id: None,
            status: CitationStatus::Unverified,
            last_checked_at: Utc::now().to_rfc3339(),
            notes: Some(
                "Placeholder for accepted outline reference; verify via Paper Base.".into(),
            ),
        })
        .collect();
    let path = sync_bibliography(base, slug, &links)?;
    Ok(Some(path))
}

/// Warns when the bibliography diverges from the expected keys.
pub fn detect_bibliography_drift(
    base: &Base,
    slug: &str,
    expected_keys: &[String],
) -> WritingResult<Vec<String>> {
    let paths = ProjectPaths::new(base, slug);
    let bib_path = &paths.bibliography_path;
    if !bib_path.exists() {
        return Ok(vec![format!(
            "{} is missing; drafts may have unresolved citations.",
            bib_path.display()
        )]);
    }
    let content = fs::read_to_string(bib_path)
        .with_context(|| format!("Failed to read bibliography {}", bib_path.display()))?;
    let mut found_keys = HashSet::new();
    for line in content.lines() {
        if let Some((key, _)) = parse_bib_key(line) {
            found_keys.insert(key.to_ascii_lowercase());
        }
    }
    let expected_set: HashSet<String> = expected_keys
        .iter()
        .map(|k| k.to_ascii_lowercase())
        .collect();
    let mut warnings = Vec::new();
    for key in expected_keys {
        if !found_keys.contains(&key.to_ascii_lowercase()) {
            warnings.push(format!(
                "Citation '{}' missing from references.bib; add metadata or re-run sync.",
                key
            ));
        }
    }
    for extra in found_keys.difference(&expected_set) {
        warnings.push(format!(
            "references.bib contains extra cite key '{}' not referenced in drafts.",
            extra
        ));
    }
    Ok(warnings)
}

fn parse_bib_key(line: &str) -> Option<(&str, &str)> {
    if !line.trim_start().starts_with('@') {
        return None;
    }
    let parts: Vec<&str> = line.split('{').collect();
    if parts.len() < 2 {
        return None;
    }
    let key_part = parts[1];
    let key = key_part.split(',').next()?.trim();
    Some((key, key_part))
}

fn render_bib_entry(link: &CitationLink) -> String {
    match link.status {
        CitationStatus::Verified => format!(
            "@article{{{key},\n  title={{Pending title for {key}}},\n  note={{Synced from Paper Base}},\n}}\n",
            key = link.cite_key
        ),
        CitationStatus::NeedsPdf => format!(
            "@article{{{key},\n  title={{Metadata-only entry; PDF needed}},\n  note={{Paper Base marked NEEDS_PDF}},\n}}\n",
            key = link.cite_key
        ),
        CitationStatus::Unverified => format!(
            "@misc{{{key},\n  title={{UNVERIFIED citation placeholder}},\n  note={{Add to Paper Base or update cite key}},\n}}\n",
            key = link.cite_key
        ),
    }
}

fn build_library_index(
    manager: &BaseManager,
    base: &Base,
) -> WritingResult<HashMap<String, LibraryEntry>> {
    let mut map = HashMap::new();
    let entries = manager.load_library_entries(base).unwrap_or_default();
    for entry in entries {
        map.insert(
            entry.entry_id.to_string().to_ascii_lowercase(),
            entry.clone(),
        );
        map.insert(entry.identifier.to_ascii_lowercase(), entry.clone());
    }
    Ok(map)
}

fn dedup(list: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    list.into_iter()
        .filter(|item| seen.insert(item.to_ascii_lowercase()))
        .collect()
}

fn record_citation_metric(
    base: &Base,
    duration: std::time::Duration,
    success: bool,
    links: &[CitationLink],
) {
    let counts = links.iter().fold(
        (0usize, 0usize, 0usize),
        |(verified, needs_pdf, unverified), link| match link.status {
            CitationStatus::Verified => (verified + 1, needs_pdf, unverified),
            CitationStatus::NeedsPdf => (verified, needs_pdf + 1, unverified),
            CitationStatus::Unverified => (verified, needs_pdf, unverified + 1),
        },
    );
    let log = OrchestrationLog::for_base(base);
    let _ = log.record_metric(&MetricRecord::Writing(WritingMetricRecord {
        kind: WritingMetricKind::CitationResolution,
        duration_ms: duration.as_millis() as i64,
        success,
        details: serde_json::json!({
            "verified": counts.0,
            "needs_pdf": counts.1,
            "unverified": counts.2,
        }),
    }));
}
