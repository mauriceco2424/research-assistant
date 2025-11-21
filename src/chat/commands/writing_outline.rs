use anyhow::{Context, Result};
use serde_json::json;
use uuid::Uuid;

use crate::bases::{Base, BaseManager};
use crate::orchestration::events::{
    log_outline_created, log_outline_modified, WritingEventDetails,
};
use crate::writing::outline::{
    generate_outline_proposals, update_outline_status, OutlineProposalRequest, OutlineStatus,
};

/// Generates outline proposals for a project and logs orchestration events.
pub fn propose_outline(
    manager: &BaseManager,
    base: &Base,
    slug: &str,
    goal: Option<String>,
    references: Vec<String>,
) -> Result<String> {
    let proposal = generate_outline_proposals(
        manager,
        base,
        slug,
        OutlineProposalRequest {
            goal,
            references,
            max_nodes: Some(150),
        },
    )?;

    let event_id = log_outline_created(
        base,
        WritingEventDetails::with_payload(
            slug,
            json!({
                "added_nodes": proposal.added_nodes.len(),
                "warnings": proposal.warnings,
            }),
        )
        .with_files_touched([proposal.outline_path.display().to_string()]),
    )?;

    let mut response = format!(
        "[OK] Generated {} outline nodes (event {}).",
        proposal.added_nodes.len(),
        event_id
    );
    if !proposal.warnings.is_empty() {
        response.push_str("\nWarnings:");
        for warning in proposal.warnings {
            response.push_str(&format!("\n- {}", warning));
        }
    }
    Ok(response)
}

/// Updates the status of an outline node and records an undo checkpoint.
pub fn update_outline_node_status(
    base: &Base,
    slug: &str,
    node_id: &str,
    status: OutlineStatus,
) -> Result<String> {
    let parsed_id = Uuid::parse_str(node_id)
        .with_context(|| format!("Invalid outline node id '{}'", node_id))?;
    // Use a dedicated event id for checkpoint tracking.
    let checkpoint_event = Uuid::new_v4();
    let (outline, checkpoint) =
        update_outline_status(base, slug, parsed_id, status, Some(checkpoint_event))?;

    let mut event_details = WritingEventDetails::with_payload(
        slug,
        json!({
            "node_id": parsed_id,
            "status": format!("{:?}", status),
        }),
    )
    .with_files_touched([format!("{}/outline.json", slug)]);

    if let Some(checkpoint) = &checkpoint {
        event_details =
            event_details.with_undo_checkpoint(checkpoint.checkpoint_path.display().to_string());
    }

    let event_id = log_outline_modified(base, event_details)?;

    let summary = outline
        .nodes
        .iter()
        .filter(|node| node.status == OutlineStatus::Accepted)
        .count();
    Ok(format!(
        "Updated outline node {} to {:?}. Undo checkpoint stored. Accepted nodes: {}. Event {}.",
        node_id, status, summary, event_id
    ))
}
