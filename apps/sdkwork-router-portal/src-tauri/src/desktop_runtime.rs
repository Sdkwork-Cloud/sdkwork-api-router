use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::desktop_runtime_config::{
    load_or_initialize_settings, persist_settings, resolve_desktop_runtime_paths,
    sync_router_config, DesktopRuntimeAccessMode, DesktopRuntimePaths, DesktopRuntimeSettings,
    DESKTOP_ADMIN_BIND, DESKTOP_GATEWAY_BIND, DESKTOP_PORTAL_BIND, DESKTOP_PUBLIC_BASE_URL,
};

const ROUTER_BINARY_NAME: &str = if cfg!(target_os = "windows") {
    "router-product-service.exe"
} else {
    "router-product-service"
};

const RUNTIME_HEALTH_TIMEOUT: Duration = Duration::from_secs(45);
const RUNTIME_HEALTH_RETRY_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalDesktopRuntimeSnapshot {
    pub mode: String,
    pub roles: Vec<String>,
    pub access_mode: DesktopRuntimeAccessMode,
    pub public_base_url: Option<String>,
    pub public_bind_addr: Option<String>,
    pub gateway_bind_addr: Option<String>,
    pub admin_bind_addr: Option<String>,
    pub portal_bind_addr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductResourceLayout {
    pub root: PathBuf,
    pub router_binary_path: PathBuf,
    pub admin_site_dir: PathBuf,
    pub portal_site_dir: PathBuf,
}

pub struct DesktopRuntimeSupervisor {
    resource_layout: ProductResourceLayout,
    runtime_paths: DesktopRuntimePaths,
    settings: DesktopRuntimeSettings,
    child: Option<Child>,
    snapshot: PortalDesktopRuntimeSnapshot,
}

impl DesktopRuntimeSupervisor {
    pub fn bootstrap(app: &AppHandle) -> Result<Self> {
        let resource_layout = resolve_product_resource_layout(app)?;
        let runtime_paths = resolve_desktop_runtime_paths(
            app.path()
                .app_config_dir()
                .context("desktop app config dir is unavailable")?,
            app.path()
                .app_data_dir()
                .context("desktop app data dir is unavailable")?,
            app.path()
                .app_log_dir()
                .context("desktop app log dir is unavailable")?,
        );

        fs::create_dir_all(&runtime_paths.config_dir).with_context(|| {
            format!(
                "failed to create desktop runtime config dir {}",
                runtime_paths.config_dir.display()
            )
        })?;
        fs::create_dir_all(&runtime_paths.data_dir).with_context(|| {
            format!(
                "failed to create desktop runtime data dir {}",
                runtime_paths.data_dir.display()
            )
        })?;
        fs::create_dir_all(&runtime_paths.log_dir).with_context(|| {
            format!(
                "failed to create desktop runtime log dir {}",
                runtime_paths.log_dir.display()
            )
        })?;

        let settings = load_or_initialize_settings(&runtime_paths)?;
        sync_router_config(&runtime_paths, &settings)?;

        let mut supervisor = Self {
            resource_layout,
            runtime_paths,
            settings,
            child: None,
            snapshot: snapshot_for_settings(DesktopRuntimeAccessMode::Local),
        };
        let snapshot = supervisor.start_child()?;
        supervisor.snapshot = snapshot;
        Ok(supervisor)
    }

    pub fn snapshot(&self) -> PortalDesktopRuntimeSnapshot {
        self.snapshot.clone()
    }

    pub fn restart(&mut self) -> Result<PortalDesktopRuntimeSnapshot> {
        sync_router_config(&self.runtime_paths, &self.settings)?;
        let snapshot = self.start_child()?;
        self.snapshot = snapshot.clone();
        Ok(snapshot)
    }

    pub fn update_access_mode(
        &mut self,
        access_mode: DesktopRuntimeAccessMode,
    ) -> Result<PortalDesktopRuntimeSnapshot> {
        if self.settings.access_mode == access_mode {
            return Ok(self.snapshot());
        }

        self.settings.access_mode = access_mode;
        persist_settings(&self.runtime_paths, &self.settings)?;
        self.restart()
    }

    fn start_child(&mut self) -> Result<PortalDesktopRuntimeSnapshot> {
        self.stop_child()?;
        sync_router_config(&self.runtime_paths, &self.settings)?;

        let stdout = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.runtime_paths.router_stdout_log_file)
            .with_context(|| {
                format!(
                    "failed to open desktop runtime stdout log {}",
                    self.runtime_paths.router_stdout_log_file.display()
                )
            })?;
        let stderr = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.runtime_paths.router_stderr_log_file)
            .with_context(|| {
                format!(
                    "failed to open desktop runtime stderr log {}",
                    self.runtime_paths.router_stderr_log_file.display()
                )
            })?;

        let mut command = Command::new(&self.resource_layout.router_binary_path);
        command
            .arg("--config-dir")
            .arg(&self.runtime_paths.config_dir)
            .arg("--admin-site-dir")
            .arg(&self.resource_layout.admin_site_dir)
            .arg("--portal-site-dir")
            .arg(&self.resource_layout.portal_site_dir)
            .current_dir(&self.resource_layout.root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));

        command.env_clear();
        command.envs(
            std::env::vars().filter(|(key, _)| !key.starts_with("SDKWORK_")),
        );

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;

            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = command.spawn().with_context(|| {
            format!(
                "failed to start desktop runtime sidecar {}",
                self.resource_layout.router_binary_path.display()
            )
        })?;
        let snapshot = snapshot_for_settings(self.settings.access_mode);

        if let Err(error) = wait_for_runtime_ready(&mut child, &snapshot) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }

        self.child = Some(child);
        Ok(snapshot)
    }

    fn stop_child(&mut self) -> Result<()> {
        let Some(mut child) = self.child.take() else {
            return Ok(());
        };

        if child
            .try_wait()
            .context("failed to inspect desktop runtime sidecar process state")?
            .is_some()
        {
            return Ok(());
        }

        let _ = child.kill();
        let _ = child.wait();
        Ok(())
    }
}

