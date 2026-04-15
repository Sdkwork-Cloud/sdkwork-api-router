use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use reqwest::{Client, StatusCode, Url};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::ProviderHealthSnapshot;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_extension_host::{
    discover_extension_packages,
    list_connector_runtime_statuses as host_connector_runtime_statuses,
    list_native_dynamic_runtime_statuses as host_native_dynamic_runtime_statuses,
    validate_discovered_extension_package, verify_discovered_extension_package_trust,
    ConnectorRuntimeStatus, DiscoveredExtensionPackage, ExtensionTrustReport,
    ManifestValidationReport, NativeDynamicRuntimeStatus,
};
use sdkwork_api_storage_core::AdminStore;
use serde::Serialize;
use serde_json::Value;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

pub use sdkwork_api_extension_host::ExtensionDiscoveryPolicy;

const BUILTIN_UPSTREAM_PROBE_TIMEOUT_MS: u64 = 750;

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
    pub distribution_name: String,
    pub crate_name: String,
    pub manifest: sdkwork_api_extension_core::ExtensionManifest,
    pub validation: ManifestValidationReport,
    pub trust: ExtensionTrustReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtensionRuntimeStatusRecord {
    pub runtime: String,
    pub extension_id: String,
    pub display_name: String,
    pub instance_id: String,
    pub base_url: Option<String>,
    pub health_url: Option<String>,
    pub process_id: Option<u32>,
    pub library_path: Option<String>,
    pub running: bool,
    pub healthy: bool,
    pub supports_health_check: bool,
    pub supports_shutdown: bool,
    pub message: Option<String>,
}

pub type ConnectorRuntimeStatusRecord = ExtensionRuntimeStatusRecord;

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
        .map(|package| DiscoveredExtensionPackageRecord::from_package(package, policy))
        .collect())
}

pub fn list_connector_runtime_statuses() -> Result<Vec<ConnectorRuntimeStatusRecord>> {
    Ok(host_connector_runtime_statuses()?
        .into_iter()
        .map(ExtensionRuntimeStatusRecord::from)
        .collect())
}

pub fn list_extension_runtime_statuses() -> Result<Vec<ExtensionRuntimeStatusRecord>> {
    let mut statuses = host_connector_runtime_statuses()?
        .into_iter()
        .map(ExtensionRuntimeStatusRecord::from)
        .collect::<Vec<_>>();
    statuses.extend(
        host_native_dynamic_runtime_statuses()?
            .into_iter()
            .map(ExtensionRuntimeStatusRecord::from),
    );
    statuses.sort_by(|left, right| {
        left.runtime
            .cmp(&right.runtime)
            .then(left.extension_id.cmp(&right.extension_id))
            .then(left.instance_id.cmp(&right.instance_id))
    });
    Ok(statuses)
}

pub async fn list_provider_health_snapshots(
    store: &dyn AdminStore,
) -> Result<Vec<ProviderHealthSnapshot>> {
    store.list_provider_health_snapshots().await
}

pub async fn capture_provider_health_snapshots(
    store: &dyn AdminStore,
) -> Result<Vec<ProviderHealthSnapshot>> {
    let providers = store.list_providers().await?;
    let installations = store
        .list_extension_installations()
        .await?
        .into_iter()
        .map(|installation| (installation.installation_id.clone(), installation))
        .collect::<HashMap<_, _>>();
    let instances = store
        .list_extension_instances()
        .await?
        .into_iter()
        .map(|instance| (instance.instance_id.clone(), instance))
        .collect::<HashMap<_, _>>();
    let statuses = list_extension_runtime_statuses()?;
    let observed_at_ms = unix_timestamp_ms();
    let probe_client = builtin_upstream_probe_client()?;

    let mut snapshots = Vec::new();
    for provider in providers {
        let instance = instances.get(&provider.id);
        let installation =
            instance.and_then(|instance| installations.get(&instance.installation_id));
        let snapshot = if let Some(snapshot) =
            provider_health_snapshot_from_runtime(&provider, instance, &statuses, observed_at_ms)
        {
            snapshot
        } else if let Some(snapshot) = provider_health_snapshot_from_probe(
            &probe_client,
            &provider,
            instance,
            installation,
            observed_at_ms,
        )
        .await
        {
            snapshot
        } else {
            continue;
        };
        store.insert_provider_health_snapshot(&snapshot).await?;
        snapshots.push(snapshot);
    }

    Ok(snapshots)
}

