use crate::MarketingCouponContext;
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord,
    MarketingSubjectScope,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarketingSubjectSet {
    pub user_id: Option<String>,
    pub project_id: Option<String>,
    pub workspace_id: Option<String>,
    pub account_id: Option<String>,
}

impl MarketingSubjectSet {
    pub fn new(
        user_id: Option<String>,
        project_id: Option<String>,
        workspace_id: Option<String>,
        account_id: Option<String>,
    ) -> Self {
        Self {
            user_id,
            project_id,
            workspace_id,
            account_id,
        }
    }

    pub fn subject_id_for_scope(&self, scope: MarketingSubjectScope) -> Option<String> {
        match scope {
            MarketingSubjectScope::User => self.user_id.clone(),
            MarketingSubjectScope::Project => self.project_id.clone(),
            MarketingSubjectScope::Workspace => self.workspace_id.clone(),
            MarketingSubjectScope::Account => self.account_id.clone(),
        }
    }

    pub fn matches(&self, scope: MarketingSubjectScope, subject_id: &str) -> bool {
        match scope {
            MarketingSubjectScope::User => self.user_id.as_deref() == Some(subject_id),
            MarketingSubjectScope::Project => self.project_id.as_deref() == Some(subject_id),
            MarketingSubjectScope::Workspace => self.workspace_id.as_deref() == Some(subject_id),
            MarketingSubjectScope::Account => self.account_id.as_deref() == Some(subject_id),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarketingRedemptionSummary {
    pub total_count: usize,
    pub redeemed_count: usize,
    pub partially_rolled_back_count: usize,
    pub rolled_back_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarketingCodeSummary {
    pub total_count: usize,
    pub available_count: usize,
    pub reserved_count: usize,
    pub redeemed_count: usize,
    pub disabled_count: usize,
    pub expired_count: usize,
}

#[derive(Debug, Clone)]
pub struct MarketingCodeView {
    pub context: MarketingCouponContext,
    pub latest_reservation: Option<CouponReservationRecord>,
    pub latest_redemption: Option<CouponRedemptionRecord>,
}

#[derive(Debug, Clone)]
pub struct MarketingReservationOwnershipView {
    pub reservation: CouponReservationRecord,
    pub code: CouponCodeRecord,
}

#[derive(Debug, Clone)]
pub struct MarketingRedemptionOwnershipView {
    pub reservation: CouponReservationRecord,
    pub redemption: CouponRedemptionRecord,
    pub code: CouponCodeRecord,
}

#[derive(Debug, Clone)]
pub struct MarketingRewardHistoryView {
    pub context: MarketingCouponContext,
    pub redemption: CouponRedemptionRecord,
    pub rollbacks: Vec<CouponRollbackRecord>,
}
