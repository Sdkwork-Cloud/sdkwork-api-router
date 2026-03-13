use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoRequest {
    pub model: String,
    pub prompt: String,
}

impl CreateVideoRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemixVideoRequest {
    pub prompt: String,
}

impl RemixVideoRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoObject {
    pub id: String,
    pub object: &'static str,
    pub url: String,
}

impl VideoObject {
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "video",
            url: url.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VideosResponse {
    pub object: &'static str,
    pub data: Vec<VideoObject>,
}

impl VideosResponse {
    pub fn new(data: Vec<VideoObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteVideoResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteVideoResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "video.deleted",
            deleted: true,
        }
    }
}
