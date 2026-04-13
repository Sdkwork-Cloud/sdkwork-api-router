#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarketingRecoveryRunReport {
    pub scanned_reservations: u64,
    pub expired_reservations: u64,
    pub released_codes: u64,
    pub released_budget_minor: u64,
    pub outbox_events_created: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct MarketingRecoveryReservationOutcome {
    pub(super) released_codes: u64,
    pub(super) released_budget_minor: u64,
}
