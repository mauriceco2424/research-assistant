use anyhow::Result;
use serde::Serialize;

use crate::bases::{Base, BaseManager};

use super::{
    linking::detect_missing_references,
    model::{
        KnowledgeProfile, MasteryLevel, ProfileType, ProjectRef, VerificationStatus, WorkProfile,
    },
    scope::ProfileScopeStore,
    service::ProfileService,
};

#[derive(Debug, Clone, Serialize)]
pub struct WorkContext {
    pub focus_statement: Option<String>,
    pub preferred_tools: Vec<String>,
    pub active_projects: Vec<ProjectRef>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeEntrySummary {
    pub concept: String,
    pub mastery_level: MasteryLevel,
    pub weakness_flags: Vec<String>,
    pub verification_status: VerificationStatus,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeSummaryCounts {
    pub strengths: usize,
    pub weaknesses: usize,
    pub unverified: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StaleEvidenceNotice {
    pub concept: String,
    pub missing_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeSummary {
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub counts: KnowledgeSummaryCounts,
    pub entries: Vec<KnowledgeEntrySummary>,
    pub stale_evidence_refs: Vec<StaleEvidenceNotice>,
}

pub fn get_work_context(manager: &BaseManager, base: &Base) -> Result<WorkContext> {
    let scope_store = ProfileScopeStore::new(base);
    scope_store.enforce_local_read(ProfileType::Work)?;
    let service = ProfileService::new(manager, base);
    let profile = service.load_work_profile()?;
    Ok(extract_work_context(&profile))
}

pub fn get_knowledge_summary(manager: &BaseManager, base: &Base) -> Result<KnowledgeSummary> {
    let scope_store = ProfileScopeStore::new(base);
    scope_store.enforce_local_read(ProfileType::Knowledge)?;
    let service = ProfileService::new(manager, base);
    let profile = service.load_knowledge_profile()?;
    let stale_refs = detect_missing_references(base, &profile);
    Ok(extract_knowledge_summary(&profile, stale_refs))
}

fn extract_work_context(profile: &WorkProfile) -> WorkContext {
    WorkContext {
        focus_statement: profile.fields.focus_statement.clone(),
        preferred_tools: profile.fields.preferred_tools.clone(),
        active_projects: profile.fields.active_projects.clone(),
        last_updated: profile.metadata.last_updated,
    }
}

fn extract_knowledge_summary(
    profile: &KnowledgeProfile,
    stale_refs: Vec<crate::orchestration::profiles::linking::StaleEvidenceRef>,
) -> KnowledgeSummary {
    let mut strengths = 0;
    let mut weaknesses = 0;
    let mut unverified = 0;
    let mut entries = Vec::new();
    for entry in &profile.entries {
        if entry.weakness_flags.is_empty() {
            strengths += 1;
        } else {
            weaknesses += 1;
        }
        if entry.verification_status != VerificationStatus::Verified {
            unverified += 1;
        }
        entries.push(KnowledgeEntrySummary {
            concept: entry.concept.clone(),
            mastery_level: entry.mastery_level,
            weakness_flags: entry.weakness_flags.clone(),
            verification_status: entry.verification_status,
            evidence_refs: entry
                .evidence_refs
                .iter()
                .map(|reference| {
                    format!(
                        "{}:{}",
                        match reference.kind {
                            super::model::EvidenceKind::Paper => "paper",
                            super::model::EvidenceKind::Note => "note",
                            super::model::EvidenceKind::Manual => "manual",
                        },
                        reference.identifier
                    )
                })
                .collect(),
        });
    }
    KnowledgeSummary {
        last_updated: profile.metadata.last_updated,
        counts: KnowledgeSummaryCounts {
            strengths,
            weaknesses,
            unverified,
        },
        entries,
        stale_evidence_refs: stale_refs
            .into_iter()
            .map(|notice| StaleEvidenceNotice {
                concept: notice.concept,
                missing_reference: notice.missing_reference,
            })
            .collect(),
    }
}
