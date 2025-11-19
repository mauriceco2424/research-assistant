use crate::bases::{Base, BaseManager, CategorySnapshotStore};
use crate::orchestration::{MetricRecord, OrchestrationLog, ReportMetricsRecord, REPORT_SLA_SECS};
use crate::reports::manifest::{hash_path, ReportBuildRequest, ReportManifest, ReportOutputEntry};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

pub struct ReportManifestWriter<'a> {
    _manager: &'a BaseManager,
    base: Base,
}

impl<'a> ReportManifestWriter<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        Self {
            _manager: manager,
            base: base.clone(),
        }
    }

    pub fn write(
        &self,
        request: &ReportBuildRequest,
        mut outputs: Vec<ReportOutputEntry>,
        config_signature: String,
        started_at: DateTime<Utc>,
        duration_ms: i64,
        orchestration_id: Uuid,
        figure_count: usize,
        entry_count: usize,
        consent_tokens: Vec<Uuid>,
        visualization_dataset_ids: Vec<Uuid>,
        mut metadata: HashMap<String, serde_json::Value>,
    ) -> Result<ReportManifest> {
        for entry in &mut outputs {
            if entry.hash.is_empty() {
                entry.hash = hash_path(&entry.path)?;
            }
        }
        let mut manifest = ReportManifest::new(&self.base, request.request_id, config_signature);
        manifest.outputs = outputs;
        manifest.started_at = started_at;
        manifest.completed_at = Utc::now();
        manifest.duration_ms = duration_ms;
        manifest.orchestration_id = Some(orchestration_id);
        manifest.ai_layer_snapshots = self.snapshot_ids()?;
        manifest.metrics_revision_id = self.latest_metric_id()?;
        manifest.consent_tokens = consent_tokens;
        manifest.visualization_dataset_ids = visualization_dataset_ids;
        manifest
            .metadata
            .entry("scope".into())
            .or_insert_with(|| serde_json::json!(request.scope.label()));
        for (key, value) in metadata.drain() {
            manifest.metadata.insert(key, value);
        }
        manifest.persist(&self.base)?;
        self.record_metrics(duration_ms, figure_count, entry_count)?;
        Ok(manifest)
    }

    fn snapshot_ids(&self) -> Result<Vec<Uuid>> {
        let store = CategorySnapshotStore::new(&self.base)?;
        Ok(store.list()?.into_iter().map(|s| s.snapshot_id).collect())
    }

    fn latest_metric_id(&self) -> Result<Option<Uuid>> {
        let metrics_store = crate::bases::CategoryMetricsStore::new(&self.base);
        let metrics = metrics_store.load()?;
        Ok(metrics.last().map(|metric| metric.metric_id))
    }

    fn record_metrics(
        &self,
        duration_ms: i64,
        figure_count: usize,
        entry_count: usize,
    ) -> Result<()> {
        let log = OrchestrationLog::for_base(&self.base);
        log.record_metric(&MetricRecord::Reports(ReportMetricsRecord {
            duration_ms,
            entry_count,
            figure_count,
            sla_breached: duration_ms > REPORT_SLA_SECS * 1000,
        }))
    }
}

pub fn unchanged_entry(path: &Path, scope: &str) -> Result<ReportOutputEntry> {
    Ok(ReportOutputEntry {
        path: path.to_path_buf(),
        scope: scope.into(),
        hash: hash_path(path)?,
        kind: "unchanged".into(),
    })
}

pub fn planned_entry(path: &Path, scope: &str) -> ReportOutputEntry {
    ReportOutputEntry {
        path: path.to_path_buf(),
        scope: scope.into(),
        hash: String::new(),
        kind: "html".into(),
    }
}

pub fn build_output_map(entries: &[ReportOutputEntry]) -> HashMap<String, ReportOutputEntry> {
    entries
        .iter()
        .map(|entry| (entry.scope.clone(), entry.clone()))
        .collect()
}
