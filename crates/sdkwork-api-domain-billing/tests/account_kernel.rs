use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountRecord, AccountStatus, AccountType, PricingPlanRecord, PricingRateRecord,
};

#[test]
fn account_record_keeps_canonical_bigint_subject_fields() {
    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status(AccountStatus::Active)
        .with_allow_overdraft(false)
        .with_overdraft_limit(0.0)
        .with_created_at_ms(1_717_171_717)
        .with_updated_at_ms(1_717_171_718);

    assert_eq!(account.account_id, 7001);
    assert_eq!(account.tenant_id, 1001);
    assert_eq!(account.organization_id, 2002);
    assert_eq!(account.user_id, 9001);
    assert_eq!(account.account_type, AccountType::Primary);
    assert_eq!(account.currency_code, "USD");
    assert_eq!(account.credit_unit_code, "credit");
    assert_eq!(account.status, AccountStatus::Active);
    assert!(!account.allow_overdraft);
    assert_eq!(account.overdraft_limit, 0.0);
}

#[test]
fn benefit_lot_and_pricing_plan_records_capture_commercial_kernel_metadata() {
    let lot = AccountBenefitLotRecord::new(8001, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
        .with_source_type(AccountBenefitSourceType::Recharge)
        .with_source_id(Some(9901))
        .with_scope_json(Some("{\"capability\":\"responses\"}".to_owned()))
        .with_original_quantity(1200.0)
        .with_remaining_quantity(1200.0)
        .with_held_quantity(0.0)
        .with_priority(10)
        .with_acquired_unit_cost(Some(1.0))
        .with_issued_at_ms(1_717_171_720)
        .with_expires_at_ms(Some(1_717_999_999))
        .with_status(AccountBenefitLotStatus::Active)
        .with_created_at_ms(1_717_171_720)
        .with_updated_at_ms(1_717_171_721);
    let plan = PricingPlanRecord::new(9101, 1001, 2002, "default-retail", 3)
        .with_display_name("Default Retail v3")
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_status("active")
        .with_created_at_ms(1_717_171_730)
        .with_updated_at_ms(1_717_171_731);
    let rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openai-official".to_owned()))
        .with_quantity_step(1.0)
        .with_unit_price(0.0025)
        .with_created_at_ms(1_717_171_740);

    assert_eq!(lot.account_id, 7001);
    assert_eq!(lot.user_id, 9001);
    assert_eq!(lot.benefit_type, AccountBenefitType::CashCredit);
    assert_eq!(lot.source_type, AccountBenefitSourceType::Recharge);
    assert_eq!(lot.source_id, Some(9901));
    assert_eq!(lot.status, AccountBenefitLotStatus::Active);
    assert_eq!(plan.plan_code, "default-retail");
    assert_eq!(plan.plan_version, 3);
    assert_eq!(rate.pricing_plan_id, 9101);
    assert_eq!(rate.metric_code, "token.input");
    assert_eq!(rate.model_code.as_deref(), Some("gpt-4.1"));
}
