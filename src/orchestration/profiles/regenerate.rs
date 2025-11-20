//! Deterministic regeneration of profile artifacts from history or exports.

use std::{fs::File, io::Read, path::Path};

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use zip::ZipArchive;

use crate::{
    bases::{Base, BaseManager, ProfileLayout},
    orchestration::{
        log_profile_event,
        profiles::{
            model::{
                HistoryRef, KnowledgeProfile, ProfileChangeKind, ProfileMetadata, ProfileType,
                UserProfile, WorkProfile, WritingProfile,
            },
            storage::{compute_hash, write_profile, write_profile_html},
            summarize::{
                summarize_knowledge, summarize_user, summarize_work, summarize_writing,
                ProfileSummary,
            },
        },
        EventType, OrchestrationLog, ProfileEventDetails,
    },
};

use super::render::build_profile_html;

#[derive(Debug, Clone)]
pub struct ProfileRegenerateOutcome {
    pub profile_type: ProfileType,
    pub replayed_events: usize,
    pub hash_after: String,
    pub event_id: Uuid,
}

pub struct ProfileRegenerator<'a> {
    manager: &'a BaseManager,
    base: Base,
    layout: ProfileLayout,
}

impl<'a> ProfileRegenerator<'a> {
    pub fn new(manager: &'a BaseManager, base: &Base) -> Self {
        Self {
            manager,
            base: base.clone(),
            layout: ProfileLayout::new(base),
        }
    }

    pub fn from_history(&self, profile_type: ProfileType) -> Result<ProfileRegenerateOutcome> {
        let log = OrchestrationLog::for_base(&self.base);
        let events = log.load_events()?;
        match profile_type {
            ProfileType::User => {
                let records = collect_records::<UserProfile>(&events, profile_type)?;
                self.replay_records(profile_type, records, summarize_user)
            }
            ProfileType::Work => {
                let records = collect_records::<WorkProfile>(&events, profile_type)?;
                self.replay_records(profile_type, records, summarize_work)
            }
            ProfileType::Writing => {
                let records = collect_records::<WritingProfile>(&events, profile_type)?;
                self.replay_records(profile_type, records, summarize_writing)
            }
            ProfileType::Knowledge => {
                let records = collect_records::<KnowledgeProfile>(&events, profile_type)?;
                self.replay_records(profile_type, records, summarize_knowledge)
            }
        }
    }

    pub fn from_archive<P: AsRef<Path>>(
        &self,
        profile_type: ProfileType,
        archive_path: P,
    ) -> Result<ProfileRegenerateOutcome> {
        let archive_path = archive_path.as_ref();
        if !archive_path.exists() {
            bail!(
                "Archive {} not found. Run `profile export {}` first.",
                archive_path.display(),
                profile_type.slug()
            );
        }
        let file = File::open(archive_path)
            .with_context(|| format!("Failed to open {}", archive_path.display()))?;
        let mut archive = ZipArchive::new(file)?;
        let json_name = format!("{}.json", profile_type.slug());
        let mut json_entry =
            archive.by_name(&json_name).with_context(|| format!("Archive missing {json_name}"))?;
        let mut contents = String::new();
        json_entry.read_to_string(&mut contents)?;
        match profile_type {
            ProfileType::User => {
                let mut profile: UserProfile = serde_json::from_str(&contents)?;
                self.persist_from_archive(profile_type, &mut profile, summarize_user, archive_path)
            }
            ProfileType::Work => {
                let mut profile: WorkProfile = serde_json::from_str(&contents)?;
                self.persist_from_archive(profile_type, &mut profile, summarize_work, archive_path)
            }
            ProfileType::Writing => {
                let mut profile: WritingProfile = serde_json::from_str(&contents)?;
                self.persist_from_archive(profile_type, &mut profile, summarize_writing, archive_path)
            }
            ProfileType::Knowledge => {
                let mut profile: KnowledgeProfile = serde_json::from_str(&contents)?;
                self.persist_from_archive(profile_type, &mut profile, summarize_knowledge, archive_path)
            }
        }
    }

    fn replay_records<P>(
        &self,
        profile_type: ProfileType,
        records: Vec<ReplayRecord<P>>,
        summarizer: fn(&P) -> ProfileSummary,
    ) -> Result<ProfileRegenerateOutcome>
    where
        P: RegenerableProfile,
    {
        if records.is_empty() {
            bail!(
                "No orchestration events found for {:?}. Cannot regenerate from history.",
                profile_type
            );
        }
        let mut profile = records
            .last()
            .ok_or_else(|| anyhow!("Missing final snapshot for {:?}", profile_type))?
            .snapshot
            .clone();
        let mut history_refs = Vec::new();
        for record in &records {
            let hash = match &record.hash_after {
                Some(hash) => hash.clone(),
                None => canonical_snapshot_hash(&record.snapshot)?,
            };
            history_refs.push(HistoryRef {
                event_id: record.event_id,
                timestamp: record.timestamp,
                hash_after: hash,
            });
        }
        profile.history_mut().clear();
        profile.history_mut().extend(history_refs.iter().cloned());
        if let Some(last) = history_refs.last() {
            profile.metadata_mut().last_updated = last.timestamp;
        }
        let hash_before = self.read_existing_hash(profile_type)?;
        let hash_after = canonical_snapshot_hash(&profile)?;
        self.persist_profile(profile_type, &profile, summarizer)?;
        let payload = json!({
            "source": "history",
            "events_replayed": history_refs.len(),
        });
        let event_id = log_profile_event(
            self.manager,
            &self.base,
            ProfileEventDetails {
                profile_type,
                change_kind: ProfileChangeKind::Regenerate,
                diff_summary: vec![format!(
                    "Regenerated {profile_type:?} from {} events",
                    history_refs.len()
                )],
                hash_before,
                hash_after: Some(hash_after.clone()),
                undo_token: None,
                payload,
            },
        )?;
        Ok(ProfileRegenerateOutcome {
            profile_type,
            replayed_events: history_refs.len(),
            hash_after,
            event_id,
        })
    }

