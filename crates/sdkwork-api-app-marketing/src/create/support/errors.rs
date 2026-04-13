use crate::governance::MarketingGovernanceError;
use crate::MarketingServiceError;

pub(crate) fn marketing_create_invalid_input(
    error: MarketingServiceError,
) -> MarketingGovernanceError {
    MarketingGovernanceError::InvalidInput(error.to_string())
}

pub(crate) fn marketing_create_storage(
    error: impl Into<anyhow::Error>,
) -> MarketingGovernanceError {
    MarketingGovernanceError::Storage(error.into())
}

pub(crate) fn marketing_create_conflicting_existing_state(
    aggregate_label: &str,
    aggregate_id: &str,
) -> MarketingGovernanceError {
    MarketingGovernanceError::Conflict(format!(
        "{aggregate_label} {aggregate_id} already exists with different state"
    ))
}
