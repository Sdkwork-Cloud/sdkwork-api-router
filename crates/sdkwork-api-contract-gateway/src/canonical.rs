#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalCapability {
    ChatCompletion,
    Responses,
    Embeddings,
    ModelListing,
    Streaming,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRequest {
    pub capability: CanonicalCapability,
    pub model: String,
}

impl CanonicalRequest {
    pub fn new(capability: CanonicalCapability, model: impl Into<String>) -> Self {
        Self {
            capability,
            model: model.into(),
        }
    }
}
