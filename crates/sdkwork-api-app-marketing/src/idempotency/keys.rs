use super::errors::MarketingIdempotencyError;

pub fn normalize_idempotency_key(
    value: Option<&str>,
) -> Result<Option<String>, MarketingIdempotencyError> {
    let Some(value) = value.map(str::trim) else {
        return Ok(None);
    };
    if value.is_empty() || value.len() > 128 || value.chars().any(|ch| ch.is_control()) {
        return Err(MarketingIdempotencyError::InvalidKey);
    }
    Ok(Some(value.to_owned()))
}

pub fn resolve_idempotency_key(
    header_value: Option<&str>,
    body_value: Option<&str>,
) -> Result<Option<String>, MarketingIdempotencyError> {
    let body_value = normalize_idempotency_key(body_value)?;
    let header_value = normalize_idempotency_key(header_value)?;
    match (body_value, header_value) {
        (Some(body_value), Some(header_value)) if body_value != header_value => {
            Err(MarketingIdempotencyError::ConflictingKeys)
        }
        (Some(body_value), Some(_)) | (Some(body_value), None) => Ok(Some(body_value)),
        (None, Some(header_value)) => Ok(Some(header_value)),
        (None, None) => Ok(None),
    }
}
