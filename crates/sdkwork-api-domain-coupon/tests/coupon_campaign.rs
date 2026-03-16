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