pub fn start_provider_health_snapshot_supervision(
    store: Arc<dyn AdminStore>,
    interval_secs: u64,
) -> Option<JoinHandle<()>> {
    if interval_secs == 0 {
        return None;
    }

    Some(tokio::spawn(async move {
        if let Err(error) = capture_provider_health_snapshots(store.as_ref()).await {
            eprintln!("provider health snapshot startup capture failed: {error}");
        }

        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        interval.tick().await;

        loop {
            interval.tick().await;
            if let Err(error) = capture_provider_health_snapshots(store.as_ref()).await {
                eprintln!("provider health snapshot capture failed: {error}");
            }
        }
    }))
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

    let mut policy = ExtensionDiscoveryPolicy::new(search_paths)
        .with_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            true,
        ))
        .with_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            true,
        ));
    for (publisher, public_key) in env_trusted_signers("SDKWORK_EXTENSION_TRUSTED_SIGNERS") {
        policy = policy.with_trusted_signer(publisher, public_key);
    }
    policy
}

impl DiscoveredExtensionPackageRecord {
    fn from_package(value: DiscoveredExtensionPackage, policy: &ExtensionDiscoveryPolicy) -> Self {
        let validation = validate_discovered_extension_package(&value);
        let trust = verify_discovered_extension_package_trust(&value, policy);
        Self {
            root_dir: value.root_dir,
            manifest_path: value.manifest_path,
            distribution_name: value.manifest.distribution_name(),
            crate_name: value.manifest.crate_name(),
            manifest: value.manifest,
            validation,
            trust,
        }
    }
}

impl From<ConnectorRuntimeStatus> for ExtensionRuntimeStatusRecord {
    fn from(value: ConnectorRuntimeStatus) -> Self {
        Self {
            runtime: "connector".to_owned(),
            extension_id: value.extension_id,
            display_name: value.display_name,
            instance_id: value.instance_id,
            base_url: Some(value.base_url),
            health_url: Some(value.health_url),
            process_id: value.process_id,
            library_path: None,
            running: value.running,
            healthy: value.healthy,
            supports_health_check: true,
            supports_shutdown: true,
            message: None,
        }
    }
}

impl From<NativeDynamicRuntimeStatus> for ExtensionRuntimeStatusRecord {
    fn from(value: NativeDynamicRuntimeStatus) -> Self {
        Self {
            runtime: "native_dynamic".to_owned(),
            extension_id: value.extension_id,
            display_name: value.display_name,
            instance_id: String::new(),
            base_url: None,
            health_url: None,
            process_id: None,
            library_path: Some(value.library_path),
            running: value.running,
            healthy: value.healthy,
            supports_health_check: value.supports_health_check,
            supports_shutdown: value.supports_shutdown,
            message: value.message,
        }
    }
}

pub fn matching_runtime_statuses_for_provider<'a>(
    provider: &ProxyProvider,
    instance: Option<&ExtensionInstance>,
    runtime_statuses: &'a [ExtensionRuntimeStatusRecord],
) -> Vec<&'a ExtensionRuntimeStatusRecord> {
    if let Some(instance) = instance {
        let exact = runtime_statuses
            .iter()
            .filter(|status| status.instance_id == instance.instance_id)
            .collect::<Vec<_>>();
        if !exact.is_empty() {
            return exact;
        }
    }

    runtime_statuses
        .iter()
        .filter(|status| {
            status.extension_id == provider.extension_id && status.instance_id.is_empty()
        })
        .collect()
}

fn provider_health_snapshot_from_runtime(
    provider: &ProxyProvider,
    instance: Option<&ExtensionInstance>,
    runtime_statuses: &[ExtensionRuntimeStatusRecord],
    observed_at_ms: u64,
) -> Option<ProviderHealthSnapshot> {
    let matching_statuses =
        matching_runtime_statuses_for_provider(provider, instance, runtime_statuses);
    let primary_status = matching_statuses.first()?;

    let instance_id = matching_statuses
        .iter()
        .find_map(|status| (!status.instance_id.is_empty()).then(|| status.instance_id.clone()));
    let running = matching_statuses.iter().any(|status| status.running);
    let healthy = matching_statuses.iter().any(|status| status.healthy);
    let message = matching_statuses
        .iter()
        .find_map(|status| status.message.clone());

    Some(
        ProviderHealthSnapshot::new(
            &provider.id,
            &provider.extension_id,
            &primary_status.runtime,
            observed_at_ms,
        )
        .with_instance_id_option(instance_id)
        .with_running(running)
        .with_healthy(healthy)
        .with_message_option(message),
    )
}

