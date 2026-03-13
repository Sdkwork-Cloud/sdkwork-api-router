use sdkwork_api_contract_openai::files::CreateFileRequest;

#[test]
fn returns_file_object() {
    let request = CreateFileRequest::new("fine-tune", "train.jsonl", b"{}".to_vec());
    let response = sdkwork_api_app_gateway::create_file("tenant-1", "project-1", &request).unwrap();
    assert_eq!(response.object, "file");
    assert_eq!(response.filename, "train.jsonl");
    assert_eq!(response.bytes, 2);
}
