use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use getrandom::fill as random_fill;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

pub const DESKTOP_PUBLIC_BASE_URL: &str = "http://127.0.0.1:3001";
pub const DESKTOP_LOCAL_PUBLIC_BIND: &str = "127.0.0.1:3001";
pub const DESKTOP_SHARED_PUBLIC_BIND: &str = "0.0.0.0:3001";
pub const DESKTOP_GATEWAY_BIND: &str = "127.0.0.1:8080";
pub const DESKTOP_ADMIN_BIND: &str = "127.0.0.1:8081";
pub const DESKTOP_PORTAL_BIND: &str = "127.0.0.1:8082";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DesktopRuntimeAccessMode {
    Local,
    Shared,
}

impl Default for DesktopRuntimeAccessMode {
    fn default() -> Self {
        Self::Local
    }
}

impl DesktopRuntimeAccessMode {
    pub fn public_bind_addr(self) -> &'static str {
        match self {
            Self::Local => DESKTOP_LOCAL_PUBLIC_BIND,
            Self::Shared => DESKTOP_SHARED_PUBLIC_BIND,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRuntimeSettings {
    pub access_mode: DesktopRuntimeAccessMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopRuntimePaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub runtime_state_file: PathBuf,
    pub router_config_file: PathBuf,
    pub router_stdout_log_file: PathBuf,
    pub router_stderr_log_file: PathBuf,
    pub router_database_file: PathBuf,
}

pub fn resolve_desktop_runtime_paths(
    config_root: impl AsRef<Path>,
    data_root: impl AsRef<Path>,
    log_root: impl AsRef<Path>,
) -> DesktopRuntimePaths {
    let config_dir = config_root.as_ref().join("router-product");
    let data_dir = data_root.as_ref().join("router-product");
    let log_dir = log_root.as_ref().join("router-product");

    DesktopRuntimePaths {
        runtime_state_file: config_dir.join("desktop-runtime.json"),
        router_config_file: config_dir.join("router.yaml"),
        router_stdout_log_file: log_dir.join("router-product-service.stdout.log"),
        router_stderr_log_file: log_dir.join("router-product-service.stderr.log"),
        router_database_file: data_dir.join("sdkwork-api-router.db"),
        config_dir,
        data_dir,
        log_dir,
    }
}

pub fn load_or_initialize_settings(_paths: &DesktopRuntimePaths) -> Result<DesktopRuntimeSettings> {
    ensure_parent_dir(&_paths.runtime_state_file)?;
    if !_paths.runtime_state_file.is_file() {
        let settings = DesktopRuntimeSettings::default();
        persist_settings(_paths, &settings)?;
        return Ok(settings);
    }

    let content = fs::read_to_string(&_paths.runtime_state_file).with_context(|| {
        format!(
            "failed to read desktop runtime state file {}",
            _paths.runtime_state_file.display()
        )
    })?;
    let settings = serde_json::from_str::<DesktopRuntimeSettings>(&content).with_context(|| {
        format!(
            "failed to parse desktop runtime state file {}",
            _paths.runtime_state_file.display()
        )
    })?;
    Ok(settings)
}

pub fn persist_settings(
    _paths: &DesktopRuntimePaths,
    _settings: &DesktopRuntimeSettings,
) -> Result<()> {
    ensure_parent_dir(&_paths.runtime_state_file)?;
    fs::write(
        &_paths.runtime_state_file,
        format!(
            "{}\n",
            serde_json::to_string_pretty(_settings)
                .context("failed to serialize desktop runtime settings")?
        ),
    )
    .with_context(|| {
        format!(
            "failed to write desktop runtime state file {}",
            _paths.runtime_state_file.display()
        )
    })?;
    Ok(())
}

pub fn sync_router_config(
    paths: &DesktopRuntimePaths,
    settings: &DesktopRuntimeSettings,
) -> Result<()> {
    fs::create_dir_all(&paths.config_dir).with_context(|| {
        format!(
            "failed to create desktop runtime config dir {}",
            paths.config_dir.display()
        )
    })?;
    fs::create_dir_all(&paths.data_dir).with_context(|| {
        format!(
            "failed to create desktop runtime data dir {}",
            paths.data_dir.display()
        )
    })?;
    fs::create_dir_all(&paths.log_dir).with_context(|| {
        format!(
            "failed to create desktop runtime log dir {}",
            paths.log_dir.display()
        )
    })?;

    let mut document = load_or_create_router_config_document(&paths.router_config_file)?;
    set_string(
        &mut document,
        "web_bind",
        settings.access_mode.public_bind_addr(),
    );
    set_string(&mut document, "gateway_bind", DESKTOP_GATEWAY_BIND);
    set_string(&mut document, "admin_bind", DESKTOP_ADMIN_BIND);
    set_string(&mut document, "portal_bind", DESKTOP_PORTAL_BIND);
    set_string_if_blank(
        &mut document,
        "database_url",
        &sqlite_url_for(&paths.router_database_file),
    );
    set_string_if_blank(
        &mut document,
        "admin_jwt_signing_secret",
        &random_secret("desktop-admin-jwt")?,
    );
    set_string_if_blank(
        &mut document,
        "portal_jwt_signing_secret",
        &random_secret("desktop-portal-jwt")?,
    );
    set_string_if_blank(
        &mut document,
        "credential_master_key",
        &random_secret("desktop-credential-master-key")?,
    );
    set_string_if_blank(
        &mut document,
        "metrics_bearer_token",
        &random_secret("desktop-metrics-token")?,
    );
    set_string_if_blank(&mut document, "bootstrap_profile", "prod");
    set_bool(&mut document, "allow_insecure_dev_defaults", false);

    let yaml = serde_yaml::to_string(&Value::Mapping(document))
        .context("failed to serialize desktop router config")?;
    fs::write(&paths.router_config_file, yaml).with_context(|| {
        format!(
            "failed to write desktop router config {}",
            paths.router_config_file.display()
        )
    })?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    let Some(parent_dir) = path.parent() else {
        bail!("path does not have a parent directory: {}", path.display());
    };
    fs::create_dir_all(parent_dir)
        .with_context(|| format!("failed to create parent dir {}", parent_dir.display()))
}

fn load_or_create_router_config_document(path: &Path) -> Result<Mapping> {
    if !path.is_file() {
        return Ok(Mapping::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read router config {}", path.display()))?;
    let parsed = serde_yaml::from_str::<Value>(&content)
        .with_context(|| format!("failed to parse router config {}", path.display()))?;

    match parsed {
        Value::Null => Ok(Mapping::new()),
        Value::Mapping(mapping) => Ok(mapping),
        other => bail!(
            "desktop router config {} must deserialize to a YAML mapping, found {:?}",
            path.display(),
            other
        ),
    }
}

fn sqlite_url_for(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.starts_with('/') {
        format!("sqlite://{normalized}")
    } else {
        format!("sqlite:///{normalized}")
    }
}

fn set_string(document: &mut Mapping, key: &str, value: &str) {
    document.insert(
        Value::String(key.to_owned()),
        Value::String(value.to_owned()),
    );
}

fn set_string_if_blank(document: &mut Mapping, key: &str, value: &str) {
    let key_value = Value::String(key.to_owned());
    if let Some(existing) = document.get(&key_value) {
        if value_is_non_empty_string(existing) {
            return;
        }
    }

    document.insert(key_value, Value::String(value.to_owned()));
}

fn set_bool(document: &mut Mapping, key: &str, value: bool) {
    document.insert(Value::String(key.to_owned()), Value::Bool(value));
}

fn value_is_non_empty_string(value: &Value) -> bool {
    matches!(value, Value::String(content) if !content.trim().is_empty())
}

fn random_secret(label: &str) -> Result<String> {
    let mut bytes = [0_u8; 24];
    random_fill(&mut bytes)
        .map_err(|error| anyhow::anyhow!("failed to generate desktop runtime secret: {error}"))?;
    Ok(format!("{label}-{}", encode_hex(&bytes)))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        load_or_initialize_settings, persist_settings, resolve_desktop_runtime_paths,
        sync_router_config, DesktopRuntimeAccessMode, DesktopRuntimePaths,
        DESKTOP_LOCAL_PUBLIC_BIND, DESKTOP_SHARED_PUBLIC_BIND,
    };

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("portal-desktop-runtime-config-{name}-{suffix}"))
    }

    fn unique_paths(name: &str) -> DesktopRuntimePaths {
        let root = unique_temp_dir(name);
        resolve_desktop_runtime_paths(root.join("config"), root.join("data"), root.join("log"))
    }

    #[test]
    fn access_mode_maps_to_the_expected_public_binds() {
        assert_eq!(
            DesktopRuntimeAccessMode::Local.public_bind_addr(),
            DESKTOP_LOCAL_PUBLIC_BIND
        );
        assert_eq!(
            DesktopRuntimeAccessMode::Shared.public_bind_addr(),
            DESKTOP_SHARED_PUBLIC_BIND
        );
    }

    #[test]
    fn settings_default_to_local_access_when_no_state_file_exists() {
        let paths = unique_paths("defaults");
        fs::create_dir_all(&paths.config_dir).expect("config dir should exist");

        let settings =
            load_or_initialize_settings(&paths).expect("settings should resolve from defaults");

        assert_eq!(settings.access_mode, DesktopRuntimeAccessMode::Local);
    }

    #[test]
    fn persisted_settings_round_trip_access_mode() {
        let paths = unique_paths("round-trip");
        fs::create_dir_all(&paths.config_dir).expect("config dir should exist");

        persist_settings(
            &paths,
            &super::DesktopRuntimeSettings {
                access_mode: DesktopRuntimeAccessMode::Shared,
            },
        )
        .expect("settings should persist");

        let settings =
            load_or_initialize_settings(&paths).expect("settings should reload after persistence");

        assert_eq!(settings.access_mode, DesktopRuntimeAccessMode::Shared);
    }

    #[test]
    fn router_config_sync_writes_desktop_runtime_defaults() {
        let paths = unique_paths("router-config");
        fs::create_dir_all(&paths.config_dir).expect("config dir should exist");
        fs::create_dir_all(&paths.data_dir).expect("data dir should exist");

        sync_router_config(
            &paths,
            &super::DesktopRuntimeSettings {
                access_mode: DesktopRuntimeAccessMode::Shared,
            },
        )
        .expect("router config should render");

        let config = fs::read_to_string(&paths.router_config_file)
            .expect("router config file should exist");

        assert!(config.contains("web_bind: 0.0.0.0:3001"));
        assert!(config.contains("gateway_bind: 127.0.0.1:8080"));
        assert!(config.contains("admin_bind: 127.0.0.1:8081"));
        assert!(config.contains("portal_bind: 127.0.0.1:8082"));
        assert!(config.contains("database_url:"));
        assert!(config.contains("admin_jwt_signing_secret:"));
        assert!(config.contains("portal_jwt_signing_secret:"));
        assert!(config.contains("credential_master_key:"));
        assert!(config.contains("metrics_bearer_token:"));
    }
}
