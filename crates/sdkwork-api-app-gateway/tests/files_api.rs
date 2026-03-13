use sdkwork_api_contract_openai::files::CreateFileRequest;

#[test]
fn returns_file_object() {
    let request = CreateFileRequest::new("fine-tune", "train.jsonl", b"{}".to_vec());
    let response = sdkwork_api_app_gateway::create_file("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.object, "file");
    assert_eq!(response.filename, "train.jsonl");
    assert_eq!(response.bytes, 2);
}

#[test]
fn lists_file_objects() {
    let response = sdkwork_api_app_gateway::list_files("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "file");
}

#[test]
fn retrieves_file_object_by_id() {
    let response = sdkwork_api_app_gateway::get_file("tenant-1", "project-1", "file_1").unwrap();
    assert_eq!(response.id, "file_1");
    assert_eq!(response.object, "file");
}

#[test]
fn deletes_file_object_by_id() {
    let response = sdkwork_api_app_gateway::delete_file("tenant-1", "project-1", "file_1").unwrap();
    assert_eq!(response.id, "file_1");
    assert!(response.deleted);
}

#[test]
fn returns_file_content_bytes() {
    let response =
        sdkwork_api_app_gateway::file_content("tenant-1", "project-1", "file_1").unwrap();
    assert_eq!(response, b"{}".to_vec());
}
