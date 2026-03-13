use sdkwork_api_app_gateway::create_eval;

#[test]
fn returns_eval_object() {
    let response = create_eval("tenant-1", "project-1", "qa-benchmark").unwrap();
    assert_eq!(response.object, "eval");
    assert_eq!(response.name, "qa-benchmark");
}