async fn provider_health_snapshot_from_probe(
    client: &Client,
    provider: &ProxyProvider,
    instance: Option<&ExtensionInstance>,
    installation: Option<&ExtensionInstallation>,
    observed_at_ms: u64,
) -> Option<ProviderHealthSnapshot> {
    if !provider_supports_builtin_upstream_probe(provider, instance, installation) {
        return None;
    }

    let probe_path = provider_probe_path(provider, instance, installation)?;
    let base_url = instance
        .and_then(|instance| instance.base_url.as_deref())
        .unwrap_or(provider.base_url.as_str());
    let outcome = match join_probe_url(base_url, &probe_path) {
        Some(probe_url) => probe_builtin_upstream_health(client, &probe_url, &probe_path).await,
        None => BuiltinUpstreamProbeOutcome::stopped(format!(
            "builtin upstream probe target is invalid for base_url {base_url}"
        )),
    };

    Some(
        ProviderHealthSnapshot::new(
            &provider.id,
            &provider.extension_id,
            "builtin",
            observed_at_ms,
        )
        .with_instance_id_option(instance.map(|instance| instance.instance_id.clone()))
        .with_running(outcome.running)
        .with_healthy(outcome.healthy)
        .with_message_option(Some(outcome.message)),
    )
}

fn builtin_upstream_probe_client() -> Result<Client> {
    sdkwork_api_kernel::ensure_reqwest_rustls_provider();
    Ok(Client::builder()
        .timeout(Duration::from_millis(BUILTIN_UPSTREAM_PROBE_TIMEOUT_MS))
        .build()?)
}

async fn probe_builtin_upstream_health(
    client: &Client,
    probe_url: &str,
    probe_path: &str,
) -> BuiltinUpstreamProbeOutcome {
    match client.get(probe_url).send().await {
        Ok(response) => builtin_upstream_probe_outcome_from_response(response.status(), probe_path),
        Err(error) => BuiltinUpstreamProbeOutcome::stopped(format!(
            "builtin upstream probe {probe_path} failed: {error}"
        )),
    }
}

fn builtin_upstream_probe_outcome_from_response(
    status: StatusCode,
    probe_path: &str,
) -> BuiltinUpstreamProbeOutcome {
    let code = status.as_u16();
    if builtin_upstream_status_is_healthy(status) {
        let message = if status.is_success() {
            format!("builtin upstream probe {probe_path} returned http {code}")
        } else {
            format!(
                "builtin upstream probe {probe_path} returned http {code} and was treated as reachable"
            )
        };
        BuiltinUpstreamProbeOutcome::healthy(message)
    } else {
        BuiltinUpstreamProbeOutcome::running(format!(
            "builtin upstream probe {probe_path} returned http {code}"
        ))
    }
}

fn builtin_upstream_status_is_healthy(status: StatusCode) -> bool {
    status.is_success()
        || matches!(
            status,
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS
        )
}

fn provider_supports_builtin_upstream_probe(
    provider: &ProxyProvider,
    instance: Option<&ExtensionInstance>,
    installation: Option<&ExtensionInstallation>,
) -> bool {
    if matches!(
        installation.map(|installation| &installation.runtime),
        Some(ExtensionRuntime::Connector | ExtensionRuntime::NativeDynamic)
    ) {
        return false;
    }

    if is_official_builtin_provider_extension_id(&provider.extension_id) {
        return true;
    }

    matches!(
        installation.map(|installation| &installation.runtime),
        Some(ExtensionRuntime::Builtin)
    ) && (explicit_probe_path(instance, installation).is_some()
        || builtin_upstream_default_probe_path(provider).is_some())
}

fn provider_probe_path(
    provider: &ProxyProvider,
    instance: Option<&ExtensionInstance>,
    installation: Option<&ExtensionInstallation>,
) -> Option<String> {
    explicit_probe_path(instance, installation)
        .or_else(|| builtin_upstream_default_probe_path(provider).map(ToOwned::to_owned))
}

fn explicit_probe_path(
    instance: Option<&ExtensionInstance>,
    installation: Option<&ExtensionInstallation>,
) -> Option<String> {
    instance
        .and_then(|instance| config_string(&instance.config, "health_path"))
        .or_else(|| {
            installation.and_then(|installation| config_string(&installation.config, "health_path"))
        })
}

fn builtin_upstream_default_probe_path(provider: &ProxyProvider) -> Option<&'static str> {
    match provider.extension_id.as_str() {
        "sdkwork.provider.openai.official"
        | "sdkwork.provider.openrouter"
        | "sdkwork.provider.ollama" => Some("/v1/models"),
        _ => match provider.adapter_kind.as_str() {
            "openai"
            | "openai-compatible"
            | "custom-openai"
            | "openrouter"
            | "openrouter-compatible"
            | "ollama"
            | "ollama-compatible" => Some("/v1/models"),
            _ => None,
        },
    }
}

