use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CreateFileRequest {
    pub purpose: String,
    pub filename: String,
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

impl CreateFileRequest {
    pub fn new(purpose: impl Into<String>, filename: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            purpose: purpose.into(),
            filename: filename.into(),
            bytes,
            content_type: None,
        }
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FileObject {
    pub id: String,
    pub object: &'static str,
    pub purpose: String,
    pub filename: String,
    pub bytes: u64,
    pub status: &'static str,
}

impl FileObject {
    pub fn new(
        id: impl Into<String>,
        filename: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        Self::with_bytes(id, filename, purpose, 0)
    }

    pub fn with_bytes(
        id: impl Into<String>,
        filename: impl Into<String>,
        purpose: impl Into<String>,
        bytes: u64,
    ) -> Self {
        Self {
            id: id.into(),
            object: "file",
            purpose: purpose.into(),
            filename: filename.into(),
            bytes,
            status: "processed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListFilesResponse {
    pub object: &'static str,
    pub data: Vec<FileObject>,
}

impl ListFilesResponse {
    pub fn new(data: Vec<FileObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteFileResponse {
    pub id: String,
    pub object: &'static str,
    pub deleted: bool,
}

impl DeleteFileResponse {
    pub fn deleted(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "file",
            deleted: true,
        }
    }
}
