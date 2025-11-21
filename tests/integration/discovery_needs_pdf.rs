use super::IntegrationHarness;
use researchbase::api::discovery::{approve_candidates, create_discovery_request, ApprovalPayload, DiscoveryRequestPayload};
use researchbase::models::discovery::{AcquisitionMode, DiscoveryMode, AcquisitionOutcomeStatus};
use researchbase::storage::ai_layer::DiscoveryStore;

#[test]
fn discovery_metadata_only_marks_needs_pdf() {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "disc-needs-pdf");

    let req = create_discovery_request(
        &manager,
        &base,
        DiscoveryRequestPayload {
            mode: DiscoveryMode::Topic,
            topic: Some("needs pdf".into()),
            gap_id: None,
            session_id: None,
            count: Some(1),
        },
    )
    .expect("request");

    let first_id = req.candidates[0].id;

    let approval = approve_candidates(
        &manager,
        &base,
        ApprovalPayload {
            request_id: req.request_id,
            acquisition_mode: AcquisitionMode::MetadataOnly,
            candidate_ids: vec![first_id],
            consent_manifest_path: None,
        },
    )
    .expect("approval");

    let store = DiscoveryStore::new(&base);
    let acquisition = store
        .load_acquisition(&approval.batch.batch_id)
        .expect("load acquisition")
        .expect("acquisition persisted");
    assert_eq!(acquisition.outcomes.len(), 1);
    assert_eq!(
        acquisition.outcomes[0].outcome,
        AcquisitionOutcomeStatus::NeedsPdf
    );
    assert!(
        acquisition.outcomes[0].error_reason.is_some(),
        "expected error reason explaining NEEDS_PDF"
    );
}
