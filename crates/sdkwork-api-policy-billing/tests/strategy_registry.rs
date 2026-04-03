use sdkwork_api_domain_billing::BillingAccountingMode;
use sdkwork_api_policy_billing::{
    builtin_billing_policy_registry, BillingPolicyExecutionInput,
    GROUP_DEFAULT_BILLING_POLICY_ID,
};

#[test]
fn builtin_registry_resolves_group_default_billing_policy() {
    let registry = builtin_billing_policy_registry();

    assert!(registry.resolve(GROUP_DEFAULT_BILLING_POLICY_ID).is_some());
}

#[test]
fn group_default_policy_uses_api_key_group_accounting_mode_when_present() {
    let registry = builtin_billing_policy_registry();
    let plugin = registry
        .resolve(GROUP_DEFAULT_BILLING_POLICY_ID)
        .expect("builtin group-default billing policy must exist");

    let result = plugin
        .execute(BillingPolicyExecutionInput {
            api_key_group_default_accounting_mode: Some("byok"),
            default_accounting_mode: BillingAccountingMode::PlatformCredit,
            upstream_cost: Some(0.42),
            customer_charge: 0.89,
        })
        .expect("group default policy should parse supported accounting modes");

    assert_eq!(result.accounting_mode, BillingAccountingMode::Byok);
    assert!((result.upstream_cost - 0.42).abs() < 1e-9);
    assert!((result.customer_charge - 0.89).abs() < 1e-9);
    assert!(
        result
            .settlement_reason
            .contains("api key group default accounting mode")
    );
}

#[test]
fn group_default_policy_falls_back_to_default_mode_and_zero_upstream_cost() {
    let registry = builtin_billing_policy_registry();
    let plugin = registry
        .resolve(GROUP_DEFAULT_BILLING_POLICY_ID)
        .expect("builtin group-default billing policy must exist");

    let result = plugin
        .execute(BillingPolicyExecutionInput {
            api_key_group_default_accounting_mode: None,
            default_accounting_mode: BillingAccountingMode::PlatformCredit,
            upstream_cost: None,
            customer_charge: 1.25,
        })
        .expect("group default policy should preserve caller defaults");

    assert_eq!(result.accounting_mode, BillingAccountingMode::PlatformCredit);
    assert!((result.upstream_cost - 0.0).abs() < 1e-9);
    assert!((result.customer_charge - 1.25).abs() < 1e-9);
    assert!(result.settlement_reason.contains("default accounting mode"));
}

#[test]
fn group_default_policy_rejects_unknown_group_accounting_mode() {
    let registry = builtin_billing_policy_registry();
    let plugin = registry
        .resolve(GROUP_DEFAULT_BILLING_POLICY_ID)
        .expect("builtin group-default billing policy must exist");

    let error = plugin
        .execute(BillingPolicyExecutionInput {
            api_key_group_default_accounting_mode: Some("enterprise_wallet"),
            default_accounting_mode: BillingAccountingMode::PlatformCredit,
            upstream_cost: Some(0.11),
            customer_charge: 0.22,
        })
        .expect_err("unknown accounting modes must be rejected");

    assert!(
        error
            .to_string()
            .contains("platform_credit, byok, passthrough")
    );
}
