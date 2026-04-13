#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketingServiceError {
    InvalidState(String),
}

impl MarketingServiceError {
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidState(message.into())
    }
}

impl std::fmt::Display for MarketingServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidState(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for MarketingServiceError {}
