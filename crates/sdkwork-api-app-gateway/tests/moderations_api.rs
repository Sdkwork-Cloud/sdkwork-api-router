use sdkwork_api_app_gateway::create_moderation;

#[test]
fn returns_unflagged_moderation_response() {
    let response = create_moderation("tenant-1", "project-1", "omni-moderation-latest").unwrap();
    assert_eq!(response.model, "omni-moderation-latest");
    assert_eq!(response.results.len(), 1);
    assert!(!response.results[0].flagged);
}
