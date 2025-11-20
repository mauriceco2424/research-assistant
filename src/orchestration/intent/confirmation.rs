//! Confirmation ticket storage for intents that require explicit approval.

use crate::bases::{ensure_intents_dir, Base};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Confirmation ticket persisted on disk per Base for auditing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationTicket {
    pub ticket_id: Uuid,
    pub intent_id: Uuid,
    pub base_id: Uuid,
    pub prompt: String,
    pub confirm_phrase: String,
    pub expires_at: DateTime<Utc>,
    pub status: ConfirmationStatus,
    #[serde(default)]
    pub consent_manifest_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConfirmationTicket {
    pub fn new(
        intent_id: Uuid,
        base_id: Uuid,
        prompt: impl Into<String>,
        confirm_phrase: impl Into<String>,
        expires_at: DateTime<Utc>,
        consent_manifest_ids: Vec<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            ticket_id: Uuid::new_v4(),
            intent_id,
            base_id,
            prompt: prompt.into(),
            confirm_phrase: confirm_phrase.into(),
            expires_at,
            status: ConfirmationStatus::Pending,
            consent_manifest_ids,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_status(&mut self, status: ConfirmationStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

/// Ticket workflow status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

/// File-backed store scoped to a single Base.
pub struct ConfirmationStore {
    root: PathBuf,
}

impl ConfirmationStore {
    pub fn for_base(base: &Base) -> Result<Self> {
        let intents_dir = ensure_intents_dir(base)?;
        let confirmations_dir = intents_dir.join("confirmations");
        fs::create_dir_all(&confirmations_dir)?;
        Ok(Self {
            root: confirmations_dir,
        })
    }

    pub fn record(&self, ticket: &ConfirmationTicket) -> Result<PathBuf> {
        fs::create_dir_all(&self.root)?;
        let path = self.path(&ticket.ticket_id);
        let data = serde_json::to_vec_pretty(ticket)?;
        fs::write(&path, data)?;
        Ok(path)
    }

    pub fn get(&self, ticket_id: &Uuid) -> Result<Option<ConfirmationTicket>> {
        let path = self.path(ticket_id);
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read(&path)?;
        let ticket = serde_json::from_slice(&data)
            .with_context(|| format!("Failed parsing confirmation ticket {:?}", path))?;
        Ok(Some(ticket))
    }

    pub fn list_all(&self) -> Result<Vec<ConfirmationTicket>> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut tickets = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let data = fs::read(entry.path())?;
                let ticket: ConfirmationTicket = serde_json::from_slice(&data)?;
                tickets.push(ticket);
            }
        }
        tickets.sort_by_key(|t| t.created_at);
        Ok(tickets)
    }

    pub fn update_status(
        &self,
        ticket_id: &Uuid,
        status: ConfirmationStatus,
    ) -> Result<Option<ConfirmationTicket>> {
        let path = self.path(ticket_id);
        if !path.exists() {
            return Ok(None);
        }
        let mut ticket = self
            .get(ticket_id)?
            .context("Ticket disappeared before update")?;
        ticket.set_status(status);
        let data = serde_json::to_vec_pretty(&ticket)?;
        fs::write(&path, data)?;
        Ok(Some(ticket))
    }

    fn path(&self, ticket_id: &Uuid) -> PathBuf {
        self.root.join(format!("{ticket_id}.json"))
    }
}
