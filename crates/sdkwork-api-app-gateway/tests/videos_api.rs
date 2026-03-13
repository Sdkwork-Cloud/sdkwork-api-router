use sdkwork_api_app_gateway::{
    create_video, delete_video, get_video, list_videos, remix_video, video_content,
};

#[test]
fn returns_video_list_for_create() {
    let response = create_video(
        "tenant-1",
        "project-1",
        "sora-1",
        "A short cinematic flyover",
    )
    .unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "video");
}

#[test]
fn lists_video_objects() {
    let response = list_videos("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].object, "video");
}

#[test]
fn retrieves_video_object() {
    let response = get_video("tenant-1", "project-1", "video_1").unwrap();
    assert_eq!(response.id, "video_1");
    assert_eq!(response.object, "video");
}

#[test]
fn deletes_video_object() {
    let response = delete_video("tenant-1", "project-1", "video_1").unwrap();
    assert_eq!(response.id, "video_1");
    assert!(response.deleted);
}

#[test]
fn returns_video_bytes() {
    let response = video_content("tenant-1", "project-1", "video_1").unwrap();
    assert_eq!(response, b"VIDEO".to_vec());
}

#[test]
fn remixes_video() {
    let response = remix_video("tenant-1", "project-1", "video_1", "Make it sunset").unwrap();
    assert_eq!(response.object, "list");
    assert_eq!(response.data[0].id, "video_1_remix");
}
