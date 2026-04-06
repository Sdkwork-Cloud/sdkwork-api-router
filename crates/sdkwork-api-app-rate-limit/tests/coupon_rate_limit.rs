use sdkwork_api_app_rate_limit::{
    check_coupon_rate_limit, create_coupon_rate_limit_policy, coupon_actor_bucket,
    coupon_code_hash, coupon_route_key, persist_rate_limit_policy, CouponRateLimitAction,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn creates_coupon_policies_with_canonical_route_and_hashed_coupon_dimension() {
    let actor_bucket = coupon_actor_bucket("user", "user-42");
    let coupon_hash = coupon_code_hash(" save10 ");
    let policy = create_coupon_rate_limit_policy(
        "coupon-validate-user",
        "project-1",
        CouponRateLimitAction::Validate,
        5,
        60,
        5,
        true,
        Some(&actor_bucket),
        Some(" save10 "),
        Some("coupon validate"),
    )
    .unwrap();

    assert_eq!(
        policy.route_key.as_deref(),
        Some(coupon_route_key(CouponRateLimitAction::Validate))
    );
    assert_eq!(policy.api_key_hash.as_deref(), Some(actor_bucket.as_str()));
    assert_eq!(policy.model_name.as_deref(), Some(coupon_hash.as_str()));
    assert_eq!(policy.requests_per_window, 5);
}

#[tokio::test]
async fn coupon_rate_limit_prefers_coupon_specific_policy_and_blocks_second_attempt() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let actor_bucket = coupon_actor_bucket("user", "user-42");

    let default_policy = create_coupon_rate_limit_policy(
        "coupon-validate-default",
        "project-1",
        CouponRateLimitAction::Validate,
        10,
        60,
        10,
        true,
        Some(&actor_bucket),
        None,
        Some("route default"),
    )
    .unwrap();
    let coupon_specific_policy = create_coupon_rate_limit_policy(
        "coupon-validate-save10",
        "project-1",
        CouponRateLimitAction::Validate,
        1,
        60,
        1,
        true,
        Some(&actor_bucket),
        Some("SAVE10"),
        Some("coupon specific"),
    )
    .unwrap();

    persist_rate_limit_policy(&store, &default_policy)
        .await
        .unwrap();
    persist_rate_limit_policy(&store, &coupon_specific_policy)
        .await
        .unwrap();

    let first = check_coupon_rate_limit(
        &store,
        "project-1",
        CouponRateLimitAction::Validate,
        Some(&actor_bucket),
        Some("save10"),
        1,
    )
    .await
    .unwrap();
    assert!(first.allowed);
    assert_eq!(
        first.policy_id.as_deref(),
        Some(coupon_specific_policy.policy_id.as_str())
    );

    let second = check_coupon_rate_limit(
        &store,
        "project-1",
        CouponRateLimitAction::Validate,
        Some(&actor_bucket),
        Some("SAVE10"),
        1,
    )
    .await
    .unwrap();
    assert!(!second.allowed);
    assert_eq!(
        second.policy_id.as_deref(),
        Some(coupon_specific_policy.policy_id.as_str())
    );
}
