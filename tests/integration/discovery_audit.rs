use super::IntegrationHarness;
use researchbase::api::discovery::{approve_candidates, create_discovery_request, ApprovalPayload, DiscoveryRequestPayload};
use researchbase::models::discovery::{AcquisitionMode, DiscoveryMode};
use researchbase::orchestration::OrchestrationLog;

#[test]
fn discovery_events_capture_manifest_path() {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "disc-audit");

    let req = create_discovery_request(
        &manager,
        &base,
        DiscoveryRequestPayload {
            mode: DiscoveryMode::Topic,
            topic: Some("audit topic".into()),
            gap_id: None,
            session_id: None,
            count: Some(2),
        },
    )
    .expect("request");

    let first_id = req.candidates[0].id;
    let manifest_path = "consent/manifests/discovery-test.json".to_string();

    approve_candidates(
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

    let log = OrchestrationLog::for_base(&base);
    let events = log.load_events().expect("load events");
    let has_manifest = events.iter().any(|evt| {
        evt.event_type == researchbase::orchestration::EventType::DiscoveryApprovalRecorded
            && evt
                .details
                .get("prompt_manifest_path")
                .and_then(|v| v.as_str())
                .map(|p| p == manifest_path)
                .unwrap_or(false)
    });
    assert!(has_manifest, "expected approval event to contain manifest path");
}
