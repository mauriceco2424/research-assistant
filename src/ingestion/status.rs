use super::batch_store::IngestionBatchState;

pub fn format_batch_status(state: &IngestionBatchState) -> String {
    let percent = if state.total_files == 0 {
        0.0
    } else {
        (state.processed_files as f64 / state.total_files as f64) * 100.0
    };
    let checkpoint = state
        .last_checkpoint
        .as_ref()
        .map(|c| format!("checkpoint {}", c))
        .unwrap_or_else(|| "no checkpoint".into());
    format!(
        "Batch {} -> {:?}: {:.1}% complete (processed {}, ingested {}, skipped {}, failed {}), {}.",
        state.batch_id,
        state.status,
        percent,
        state.processed_files,
        state.ingested_files,
        state.skipped_files,
        state.failed_files,
        checkpoint
    )
}
