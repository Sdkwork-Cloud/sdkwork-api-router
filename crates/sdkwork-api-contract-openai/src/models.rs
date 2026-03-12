use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModelObject {
    pub id: String,
    pub object: &'static str,
    pub owned_by: String,
}

impl ModelObject {
    pub fn new(id: impl Into<String>, owned_by: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            object: "model",
            owned_by: owned_by.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ListModelsResponse {
    pub object: &'static str,
    pub data: Vec<ModelObject>,
}

impl ListModelsResponse {
    pub fn new(data: Vec<ModelObject>) -> Self {
        Self {
            object: "list",
            data,
        }
    }
}
