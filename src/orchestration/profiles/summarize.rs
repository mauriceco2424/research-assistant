use super::model::{
    KnowledgeProfile, UserProfile, WorkProfile, WritingProfile,
};

#[derive(Debug, Clone)]
pub struct ProfileSummary {
    pub highlights: Vec<String>,
    pub fields: Vec<(String, String)>,
}

impl ProfileSummary {
    pub fn new() -> Self {
        Self {
            highlights: Vec::new(),
            fields: Vec::new(),
        }
    }
}

pub fn summarize_user(profile: &UserProfile) -> ProfileSummary {
    let mut summary = ProfileSummary::new();
    if profile.summary.is_empty() {
        if !profile.fields.name.is_empty() {
            summary
                .highlights
                .push(format!("Primary contact: {}", profile.fields.name));
        }
        if !profile.fields.communication_style.is_empty() {
            summary.highlights.push(format!(
                "Prefers {} communication",
                join_list(&profile.fields.communication_style)
            ));
        }
    } else {
        summary.highlights.extend(profile.summary.clone());
    }
    summary.fields.push((
        "Name".into(),
        null_safe(&profile.fields.name),
    ));
    summary.fields.push((
        "Affiliations".into(),
        join_list(&profile.fields.affiliations),
    ));
    summary.fields.push((
        "Communication Style".into(),
        join_list(&profile.fields.communication_style),
    ));
    summary.fields.push((
        "Availability".into(),
        profile.fields.availability.clone().unwrap_or_else(|| "Unset".into()),
    ));
    summary
}

pub fn summarize_work(profile: &WorkProfile) -> ProfileSummary {
    let mut summary = ProfileSummary::new();
    if let Some(focus) = &profile.fields.focus_statement {
        summary
            .highlights
            .push(format!("Primary focus: {}", focus));
    } else {
        summary
            .highlights
            .push("No focus statement recorded yet.".into());
    }
    if !profile.fields.milestones.is_empty() {
        summary.highlights.push(format!(
            "{} milestones tracked",
            profile.fields.milestones.len()
        ));
    }
    summary.fields.push((
        "Focus Statement".into(),
        profile
            .fields
            .focus_statement
            .clone()
            .unwrap_or_else(|| "Unset".into()),
    ));
    summary.fields.push((
        "Active Projects".into(),
        profile
            .fields
            .active_projects
            .iter()
            .map(|project| {
                format!(
                    "{} ({})",
                    project.name,
                    project
                        .status
                        .clone()
                        .unwrap_or_else(|| "status: n/a".into())
                )
            })
            .collect::<Vec<String>>()
            .join("; "),
    ));
    summary.fields.push((
        "Preferred Tools".into(),
        join_list(&profile.fields.preferred_tools),
    ));
    summary.fields.push(("Risks".into(), join_list(&profile.fields.risks)));
    summary
}

pub fn summarize_writing(profile: &WritingProfile) -> ProfileSummary {
    let mut summary = ProfileSummary::new();
    if profile.summary.is_empty() {
        if !profile.fields.tone_descriptors.is_empty() {
            summary.highlights.push(format!(
                "Tone: {}",
                join_list(&profile.fields.tone_descriptors)
            ));
        }
        if !profile.fields.structure_preferences.is_empty() {
            summary.highlights.push(format!(
                "Structure preferences: {}",
                join_list(&profile.fields.structure_preferences)
            ));
        }
    } else {
        summary.highlights.extend(profile.summary.clone());
    }
    summary.fields.push((
        "Tone Descriptors".into(),
        join_list(&profile.fields.tone_descriptors),
    ));
    summary.fields.push((
        "Structure Preferences".into(),
        join_list(&profile.fields.structure_preferences),
    ));
    summary.fields.push((
        "Style Examples".into(),
        profile
            .fields
            .style_examples
            .iter()
            .map(|example| format!("{} ({})", example.source, example.citation.clone().unwrap_or_else(|| "n/a".into())))
            .collect::<Vec<String>>()
            .join("; "),
    ));
    summary
}

pub fn summarize_knowledge(profile: &KnowledgeProfile) -> ProfileSummary {
    let mut summary = ProfileSummary::new();
    if profile.summary.is_empty() {
        let strengths = profile
            .entries
            .iter()
            .filter(|entry| entry.weakness_flags.is_empty())
            .count();
        let weaknesses = profile
            .entries
            .iter()
            .filter(|entry| !entry.weakness_flags.is_empty())
            .count();
        summary.highlights.push(format!(
            "{} strengths, {} weaknesses recorded",
            strengths, weaknesses
        ));
    } else {
        summary.highlights.extend(profile.summary.clone());
    }
    summary.fields.push((
        "Entries".into(),
        profile.entries.len().to_string(),
    ));
    let stale = profile
        .entries
        .iter()
        .filter(|entry| entry.verification_status != super::model::VerificationStatus::Verified)
        .count();
    summary.fields.push(("Unverified/STALE".into(), stale.to_string()));
    summary
}

fn join_list(values: &[String]) -> String {
    if values.is_empty() {
        "None".into()
    } else {
        values.join(", ")
    }
}

fn null_safe(value: &str) -> String {
    if value.trim().is_empty() {
        "Unset".into()
    } else {
        value.to_string()
    }
}
