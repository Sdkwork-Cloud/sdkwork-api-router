use sdkwork_api_app_gateway::create_realtime_session;

#[test]
fn returns_realtime_session_object() {
    let response =
        create_realtime_session("tenant-1", "project-1", "gpt-4o-realtime-preview").unwrap();
    assert_eq!(response.object, "realtime.session");
    assert_eq!(response.model, "gpt-4o-realtime-preview");
}
