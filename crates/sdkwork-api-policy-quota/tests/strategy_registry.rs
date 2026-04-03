use sdkwork_api_domain_billing::{QuotaCheckResult, QuotaPolicy};
use sdkwork_api_policy_quota::{
    builtin_quota_policy_registry, QuotaPolicyExecutionInput, STRICTEST_LIMIT_QUOTA_POLICY_ID,
};

#[test]
fn builtin_registry_resolves_strictest_limit_policy() {
    let registry = builtin_quota_policy_registry();

    assert!(registry.resolve(STRICTEST_LIMIT_QUOTA_POLICY_ID).is_some());
}

#[test]
fn strictest_limit_policy_uses_smallest_enabled_limit() {
    let registry = builtin_quota_policy_registry();
    let plugin = registry
        .resolve(STRICTEST_LIMIT_QUOTA_POLICY_ID)
        .expect("builtin strictest-limit quota policy must exist");
    let policies = vec![
        QuotaPolicy::new("quota-loose", "project-1", 500).with_enabled(true),
        QuotaPolicy::new("quota-strict", "project-1", 100).with_enabled(true),
        QuotaPolicy::new("quota-disabled", "project-1", 10).with_enabled(false),
    ];

    let evaluation = plugin.execute(QuotaPolicyExecutionInput {
        policies: &policies,
        used_units: 70,
        requested_units: 40,
    });

    assert_eq!(
        evaluation,
        QuotaCheckResult {
            allowed: false,
            policy_id: Some("quota-strict".to_owned()),
            requested_units: 40,
            used_units: 70,
            limit_units: Some(100),
            remaining_units: Some(30),
        }
    );
}

#[test]
fn strictest_limit_policy_breaks_equal_limits_by_policy_id() {
    let registry = builtin_quota_policy_registry();
    let plugin = registry
        .resolve(STRICTEST_LIMIT_QUOTA_POLICY_ID)
        .expect("builtin strictest-limit quota policy must exist");
    let policies = vec![
        QuotaPolicy::new("quota-b", "project-1", 100).with_enabled(true),
        QuotaPolicy::new("quota-a", "project-1", 100).with_enabled(true),
    ];

    let evaluation = plugin.execute(QuotaPolicyExecutionInput {
        policies: &policies,
        used_units: 40,
        requested_units: 10,
    });

    assert_eq!(evaluation.policy_id.as_deref(), Some("quota-a"));
    assert!(evaluation.allowed);
}

#[test]
fn strictest_limit_policy_allows_when_no_enabled_policy_exists() {
    let registry = builtin_quota_policy_registry();
    let plugin = registry
        .resolve(STRICTEST_LIMIT_QUOTA_POLICY_ID)
        .expect("builtin strictest-limit quota policy must exist");
    let policies = vec![
        QuotaPolicy::new("quota-disabled", "project-1", 10).with_enabled(false),
    ];

    let evaluation = plugin.execute(QuotaPolicyExecutionInput {
        policies: &policies,
        used_units: 90,
        requested_units: 50,
    });

    assert_eq!(
        evaluation,
        QuotaCheckResult::allowed_without_policy(50, 90)
    );
}
