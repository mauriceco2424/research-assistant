use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::bases::{Base, BaseManager};
use crate::orchestration::profiles::model::{RemoteInferenceStatus, StyleExample, WritingProfile};
use crate::orchestration::{require_remote_operation_consent, ConsentOperation, ConsentScope};
use crate::profiles::writing_profile::{
    StyleAnalysisMethod, StyleModelRecord, WritingProfileStore,
};

use super::WritingResult;

const REMOTE_SIZE_THRESHOLD_BYTES: u64 = 15 * 1024 * 1024;
const REMOTE_PAGE_THRESHOLD: usize = 60;

/// Enumerates the structured interview questions captured for each project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleInterviewQuestionId {
    Tone,
    Venue,
    SectionEmphasis,
    CitationDensity,
}

impl StyleInterviewQuestionId {
    pub fn as_str(&self) -> &'static str {
        match self {
            StyleInterviewQuestionId::Tone => "tone",
            StyleInterviewQuestionId::Venue => "venue",
            StyleInterviewQuestionId::SectionEmphasis => "section_emphasis",
            StyleInterviewQuestionId::CitationDensity => "citation_density",
        }
    }
}

/// Static interview question descriptor used by chat commands.
#[derive(Debug, Clone, Copy)]
pub struct StyleInterviewQuestion {
    pub id: StyleInterviewQuestionId,
    pub prompt: &'static str,
    pub hint: &'static str,
}

/// User response captured for a specific question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleInterviewResponse {
    pub question_id: StyleInterviewQuestionId,
    pub answer: String,
}

/// Result of applying interview responses to the WritingProfile.
#[derive(Debug, Clone)]
pub struct StyleInterviewOutcome {
    pub profile: WritingProfile,
    pub responses: Vec<StyleInterviewResponse>,
    pub summary: Vec<String>,
}

/// Request describing a user-provided PDF for style analysis.
#[derive(Debug, Clone)]
pub struct StyleModelSource {
    pub path: PathBuf,
    pub require_remote: bool,
    pub provider_hint: Option<String>,
    pub notes: Option<String>,
}

impl StyleModelSource {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            require_remote: false,
            provider_hint: None,
            notes: None,
        }
    }

    pub fn with_remote(mut self) -> Self {
        self.require_remote = true;
        self
    }
}

/// Summary of newly recorded style models plus any required consents.
#[derive(Debug)]
pub struct StyleModelIngestionResult {
    pub profile: WritingProfile,
    pub records: Vec<StyleModelRecord>,
    pub remote_consents: Vec<RemoteStyleConsent>,
}

/// Lightweight description of a consent manifest logged for remote analysis.
#[derive(Debug, Clone)]
pub struct RemoteStyleConsent {
    pub manifest_id: Uuid,
    pub approval_text: String,
    pub prompt_manifest: Value,
}

const STYLE_QUESTIONS: [StyleInterviewQuestion; 4] = [
    StyleInterviewQuestion {
        id: StyleInterviewQuestionId::Tone,
        prompt: "What tone should the draft adopt (e.g., confident, conversational, rigorous)?",
        hint: "Provide 1-3 comma-separated descriptors.",
    },
    StyleInterviewQuestion {
        id: StyleInterviewQuestionId::Venue,
        prompt: "What target venue or audience should this project align with?",
        hint: "Conference/journal name or short description.",
    },
    StyleInterviewQuestion {
        id: StyleInterviewQuestionId::SectionEmphasis,
        prompt: "Are there sections or evidence types that deserve special emphasis?",
        hint: "E.g., 'methods should highlight ablation results'.",
    },
    StyleInterviewQuestion {
        id: StyleInterviewQuestionId::CitationDensity,
        prompt: "How dense should citations be and are there citation styles to follow/avoid?",
        hint: "Describe expected frequency or formatting preferences.",
    },
];

/// Returns the fixed set of questions used for the writing style interview.
pub fn style_interview_questions() -> &'static [StyleInterviewQuestion] {
    &STYLE_QUESTIONS
}

