use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMusicRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrumental: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_at_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_track_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_format: Option<String>,
}

impl CreateMusicRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            title: None,
            tags: Vec::new(),
            lyrics: None,
            instrumental: None,
            duration_seconds: None,
            continue_at_seconds: None,
            continue_track_id: None,
            audio_format: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_lyrics(mut self, lyrics: impl Into<String>) -> Self {
        self.lyrics = Some(lyrics.into());
        self
    }

    pub fn with_instrumental(mut self, instrumental: bool) -> Self {
        self.instrumental = Some(instrumental);
        self
    }

    pub fn with_duration_seconds(mut self, duration_seconds: f64) -> Self {
        self.duration_seconds = Some(duration_seconds);
        self
    }

    pub fn with_continue_at_seconds(mut self, continue_at_seconds: f64) -> Self {
        self.continue_at_seconds = Some(continue_at_seconds);
        self
    }

    pub fn with_continue_track_id(mut self, continue_track_id: impl Into<String>) -> Self {
        self.continue_track_id = Some(continue_track_id.into());
        self
    }

    pub fn with_audio_format(mut self, audio_format: impl Into<String>) -> Self {
        self.audio_format = Some(audio_format.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMusicLyricsRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl CreateMusicLyricsRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            title: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicObject {
    pub id: String,
    pub object: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
}

impl MusicObject {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "music",
            status: None,
            model: None,
            title: None,
            audio_url: None,
            image_url: None,
            lyrics: None,
            duration_seconds: None,
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_audio_url(mut self, audio_url: impl Into<String>) -> Self {
        self.audio_url = Some(audio_url.into());
        self
    }

    pub fn with_image_url(mut self, image_url: impl Into<String>) -> Self {
        self.image_url = Some(image_url.into());
        self
    }

    pub fn with_lyrics(mut self, lyrics: impl Into<String>) -> Self {
        self.lyrics = Some(lyrics.into());
        self
    }

    pub fn with_duration_seconds(mut self, duration_seconds: f64) -> Self {
        self.duration_seconds = Some(duration_seconds);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTracksResponse {
    pub object: &'static str,
    pub data: Vec<MusicObject>,
}

impl MusicTracksResponse {
    pub fn new(data: Vec<MusicObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicLyricsObject {
    pub id: String,
    pub object: &'static str,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub text: String,
}

impl MusicLyricsObject {
    pub fn new(
        id: impl Into<String>,
        status: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            object: "music.lyrics",
            status: status.into(),
            title: None,
            text: text.into(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMusicResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteMusicResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "music.deleted",
            deleted: true,
        }
    }
}
