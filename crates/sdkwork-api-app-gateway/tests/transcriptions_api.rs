use sdkwork_api_app_gateway::create_transcription;

#[test]
fn returns_transcription_object() {
    let response = create_transcription("tenant-1", "project-1", "gpt-4o-mini-transcribe").unwrap();
    assert_eq!(response.text, "sdkwork transcription");
}