/// Applies interview responses to the WritingProfile and persists the result.
pub fn run_style_interview(
    base: &Base,
    responses: &[StyleInterviewResponse],
) -> WritingResult<StyleInterviewOutcome> {
    let store = WritingProfileStore::new(base);
    let mut profile = store.load_profile()?;
    if responses.is_empty() {
        return Ok(StyleInterviewOutcome {
            profile,
            responses: Vec::new(),
            summary: Vec::new(),
        });
    }

    let mut updated_tone = profile.fields.tone_descriptors.clone();
    let mut structure_notes = profile.fields.structure_preferences.clone();
    let mut summary_lines = Vec::new();

    for response in responses {
        let answer = response.answer.trim();
        if answer.is_empty() {
            continue;
        }
        match response.question_id {
            StyleInterviewQuestionId::Tone => {
                updated_tone = parse_list(answer);
                if updated_tone.is_empty() {
                    updated_tone.push(answer.to_string());
                }
                summary_lines.push(format!("Tone guidance: {}", answer));
            }
            StyleInterviewQuestionId::Venue => {
                structure_notes.push(format!("Target venue/audience: {}", answer));
                summary_lines.push(format!("Audience: {}", answer));
            }
            StyleInterviewQuestionId::SectionEmphasis => {
                structure_notes.push(format!("Section emphasis: {}", answer));
                summary_lines.push(format!("Section emphasis: {}", answer));
            }
            StyleInterviewQuestionId::CitationDensity => {
                structure_notes.push(format!("Citation expectations: {}", answer));
                summary_lines.push(format!("Citation preferences: {}", answer));
            }
        }
    }

    dedup_preserve_order(&mut updated_tone);
    dedup_preserve_order(&mut structure_notes);

    if !summary_lines.is_empty() {
        profile.summary = summary_lines.clone();
    }
    profile.fields.tone_descriptors = updated_tone;
    profile.fields.structure_preferences = structure_notes;
    profile.metadata.last_updated = Utc::now();
    store.save_profile(&profile)?;

    Ok(StyleInterviewOutcome {
        profile,
        responses: responses.to_vec(),
        summary: summary_lines,
    })
}

/// Runs local style model analysis for each source and records metadata.
pub fn ingest_style_models(
    manager: &BaseManager,
    base: &Base,
    sources: &[StyleModelSource],
) -> WritingResult<StyleModelIngestionResult> {
    if sources.is_empty() {
        let store = WritingProfileStore::new(base);
        let profile = store.load_profile()?;
        return Ok(StyleModelIngestionResult {
            profile,
            records: Vec::new(),
            remote_consents: Vec::new(),
        });
    }

    let store = WritingProfileStore::new(base);
    let mut profile = store.load_profile()?;
    let mut records = Vec::new();
    let mut remote_consents = Vec::new();

    for source in sources {
        let metrics = StyleModelMetrics::from_path(&source.path)?;
        let mut analysis_method = StyleAnalysisMethod::Local;
        let mut consent_token = None;
        if let Some(reason) = remote_reason(&metrics, source.require_remote) {
            let manifest = require_remote_operation_consent(
                manager,
                base,
                ConsentOperation::ProfileInterviewRemote,
                "Remote style model analysis requested",
                ConsentScope::default(),
                json!({
                    "operation": "style_model_remote_analysis",
                    "file": source.path.display().to_string(),
                    "reason": reason,
                }),
            )?;
            consent_token = Some(manifest.manifest_id.to_string());
            remote_consents.push(RemoteStyleConsent {
                manifest_id: manifest.manifest_id,
                approval_text: manifest.approval_text.clone(),
                prompt_manifest: manifest.prompt_manifest.clone(),
            });
            profile.fields.remote_inference_metadata.last_remote_source =
                Some(source.path.display().to_string());
            profile.fields.remote_inference_metadata.consent_manifest_id =
                Some(manifest.manifest_id);
            profile.fields.remote_inference_metadata.status = RemoteInferenceStatus::Approved;
            analysis_method = StyleAnalysisMethod::Remote {
                provider_id: source
                    .provider_hint
                    .clone()
                    .unwrap_or_else(|| "remote-style-analysis".to_string()),
            };
        }

        let record = store.record_style_model(
            source.path.clone(),
            analysis_method,
            metrics.as_value(),
            consent_token,
            source.notes.clone(),
        )?;
        append_style_example(&mut profile, &record, &metrics);
        records.push(record);
    }

    if !records.is_empty() {
        profile.summary.push(format!(
            "Recorded {} style model(s) on {}",
            records.len(),
            Utc::now().format("%Y-%m-%d")
        ));
        profile.metadata.last_updated = Utc::now();
        store.save_profile(&profile)?;
    }

    Ok(StyleModelIngestionResult {
        profile,
        records,
        remote_consents,
    })
}

