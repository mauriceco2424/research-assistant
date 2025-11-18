pub mod figure_store;

pub use figure_store::{FigureAssetRecord, FigureExtractionStatus, FigureStore};

use crate::bases::{Base, BaseManager, LibraryEntry};
use crate::orchestration::{
    log_event, AcquisitionBatch, AcquisitionRecord, EventType, OrchestrationLog,
};
use anyhow::Result;
use chrono::Utc;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// User-provided answers for the Path B interview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewAnswers {
    pub topic: String,
    pub research_questions: Vec<String>,
    pub expertise_level: String,
}

/// Candidate paper returned by the AI layer for acquisition consideration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePaper {
    pub title: String,
    pub authors: Vec<String>,
    pub venue: Option<String>,
    pub year: Option<i32>,
    pub identifier: String,
    pub open_access: bool,
}

impl CandidatePaper {
    pub fn metadata_summary(&self) -> String {
        format!(
            "{} ({})",
            self.title,
            self.year
                .map(|y| y.to_string())
                .unwrap_or_else(|| "n.d.".into())
        )
    }
}

/// Simple heuristic generator for candidate papers based on interview data.
pub fn generate_candidates_from_interview(answers: &InterviewAnswers) -> Vec<CandidatePaper> {
    let mut candidates = Vec::new();
    for (idx, question) in answers.research_questions.iter().enumerate() {
        let identifier = format!(
            "doi:10.1234/{}/{}",
            answers.topic.replace(' ', "_"),
            idx + 1
        );
        candidates.push(CandidatePaper {
            title: format!("{} - {}", answers.topic, question),
            authors: vec!["ResearchBot".into(), answers.expertise_level.clone()],
            venue: Some("ResearchBase Suggestions".into()),
            year: Some(2020 + idx as i32),
            identifier,
            open_access: idx % 2 == 0,
        });
    }
    candidates
}

/// Generate discovery candidates based on a free-form request.
pub fn discover_candidates(topic: &str, count: usize) -> Vec<CandidatePaper> {
    let mut rng = rand::thread_rng();
    let mut candidates = Vec::new();
    let venues = ["ArXiv", "OpenReview", "JMLR", "Nature", "Science"];
    for idx in 0..count {
        candidates.push(CandidatePaper {
            title: format!("{} - lead {}", topic, idx + 1),
            authors: vec!["AutoDiscover".into()],
            venue: Some(
                venues
                    .choose(&mut rng)
                    .unwrap_or(&"ResearchBase")
                    .to_string(),
            ),
            year: Some(2019 + (idx as i32 % 5)),
            identifier: format!("doi:10.5678/{topic}/{}", idx + 1),
            open_access: idx % 3 != 0,
        });
    }
    candidates
}

/// Executes the Paper Acquisition Workflow for an approved set of candidates.
pub fn run_acquisition_batch(
    manager: &BaseManager,
    base: &Base,
    candidates: &[CandidatePaper],
    approval_text: &str,
) -> Result<AcquisitionBatch> {
    let mut entries = manager.load_library_entries(base)?;
    let mut records = Vec::new();

    for candidate in candidates {
        let entry_id = Uuid::new_v4();
        let mut entry = LibraryEntry {
            entry_id,
            title: candidate.title.clone(),
            authors: candidate.authors.clone(),
            venue: candidate.venue.clone(),
            year: candidate.year,
            identifier: candidate.identifier.clone(),
            pdf_paths: Vec::new(),
            needs_pdf: true,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut pdf_attached = false;
        if candidate.open_access {
            let pdf_path = write_placeholder_pdf(&base.user_layer_path, &candidate.identifier)?;
            entry.mark_pdf_attached(pdf_path);
            pdf_attached = true;
        }

        let needs_pdf = entry.pdf_paths.is_empty();
        entries.push(entry);
        records.push(AcquisitionRecord {
            candidate_identifier: candidate.identifier.clone(),
            title: candidate.title.clone(),
            authors: candidate.authors.clone(),
            pdf_attached,
            needs_pdf,
            library_entry_id: Some(entry_id),
        });
    }

    manager.save_library_entries(base, &entries)?;

    let batch = AcquisitionBatch::new(base.id, approval_text.to_string(), records, Utc::now());
    let log = OrchestrationLog::for_base(base);
    log.record_batch(&batch)?;

    log_event(
        manager,
        base,
        EventType::AcquisitionApproved,
        serde_json::json!({
            "batch_id": batch.batch_id,
            "selection_count": candidates.len(),
            "approval_text": approval_text
        }),
    )?;

    Ok(batch)
}

/// Undo the last acquisition batch for a base.
pub fn undo_last_batch(manager: &BaseManager, base: &Base) -> Result<Option<AcquisitionBatch>> {
    let log = OrchestrationLog::for_base(base);
    log.undo_last_batch(base, manager)
}

fn write_placeholder_pdf(user_layer: &PathBuf, identifier: &str) -> Result<PathBuf> {
    let safe_name = identifier.replace(['/', ':'], "_");
    let pdf_dir = user_layer.join("pdfs");
    fs::create_dir_all(&pdf_dir)?;
    let pdf_path = pdf_dir.join(format!("{safe_name}.txt"));
    fs::write(
        &pdf_path,
        format!("Placeholder for {identifier}. Actual PDF should be downloaded manually."),
    )?;
    Ok(pdf_path)
}
