use super::IntegrationHarness;
use anyhow::Result;
use researchbase::ingestion::{IngestionBatchStatus, IngestionRunner};
use std::fs;

#[test]
fn ingestion_runner_pauses_and_resumes_batches() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "ingestion-runner");
    let source_dir = harness.workspace_path().join("ingest-source");
    fs::create_dir_all(&source_dir)?;

    for idx in 0..30 {
        let file = source_dir.join(format!("paper_{idx}.pdf"));
        fs::write(file, format!("test file {idx}"))?;
    }

    let runner = IngestionRunner::new(&manager, base.clone());
    let start_outcome = runner.start_batch(&source_dir)?;
    assert!(
        matches!(start_outcome.state.status, IngestionBatchStatus::Paused | IngestionBatchStatus::Completed),
        "initial run should produce a Paused or Completed state"
    );

    if matches!(start_outcome.state.status, IngestionBatchStatus::Paused) {
        let resume_outcome = runner.resume_latest()?;
        assert!(
            matches!(resume_outcome.state.status, IngestionBatchStatus::Completed),
            "resume should finish the batch"
        );
        assert_eq!(
            resume_outcome.state.ingested_files,
            start_outcome.state.total_files
        );
    }

    Ok(())
}
