use sdkwork_api_domain_coupon::CouponCampaign;

#[test]
fn coupon_campaign_keeps_operational_fields() {
    let coupon = CouponCampaign::new(
        "coupon_launch",
        "LAUNCH20",
        "20% launch discount",
        "new_signup",
        200,
        true,
        "Launch campaign",
        "2026-06-30",
    )
    .with_created_at_ms(1_234_567);

    assert_eq!(coupon.code, "LAUNCH20");
    assert_eq!(coupon.remaining, 200);
    assert!(coupon.active);
    assert_eq!(coupon.created_at_ms, 1_234_567);
}

#[test]
fn coupon_campaign_reports_compatibility_live_availability() {
    let live_coupon = CouponCampaign::new(
        "coupon_live",
        "LIVE10",
        "10% live discount",
        "new_signup",
        10,
        true,
        "Compatibility live campaign",
        "2026-06-30",
    );
    let inactive_coupon = CouponCampaign::new(
        "coupon_inactive",
        "INACTIVE10",
        "10% inactive discount",
        "new_signup",
        10,
        false,
        "Inactive compatibility campaign",
        "2026-06-30",
    );
    let exhausted_coupon = CouponCampaign::new(
        "coupon_exhausted",
        "EMPTY10",
        "10% exhausted discount",
        "new_signup",
        0,
        true,
        "Exhausted compatibility campaign",
        "2026-06-30",
    );

    assert!(live_coupon.is_compatibility_live());
    assert!(!inactive_coupon.is_compatibility_live());
    assert!(!exhausted_coupon.is_compatibility_live());
}
