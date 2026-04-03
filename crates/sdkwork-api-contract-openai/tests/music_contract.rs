use sdkwork_api_contract_openai::music::{
    CreateMusicLyricsRequest, CreateMusicRequest, DeleteMusicResponse, MusicLyricsObject,
    MusicObject, MusicTracksResponse,
};

#[test]
fn serializes_music_resource_contracts() {
    let request = CreateMusicRequest::new("suno-v4", "Write a soaring electronic anthem")
        .with_title("Skyline Pulse")
        .with_tags(vec!["electronic".to_owned(), "anthem".to_owned()])
        .with_lyrics("We rise with the skyline".to_owned())
        .with_duration_seconds(123.0)
        .with_instrumental(false);
    let request_json = serde_json::to_value(request).unwrap();
    assert_eq!(request_json["model"], "suno-v4");
    assert_eq!(request_json["prompt"], "Write a soaring electronic anthem");
    assert_eq!(request_json["title"], "Skyline Pulse");
    assert_eq!(request_json["tags"][0], "electronic");
    assert_eq!(request_json["duration_seconds"], 123.0);
    assert_eq!(request_json["instrumental"], false);

    let lyrics_request = CreateMusicLyricsRequest::new("Write uplifting synth-pop lyrics")
        .with_title("Skyline Pulse");
    let lyrics_request_json = serde_json::to_value(lyrics_request).unwrap();
    assert_eq!(
        lyrics_request_json["prompt"],
        "Write uplifting synth-pop lyrics"
    );
    assert_eq!(lyrics_request_json["title"], "Skyline Pulse");

    let track = MusicObject::new("music_1")
        .with_status("completed")
        .with_model("suno-v4")
        .with_title("Skyline Pulse")
        .with_audio_url("https://example.com/music.mp3")
        .with_duration_seconds(123.0)
        .with_lyrics("We rise with the skyline");
    let track_json = serde_json::to_value(&track).unwrap();
    assert_eq!(track_json["object"], "music");
    assert_eq!(track_json["status"], "completed");
    assert_eq!(track_json["duration_seconds"], 123.0);

    let response = MusicTracksResponse::new(vec![track]);
    let response_json = serde_json::to_value(response).unwrap();
    assert_eq!(response_json["object"], "list");
    assert_eq!(response_json["data"][0]["id"], "music_1");

    let lyrics = MusicLyricsObject::new("lyrics_1", "completed", "We rise with the skyline")
        .with_title("Skyline Pulse");
    let lyrics_json = serde_json::to_value(lyrics).unwrap();
    assert_eq!(lyrics_json["object"], "music.lyrics");
    assert_eq!(lyrics_json["text"], "We rise with the skyline");

    let deleted = DeleteMusicResponse::deleted("music_1");
    let deleted_json = serde_json::to_value(deleted).unwrap();
    assert_eq!(deleted_json["object"], "music.deleted");
    assert_eq!(deleted_json["deleted"], true);
}
