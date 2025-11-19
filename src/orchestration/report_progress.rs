use crate::bases::{Base, BaseManager};
use crate::orchestration::{log_event, EventType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct ProgressPayload<'a> {
    job_id: Uuid,
    stage: &'a str,
    message: &'a str,
    percent: u8,
    elapsed_ms: i64,
    started_at: DateTime<Utc>,
}

pub struct ReportProgressTracker<'a> {
    manager: &'a BaseManager,
    base: Base,
    job_id: Uuid,
    started_at: DateTime<Utc>,
    started_instant: Instant,
    last_emit: Instant,
    max_interval: Duration,
    last_percent: u8,
}

impl<'a> ReportProgressTracker<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        let now = Instant::now();
        Self {
            manager,
            base: base.clone(),
            job_id: Uuid::new_v4(),
            started_at: Utc::now(),
            started_instant: now,
            last_emit: now,
            max_interval: Duration::from_secs(5),
            last_percent: 0,
        }
    }

    pub fn start(&mut self, scope: &str) -> Result<()> {
        self.emit("start", scope, 0)
    }

    pub fn update(&mut self, message: &str, percent: u8) -> Result<()> {
        if self.last_emit.elapsed() < self.max_interval && percent == self.last_percent {
            return Ok(());
        }
        self.emit("progress", message, percent)
    }

    pub fn finish(&mut self, message: &str) -> Result<()> {
        self.emit("finish", message, 100)
    }

    fn emit(&mut self, stage: &str, message: &str, percent: u8) -> Result<()> {
        let payload = ProgressPayload {
            job_id: self.job_id,
            stage,
            message,
            percent,
            elapsed_ms: self.started_instant.elapsed().as_millis() as i64,
            started_at: self.started_at,
        };
        self.last_emit = Instant::now();
        self.last_percent = percent;
        log_event(
            self.manager,
            &self.base,
            EventType::ReportsGenerated,
            serde_json::to_value(payload)?,
        )
    }

    pub fn job_id(&self) -> Uuid {
        self.job_id
    }

    pub fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    pub fn elapsed_ms(&self) -> i64 {
        self.started_instant.elapsed().as_millis() as i64
    }
}
