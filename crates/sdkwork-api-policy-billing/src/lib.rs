use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use sdkwork_api_domain_billing::BillingAccountingMode;

pub const GROUP_DEFAULT_BILLING_POLICY_ID: &str = "group_default";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BillingPolicyPluginMetadata {
    pub plugin_id: &'static str,
    pub policy_id: &'static str,
}

pub struct BillingPolicyExecutionInput<'a> {
    pub api_key_group_default_accounting_mode: Option<&'a str>,
    pub default_accounting_mode: BillingAccountingMode,
    pub upstream_cost: Option<f64>,
    pub customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BillingPolicyExecutionResult {
    pub accounting_mode: BillingAccountingMode,
    pub upstream_cost: f64,
    pub customer_charge: f64,
    pub settlement_reason: String,
}

impl BillingPolicyExecutionResult {
    pub fn new(
        accounting_mode: BillingAccountingMode,
        upstream_cost: f64,
        customer_charge: f64,
        settlement_reason: impl Into<String>,
    ) -> Self {
        Self {
            accounting_mode,
            upstream_cost,
            customer_charge,
            settlement_reason: settlement_reason.into(),
        }
    }
}

pub trait BillingPolicyPlugin: Send + Sync {
    fn metadata(&self) -> BillingPolicyPluginMetadata;

    fn execute(
        &self,
        input: BillingPolicyExecutionInput<'_>,
    ) -> Result<BillingPolicyExecutionResult>;
}

#[derive(Default)]
pub struct BillingPolicyPluginRegistry {
    plugins: HashMap<&'static str, Arc<dyn BillingPolicyPlugin>>,
}

impl BillingPolicyPluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, plugin: P) -> Option<Arc<dyn BillingPolicyPlugin>>
    where
        P: BillingPolicyPlugin + 'static,
    {
        self.register_arc(Arc::new(plugin))
    }

    pub fn register_arc(
        &mut self,
        plugin: Arc<dyn BillingPolicyPlugin>,
    ) -> Option<Arc<dyn BillingPolicyPlugin>> {
        let metadata = plugin.metadata();
        self.plugins.insert(metadata.policy_id, plugin)
    }

    pub fn resolve(&self, policy_id: &str) -> Option<Arc<dyn BillingPolicyPlugin>> {
        self.plugins.get(policy_id).cloned()
    }
}

pub fn builtin_billing_policy_registry() -> BillingPolicyPluginRegistry {
    let mut registry = BillingPolicyPluginRegistry::new();
    registry.register(GroupDefaultBillingPolicyPlugin);
    registry
}

struct GroupDefaultBillingPolicyPlugin;

impl BillingPolicyPlugin for GroupDefaultBillingPolicyPlugin {
    fn metadata(&self) -> BillingPolicyPluginMetadata {
        BillingPolicyPluginMetadata {
            plugin_id: "sdkwork.billing.policy.group_default",
            policy_id: GROUP_DEFAULT_BILLING_POLICY_ID,
        }
    }

    fn execute(
        &self,
        input: BillingPolicyExecutionInput<'_>,
    ) -> Result<BillingPolicyExecutionResult> {
        let upstream_cost = input.upstream_cost.unwrap_or(0.0);
        let resolved = resolve_accounting_mode(
            input.api_key_group_default_accounting_mode,
            input.default_accounting_mode,
        )?;

        Ok(BillingPolicyExecutionResult::new(
            resolved.accounting_mode,
            upstream_cost,
            input.customer_charge,
            resolved.settlement_reason,
        ))
    }
}

struct ResolvedAccountingMode {
    accounting_mode: BillingAccountingMode,
    settlement_reason: String,
}

fn resolve_accounting_mode(
    api_key_group_default_accounting_mode: Option<&str>,
    default_accounting_mode: BillingAccountingMode,
) -> Result<ResolvedAccountingMode> {
    let Some(group_default) = api_key_group_default_accounting_mode
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(ResolvedAccountingMode {
            accounting_mode: default_accounting_mode,
            settlement_reason: format!(
                "applied default accounting mode {} because no api key group override was configured",
                default_accounting_mode.as_str()
            ),
        });
    };

    let normalized = group_default.to_ascii_lowercase();
    let accounting_mode =
        BillingAccountingMode::from_str(&normalized).map_err(|_| {
            anyhow!(
                "api key group default_accounting_mode must be one of: platform_credit, byok, passthrough"
            )
        })?;

    Ok(ResolvedAccountingMode {
        accounting_mode,
        settlement_reason: format!(
            "applied api key group default accounting mode {}",
            accounting_mode.as_str()
        ),
    })
}