fn append_style_example(
    profile: &mut WritingProfile,
    record: &StyleModelRecord,
    metrics: &StyleModelMetrics,
) {
    profile.fields.style_examples.push(StyleExample {
        source: record.source_pdf_path.display().to_string(),
        excerpt: format!(
            "{} words · {:.3} citation density",
            metrics.word_estimate, metrics.citation_density
        ),
        citation: None,
    });
    const STYLE_EXAMPLE_LIMIT: usize = 5;
    if profile.fields.style_examples.len() > STYLE_EXAMPLE_LIMIT {
        let drop_count = profile.fields.style_examples.len() - STYLE_EXAMPLE_LIMIT;
        profile.fields.style_examples.drain(0..drop_count);
    }
}

fn remote_reason(metrics: &StyleModelMetrics, force_remote: bool) -> Option<String> {
    if force_remote {
        return Some("User requested remote-only analysis".to_string());
    }
    if metrics.byte_len > REMOTE_SIZE_THRESHOLD_BYTES {
        return Some("File exceeds local analysis size threshold".to_string());
    }
    if metrics.page_estimate > REMOTE_PAGE_THRESHOLD {
        return Some("Estimated page count exceeds local analyzer capacity".to_string());
    }
    None
}

fn parse_list(answer: &str) -> Vec<String> {
    answer
        .split(|ch: char| ch == ',' || ch == '\n')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

fn dedup_preserve_order(items: &mut Vec<String>) {
    let mut seen = Vec::new();
    items.retain(|entry| {
        if seen
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(entry))
        {
            false
        } else {
            seen.push(entry.clone());
            true
        }
    });
}

struct StyleModelMetrics {
    byte_len: u64,
    word_estimate: usize,
    citation_hits: usize,
    citation_density: f32,
    fingerprint: String,
    page_estimate: usize,
}

impl StyleModelMetrics {
    fn from_path(path: &Path) -> WritingResult<Self> {
        let data = fs::read(path)
            .with_context(|| format!("Failed to read style model source {}", path.display()))?;
        if data.is_empty() {
            bail!(
                "Style model source {} is empty; provide a valid PDF or text export.",
                path.display()
            );
        }
        let bytes = data.len() as u64;
        let text = String::from_utf8_lossy(&data);
        let word_estimate = text.split_whitespace().count();
        let lowered = text.to_ascii_lowercase();
        let citation_hits = lowered.matches("\\cite").count() + lowered.matches(" et al").count();
        let citation_density = if word_estimate == 0 {
            0.0
        } else {
            citation_hits as f32 / word_estimate as f32
        };
        let digest = Sha256::digest(&data);
        let fingerprint = format!("{:x}", digest);
        let page_estimate = ((word_estimate as f32 / 4000.0).ceil() as usize).max(1);
        Ok(Self {
            byte_len: bytes,
            word_estimate,
            citation_hits,
            citation_density,
            fingerprint,
            page_estimate,
        })
    }

