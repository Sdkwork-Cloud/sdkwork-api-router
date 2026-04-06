use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingBenefitKind {
    PercentageOff,
    FixedAmountOff,
    GrantUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingStackingPolicy {
    Exclusive,
    Stackable,
    BestOfGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingSubjectScope {
    User,
    Project,
    Workspace,
    Account,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponTemplateStatus {
    Draft,
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponDistributionKind {
    SharedCode,
    UniqueCode,
    AutoClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingCampaignStatus {
    Draft,
    Scheduled,
    Active,
    Paused,
    Ended,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CampaignBudgetStatus {
    Draft,
    Active,
    Exhausted,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponCodeStatus {
    Available,
    Reserved,
    Redeemed,
    Expired,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponReservationStatus {
    Reserved,
    Released,
    Confirmed,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponRedemptionStatus {
    Pending,
    Redeemed,
    PartiallyRolledBack,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponRollbackType {
    Cancel,
    Refund,
    PartialRefund,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CouponRollbackStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketingOutboxEventStatus {
    Pending,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponBenefitSpec {
    pub benefit_kind: MarketingBenefitKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_amount_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_discount_minor: Option<u64>,
}

impl CouponBenefitSpec {
    pub fn new(benefit_kind: MarketingBenefitKind) -> Self {
        Self {
            benefit_kind,
            discount_percent: None,
            discount_amount_minor: None,
            grant_units: None,
            currency_code: None,
            max_discount_minor: None,
        }
    }

    pub fn with_discount_percent(mut self, discount_percent: Option<u8>) -> Self {
        self.discount_percent = discount_percent.map(|value| value.min(100));
        self
    }

    pub fn with_discount_amount_minor(mut self, discount_amount_minor: Option<u64>) -> Self {
        self.discount_amount_minor = discount_amount_minor;
        self
    }

    pub fn with_grant_units(mut self, grant_units: Option<u64>) -> Self {
        self.grant_units = grant_units;
        self
    }

    pub fn with_currency_code(mut self, currency_code: Option<String>) -> Self {
        self.currency_code = currency_code;
        self
    }

    pub fn with_max_discount_minor(mut self, max_discount_minor: Option<u64>) -> Self {
        self.max_discount_minor = max_discount_minor;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponRestrictionSpec {
    pub subject_scope: MarketingSubjectScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_order_amount_minor: Option<u64>,
    #[serde(default)]
    pub first_order_only: bool,
    #[serde(default)]
    pub new_customer_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusive_group: Option<String>,
    pub stacking_policy: MarketingStackingPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_redemptions_per_subject: Option<u64>,
    #[serde(default)]
    pub eligible_target_kinds: Vec<String>,
}

impl CouponRestrictionSpec {
    pub fn new(subject_scope: MarketingSubjectScope) -> Self {
        Self {
            subject_scope,
            min_order_amount_minor: None,
            first_order_only: false,
            new_customer_only: false,
            exclusive_group: None,
            stacking_policy: MarketingStackingPolicy::Exclusive,
            max_redemptions_per_subject: None,
            eligible_target_kinds: Vec::new(),
        }
    }

    pub fn with_min_order_amount_minor(mut self, min_order_amount_minor: Option<u64>) -> Self {
        self.min_order_amount_minor = min_order_amount_minor;
        self
    }

    pub fn with_first_order_only(mut self, first_order_only: bool) -> Self {
        self.first_order_only = first_order_only;
        self
    }

    pub fn with_new_customer_only(mut self, new_customer_only: bool) -> Self {
        self.new_customer_only = new_customer_only;
        self
    }

    pub fn with_exclusive_group(mut self, exclusive_group: Option<String>) -> Self {
        self.exclusive_group = exclusive_group;
        self
    }

    pub fn with_stacking_policy(mut self, stacking_policy: MarketingStackingPolicy) -> Self {
        self.stacking_policy = stacking_policy;
        self
    }

    pub fn with_max_redemptions_per_subject(
        mut self,
        max_redemptions_per_subject: Option<u64>,
    ) -> Self {
        self.max_redemptions_per_subject = max_redemptions_per_subject;
        self
    }

    pub fn with_eligible_target_kinds(mut self, eligible_target_kinds: Vec<String>) -> Self {
        self.eligible_target_kinds = eligible_target_kinds;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponTemplateRecord {
    pub coupon_template_id: String,
    pub template_key: String,
    pub display_name: String,
    pub status: CouponTemplateStatus,
    pub distribution_kind: CouponDistributionKind,
    pub benefit: CouponBenefitSpec,
    pub restriction: CouponRestrictionSpec,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponTemplateRecord {
    pub fn new(
        coupon_template_id: impl Into<String>,
        template_key: impl Into<String>,
        benefit_kind: MarketingBenefitKind,
    ) -> Self {
        Self {
            coupon_template_id: coupon_template_id.into(),
            template_key: template_key.into(),
            display_name: String::new(),
            status: CouponTemplateStatus::Draft,
            distribution_kind: CouponDistributionKind::SharedCode,
            benefit: CouponBenefitSpec::new(benefit_kind),
            restriction: CouponRestrictionSpec::new(MarketingSubjectScope::Project),
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_status(mut self, status: CouponTemplateStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_distribution_kind(mut self, distribution_kind: CouponDistributionKind) -> Self {
        self.distribution_kind = distribution_kind;
        self
    }

    pub fn with_benefit(mut self, benefit: CouponBenefitSpec) -> Self {
        self.benefit = benefit;
        self
    }

    pub fn with_restriction(mut self, restriction: CouponRestrictionSpec) -> Self {
        self.restriction = restriction;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MarketingCampaignRecord {
    pub marketing_campaign_id: String,
    pub coupon_template_id: String,
    pub display_name: String,
    pub status: MarketingCampaignStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl MarketingCampaignRecord {
    pub fn new(
        marketing_campaign_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
    ) -> Self {
        Self {
            marketing_campaign_id: marketing_campaign_id.into(),
            coupon_template_id: coupon_template_id.into(),
            display_name: String::new(),
            status: MarketingCampaignStatus::Draft,
            start_at_ms: None,
            end_at_ms: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_status(mut self, status: MarketingCampaignStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_start_at_ms(mut self, start_at_ms: Option<u64>) -> Self {
        self.start_at_ms = start_at_ms;
        self
    }

    pub fn with_end_at_ms(mut self, end_at_ms: Option<u64>) -> Self {
        self.end_at_ms = end_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn is_effective_at(&self, now_ms: u64) -> bool {
        if self.status != MarketingCampaignStatus::Active {
            return false;
        }

        let after_start = self.start_at_ms.is_none_or(|value| now_ms >= value);
        let before_end = self.end_at_ms.is_none_or(|value| now_ms <= value);
        after_start && before_end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CampaignBudgetRecord {
    pub campaign_budget_id: String,
    pub marketing_campaign_id: String,
    pub status: CampaignBudgetStatus,
    pub total_budget_minor: u64,
    pub reserved_budget_minor: u64,
    pub consumed_budget_minor: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CampaignBudgetRecord {
    pub fn new(
        campaign_budget_id: impl Into<String>,
        marketing_campaign_id: impl Into<String>,
    ) -> Self {
        Self {
            campaign_budget_id: campaign_budget_id.into(),
            marketing_campaign_id: marketing_campaign_id.into(),
            status: CampaignBudgetStatus::Draft,
            total_budget_minor: 0,
            reserved_budget_minor: 0,
            consumed_budget_minor: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_status(mut self, status: CampaignBudgetStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_total_budget_minor(mut self, total_budget_minor: u64) -> Self {
        self.total_budget_minor = total_budget_minor;
        self
    }

    pub fn with_reserved_budget_minor(mut self, reserved_budget_minor: u64) -> Self {
        self.reserved_budget_minor = reserved_budget_minor;
        self
    }

    pub fn with_consumed_budget_minor(mut self, consumed_budget_minor: u64) -> Self {
        self.consumed_budget_minor = consumed_budget_minor;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn available_budget_minor(&self) -> u64 {
        self.total_budget_minor
            .saturating_sub(self.reserved_budget_minor)
            .saturating_sub(self.consumed_budget_minor)
    }

    pub fn can_reserve(&self, amount_minor: u64) -> bool {
        self.status == CampaignBudgetStatus::Active && amount_minor <= self.available_budget_minor()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponCodeRecord {
    pub coupon_code_id: String,
    pub coupon_template_id: String,
    pub code_value: String,
    pub status: CouponCodeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_subject_scope: Option<MarketingSubjectScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_subject_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponCodeRecord {
    pub fn new(
        coupon_code_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
        code_value: impl Into<String>,
    ) -> Self {
        Self {
            coupon_code_id: coupon_code_id.into(),
            coupon_template_id: coupon_template_id.into(),
            code_value: code_value.into(),
            status: CouponCodeStatus::Available,
            claimed_subject_scope: None,
            claimed_subject_id: None,
            expires_at_ms: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_status(mut self, status: CouponCodeStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_claimed_subject(
        mut self,
        claimed_subject_scope: Option<MarketingSubjectScope>,
        claimed_subject_id: Option<String>,
    ) -> Self {
        self.claimed_subject_scope = claimed_subject_scope;
        self.claimed_subject_id = claimed_subject_id;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn is_redeemable_at(&self, now_ms: u64) -> bool {
        self.status == CouponCodeStatus::Available
            && self.expires_at_ms.is_none_or(|value| now_ms <= value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponReservationRecord {
    pub coupon_reservation_id: String,
    pub coupon_code_id: String,
    pub subject_scope: MarketingSubjectScope,
    pub subject_id: String,
    pub reservation_status: CouponReservationStatus,
    pub budget_reserved_minor: u64,
    pub expires_at_ms: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponReservationRecord {
    pub fn new(
        coupon_reservation_id: impl Into<String>,
        coupon_code_id: impl Into<String>,
        subject_scope: MarketingSubjectScope,
        subject_id: impl Into<String>,
        expires_at_ms: u64,
    ) -> Self {
        Self {
            coupon_reservation_id: coupon_reservation_id.into(),
            coupon_code_id: coupon_code_id.into(),
            subject_scope,
            subject_id: subject_id.into(),
            reservation_status: CouponReservationStatus::Reserved,
            budget_reserved_minor: 0,
            expires_at_ms,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_status(mut self, reservation_status: CouponReservationStatus) -> Self {
        self.reservation_status = reservation_status;
        self
    }

    pub fn with_budget_reserved_minor(mut self, budget_reserved_minor: u64) -> Self {
        self.budget_reserved_minor = budget_reserved_minor;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: u64) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn is_active_at(&self, now_ms: u64) -> bool {
        self.reservation_status == CouponReservationStatus::Reserved && now_ms <= self.expires_at_ms
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponRedemptionRecord {
    pub coupon_redemption_id: String,
    pub coupon_reservation_id: String,
    pub coupon_code_id: String,
    pub coupon_template_id: String,
    pub redemption_status: CouponRedemptionStatus,
    pub subsidy_amount_minor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_event_id: Option<String>,
    pub redeemed_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponRedemptionRecord {
    pub fn new(
        coupon_redemption_id: impl Into<String>,
        coupon_reservation_id: impl Into<String>,
        coupon_code_id: impl Into<String>,
        coupon_template_id: impl Into<String>,
        redeemed_at_ms: u64,
    ) -> Self {
        Self {
            coupon_redemption_id: coupon_redemption_id.into(),
            coupon_reservation_id: coupon_reservation_id.into(),
            coupon_code_id: coupon_code_id.into(),
            coupon_template_id: coupon_template_id.into(),
            redemption_status: CouponRedemptionStatus::Pending,
            subsidy_amount_minor: 0,
            order_id: None,
            payment_event_id: None,
            redeemed_at_ms,
            updated_at_ms: redeemed_at_ms,
        }
    }

    pub fn with_status(mut self, redemption_status: CouponRedemptionStatus) -> Self {
        self.redemption_status = redemption_status;
        self
    }

    pub fn with_subsidy_amount_minor(mut self, subsidy_amount_minor: u64) -> Self {
        self.subsidy_amount_minor = subsidy_amount_minor;
        self
    }

    pub fn with_order_id(mut self, order_id: Option<String>) -> Self {
        self.order_id = order_id;
        self
    }

    pub fn with_payment_event_id(mut self, payment_event_id: Option<String>) -> Self {
        self.payment_event_id = payment_event_id;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CouponRollbackRecord {
    pub coupon_rollback_id: String,
    pub coupon_redemption_id: String,
    pub rollback_type: CouponRollbackType,
    pub rollback_status: CouponRollbackStatus,
    pub restored_budget_minor: u64,
    pub restored_inventory_count: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CouponRollbackRecord {
    pub fn new(
        coupon_rollback_id: impl Into<String>,
        coupon_redemption_id: impl Into<String>,
        rollback_type: CouponRollbackType,
        created_at_ms: u64,
    ) -> Self {
        Self {
            coupon_rollback_id: coupon_rollback_id.into(),
            coupon_redemption_id: coupon_redemption_id.into(),
            rollback_type,
            rollback_status: CouponRollbackStatus::Pending,
            restored_budget_minor: 0,
            restored_inventory_count: 0,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, rollback_status: CouponRollbackStatus) -> Self {
        self.rollback_status = rollback_status;
        self
    }

    pub fn with_restored_budget_minor(mut self, restored_budget_minor: u64) -> Self {
        self.restored_budget_minor = restored_budget_minor;
        self
    }

    pub fn with_restored_inventory_count(mut self, restored_inventory_count: u64) -> Self {
        self.restored_inventory_count = restored_inventory_count;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MarketingOutboxEventRecord {
    pub marketing_outbox_event_id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub status: MarketingOutboxEventStatus,
    pub payload_json: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl MarketingOutboxEventRecord {
    pub fn new(
        marketing_outbox_event_id: impl Into<String>,
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            marketing_outbox_event_id: marketing_outbox_event_id.into(),
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            event_type: event_type.into(),
            status: MarketingOutboxEventStatus::Pending,
            payload_json: payload_json.into(),
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_status(mut self, status: MarketingOutboxEventStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}
