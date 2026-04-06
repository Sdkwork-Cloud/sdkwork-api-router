use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommerceOrderRecord {
    pub order_id: String,
    pub project_id: String,
    pub user_id: String,
    pub target_kind: String,
    pub target_id: String,
    pub target_name: String,
    pub list_price_cents: u64,
    pub payable_price_cents: u64,
    pub list_price_label: String,
    pub payable_price_label: String,
    pub granted_units: u64,
    pub bonus_units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_coupon_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupon_reservation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupon_redemption_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketing_campaign_id: Option<String>,
    #[serde(default)]
    pub subsidy_amount_minor: u64,
    pub status: String,
    pub source: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CommerceOrderRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        order_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
        target_name: impl Into<String>,
        list_price_cents: u64,
        payable_price_cents: u64,
        list_price_label: impl Into<String>,
        payable_price_label: impl Into<String>,
        granted_units: u64,
        bonus_units: u64,
        status: impl Into<String>,
        source: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            order_id: order_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            target_kind: target_kind.into(),
            target_id: target_id.into(),
            target_name: target_name.into(),
            list_price_cents,
            payable_price_cents,
            list_price_label: list_price_label.into(),
            payable_price_label: payable_price_label.into(),
            granted_units,
            bonus_units,
            applied_coupon_code: None,
            coupon_reservation_id: None,
            coupon_redemption_id: None,
            marketing_campaign_id: None,
            subsidy_amount_minor: 0,
            status: status.into(),
            source: source.into(),
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_applied_coupon_code_option(mut self, applied_coupon_code: Option<String>) -> Self {
        self.applied_coupon_code = applied_coupon_code;
        self
    }

    pub fn with_coupon_reservation_id_option(
        mut self,
        coupon_reservation_id: Option<String>,
    ) -> Self {
        self.coupon_reservation_id = coupon_reservation_id;
        self
    }

    pub fn with_coupon_redemption_id_option(
        mut self,
        coupon_redemption_id: Option<String>,
    ) -> Self {
        self.coupon_redemption_id = coupon_redemption_id;
        self
    }

    pub fn with_marketing_campaign_id_option(
        mut self,
        marketing_campaign_id: Option<String>,
    ) -> Self {
        self.marketing_campaign_id = marketing_campaign_id;
        self
    }

    pub fn with_subsidy_amount_minor(mut self, subsidy_amount_minor: u64) -> Self {
        self.subsidy_amount_minor = subsidy_amount_minor;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommercePaymentEventProcessingStatus {
    Received,
    Processed,
    Ignored,
    Rejected,
    Failed,
}

impl CommercePaymentEventProcessingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Processed => "processed",
            Self::Ignored => "ignored",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for CommercePaymentEventProcessingStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "received" => Ok(Self::Received),
            "processed" => Ok(Self::Processed),
            "ignored" => Ok(Self::Ignored),
            "rejected" => Ok(Self::Rejected),
            "failed" => Ok(Self::Failed),
            other => Err(format!(
                "unknown commerce payment event processing status: {other}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommercePaymentEventRecord {
    pub payment_event_id: String,
    pub order_id: String,
    pub project_id: String,
    pub user_id: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_event_id: Option<String>,
    pub dedupe_key: String,
    pub event_type: String,
    pub payload_json: String,
    pub processing_status: CommercePaymentEventProcessingStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processing_message: Option<String>,
    pub received_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_status_after: Option<String>,
}

impl CommercePaymentEventRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_event_id: impl Into<String>,
        order_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        provider: impl Into<String>,
        dedupe_key: impl Into<String>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        received_at_ms: u64,
    ) -> Self {
        Self {
            payment_event_id: payment_event_id.into(),
            order_id: order_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            provider: provider.into(),
            provider_event_id: None,
            dedupe_key: dedupe_key.into(),
            event_type: event_type.into(),
            payload_json: payload_json.into(),
            processing_status: CommercePaymentEventProcessingStatus::Received,
            processing_message: None,
            received_at_ms,
            processed_at_ms: None,
            order_status_after: None,
        }
    }

    pub fn with_provider_event_id(mut self, provider_event_id: Option<String>) -> Self {
        self.provider_event_id = provider_event_id;
        self
    }

    pub fn with_processing_status(
        mut self,
        processing_status: CommercePaymentEventProcessingStatus,
    ) -> Self {
        self.processing_status = processing_status;
        self
    }

    pub fn with_processing_message(mut self, processing_message: Option<String>) -> Self {
        self.processing_message = processing_message;
        self
    }

    pub fn with_processed_at_ms(mut self, processed_at_ms: Option<u64>) -> Self {
        self.processed_at_ms = processed_at_ms;
        self
    }

    pub fn with_order_status_after(mut self, order_status_after: Option<String>) -> Self {
        self.order_status_after = order_status_after;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProjectMembershipRecord {
    pub membership_id: String,
    pub project_id: String,
    pub user_id: String,
    pub plan_id: String,
    pub plan_name: String,
    pub price_cents: u64,
    pub price_label: String,
    pub cadence: String,
    pub included_units: u64,
    pub status: String,
    pub source: String,
    pub activated_at_ms: u64,
    pub updated_at_ms: u64,
}

impl ProjectMembershipRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        membership_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        plan_id: impl Into<String>,
        plan_name: impl Into<String>,
        price_cents: u64,
        price_label: impl Into<String>,
        cadence: impl Into<String>,
        included_units: u64,
        status: impl Into<String>,
        source: impl Into<String>,
        activated_at_ms: u64,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            membership_id: membership_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            plan_id: plan_id.into(),
            plan_name: plan_name.into(),
            price_cents,
            price_label: price_label.into(),
            cadence: cadence.into(),
            included_units,
            status: status.into(),
            source: source.into(),
            activated_at_ms,
            updated_at_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CommerceOrderRecord, CommercePaymentEventProcessingStatus, CommercePaymentEventRecord,
        ProjectMembershipRecord,
    };

    #[test]
    fn commerce_order_keeps_operational_fields() {
        let order = CommerceOrderRecord::new(
            "order_1",
            "project_demo",
            "user_demo",
            "recharge_pack",
            "pack-100k",
            "Boost 100k",
            4_000,
            3_200,
            "$40.00",
            "$32.00",
            100_000,
            0,
            "fulfilled",
            "workspace_seed",
            1_710_000_001,
        )
        .with_applied_coupon_code_option(Some("SPRING20".to_owned()));

        assert_eq!(order.order_id, "order_1");
        assert_eq!(order.project_id, "project_demo");
        assert_eq!(order.user_id, "user_demo");
        assert_eq!(order.target_kind, "recharge_pack");
        assert_eq!(order.payable_price_cents, 3_200);
        assert_eq!(order.granted_units, 100_000);
        assert_eq!(order.applied_coupon_code.as_deref(), Some("SPRING20"));
        assert!(order.coupon_reservation_id.is_none());
        assert!(order.coupon_redemption_id.is_none());
        assert!(order.marketing_campaign_id.is_none());
        assert_eq!(order.subsidy_amount_minor, 0);
        assert_eq!(order.created_at_ms, 1_710_000_001);
        assert_eq!(order.updated_at_ms, 1_710_000_001);
    }

    #[test]
    fn project_membership_captures_active_plan_entitlements() {
        let membership = ProjectMembershipRecord::new(
            "membership_1",
            "project_demo",
            "user_demo",
            "growth",
            "Growth",
            7_900,
            "$79.00",
            "/month",
            100_000,
            "active",
            "workspace_seed",
            1_710_000_100,
            1_710_000_100,
        );

        assert_eq!(membership.project_id, "project_demo");
        assert_eq!(membership.plan_id, "growth");
        assert_eq!(membership.plan_name, "Growth");
        assert_eq!(membership.included_units, 100_000);
        assert_eq!(membership.status, "active");
    }

    #[test]
    fn commerce_payment_event_keeps_audit_and_processing_fields() {
        let event = CommercePaymentEventRecord::new(
            "payment_event_1",
            "order_1",
            "project_demo",
            "user_demo",
            "stripe",
            "stripe:evt_1",
            "settled",
            "{\"event_type\":\"settled\"}",
            1_710_000_200,
        )
        .with_provider_event_id(Some("evt_1".to_owned()))
        .with_processing_status(CommercePaymentEventProcessingStatus::Processed)
        .with_processed_at_ms(Some(1_710_000_250))
        .with_order_status_after(Some("fulfilled".to_owned()));

        assert_eq!(event.order_id, "order_1");
        assert_eq!(event.provider, "stripe");
        assert_eq!(event.provider_event_id.as_deref(), Some("evt_1"));
        assert_eq!(event.dedupe_key, "stripe:evt_1");
        assert_eq!(
            event.processing_status,
            CommercePaymentEventProcessingStatus::Processed
        );
        assert_eq!(event.order_status_after.as_deref(), Some("fulfilled"));
    }
}
