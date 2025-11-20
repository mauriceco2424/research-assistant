//! Intent log scaffolding, used to persist router events in the AI layer.

use super::payload::IntentPayload;
use crate::bases::{ensure_intents_dir, Base};
use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// File-backed JSONL log storing intent payloads per Base.
pub struct IntentLog {
    path: PathBuf,
}

impl IntentLog {
    pub fn for_base(base: &Base) -> Result<Self> {
        let dir = ensure_intents_dir(base)?;
        let path = dir.join("log.jsonl");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            File::create(&path)?;
        }
        Ok(Self { path })
    }

    pub fn append(&self, payload: &IntentPayload) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = payload.to_json_line()?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn read_all(&self) -> Result<Vec<IntentPayload>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut payloads = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let payload = IntentPayload::from_json_line(&line)
                .with_context(|| "Failed parsing intent log")?;
            payloads.push(payload);
        }
        Ok(payloads)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
