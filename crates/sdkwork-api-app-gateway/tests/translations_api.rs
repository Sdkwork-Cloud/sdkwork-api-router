use sdkwork_api_app_gateway::create_translation;

#[test]
fn returns_translation_object() {
    let response = create_translation("tenant-1", "project-1", "gpt-4o-mini-transcribe").unwrap();
    assert_eq!(response.text, "sdkwork translation");
}
