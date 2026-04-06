use sdkwork_api_config::StandaloneConfig;

#[test]
fn allows_local_loopback_defaults_for_developer_startup() {
    StandaloneConfig::default()
        .validate_security_posture()
        .expect("loopback local defaults should remain usable for local development");
}

#[test]
fn rejects_default_secrets_when_any_service_binds_non_loopback() {
    let config = StandaloneConfig {
        gateway_bind: "0.0.0.0:8080".to_owned(),
        ..StandaloneConfig::default()
    };

    let error = config
        .validate_security_posture()
        .expect_err("non-loopback bindings must not use built-in development secrets");
    let message = error.to_string();
    assert!(message.contains("admin_jwt_signing_secret"));
    assert!(message.contains("portal_jwt_signing_secret"));
    assert!(message.contains("credential_master_key"));
    assert!(message.contains("metrics_bearer_token"));
}

#[test]
fn allows_explicit_override_for_insecure_non_loopback_development() {
    let config = StandaloneConfig {
        gateway_bind: "0.0.0.0:8080".to_owned(),
        allow_insecure_dev_defaults: true,
        ..StandaloneConfig::default()
    };

    config
        .validate_security_posture()
        .expect("explicit override should preserve non-production development workflows");
}