    fn as_value(&self) -> Value {
        json!({
            "byteLength": self.byte_len,
            "wordEstimate": self.word_estimate,
            "citationHits": self.citation_hits,
            "citationDensity": self.citation_density,
            "fingerprint": self.fingerprint,
            "pageEstimate": self.page_estimate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bases::{AppConfig, Base, BaseManager, WorkspacePaths};
    use tempfile::TempDir;

    fn temp_base_context() -> (TempDir, BaseManager, Base) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_path_buf();
        let user_dir = root.join("User");
        let ai_dir = root.join("AI");
        fs::create_dir_all(&user_dir).unwrap();
        fs::create_dir_all(&ai_dir).unwrap();
        let workspace = WorkspacePaths {
            root: root.clone(),
            user_dir: user_dir.clone(),
            ai_dir: ai_dir.clone(),
        };
        let config_path = root.join("config").join("config.toml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        let manager = BaseManager {
            config: AppConfig::default(),
            paths: workspace,
            config_path,
        };
        let base = Base {
            id: Uuid::new_v4(),
            name: "Style Test".into(),
            slug: "style-test".into(),
            user_layer_path: user_dir.join("style-test"),
            ai_layer_path: ai_dir.join("style-test"),
            created_at: Utc::now(),
            last_active_at: None,
        };
        fs::create_dir_all(&base.user_layer_path.join("profiles")).unwrap();
        fs::create_dir_all(&base.ai_layer_path.join("profiles")).unwrap();
        fs::create_dir_all(base.ai_layer_path.join("consent/manifests")).unwrap();
        (tmp, manager, base)
    }

    #[test]
    fn interview_updates_profile_fields() {
        let (_tmp, _manager, base) = temp_base_context();
        let responses = vec![
            StyleInterviewResponse {
                question_id: StyleInterviewQuestionId::Tone,
                answer: "Confident, precise".into(),
            },
            StyleInterviewResponse {
                question_id: StyleInterviewQuestionId::Venue,
                answer: "ICLR main track".into(),
            },
            StyleInterviewResponse {
                question_id: StyleInterviewQuestionId::SectionEmphasis,
                answer: "Related work should highlight contrastive learning".into(),
            },
        ];
        let outcome = run_style_interview(&base, &responses).unwrap();
        assert!(outcome
            .profile
            .fields
            .tone_descriptors
            .contains(&"Confident".to_string()));
        assert!(outcome
            .profile
            .fields
            .structure_preferences
            .iter()
            .any(|entry| entry.contains("ICLR")));
    }

    #[test]
    fn ingest_local_style_model_records_metrics() {
        let (_tmp, manager, base) = temp_base_context();
        let sample_path = base.user_layer_path.join("sample.txt");
        fs::create_dir_all(sample_path.parent().unwrap()).unwrap();
        fs::write(
            &sample_path,
            "This draft cites prior work \\cite{smith2024}. Et al references abound.",
        )
        .unwrap();

        let result =
            ingest_style_models(&manager, &base, &[StyleModelSource::new(&sample_path)]).unwrap();
        assert_eq!(result.records.len(), 1);
        assert!(result.profile.fields.style_examples.len() >= 1);
    }

    #[test]
    fn remote_ingestion_logs_consent() {
        let (_tmp, mut manager, base) = temp_base_context();
        manager.config.acquisition.remote_allowed = true;
        let sample_path = base.user_layer_path.join("remote.txt");
        fs::create_dir_all(sample_path.parent().unwrap()).unwrap();
        fs::write(&sample_path, "Large document requiring GPU.").unwrap();

        let source = StyleModelSource {
            path: sample_path,
            require_remote: true,
            provider_hint: Some("gpu-style".into()),
            notes: Some("Uploaded from laptop".into()),
        };
        let result = ingest_style_models(&manager, &base, &[source]).unwrap();
        assert_eq!(result.remote_consents.len(), 1);
        assert!(result
            .profile
            .fields
            .remote_inference_metadata
            .consent_manifest_id
            .is_some());
    }
}
