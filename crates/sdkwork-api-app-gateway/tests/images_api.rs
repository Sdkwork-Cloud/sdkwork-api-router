use sdkwork_api_app_gateway::create_image_generation;

#[test]
fn returns_images_response() {
    let response = create_image_generation("tenant-1", "project-1", "gpt-image-1").unwrap();
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].b64_json, "sdkwork-image");
}
