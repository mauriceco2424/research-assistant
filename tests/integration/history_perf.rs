use super::IntegrationHarness;
use anyhow::Result;
use chrono::{Duration, Utc};
use researchbase::chat::ChatSession;
use researchbase::orchestration::{EventType, OrchestrationEvent, OrchestrationLog};
use std::time::Instant;
use uuid::Uuid;

#[test]
fn history_handles_many_batches_quickly() -> Result<()> {
    let _harness = IntegrationHarness::new();
    let mut chat = ChatSession::new()?;
    let base = chat.create_base("history-perf")?;
    chat.select_base(&base.id)?;
    let log = OrchestrationLog::for_base(&base);

    for idx in 0..50 {
        let event = OrchestrationEvent {
            event_id: Uuid::new_v4(),
            base_id: base.id,
            event_type: EventType::IngestionBatchCompleted,
            timestamp: Utc::now() - Duration::minutes(idx),
            details: serde_json::json!({
                "batch_id": idx,
                "ingested": 10
            }),
        };
        log.append_event(&event)?;
    }

    let started = Instant::now();
    let history = chat.history_show(Some("30d"))?;
    let elapsed = started.elapsed();
    assert!(
        elapsed.as_millis() < 2000,
        "history command should stay responsive, took {:?}",
        elapsed
    );
    let ingestion_lines = history
        .into_iter()
        .filter(|line| line.contains("IngestionBatchCompleted"))
        .count();
    assert!(
        ingestion_lines >= 50,
        "history output should include at least 50 ingestion entries"
    );

    Ok(())
}
