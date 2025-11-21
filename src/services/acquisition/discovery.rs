use crate::acquisition::{run_acquisition_batch, CandidatePaper};
use crate::bases::{Base, BaseManager};
use crate::models::discovery::{
    AcquisitionMode, AcquisitionOutcomeStatus, DiscoveryAcquisitionOutcome,
    DiscoveryAcquisitionRecord, DiscoveryApprovalBatch, DiscoveryRequestRecord,
};
use crate::orchestration::events::{log_discovery_acquisition, log_discovery_approval};
use crate::storage::ai_layer::DiscoveryStore;
use anyhow::{anyhow, Result};

fn to_candidate_paper(candidate: &crate::models::discovery::DiscoveryCandidate) -> CandidatePaper {
    CandidatePaper {
        title: candidate.title.clone(),
        authors: candidate.authors.clone(),
        venue: candidate.venue.clone(),
        year: candidate.year,
        identifier: candidate
            .identifiers
            .doi
            .clone()
            .or(candidate.identifiers.arxiv.clone())
            .unwrap_or_else(|| candidate.id.to_string()),
        open_access: true,
    }
}

pub fn approve_and_acquire(
    manager: &BaseManager,
    base: &Base,
    request: &DiscoveryRequestRecord,
    approval: &DiscoveryApprovalBatch,
) -> Result<DiscoveryAcquisitionRecord> {
    let mut outcomes = Vec::new();
    let mut selected = Vec::new();

    for candidate_id in &approval.candidate_ids {
        let candidate = request
            .candidates
            .iter()
            .find(|c| &c.id == candidate_id)
            .ok_or_else(|| anyhow!("Candidate {} not found", candidate_id))?;
        selected.push(candidate.clone());
    }

    log_discovery_approval(manager, base, request, approval)?;

    match approval.acquisition_mode {
        AcquisitionMode::MetadataOnly => {
            let records: Vec<DiscoveryAcquisitionOutcome> = selected
                .iter()
                .map(|c| DiscoveryAcquisitionOutcome {
                    candidate_id: c.id,
                    outcome: AcquisitionOutcomeStatus::NeedsPdf,
                    pdf_path: None,
                    error_reason: Some("Metadata-only approval; PDF not requested.".into()),
                })
                .collect();
            let store = DiscoveryStore::new(base);
            let record = DiscoveryAcquisitionRecord {
                batch_id: approval.batch_id,
                base_id: base.id,
                outcomes: records.clone(),
                recorded_at: chrono::Utc::now(),
            };
            store.save_acquisition(&record)?;
            log_discovery_acquisition(manager, base, approval, &record)?;
            return Ok(record);
        }
        AcquisitionMode::MetadataAndPdf => {
            let converted: Vec<CandidatePaper> = selected.iter().map(to_candidate_paper).collect();
            let batch = run_acquisition_batch(manager, base, &converted, "discovery approval")?;
            for (idx, record) in batch.records.iter().enumerate() {
                let candidate = &selected[idx];
                outcomes.push(DiscoveryAcquisitionOutcome {
                    candidate_id: candidate.id,
                    outcome: if record.pdf_attached {
                        AcquisitionOutcomeStatus::Success
                    } else {
                        AcquisitionOutcomeStatus::NeedsPdf
                    },
                    pdf_path: record.library_entry_id.and_then(|_| {
                        record.pdf_attached.then(|| {
                            format!(
                                "pdfs/{}",
                                record.candidate_identifier.replace(['/', ':'], "_")
                            )
                        })
                    }),
                    error_reason: if record.pdf_attached {
                        None
                    } else {
                        Some("PDF not attached; manual follow-up required.".into())
                    },
                });
            }
            let store = DiscoveryStore::new(base);
            let record = DiscoveryAcquisitionRecord {
                batch_id: batch.batch_id,
                base_id: base.id,
                outcomes: outcomes.clone(),
                recorded_at: chrono::Utc::now(),
            };
            store.save_acquisition(&record)?;
            log_discovery_acquisition(manager, base, approval, &record)?;
            return Ok(record);
        }
    }
}