fn is_official_builtin_provider_extension_id(extension_id: &str) -> bool {
    matches!(
        extension_id,
        "sdkwork.provider.openai.official"
            | "sdkwork.provider.openrouter"
            | "sdkwork.provider.ollama"
    )
}

fn join_probe_url(base_url: &str, probe_path: &str) -> Option<String> {
    let mut url = Url::parse(base_url).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }

    let base_segments = url
        .path_segments()
        .map(|segments| {
            segments
                .filter(|segment| !segment.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let probe_segments = probe_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let overlap = overlapping_path_segments(&base_segments, &probe_segments);
    let combined_segments = base_segments
        .into_iter()
        .chain(
            probe_segments
                .into_iter()
                .skip(overlap)
                .map(ToOwned::to_owned),
        )
        .collect::<Vec<_>>();

    if combined_segments.is_empty() {
        url.set_path("/");
    } else {
        url.set_path(&format!("/{}", combined_segments.join("/")));
    }

    Some(url.into())
}

fn overlapping_path_segments(base_segments: &[String], probe_segments: &[&str]) -> usize {
    let max_overlap = base_segments.len().min(probe_segments.len());
    for overlap in (1..=max_overlap).rev() {
        if base_segments[base_segments.len() - overlap..]
            .iter()
            .map(String::as_str)
            .eq(probe_segments[..overlap].iter().copied())
        {
            return overlap;
        }
    }

    0
}

#[derive(Debug, Clone)]
struct BuiltinUpstreamProbeOutcome {
    running: bool,
    healthy: bool,
    message: String,
}

impl BuiltinUpstreamProbeOutcome {
    fn healthy(message: String) -> Self {
        Self {
            running: true,
            healthy: true,
            message,
        }
    }

    fn running(message: String) -> Self {
        Self {
            running: true,
            healthy: false,
            message,
        }
    }

    fn stopped(message: String) -> Self {
        Self {
            running: false,
            healthy: false,
            message,
        }
    }
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn config_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(default)
}

fn env_trusted_signers(key: &str) -> Vec<(String, String)> {
    std::env::var(key)
        .ok()
        .map(|value| parse_trusted_signers(&value))
        .unwrap_or_default()
}

fn parse_trusted_signers(value: &str) -> Vec<(String, String)> {
    value
        .split(';')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            let (publisher, public_key) = entry.split_once('=')?;
            let publisher = publisher.trim();
            let public_key = public_key.trim();
            if publisher.is_empty() || public_key.is_empty() {
                return None;
            }
            Some((publisher.to_owned(), public_key.to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_api_domain_catalog::ProxyProvider;
    use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
    use serde_json::json;

    #[test]
    fn join_probe_url_preserves_base_url_path_prefix_without_duplicate_segments() {
        assert_eq!(
            join_probe_url("https://openrouter.ai/api/v1", "/v1/models").as_deref(),
            Some("https://openrouter.ai/api/v1/models")
        );
    }

    #[tokio::test]
    async fn explicit_health_path_enables_probe_for_builtin_installation_without_default_family() {
        let client = builtin_upstream_probe_client().unwrap();
        let provider = ProxyProvider::new(
            "provider-custom-builtin",
            "openai",
            "custom-http",
            "not-a-valid-base-url",
            "Custom Builtin",
        )
        .with_extension_id("sdkwork.provider.custom-builtin");
        let installation = ExtensionInstallation::new(
            "custom-builtin-installation",
            "sdkwork.provider.custom-builtin",
            ExtensionRuntime::Builtin,
        )
        .with_enabled(true)
        .with_config(json!({
            "health_path": "/custom-health"
        }));
        let instance = ExtensionInstance::new(
            "provider-custom-builtin",
            "custom-builtin-installation",
            "sdkwork.provider.custom-builtin",
        )
        .with_enabled(true)
        .with_base_url("not-a-valid-base-url")
        .with_config(json!({}));

        let snapshot = provider_health_snapshot_from_probe(
            &client,
            &provider,
            Some(&instance),
            Some(&installation),
            42,
        )
        .await;

        assert!(snapshot.is_some());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.provider_id, "provider-custom-builtin");
        assert_eq!(snapshot.runtime, "builtin");
        assert!(!snapshot.running);
        assert!(!snapshot.healthy);
        assert!(snapshot
            .message
            .as_deref()
            .is_some_and(|message| message.contains("invalid")));
    }
}
