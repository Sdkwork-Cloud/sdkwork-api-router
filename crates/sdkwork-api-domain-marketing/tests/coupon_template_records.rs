use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponReservationRecord,
    CouponReservationStatus, CouponRestrictionSpec, CouponTemplateRecord,
    CouponTemplateStatus, MarketingBenefitKind, MarketingCampaignRecord,
    MarketingCampaignStatus, MarketingSubjectScope,
};

#[test]
fn coupon_template_captures_benefit_distribution_and_restrictions() {
    let template = CouponTemplateRecord::new(
        "tpl_launch_20",
        "launch-20",
        MarketingBenefitKind::PercentageOff,
    )
    .with_display_name("Launch 20% Off")
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::SharedCode)
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff)
            .with_discount_percent(Some(20))
            .with_max_discount_minor(Some(5_000)),
    )
    .with_restriction(
        CouponRestrictionSpec::new(MarketingSubjectScope::Project)
            .with_min_order_amount_minor(Some(1_000))
            .with_first_order_only(true)
            .with_exclusive_group(Some("launch-only".to_owned())),
    );

    assert_eq!(template.coupon_template_id, "tpl_launch_20");
    assert_eq!(template.status, CouponTemplateStatus::Active);
    assert_eq!(template.distribution_kind, CouponDistributionKind::SharedCode);
    assert_eq!(template.benefit.discount_percent, Some(20));
    assert_eq!(template.restriction.subject_scope, MarketingSubjectScope::Project);
    assert!(template.restriction.first_order_only);
    assert_eq!(
        template.restriction.exclusive_group.as_deref(),
        Some("launch-only")
    );
}

#[test]
fn coupon_code_and_reservation_expose_safety_helpers() {
    let code = CouponCodeRecord::new("code_launch_20", "tpl_launch_20", "shared_launch_20")
        .with_status(CouponCodeStatus::Available)
        .with_expires_at_ms(Some(2_000));
    let reservation = CouponReservationRecord::new(
        "res_launch_20",
        "code_launch_20",
        MarketingSubjectScope::Project,
        "project_demo",
        1_500,
    )
    .with_status(CouponReservationStatus::Reserved)
    .with_expires_at_ms(2_000);

    assert!(code.is_redeemable_at(1_500));
    assert!(!code.is_redeemable_at(2_100));
    assert!(reservation.is_active_at(1_500));
    assert!(!reservation.is_active_at(2_100));
}

#[test]
fn campaign_budget_and_campaign_report_effective_headroom() {
    let campaign = MarketingCampaignRecord::new("camp_launch", "tpl_launch_20")
        .with_status(MarketingCampaignStatus::Active)
        .with_start_at_ms(Some(1_000))
        .with_end_at_ms(Some(5_000));
    let budget = CampaignBudgetRecord::new("budget_launch", "camp_launch")
        .with_status(CampaignBudgetStatus::Active)
        .with_total_budget_minor(10_000)
        .with_reserved_budget_minor(1_500)
        .with_consumed_budget_minor(3_000);

    assert!(campaign.is_effective_at(2_000));
    assert!(!campaign.is_effective_at(5_100));
    assert_eq!(budget.available_budget_minor(), 5_500);
    assert!(budget.can_reserve(5_000));
    assert!(!budget.can_reserve(5_600));
}