pub fn resolve_product_resource_layout_for_paths(
    resource_root: Option<&Path>,
    prepared_workspace_root: Option<&Path>,
    workspace_root: &Path,
) -> Result<ProductResourceLayout> {
    if let Some(layout) = resource_root.and_then(release_like_layout) {
        return Ok(layout);
    }
    if let Some(layout) = prepared_workspace_root.and_then(release_like_layout) {
        return Ok(layout);
    }

    let workspace_layout = ProductResourceLayout {
        root: workspace_root.to_path_buf(),
        router_binary_path: workspace_root
            .join("target")
            .join("release")
            .join(ROUTER_BINARY_NAME),
        admin_site_dir: workspace_root
            .join("apps")
            .join("sdkwork-router-admin")
            .join("dist"),
        portal_site_dir: workspace_root
            .join("apps")
            .join("sdkwork-router-portal")
            .join("dist"),
    };
    if workspace_layout.router_binary_path.is_file()
        && workspace_layout.admin_site_dir.is_dir()
        && workspace_layout.portal_site_dir.is_dir()
    {
        return Ok(workspace_layout);
    }

    bail!(
        "unable to resolve portal desktop router-product resources; checked release resources, prepared workspace payload, and raw workspace fallback under {}",
        workspace_root.display()
    )
}

impl Drop for DesktopRuntimeSupervisor {
    fn drop(&mut self) {
        let _ = self.stop_child();
    }
}

fn resolve_product_resource_layout(app: &AppHandle) -> Result<ProductResourceLayout> {
    let resource_root = app
        .path()
        .resource_dir()
        .ok()
        .map(|path| path.join("router-product"));
    let workspace_root = workspace_root();
    let prepared_workspace_root = workspace_root
        .join("artifacts")
        .join("router-portal-desktop")
        .join("resources")
        .join("router-product");

    resolve_product_resource_layout_for_paths(
        resource_root.as_deref(),
        Some(prepared_workspace_root.as_path()),
        &workspace_root,
    )
}

