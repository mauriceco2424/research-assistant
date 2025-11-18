use super::IntegrationHarness;
use anyhow::Result;
use researchbase::ingestion::{IngestionBatchStatus, IngestionRunner};
use researchbase::orchestration::{MetricRecord, OrchestrationLog};
use std::fs;

#[test]
fn ingestion_scales_to_large_batches_with_metrics() -> Result<()> {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    manager.config.ingestion.checkpoint_interval_files = 600;
    manager.config.ingestion.max_parallel_file_copies = 8;
    let base = harness.create_base(&mut manager, "scale-batch");
    let source_dir = harness.workspace_path().join("scale-source");
    fs::create_dir_all(&source_dir)?;

    for idx in 0..550 {
        fs::write(
            source_dir.join(format!("scale_{idx}.pdf")),
            format!("pdf data {idx}"),
        )?;
    }

    let runner = IngestionRunner::new(&manager, base.clone());
    let outcome = runner.start_batch(&source_dir)?;
    assert!(
        matches!(outcome.state.status, IngestionBatchStatus::Completed),
        "expected run to finish in one chunk"
    );
    assert_eq!(outcome.state.ingested_files, 550);

    let log = OrchestrationLog::for_base(&base);
    let metrics = log.load_metrics()?;
    assert!(
        metrics.iter().any(|record| matches!(
            record,
            MetricRecord::Ingestion(metric) if metric.batch_id == outcome.state.batch_id
        )),
        "ingestion metrics should record batch completion"
    );

    Ok(())
}
