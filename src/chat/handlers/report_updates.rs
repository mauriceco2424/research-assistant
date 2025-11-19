use crate::bases::Base;
use crate::reports::build_service::ReportBuildResult;
use chrono::Duration;
use uuid::Uuid;

pub fn build_completion_summary(base: &Base, result: &ReportBuildResult) -> String {
    let manifest_path = base
        .ai_layer_path
        .join("reports")
        .join("manifests")
        .join(format!("{}.json", result.manifest.manifest_id));
    let duration = Duration::milliseconds(result.duration_ms);
    let seconds = duration.num_milliseconds() as f64 / 1000.0;
    let mut message = format!(
        "reports regenerate ({}) completed in {:.1}s. Orchestration ID: {}.",
        result.scope_label, seconds, result.orchestration_id
    );
    if !result.updated_files.is_empty() {
        message.push_str("\nUpdated files:");
        for path in &result.updated_files {
            message.push_str(&format!("\n- {}", path.display()));
        }
    }
    if result.figures_enabled {
        message.push_str("\nFigures: enabled");
    } else {
        message.push_str("\nFigures: disabled");
    }
    if !result.visualization_types.is_empty() {
        message.push_str(&format!(
            "\nVisualizations: {}",
            result.visualization_types.join(", ")
        ));
    }
    if !result.manifest.consent_tokens.is_empty() {
        message.push_str("\nConsent tokens:");
        for token in &result.manifest.consent_tokens {
            message.push_str(&format!("\n  - {}", token));
        }
    }
    message.push_str(&format!("\nManifest: {}", manifest_path.display()));
    message
}

pub fn queued_message(request_id: Uuid, scope: &str) -> String {
    format!(
        "Another report job is running. Queued request {request_id} for scope '{scope}'. The system will run it automatically when the active job completes."
    )
}