fn release_like_layout(root: &Path) -> Option<ProductResourceLayout> {
    let layout = ProductResourceLayout {
        root: root.to_path_buf(),
        router_binary_path: root.join("bin").join(ROUTER_BINARY_NAME),
        admin_site_dir: root.join("sites").join("admin").join("dist"),
        portal_site_dir: root.join("sites").join("portal").join("dist"),
    };
    let bootstrap_data_dir = root.join("data");
    let release_manifest_path = root.join("release-manifest.json");
    let release_payload_readme_path = root.join("README.txt");

    (layout.router_binary_path.is_file()
        && layout.admin_site_dir.is_dir()
        && layout.portal_site_dir.is_dir()
        && bootstrap_data_dir.is_dir()
        && release_manifest_path.is_file()
        && release_payload_readme_path.is_file())
    .then_some(layout)
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("portal src-tauri must live inside apps/<app>/src-tauri within the workspace")
        .to_path_buf()
}

fn snapshot_for_settings(access_mode: DesktopRuntimeAccessMode) -> PortalDesktopRuntimeSnapshot {
    PortalDesktopRuntimeSnapshot {
        mode: "desktop".to_owned(),
        roles: vec![
            "web".to_owned(),
            "gateway".to_owned(),
            "admin".to_owned(),
            "portal".to_owned(),
        ],
        access_mode,
        public_base_url: Some(DESKTOP_PUBLIC_BASE_URL.to_owned()),
        public_bind_addr: Some(access_mode.public_bind_addr().to_owned()),
        gateway_bind_addr: Some(DESKTOP_GATEWAY_BIND.to_owned()),
        admin_bind_addr: Some(DESKTOP_ADMIN_BIND.to_owned()),
        portal_bind_addr: Some(DESKTOP_PORTAL_BIND.to_owned()),
    }
}

fn wait_for_runtime_ready(
    child: &mut Child,
    snapshot: &PortalDesktopRuntimeSnapshot,
) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .context("failed to create desktop runtime health client")?;
    let Some(public_base_url) = snapshot.public_base_url.as_deref() else {
        bail!("desktop runtime snapshot did not expose a public base URL");
    };
    let deadline = Instant::now() + RUNTIME_HEALTH_TIMEOUT;
    let health_urls = health_probe_urls(public_base_url);

    while Instant::now() < deadline {
        if let Some(exit_status) = child
            .try_wait()
            .context("failed to inspect desktop runtime sidecar process state")?
        {
            bail!("desktop runtime sidecar exited early with status {exit_status}");
        }

        if health_urls.iter().all(|url| endpoint_is_ready(&client, url)) {
            return Ok(());
        }

        std::thread::sleep(RUNTIME_HEALTH_RETRY_INTERVAL);
    }

    bail!(
        "desktop runtime sidecar did not become healthy within {:?}",
        RUNTIME_HEALTH_TIMEOUT
    )
}

fn health_probe_urls(public_base_url: &str) -> Vec<String> {
    vec![
        format!("{public_base_url}/"),
        format!("{public_base_url}/api/v1/health"),
        format!("{public_base_url}/api/admin/health"),
        format!("{public_base_url}/api/portal/health"),
    ]
}

