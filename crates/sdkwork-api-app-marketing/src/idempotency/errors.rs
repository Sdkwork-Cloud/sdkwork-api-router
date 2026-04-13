#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketingIdempotencyError {
    InvalidKey,
    ConflictingKeys,
}

impl std::fmt::Display for MarketingIdempotencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "invalid idempotency key"),
            Self::ConflictingKeys => {
                write!(f, "conflicting idempotency keys between header and body")
            }
        }
    }
}

impl std::error::Error for MarketingIdempotencyError {}
