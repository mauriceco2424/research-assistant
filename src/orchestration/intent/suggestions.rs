use crate::bases::{Base, BaseManager, ProfileLayout};
use crate::orchestration::consent::{ConsentStatus, ConsentStore};
use crate::orchestration::profiles::model::{KnowledgeProfile, VerificationStatus};
use crate::orchestration::profiles::storage::read_profile;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Snapshot of AI-layer signals used to guide proactive suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionContext {
    pub snapshot_id: Uuid,
    pub pending_consent: Vec<Uuid>,
    pub stale_knowledge_entries: Vec<String>,
    pub ingestion_backlog: usize,
    pub generated_at: DateTime<Utc>,
    #[serde(default)]
    pub evidence_paths: Vec<PathBuf>,
}

impl SuggestionContext {
    pub fn build(manager: &BaseManager, base: &Base) -> Result<Self> {
        let mut evidence_paths = Vec::new();
        let mut pending_consent = Vec::new();
        let store = ConsentStore::for_base(base);
        let manifests = store.load_all()?;
        let manifest_dir = ProfileLayout::new(base).consent_manifests_dir;
        for manifest in manifests {
            let is_pending = manifest.status != ConsentStatus::Approved
                || manifest
                    .expires_at
                    .map(|timestamp| timestamp <= Utc::now())
                    .unwrap_or(false);
            if is_pending {
                pending_consent.push(manifest.manifest_id);
                evidence_paths.push(manifest_dir.join(format!("{}.json", manifest.manifest_id)));
            }
        }

        let mut stale_knowledge_entries = Vec::new();
        let layout = ProfileLayout::new(base);
        let knowledge_path = layout.profile_json("knowledge");
        if let Some(profile) = read_profile::<KnowledgeProfile, _>(&knowledge_path)? {
            for entry in profile.entries {
                if entry.verification_status == VerificationStatus::Stale {
                    stale_knowledge_entries.push(entry.concept.clone());
                }
            }
            if !stale_knowledge_entries.is_empty() {
                evidence_paths.push(knowledge_path);
            }
        }

        let backlog = manager
            .load_library_entries(base)?
            .into_iter()
            .filter(|entry| entry.needs_pdf)
            .count();

        Ok(Self {
            snapshot_id: Uuid::new_v4(),
            pending_consent,
            stale_knowledge_entries,
            ingestion_backlog: backlog,
            generated_at: Utc::now(),
            evidence_paths,
        })
    }
}
