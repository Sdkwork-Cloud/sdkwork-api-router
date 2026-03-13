use sdkwork_api_app_gateway::create_completion;

#[test]
fn returns_completion_object() {
    let response = create_completion("tenant-1", "project-1", "gpt-3.5-turbo-instruct").unwrap();
    assert_eq!(response.object, "text_completion");
}
