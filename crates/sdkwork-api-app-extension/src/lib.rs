use anyhow::Result;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_extension_host::{
    discover_extension_packages,
    list_connector_runtime_statuses as host_connector_runtime_statuses, ConnectorRuntimeStatus,
    DiscoveredExtensionPackage,
};
use sdkwork_api_storage_core::AdminStore;
use serde::Serialize;
use serde_json::Value;

pub use sdkwork_api_extension_host::ExtensionDiscoveryPolicy;

pub struct PersistExtensionInstanceInput<'a> {
    pub instance_id: &'a str,
    pub installation_id: &'a str,
    pub extension_id: &'a str,
    pub enabled: bool,
    pub base_url: Option<&'a str>,
    pub credential_ref: Option<&'a str>,
    pub config: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveredExtensionPackageRecord {
    pub root_dir: std::path::PathBuf,
    pub manifest_path: std::path::PathBuf,
    pub manifest: sdkwork_api_extension_core::ExtensionManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConnectorRuntimeStatusRecord {
    pub instance_id: String,
    pub base_url: String,
    pub health_url: String,
    pub process_id: Option<u32>,
    pub running: bool,
    pub healthy: bool,
}

pub async fn list_extension_installations(
    store: &dyn AdminStore,
) -> Result<Vec<ExtensionInstallation>> {
    store.list_extension_installations().await
}

pub async fn persist_extension_installation(
    store: &dyn AdminStore,
    installation_id: &str,
    extension_id: &str,
    runtime: ExtensionRuntime,
    enabled: bool,
    entrypoint: Option<&str>,
    config: Value,
) -> Result<ExtensionInstallation> {
    let mut installation =
        ExtensionInstallation::new(installation_id, extension_id, runtime).with_enabled(enabled);
    if let Some(entrypoint) = entrypoint {
        installation = installation.with_entrypoint(entrypoint);
    }
    installation = installation.with_config(config);
    store.insert_extension_installation(&installation).await
}

pub async fn list_extension_instances(store: &dyn AdminStore) -> Result<Vec<ExtensionInstance>> {
    store.list_extension_instances().await
}

pub fn list_discovered_extension_packages(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<Vec<DiscoveredExtensionPackageRecord>> {
    Ok(discover_extension_packages(policy)?
        .into_iter()
        .map(DiscoveredExtensionPackageRecord::from)
        .collect())
}

pub fn list_connector_runtime_statuses() -> Result<Vec<ConnectorRuntimeStatusRecord>> {
    Ok(host_connector_runtime_statuses()?
        .into_iter()
        .map(ConnectorRuntimeStatusRecord::from)
        .collect())
}

pub async fn persist_extension_instance(
    store: &dyn AdminStore,
    input: PersistExtensionInstanceInput<'_>,
) -> Result<ExtensionInstance> {
    let mut instance =
        ExtensionInstance::new(input.instance_id, input.installation_id, input.extension_id)
            .with_enabled(input.enabled);
    if let Some(base_url) = input.base_url {
        instance = instance.with_base_url(base_url);
    }
    if let Some(credential_ref) = input.credential_ref {
        instance = instance.with_credential_ref(credential_ref);
    }
    instance = instance.with_config(input.config);
    store.insert_extension_instance(&instance).await
}

pub fn configured_extension_discovery_policy_from_env() -> ExtensionDiscoveryPolicy {
    let search_paths = std::env::var_os("SDKWORK_EXTENSION_PATHS")
        .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    ExtensionDiscoveryPolicy::new(search_paths)
        .with_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            true,
        ))
        .with_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            false,
        ))
}

impl From<DiscoveredExtensionPackage> for DiscoveredExtensionPackageRecord {
    fn from(value: DiscoveredExtensionPackage) -> Self {
        Self {
            root_dir: value.root_dir,
            manifest_path: value.manifest_path,
            manifest: value.manifest,
        }
    }
}

impl From<ConnectorRuntimeStatus> for ConnectorRuntimeStatusRecord {
    fn from(value: ConnectorRuntimeStatus) -> Self {
        Self {
            instance_id: value.instance_id,
            base_url: value.base_url,
            health_url: value.health_url,
            process_id: value.process_id,
            running: value.running,
            healthy: value.healthy,
        }
    }
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(default)
}
