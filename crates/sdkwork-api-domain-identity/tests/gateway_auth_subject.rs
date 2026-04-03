use sdkwork_api_domain_identity::{GatewayAuthSubject, GatewayAuthType};

#[test]
fn jwt_gateway_auth_subject_normalizes_canonical_request_principal() {
    let subject = GatewayAuthSubject::for_jwt(1001, 0, 9001, "plus-auth:user-9001")
        .with_platform("web")
        .with_owner("tenant-owner");

    assert_eq!(subject.tenant_id, 1001);
    assert_eq!(subject.organization_id, 0);
    assert_eq!(subject.user_id, 9001);
    assert_eq!(subject.auth_type, GatewayAuthType::Jwt);
    assert_eq!(subject.api_key_id, None);
    assert_eq!(subject.api_key_hash, None);
    assert_eq!(subject.jwt_subject.as_deref(), Some("plus-auth:user-9001"));
    assert_eq!(subject.platform.as_deref(), Some("web"));
    assert_eq!(subject.owner.as_deref(), Some("tenant-owner"));
    assert_eq!(subject.request_principal, "jwt:plus-auth:user-9001");
}

#[test]
fn api_key_gateway_auth_subject_keeps_api_key_attribution() {
    let subject = GatewayAuthSubject::for_api_key(1001, 2002, 9001, 778899, "key_hash_live");

    assert_eq!(subject.tenant_id, 1001);
    assert_eq!(subject.organization_id, 2002);
    assert_eq!(subject.user_id, 9001);
    assert_eq!(subject.auth_type, GatewayAuthType::ApiKey);
    assert_eq!(subject.api_key_id, Some(778899));
    assert_eq!(subject.api_key_hash.as_deref(), Some("key_hash_live"));
    assert_eq!(subject.jwt_subject, None);
    assert_eq!(subject.request_principal, "api_key:778899");
}
