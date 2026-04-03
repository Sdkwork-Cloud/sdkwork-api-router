use std::collections::HashMap;
use std::sync::Arc;

use sdkwork_api_domain_billing::{QuotaCheckResult, QuotaPolicy};

pub const STRICTEST_LIMIT_QUOTA_POLICY_ID: &str = "strictest_limit";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuotaPolicyPluginMetadata {
    pub plugin_id: &'static str,
    pub policy_id: &'static str,
}

pub struct QuotaPolicyExecutionInput<'a> {
    pub policies: &'a [QuotaPolicy],
    pub used_units: u64,
    pub requested_units: u64,
}

pub trait QuotaPolicyPlugin: Send + Sync {
    fn metadata(&self) -> QuotaPolicyPluginMetadata;

    fn execute(&self, input: QuotaPolicyExecutionInput<'_>) -> QuotaCheckResult;
}

#[derive(Default)]
pub struct QuotaPolicyPluginRegistry {
    plugins: HashMap<&'static str, Arc<dyn QuotaPolicyPlugin>>,
}

impl QuotaPolicyPluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, plugin: P) -> Option<Arc<dyn QuotaPolicyPlugin>>
    where
        P: QuotaPolicyPlugin + 'static,
    {
        self.register_arc(Arc::new(plugin))
    }

    pub fn register_arc(
        &mut self,
        plugin: Arc<dyn QuotaPolicyPlugin>,
    ) -> Option<Arc<dyn QuotaPolicyPlugin>> {
        let metadata = plugin.metadata();
        self.plugins.insert(metadata.policy_id, plugin)
    }

    pub fn resolve(&self, policy_id: &str) -> Option<Arc<dyn QuotaPolicyPlugin>> {
        self.plugins.get(policy_id).cloned()
    }
}

pub fn builtin_quota_policy_registry() -> QuotaPolicyPluginRegistry {
    let mut registry = QuotaPolicyPluginRegistry::new();
    registry.register(StrictestLimitQuotaPolicyPlugin);
    registry
}

struct StrictestLimitQuotaPolicyPlugin;

impl QuotaPolicyPlugin for StrictestLimitQuotaPolicyPlugin {
    fn metadata(&self) -> QuotaPolicyPluginMetadata {
        QuotaPolicyPluginMetadata {
            plugin_id: "sdkwork.quota.policy.strictest_limit",
            policy_id: STRICTEST_LIMIT_QUOTA_POLICY_ID,
        }
    }

    fn execute(&self, input: QuotaPolicyExecutionInput<'_>) -> QuotaCheckResult {
        let effective_policy = input
            .policies
            .iter()
            .filter(|policy| policy.enabled)
            .min_by(|left, right| {
                left.max_units
                    .cmp(&right.max_units)
                    .then_with(|| left.policy_id.cmp(&right.policy_id))
            });

        match effective_policy {
            Some(policy) => {
                QuotaCheckResult::from_policy(policy, input.used_units, input.requested_units)
            }
            None => QuotaCheckResult::allowed_without_policy(input.requested_units, input.used_units),
        }
    }
}
