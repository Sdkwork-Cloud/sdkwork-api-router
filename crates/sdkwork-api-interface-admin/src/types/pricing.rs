use super::*;

fn default_currency_code() -> String {
    "USD".to_owned()
}

fn default_credit_unit_code() -> String {
    "credit".to_owned()
}

fn default_charge_unit() -> String {
    "unit".to_owned()
}

fn default_pricing_method() -> String {
    "per_unit".to_owned()
}

fn default_rounding_increment() -> f64 {
    1.0
}

fn default_rounding_mode() -> String {
    "none".to_owned()
}

fn default_pricing_status() -> String {
    "draft".to_owned()
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCommercialPricingPlanRequest {
    pub(crate) tenant_id: u64,
    #[serde(default)]
    pub(crate) organization_id: u64,
    pub(crate) plan_code: String,
    pub(crate) plan_version: u64,
    pub(crate) display_name: String,
    #[serde(default = "default_currency_code")]
    pub(crate) currency_code: String,
    #[serde(default = "default_credit_unit_code")]
    pub(crate) credit_unit_code: String,
    #[serde(default = "default_pricing_status")]
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) effective_from_ms: u64,
    #[serde(default)]
    pub(crate) effective_to_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CloneCommercialPricingPlanRequest {
    #[serde(default)]
    pub(crate) plan_version: Option<u64>,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
    #[serde(default = "default_pricing_status")]
    pub(crate) status: String,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct PublishCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ScheduleCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RetireCommercialPricingPlanRequest {}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateCommercialPricingRateRequest {
    pub(crate) tenant_id: u64,
    #[serde(default)]
    pub(crate) organization_id: u64,
    pub(crate) pricing_plan_id: u64,
    pub(crate) metric_code: String,
    pub(crate) capability_code: Option<String>,
    pub(crate) model_code: Option<String>,
    pub(crate) provider_code: Option<String>,
    #[serde(default = "default_charge_unit")]
    pub(crate) charge_unit: String,
    #[serde(default = "default_pricing_method")]
    pub(crate) pricing_method: String,
    #[serde(default = "default_rounding_increment")]
    pub(crate) quantity_step: f64,
    #[serde(default)]
    pub(crate) unit_price: f64,
    #[serde(default)]
    pub(crate) display_price_unit: String,
    #[serde(default)]
    pub(crate) minimum_billable_quantity: f64,
    #[serde(default)]
    pub(crate) minimum_charge: f64,
    #[serde(default = "default_rounding_increment")]
    pub(crate) rounding_increment: f64,
    #[serde(default = "default_rounding_mode")]
    pub(crate) rounding_mode: String,
    #[serde(default)]
    pub(crate) included_quantity: f64,
    #[serde(default)]
    pub(crate) priority: u64,
    pub(crate) notes: Option<String>,
    #[serde(default = "default_pricing_status")]
    pub(crate) status: String,
}
