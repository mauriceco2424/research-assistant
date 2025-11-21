use super::IntegrationHarness;
use researchbase::api::discovery::{approve_candidates, create_discovery_request, ApprovalPayload, DiscoveryRequestPayload};
use researchbase::models::discovery::{AcquisitionMode, DiscoveryMode};
use researchbase::storage::ai_layer::DiscoveryStore;

#[test]
fn discovery_provenance_persisted() {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "disc-provenance");

    let req = create_discovery_request(
        &manager,
        &base,
        DiscoveryRequestPayload {
            mode: DiscoveryMode::Topic,
            topic: Some("provenance topic".into()),
            gap_id: None,
            session_id: None,
            count: Some(2),
        },
    )
    .expect("request");

    let first_id = req.candidates[0].id;
    let manifest_path = "consent/manifests/provenance.json".to_string();

    let approval = approve_candidates(
        &manager,
        &base,
        ApprovalPayload {
            request_id: req.request_id,
            acquisition_mode: AcquisitionMode::MetadataOnly,
            candidate_ids: vec![first_id],
            consent_manifest_path: Some(manifest_path.clone()),
        },
    )
    .expect("approval");

    let store = DiscoveryStore::new(&base);
    let stored_approval = store
        .load_approval(&approval.batch.batch_id)
        .expect("load approval")
        .expect("approval persisted");
    assert_eq!(stored_approval.batch_id, approval.batch.batch_id);
    assert_eq!(
        stored_approval.consent_manifest_path,
        Some(manifest_path.clone())
    );
    let acquisition = store
        .load_acquisition(&approval.batch.batch_id)
        .expect("load acquisition")
        .expect("acquisition persisted");
    assert_eq!(acquisition.base_id, base.id);
    assert_eq!(acquisition.outcomes.len(), 1);
}