fn endpoint_is_ready(client: &Client, url: &str) -> bool {
    client
        .get(url)
        .send()
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        health_probe_urls, release_like_layout, resolve_product_resource_layout_for_paths,
    };

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("portal-desktop-runtime-{name}-{suffix}"))
    }

    fn create_release_like_router_product(root: &Path, binary_name: &str) {
        fs::create_dir_all(root.join("bin")).expect("router-product bin dir should exist");
        fs::create_dir_all(root.join("sites").join("admin").join("dist"))
            .expect("admin site dir should exist");
        fs::create_dir_all(root.join("sites").join("portal").join("dist"))
            .expect("portal site dir should exist");
        fs::create_dir_all(root.join("data").join("profiles"))
            .expect("bootstrap data dir should exist");
        fs::write(root.join("bin").join(binary_name), []).expect("binary placeholder should exist");
        fs::write(
            root.join("release-manifest.json"),
            r#"{"type":"portal-desktop-router-product"}"#,
        )
        .expect("release manifest should exist");
        fs::write(root.join("README.txt"), "portal desktop router-product payload")
            .expect("release payload readme should exist");
        fs::write(
            root.join("data").join("profiles").join("prod.json"),
            r#"{"profile":"prod"}"#,
        )
        .expect("bootstrap profile manifest should exist");
    }

    fn create_workspace_layout(root: &Path, binary_name: &str) {
        fs::create_dir_all(root.join("target").join("release"))
            .expect("workspace target release dir should exist");
        fs::create_dir_all(
            root.join("apps")
                .join("sdkwork-router-admin")
                .join("dist"),
        )
        .expect("workspace admin dist dir should exist");
        fs::create_dir_all(
            root.join("apps")
                .join("sdkwork-router-portal")
                .join("dist"),
        )
        .expect("workspace portal dist dir should exist");
        fs::write(
            root.join("target").join("release").join(binary_name),
            [],
        )
        .expect("workspace router-product-service binary should exist");
    }

    #[test]
    fn resource_root_layout_is_preferred_over_workspace_fallback() {
        let workspace_root = unique_temp_dir("resource-root-preferred");
        let resource_root = unique_temp_dir("resource-root");
        create_workspace_layout(&workspace_root, "router-product-service.exe");
        create_release_like_router_product(&resource_root, "router-product-service.exe");

        let layout = resolve_product_resource_layout_for_paths(
            Some(&resource_root),
            None,
            &workspace_root,
        )
        .expect("resource layout should resolve");

        assert_eq!(layout.root, resource_root);
        assert_eq!(
            layout.router_binary_path,
            resource_root.join("bin").join("router-product-service.exe")
        );
    }

    #[test]
    fn prepared_workspace_payload_is_used_before_raw_workspace_fallback() {
        let workspace_root = unique_temp_dir("prepared-workspace");
        let prepared_root = unique_temp_dir("prepared-root");
        create_workspace_layout(&workspace_root, "router-product-service.exe");
        create_release_like_router_product(&prepared_root, "router-product-service.exe");

        let layout = resolve_product_resource_layout_for_paths(
            None,
            Some(&prepared_root),
            &workspace_root,
        )
        .expect("prepared workspace layout should resolve");

        assert_eq!(layout.root, prepared_root);
    }

    #[test]
    fn raw_workspace_layout_is_used_when_release_like_roots_are_missing() {
        let workspace_root = unique_temp_dir("workspace-fallback");
        create_workspace_layout(&workspace_root, "router-product-service.exe");

        let layout = resolve_product_resource_layout_for_paths(None, None, &workspace_root)
            .expect("workspace layout should resolve");

        assert_eq!(
            layout.router_binary_path,
            workspace_root
                .join("target")
                .join("release")
                .join("router-product-service.exe")
        );
        assert_eq!(
            layout.admin_site_dir,
            workspace_root
                .join("apps")
                .join("sdkwork-router-admin")
                .join("dist")
        );
        assert_eq!(
            layout.portal_site_dir,
            workspace_root
                .join("apps")
                .join("sdkwork-router-portal")
                .join("dist")
        );
    }

    #[test]
    fn release_like_layout_requires_bootstrap_data_and_payload_metadata() {
        let resource_root = unique_temp_dir("release-like-validation");
        create_release_like_router_product(&resource_root, "router-product-service.exe");

        assert!(release_like_layout(&resource_root).is_some());

        fs::remove_file(resource_root.join("release-manifest.json"))
            .expect("release manifest removal should succeed");
        assert!(release_like_layout(&resource_root).is_none());
    }

    #[test]
    fn desktop_runtime_health_probes_use_gateway_v1_contract() {
        let urls = health_probe_urls("http://127.0.0.1:3001");

        assert_eq!(
            urls,
            vec![
                "http://127.0.0.1:3001/".to_owned(),
                "http://127.0.0.1:3001/api/v1/health".to_owned(),
                "http://127.0.0.1:3001/api/admin/health".to_owned(),
                "http://127.0.0.1:3001/api/portal/health".to_owned(),
            ]
        );
        assert!(!urls.iter().any(|url| url.ends_with("/api/health")));
    }
}
