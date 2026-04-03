use sdkwork_api_domain_identity::{
    CanonicalApiKeyRecord, IdentityBindingRecord, IdentityUserRecord,
};

#[test]
fn canonical_identity_user_defaults_to_active_status() {
    let user = IdentityUserRecord::new(9001, 1001, 2002)
        .with_external_user_ref(Some("portal-user-1".to_owned()))
        .with_display_name(Some("Portal User".to_owned()))
        .with_email(Some("portal@example.com".to_owned()))
        .with_created_at_ms(10)
        .with_updated_at_ms(20);

    assert_eq!(user.user_id, 9001);
    assert_eq!(user.tenant_id, 1001);
    assert_eq!(user.organization_id, 2002);
    assert_eq!(user.external_user_ref.as_deref(), Some("portal-user-1"));
    assert_eq!(user.display_name.as_deref(), Some("Portal User"));
    assert_eq!(user.email.as_deref(), Some("portal@example.com"));
    assert_eq!(user.status, "active");
    assert_eq!(user.created_at_ms, 10);
    assert_eq!(user.updated_at_ms, 20);
}

#[test]
fn canonical_api_key_keeps_numeric_subject_scope() {
    let api_key = CanonicalApiKeyRecord::new(778899, 1001, 2002, 9001, "key_hash_live")
        .with_key_prefix("skw_live")
        .with_display_name("Production key")
        .with_expires_at_ms(Some(1_900_000_000_000))
        .with_created_at_ms(30)
        .with_updated_at_ms(40);

    assert_eq!(api_key.api_key_id, 778899);
    assert_eq!(api_key.tenant_id, 1001);
    assert_eq!(api_key.organization_id, 2002);
    assert_eq!(api_key.user_id, 9001);
    assert_eq!(api_key.key_hash, "key_hash_live");
    assert_eq!(api_key.status, "active");
    assert_eq!(api_key.expires_at_ms, Some(1_900_000_000_000));
}

#[test]
fn identity_binding_captures_subject_and_platform_evidence() {
    let binding = IdentityBindingRecord::new(7001, 1001, 2002, 9001, "jwt_subject")
        .with_issuer(Some("plus-auth".to_owned()))
        .with_subject(Some("user-9001".to_owned()))
        .with_platform(Some("web".to_owned()))
        .with_owner(Some("tenant-owner".to_owned()))
        .with_status("active")
        .with_created_at_ms(50)
        .with_updated_at_ms(60);

    assert_eq!(binding.identity_binding_id, 7001);
    assert_eq!(binding.binding_type, "jwt_subject");
    assert_eq!(binding.issuer.as_deref(), Some("plus-auth"));
    assert_eq!(binding.subject.as_deref(), Some("user-9001"));
    assert_eq!(binding.platform.as_deref(), Some("web"));
    assert_eq!(binding.owner.as_deref(), Some("tenant-owner"));
    assert_eq!(binding.status, "active");
}
