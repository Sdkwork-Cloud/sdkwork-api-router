#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouponValidationDecision {
    pub eligible: bool,
    pub rejection_reason: Option<String>,
    pub reservable_budget_minor: u64,
}

impl CouponValidationDecision {
    pub fn eligible(reservable_budget_minor: u64) -> Self {
        Self {
            eligible: true,
            rejection_reason: None,
            reservable_budget_minor,
        }
    }

    pub fn rejected(reason: &'static str) -> Self {
        Self {
            eligible: false,
            rejection_reason: Some(reason.to_owned()),
            reservable_budget_minor: 0,
        }
    }
}
