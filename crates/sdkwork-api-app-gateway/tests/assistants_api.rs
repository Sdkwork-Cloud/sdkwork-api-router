use sdkwork_api_app_gateway::create_assistant;

#[test]
fn returns_assistant_object() {
    let response = create_assistant("tenant-1", "project-1", "Support", "gpt-4.1").unwrap();
    assert_eq!(response.object, "assistant");
    assert_eq!(response.model, "gpt-4.1");
}
