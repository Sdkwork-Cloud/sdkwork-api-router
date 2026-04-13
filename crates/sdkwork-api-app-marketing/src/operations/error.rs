use crate::MarketingServiceError;

#[derive(Debug)]
pub enum MarketingOperationError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Forbidden(String),
    Storage(anyhow::Error),
}

impl MarketingOperationError {
    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub(crate) fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub(crate) fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }

    pub(crate) fn storage(error: impl Into<anyhow::Error>) -> Self {
        Self::Storage(error.into())
    }
}

impl std::fmt::Display for MarketingOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message)
            | Self::NotFound(message)
            | Self::Conflict(message)
            | Self::Forbidden(message) => write!(f, "{message}"),
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MarketingOperationError {}

impl From<MarketingServiceError> for MarketingOperationError {
    fn from(error: MarketingServiceError) -> Self {
        Self::Conflict(error.to_string())
    }
}
