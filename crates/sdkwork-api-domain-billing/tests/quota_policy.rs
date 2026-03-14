use sdkwork_api_domain_billing::QuotaPolicy;

#[test]
fn quota_policy_captures_project_budget() {
    let policy = QuotaPolicy::new("quota-project-1", "project-1", 5_000).with_enabled(true);

    assert_eq!(policy.policy_id, "quota-project-1");
    assert_eq!(policy.project_id, "project-1");
    assert_eq!(policy.max_units, 5_000);
    assert!(policy.enabled);
}
