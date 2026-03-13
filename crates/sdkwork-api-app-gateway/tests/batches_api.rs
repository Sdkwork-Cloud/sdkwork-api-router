use sdkwork_api_app_gateway::create_batch;

#[test]
fn returns_batch_object() {
    let response = create_batch("tenant-1", "project-1", "/v1/responses", "file_1").unwrap();
    assert_eq!(response.object, "batch");
    assert_eq!(response.endpoint, "/v1/responses");
    assert_eq!(response.input_file_id, "file_1");
}

#[test]
fn lists_batch_objects() {
    let response = sdkwork_api_app_gateway::list_batches("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "batch");
}

#[test]
fn retrieves_batch_object() {
    let response = sdkwork_api_app_gateway::get_batch("tenant-1", "project-1", "batch_1").unwrap();
    assert_eq!(response.id, "batch_1");
    assert_eq!(response.object, "batch");
}

#[test]
fn cancels_batch_object() {
    let response =
        sdkwork_api_app_gateway::cancel_batch("tenant-1", "project-1", "batch_1").unwrap();
    assert_eq!(response.id, "batch_1");
    assert_eq!(response.status, "cancelled");
}
