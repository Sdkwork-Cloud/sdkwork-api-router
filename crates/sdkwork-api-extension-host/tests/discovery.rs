use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_extension_core::{ExtensionProtocol, ExtensionRuntime};
use sdkwork_api_extension_host::{
    discover_extension_packages, validate_discovered_extension_package, ExtensionDiscoveryPolicy,
    ManifestValidationSeverity,
};

#[test]
fn discovers_sdkwork_extension_manifests_from_configured_directories() {
    let root = temp_extension_root("connector-openai");
    let package_dir = root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest("sdkwork.provider.custom-openai", "connector", "openai"),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false);

    let packages = discover_extension_packages(&policy).expect("discovered packages");

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].root_dir, package_dir);
    assert_eq!(
        packages[0].manifest_path,
        package_dir.join("sdkwork-extension.toml")
    );
    assert_eq!(packages[0].manifest.id, "sdkwork.provider.custom-openai");
    assert_eq!(packages[0].manifest.runtime, ExtensionRuntime::Connector);
    assert_eq!(
        packages[0].manifest.protocol,
        Some(ExtensionProtocol::OpenAi)
    );
    let report = validate_discovered_extension_package(&packages[0]);
    assert!(report.valid);
    assert!(report.issues.is_empty());

    cleanup_dir(&root);
}

#[test]
fn discovery_filters_disabled_runtimes() {
    let root = temp_extension_root("native-dynamic");
    let package_dir = root.join("sdkwork-provider-native-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest("sdkwork.provider.native-openai", "native_dynamic", "openai"),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false);

    let packages = discover_extension_packages(&policy).expect("discovered packages");
    assert!(packages.is_empty());

    cleanup_dir(&root);
}

#[test]
fn discovery_validation_reports_missing_permissions_and_health_contract() {
    let root = temp_extension_root("validation");
    let package_dir = root.join("sdkwork-provider-validation-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        incomplete_connector_manifest("sdkwork.provider.validation-openai"),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false);

    let packages = discover_extension_packages(&policy).expect("discovered packages");
    let report = validate_discovered_extension_package(&packages[0]);

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "missing_permissions"
            && issue.severity == ManifestValidationSeverity::Error));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "missing_health_contract"
            && issue.severity == ManifestValidationSeverity::Warning));

    cleanup_dir(&root);
}

fn connector_manifest(extension_id: &str, runtime: &str, protocol: &str) -> String {
    format!(
        r#"
api_version = "sdkwork.extension/v1"
id = "{extension_id}"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI"
runtime = "{runtime}"
protocol = "{protocol}"
entrypoint = "bin/sdkwork-provider-custom-openai"
channel_bindings = ["sdkwork.channel.openai"]
permissions = ["network_outbound", "spawn_process"]

[health]
path = "/health"
interval_secs = 30

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#
    )
}

fn incomplete_connector_manifest(extension_id: &str) -> String {
    format!(
        r#"
api_version = "sdkwork.extension/v1"
id = "{extension_id}"
kind = "provider"
version = "0.1.0"
display_name = "Validation OpenAI"
runtime = "connector"
protocol = "openai"
entrypoint = "bin/sdkwork-provider-validation-openai"
channel_bindings = ["sdkwork.channel.openai"]

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#
    )
}

fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-extension-host-{suffix}-{millis}"));
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}
