use crate::MarketingServiceError;

pub(crate) fn normalize_required_identifier(
    value: &str,
    aggregate_label: &str,
    field_label: &str,
) -> Result<String, MarketingServiceError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(MarketingServiceError::invalid_state(format!(
            "{aggregate_label} create requires {field_label}"
        )));
    }
    Ok(normalized.to_owned())
}

pub(crate) fn normalize_optional_text(value: String) -> String {
    value.trim().to_owned()
}

pub(crate) fn normalize_optional_identifier(value: Option<String>) -> Option<String> {
    value
        .map(|entry| entry.trim().to_owned())
        .filter(|entry| !entry.is_empty())
}
