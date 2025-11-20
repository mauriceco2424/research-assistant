//! Knowledge profile utilities for strengths, weaknesses, and evidence maintenance.

use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Utc};

use super::model::{KnowledgeEntry, KnowledgeProfile, MasteryLevel, VerificationStatus};
use crate::orchestration::profiles::service::ProfileFieldChange;

impl KnowledgeProfile {
    pub fn find_entry_mut(&mut self, concept: &str) -> Option<&mut KnowledgeEntry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.concept.eq_ignore_ascii_case(concept))
    }
}

pub fn apply_knowledge_mutations(
    profile: &mut KnowledgeProfile,
    changes: &[ProfileFieldChange],
) -> Result<Vec<String>> {
    let mut diff = Vec::new();
    for change in changes {
        let (concept, field) = parse_concept_field(&change.field)?;
        let entry = profile
            .find_entry_mut(concept)
            .ok_or_else(|| anyhow!("Unknown knowledge concept '{concept}'"))?;
        match field {
            "mastery_level" | "mastery" => {
                entry.mastery_level = parse_mastery(&change.value)?;
                diff.push(format!("Updated mastery for {concept}"));
            }
            "weakness_flags" | "weaknesses" => {
                entry.weakness_flags = split_list(&change.value);
                diff.push(format!("Updated weaknesses for {concept}"));
            }
            "last_reviewed" => {
                entry.last_reviewed = parse_timestamp(&change.value)?;
                diff.push(format!("Updated last reviewed for {concept}"));
            }
            "verification_status" => {
                entry.verification_status = parse_verification(&change.value)?;
                diff.push(format!("Updated verification for {concept}"));
            }
            other => bail!("Unsupported field '{other}' for knowledge concept '{concept}'"),
        }
    }
    Ok(diff)
}

fn parse_concept_field(field: &str) -> Result<(&str, &str)> {
    let mut parts = field.splitn(2, '.');
    let concept = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Knowledge change requires concept.field syntax"))?;
    let attribute = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Knowledge change requires concept.field syntax"))?;
    Ok((concept, attribute))
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)
        .map_err(|err| anyhow!("Invalid timestamp '{value}': {err}"))?
        .with_timezone(&Utc))
}

fn parse_mastery(value: &str) -> Result<MasteryLevel> {
    match value.to_ascii_lowercase().as_str() {
        "novice" => Ok(MasteryLevel::Novice),
        "developing" => Ok(MasteryLevel::Developing),
        "proficient" => Ok(MasteryLevel::Proficient),
        "expert" => Ok(MasteryLevel::Expert),
        other => bail!("Invalid mastery level '{other}'"),
    }
}

fn parse_verification(value: &str) -> Result<VerificationStatus> {
    match value.to_ascii_lowercase().as_str() {
        "verified" => Ok(VerificationStatus::Verified),
        "unverified" => Ok(VerificationStatus::Unverified),
        "stale" => Ok(VerificationStatus::Stale),
        other => bail!("Invalid verification status '{other}'"),
    }
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}
