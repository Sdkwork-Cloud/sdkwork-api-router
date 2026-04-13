use super::*;

#[derive(Debug, Serialize)]
pub(crate) struct CommercialAccountSummaryResponse {
    pub(crate) account: AccountRecord,
    pub(crate) available_balance: f64,
    pub(crate) held_balance: f64,
    pub(crate) consumed_balance: f64,
    pub(crate) grant_balance: f64,
    pub(crate) active_lot_count: u64,
}

impl CommercialAccountSummaryResponse {
    pub(crate) fn from_balance(account: AccountRecord, balance: &AccountBalanceSnapshot) -> Self {
        Self {
            account,
            available_balance: balance.available_balance,
            held_balance: balance.held_balance,
            consumed_balance: balance.consumed_balance,
            grant_balance: balance.grant_balance,
            active_lot_count: balance.active_lot_count,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateQuotaPolicyRequest {
    pub(crate) policy_id: String,
    pub(crate) project_id: String,
    pub(crate) max_units: u64,
    #[serde(default = "default_true")]
    pub(crate) enabled: bool,
}
