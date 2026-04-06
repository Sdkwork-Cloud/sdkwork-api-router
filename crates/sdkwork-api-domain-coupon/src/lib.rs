use serde::{Deserialize, Serialize};

/// Compatibility-era coupon record.
///
/// This model exists to preserve current admin and portal behavior while the
/// canonical marketing kernel is introduced. New business semantics such as
/// reservation, redemption, rollback, budget, and stackability must not be
/// added here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponCampaign {
    pub id: String,
    pub code: String,
    pub discount_label: String,
    pub audience: String,
    pub remaining: u64,
    pub active: bool,
    pub note: String,
    pub expires_on: String,
    #[serde(default)]
    pub created_at_ms: u64,
}

impl CouponCampaign {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        code: impl Into<String>,
        discount_label: impl Into<String>,
        audience: impl Into<String>,
        remaining: u64,
        active: bool,
        note: impl Into<String>,
        expires_on: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            discount_label: discount_label.into(),
            audience: audience.into(),
            remaining,
            active,
            note: note.into(),
            expires_on: expires_on.into(),
            created_at_ms: 0,
        }
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    /// Compatibility-layer availability used by legacy coupon reads.
    pub fn is_compatibility_live(&self) -> bool {
        self.active && self.remaining > 0
    }
}
