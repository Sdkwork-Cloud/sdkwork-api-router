use sdkwork_api_contract_gateway::canonical::{CanonicalCapability, CanonicalRequest};

#[test]
fn canonical_request_tracks_capability() {
    let request = CanonicalRequest::new(CanonicalCapability::ChatCompletion, "gpt-4.1");
    assert_eq!(request.capability, CanonicalCapability::ChatCompletion);
    assert_eq!(request.model, "gpt-4.1");
}
