use anyhow::{bail, Context, Result};
use serde_json::json;
use uuid::Uuid;

use crate::bases::{Base, BaseManager};
use crate::orchestration::events::{log_draft_generated, WritingEventDetails};
use crate::writing::drafting::{generate_draft, DraftOutcome, DraftRequest};
use crate::writing::outline::OutlineStore;

/// Generates a draft for a specific outline node and logs orchestration metadata.
pub fn generate_draft_for_node(
    manager: &BaseManager,
    base: &Base,
    slug: &str,
    node_id: &str,
    instructions: Option<String>,
    cite_sources: Vec<String>,
) -> Result<String> {
    let parsed_id = Uuid::parse_str(node_id)
        .with_context(|| format!("Invalid outline node id '{}'", node_id))?;

    let outline = OutlineStore::new(base).load(slug)?;
    if !outline.nodes.iter().any(|node| node.id == parsed_id) {
        bail!(
            "Outline node {} not found; generate proposals and accept nodes before drafting.",
            node_id
        );
    }

    let outcome = generate_draft(
        manager,
        base,
        DraftRequest {
            slug: slug.to_string(),
            node_id: parsed_id,
            instructions,
            cite_sources,
        },
        Some(&outline),
        None,
    )?;

    let event_id = log_draft_generated(
        base,
        WritingEventDetails::with_payload(
            slug,
            json!({
                "node_id": parsed_id,
                "citations": outcome.metadata.citations,
            }),
        )
        .with_files_touched([
            outcome.path.display().to_string(),
            format!("{}/outline.json", slug),
            outcome.metadata.file_path.clone(),
            outcome.bibliography_path.display().to_string(),
        ]),
    )?;

    Ok(format!(
        "Draft generated for node {} at {} (event {}).{}",
        node_id,
        outcome.path.display(),
        event_id,
        format_drift_warnings(&outcome)
    ))
}

fn format_drift_warnings(outcome: &DraftOutcome) -> String {
    if outcome.drift_warnings.is_empty() {
        String::new()
    } else {
        let bullets = outcome
            .drift_warnings
            .iter()
            .map(|w| format!("\n- {}", w))
            .collect::<String>();
        format!("\nWarnings:{}", bullets)
    }
}
