use sdkwork_api_contract_openai::videos::{
    CreateVideoRequest, DeleteVideoResponse, RemixVideoRequest, VideoObject, VideosResponse,
};

#[test]
fn serializes_video_resource_contracts() {
    let request = CreateVideoRequest::new("sora-1", "A short cinematic flyover");
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["model"], "sora-1");
    assert_eq!(request_json["prompt"], "A short cinematic flyover");

    let remix = RemixVideoRequest::new("Make it sunset");
    let remix_json = serde_json::to_value(remix).unwrap();
    assert_eq!(remix_json["prompt"], "Make it sunset");

    let video = VideoObject::new("video_1", "https://example.com/video.mp4");
    let video_json = serde_json::to_value(video).unwrap();
    assert_eq!(video_json["object"], "video");

    let response = VideosResponse::new(vec![VideoObject::new(
        "video_1",
        "https://example.com/video.mp4",
    )]);
    let response_json = serde_json::to_value(response).unwrap();
    assert_eq!(response_json["object"], "list");
    assert_eq!(response_json["data"][0]["id"], "video_1");

    let deleted = DeleteVideoResponse::deleted("video_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "video.deleted");
    assert_eq!(deleted_json["deleted"], true);
}
