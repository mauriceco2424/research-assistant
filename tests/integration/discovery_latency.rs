use super::IntegrationHarness;
use researchbase::api::discovery::{create_discovery_request, DiscoveryRequestPayload};
use researchbase::models::discovery::DiscoveryMode;
use std::time::Instant;

#[test]
fn discovery_request_returns_under_30s() {
    let harness = IntegrationHarness::new();
    let mut manager = harness.base_manager();
    let base = harness.create_base(&mut manager, "disc-latency");

    let payload = DiscoveryRequestPayload {
        mode: DiscoveryMode::Topic,
        topic: Some("test topic".into()),
        gap_id: None,
        session_id: None,
        count: Some(3),
    };
    let start = Instant::now();
    let resp = create_discovery_request(&manager, &base, payload).expect("request should succeed");
    let elapsed = start.elapsed();
    assert!(!resp.candidates.is_empty(), "expected candidates");
    assert!(
        elapsed.as_secs() < 30,
        "discovery request exceeded 30s: {:?}",
        elapsed
    );
}