    fn persist_from_archive<P>(
        &self,
        profile_type: ProfileType,
        profile: &mut P,
        summarizer: fn(&P) -> ProfileSummary,
        archive_path: &Path,
    ) -> Result<ProfileRegenerateOutcome>
    where
        P: RegenerableProfile,
    {
        let history_entries = profile.history().len();
        let hash_before = self.read_existing_hash(profile_type)?;
        let hash_after = canonical_snapshot_hash(profile)?;
        self.persist_profile(profile_type, profile, summarizer)?;
        let payload = json!({
            "source": "archive",
            "archive_path": archive_path,
            "history_entries": history_entries,
        });
        let event_id = log_profile_event(
            self.manager,
            &self.base,
            ProfileEventDetails {
                profile_type,
                change_kind: ProfileChangeKind::Regenerate,
                diff_summary: vec![format!(
                    "Regenerated {profile_type:?} from archive {}",
                    archive_path.display()
                )],
                hash_before,
                hash_after: Some(hash_after.clone()),
                undo_token: None,
                payload,
            },
        )?;
        Ok(ProfileRegenerateOutcome {
            profile_type,
            replayed_events: history_entries,
            hash_after,
            event_id,
        })
    }

    fn persist_profile<P>(
        &self,
        profile_type: ProfileType,
        profile: &P,
        summarizer: fn(&P) -> ProfileSummary,
    ) -> Result<()>
    where
        P: RegenerableProfile,
    {
        let json_path = self.layout.profile_json(profile_type.slug());
        write_profile(&json_path, profile)?;
        let summary = summarizer(profile);
        let html = build_profile_html(
            profile_type,
            profile.metadata(),
            &summary.highlights,
            &summary.fields,
        );
        let html_path = self.layout.profile_html(profile_type.slug());
        write_profile_html(html_path, &html)?;
        Ok(())
    }

    fn read_existing_hash(&self, profile_type: ProfileType) -> Result<Option<String>> {
        let json_path = self.layout.profile_json(profile_type.slug());
        if !json_path.exists() {
            return Ok(None);
        }
        let data = std::fs::read(&json_path)?;
        let mut value: serde_json::Value = serde_json::from_slice(&data)?;
        if let serde_json::Value::Object(ref mut map) = value {
            map.remove("history");
        }
        let canonical_bytes = serde_json::to_vec(&value)?;
        Ok(Some(compute_hash(&canonical_bytes)))
    }
}

#[derive(Debug, Clone)]
struct ReplayRecord<P> {
    event_id: Uuid,
    timestamp: DateTime<Utc>,
    hash_after: Option<String>,
    snapshot: P,
}

fn collect_records<T>(
    events: &[crate::orchestration::OrchestrationEvent],
    profile_type: ProfileType,
) -> Result<Vec<ReplayRecord<T>>>
where
    T: Clone + Serialize + DeserializeOwned,
{
    let mut records = Vec::new();
    for event in events {
        if event.event_type != EventType::ProfileChange {
            continue;
        }
        let details: ProfileEventDetails = serde_json::from_value(event.details.clone())?;
        if details.profile_type != profile_type {
            continue;
        }
        let Some(snapshot) = details.payload.get("snapshot").cloned() else {
            // Governance events (export/delete) do not persist snapshots, so skip them.
            continue;
        };
        let snapshot: T = serde_json::from_value(snapshot)?;
        records.push(ReplayRecord {
            event_id: event.event_id,
            timestamp: event.timestamp,
            hash_after: details.hash_after.clone(),
            snapshot,
        });
    }
    Ok(records)
}

fn canonical_snapshot_hash<P>(profile: &P) -> Result<String>
where
    P: RegenerableProfile,
{
    let mut clone = profile.clone();
    clone.history_mut().clear();
    let canonical = serde_json::to_vec(&clone)?;
    Ok(compute_hash(&canonical))
}

trait RegenerableProfile: Clone + Serialize {
    fn metadata(&self) -> &ProfileMetadata;
    fn metadata_mut(&mut self) -> &mut ProfileMetadata;
    fn history(&self) -> &Vec<HistoryRef>;
    fn history_mut(&mut self) -> &mut Vec<HistoryRef>;
}

impl RegenerableProfile for UserProfile {
    fn metadata(&self) -> &ProfileMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ProfileMetadata {
        &mut self.metadata
    }

    fn history(&self) -> &Vec<HistoryRef> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<HistoryRef> {
        &mut self.history
    }
}

impl RegenerableProfile for WorkProfile {
    fn metadata(&self) -> &ProfileMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ProfileMetadata {
        &mut self.metadata
    }

    fn history(&self) -> &Vec<HistoryRef> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<HistoryRef> {
        &mut self.history
    }
}

impl RegenerableProfile for WritingProfile {
    fn metadata(&self) -> &ProfileMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ProfileMetadata {
        &mut self.metadata
    }

    fn history(&self) -> &Vec<HistoryRef> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<HistoryRef> {
        &mut self.history
    }
}

impl RegenerableProfile for KnowledgeProfile {
    fn metadata(&self) -> &ProfileMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ProfileMetadata {
        &mut self.metadata
    }

    fn history(&self) -> &Vec<HistoryRef> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<HistoryRef> {
        &mut self.history
    }
}
