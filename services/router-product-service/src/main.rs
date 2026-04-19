use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use sdkwork_api_config::{StandaloneConfig, StandaloneConfigLoader};
use sdkwork_api_observability::init_tracing;
use sdkwork_api_product_runtime::{
    ProductRuntimeRole, ProductSiteDirs, RouterProductRuntime, RouterProductRuntimeOptions,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

const SDKWORK_CONFIG_DIR: &str = "SDKWORK_CONFIG_DIR";
const SDKWORK_CONFIG_FILE: &str = "SDKWORK_CONFIG_FILE";
const SDKWORK_DATABASE_URL: &str = "SDKWORK_DATABASE_URL";
const SDKWORK_ROUTER_INSTALL_MODE: &str = "SDKWORK_ROUTER_INSTALL_MODE";
const SDKWORK_WEB_BIND: &str = "SDKWORK_WEB_BIND";
const SDKWORK_ROUTER_ROLES: &str = "SDKWORK_ROUTER_ROLES";
const SDKWORK_ROUTER_NODE_ID_PREFIX: &str = "SDKWORK_ROUTER_NODE_ID_PREFIX";
const SDKWORK_GATEWAY_BIND: &str = "SDKWORK_GATEWAY_BIND";
const SDKWORK_ADMIN_BIND: &str = "SDKWORK_ADMIN_BIND";
const SDKWORK_PORTAL_BIND: &str = "SDKWORK_PORTAL_BIND";
const SDKWORK_ADMIN_PROXY_TARGET: &str = "SDKWORK_ADMIN_PROXY_TARGET";
const SDKWORK_PORTAL_PROXY_TARGET: &str = "SDKWORK_PORTAL_PROXY_TARGET";
const SDKWORK_GATEWAY_PROXY_TARGET: &str = "SDKWORK_GATEWAY_PROXY_TARGET";
const SDKWORK_ADMIN_SITE_DIR: &str = "SDKWORK_ADMIN_SITE_DIR";
const SDKWORK_PORTAL_SITE_DIR: &str = "SDKWORK_PORTAL_SITE_DIR";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
enum PlanFormat {
    #[default]
    Text,
    Json,
}

impl PlanFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
        }
    }
}

#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(
    name = "router-product-service",
    about = "Start the integrated SDKWork Router product host in server mode."
)]
struct RouterProductServiceCli {
    #[arg(long = "config-dir", value_name = "DIR")]
    config_dir: Option<String>,
    #[arg(long = "config-file", value_name = "FILE")]
    config_file: Option<String>,
    #[arg(long = "runtime-home", value_name = "DIR")]
    runtime_home: Option<PathBuf>,
    #[arg(long = "database-url", value_name = "URL")]
    database_url: Option<String>,
    #[arg(long = "bind", value_name = "HOST:PORT")]
    public_web_bind: Option<String>,
    #[arg(long = "roles", value_name = "ROLES")]
    roles: Option<String>,
    #[arg(long = "node-id-prefix", value_name = "PREFIX")]
    node_id_prefix: Option<String>,
    #[arg(long = "gateway-bind", value_name = "HOST:PORT")]
    gateway_bind: Option<String>,
    #[arg(long = "admin-bind", value_name = "HOST:PORT")]
    admin_bind: Option<String>,
    #[arg(long = "portal-bind", value_name = "HOST:PORT")]
    portal_bind: Option<String>,
    #[arg(long = "admin-upstream", value_name = "HOST:PORT")]
    admin_upstream: Option<String>,
    #[arg(long = "portal-upstream", value_name = "HOST:PORT")]
    portal_upstream: Option<String>,
    #[arg(long = "gateway-upstream", value_name = "HOST:PORT")]
    gateway_upstream: Option<String>,
    #[arg(long = "admin-site-dir", value_name = "DIR")]
    admin_site_dir: Option<PathBuf>,
    #[arg(long = "portal-site-dir", value_name = "DIR")]
    portal_site_dir: Option<PathBuf>,
    #[arg(
        long = "backup-output",
        value_name = "DIR",
        conflicts_with = "restore_source"
    )]
    backup_output: Option<PathBuf>,
    #[arg(
        long = "restore-source",
        value_name = "DIR",
        conflicts_with = "backup_output"
    )]
    restore_source: Option<PathBuf>,
    #[arg(long = "force", default_value_t = false)]
    force: bool,
    #[arg(long = "plan-format", value_enum, default_value_t = PlanFormat::Text)]
    plan_format: PlanFormat,
    #[arg(long = "dry-run", default_value_t = false)]
    dry_run: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ProductServiceSettings {
    config_dir: Option<String>,
    config_file: Option<String>,
    runtime_home: Option<PathBuf>,
    database_url: Option<String>,
    public_web_bind: Option<String>,
    roles: Option<Vec<ProductRuntimeRole>>,
    node_id_prefix: Option<String>,
    gateway_bind: Option<String>,
    admin_bind: Option<String>,
    portal_bind: Option<String>,
    admin_upstream: Option<String>,
    portal_upstream: Option<String>,
    gateway_upstream: Option<String>,
    admin_site_dir: Option<PathBuf>,
    portal_site_dir: Option<PathBuf>,
    backup_output: Option<PathBuf>,
    restore_source: Option<PathBuf>,
    force: bool,
    plan_format: PlanFormat,
    dry_run: bool,
}

impl ProductServiceSettings {
    fn operation_kind(&self) -> Option<RuntimeOperationKind> {
        if self.backup_output.is_some() {
            Some(RuntimeOperationKind::Backup)
        } else if self.restore_source.is_some() {
            Some(RuntimeOperationKind::Restore)
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = RouterProductServiceCli::parse();
    let mut settings = resolve_service_settings(&cli, env::vars())?;
    let runtime_context = resolve_runtime_context(settings.runtime_home.as_deref())?;
    apply_runtime_context_defaults(&mut settings, runtime_context.as_ref());
    load_router_env_defaults(&settings)?;
    let (loader, config) = load_runtime_config(&settings, &cli)?;

    if settings.dry_run {
        print!("{}", render_service_plan(&settings, &config)?);
        return Ok(());
    }

    if let Some(operation) = settings.operation_kind() {
        execute_runtime_operation(operation, &settings, &config, runtime_context.as_ref())?;
        return Ok(());
    }

    let effective_public_web_bind = resolve_effective_public_web_bind(&settings, &config);
    let install_mode = env::var(SDKWORK_ROUTER_INSTALL_MODE).ok();
    validate_runtime_config_for_install_mode(
        &config,
        effective_public_web_bind,
        install_mode.as_deref(),
    )?;

    init_tracing("router-product-service");
    let options = build_runtime_options(&settings, &config)?;
    let runtime = RouterProductRuntime::start(loader, config, options).await?;
    print_runtime_summary(&runtime);

    tokio::signal::ctrl_c().await?;
    drop(runtime);
    Ok(())
}

fn resolve_service_settings<I, K, V>(
    cli: &RouterProductServiceCli,
    env_pairs: I,
) -> anyhow::Result<ProductServiceSettings>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let env_values = collect_env_values(env_pairs);

    if (cli.backup_output.is_some() || cli.restore_source.is_some()) && cli.runtime_home.is_none() {
        anyhow::bail!("backup and restore operations require --runtime-home");
    }

    Ok(ProductServiceSettings {
        config_dir: resolve_string_option(
            cli.config_dir.as_deref(),
            &env_values,
            SDKWORK_CONFIG_DIR,
        ),
        config_file: resolve_string_option(
            cli.config_file.as_deref(),
            &env_values,
            SDKWORK_CONFIG_FILE,
        ),
        runtime_home: cli.runtime_home.clone(),
        database_url: resolve_string_option(
            cli.database_url.as_deref(),
            &env_values,
            SDKWORK_DATABASE_URL,
        ),
        public_web_bind: resolve_string_option(
            cli.public_web_bind.as_deref(),
            &env_values,
            SDKWORK_WEB_BIND,
        ),
        roles: resolve_string_option(cli.roles.as_deref(), &env_values, SDKWORK_ROUTER_ROLES)
            .map(|value| parse_roles(&value))
            .transpose()?,
        node_id_prefix: resolve_string_option(
            cli.node_id_prefix.as_deref(),
            &env_values,
            SDKWORK_ROUTER_NODE_ID_PREFIX,
        ),
        gateway_bind: resolve_string_option(
            cli.gateway_bind.as_deref(),
            &env_values,
            SDKWORK_GATEWAY_BIND,
        ),
        admin_bind: resolve_string_option(
            cli.admin_bind.as_deref(),
            &env_values,
            SDKWORK_ADMIN_BIND,
        ),
        portal_bind: resolve_string_option(
            cli.portal_bind.as_deref(),
            &env_values,
            SDKWORK_PORTAL_BIND,
        ),
        admin_upstream: resolve_string_option(
            cli.admin_upstream.as_deref(),
            &env_values,
            SDKWORK_ADMIN_PROXY_TARGET,
        ),
        portal_upstream: resolve_string_option(
            cli.portal_upstream.as_deref(),
            &env_values,
            SDKWORK_PORTAL_PROXY_TARGET,
        ),
        gateway_upstream: resolve_string_option(
            cli.gateway_upstream.as_deref(),
            &env_values,
            SDKWORK_GATEWAY_PROXY_TARGET,
        ),
        admin_site_dir: resolve_path_option(
            cli.admin_site_dir.as_ref(),
            &env_values,
            SDKWORK_ADMIN_SITE_DIR,
        ),
        portal_site_dir: resolve_path_option(
            cli.portal_site_dir.as_ref(),
            &env_values,
            SDKWORK_PORTAL_SITE_DIR,
        ),
        backup_output: cli.backup_output.clone(),
        restore_source: cli.restore_source.clone(),
        force: cli.force,
        plan_format: cli.plan_format,
        dry_run: cli.dry_run,
    })
}

fn resolve_runtime_context(
    runtime_home: Option<&Path>,
) -> anyhow::Result<Option<InstalledRuntimeContext>> {
    let Some(runtime_home) = runtime_home else {
        return Ok(None);
    };

    let runtime_home = resolve_installed_runtime_home(runtime_home)?;
    let manifest_path = runtime_home.join("release-manifest.json");
    let manifest_contents = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "failed to read installed runtime manifest {}",
            manifest_path.display()
        )
    })?;
    let manifest: InstalledRuntimeManifest = serde_json::from_str(&manifest_contents)
        .with_context(|| {
            format!(
                "failed to parse installed runtime manifest {}",
                manifest_path.display()
            )
        })?;

    Ok(Some(InstalledRuntimeContext {
        runtime_home,
        manifest_path,
        manifest,
    }))
}

fn resolve_installed_runtime_home(candidate: &Path) -> anyhow::Result<PathBuf> {
    let absolute_candidate = absolutize_path(candidate)?;
    if absolute_candidate.join("release-manifest.json").is_file() {
        return Ok(absolute_candidate);
    }

    let current_candidate = absolute_candidate.join("current");
    if current_candidate.join("release-manifest.json").is_file() {
        return Ok(current_candidate);
    }

    anyhow::bail!(
        "runtime home {} does not contain current/release-manifest.json",
        absolute_candidate.display()
    );
}

fn absolutize_path(candidate: &Path) -> anyhow::Result<PathBuf> {
    if candidate.is_absolute() {
        Ok(candidate.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(candidate))
    }
}

fn apply_runtime_context_defaults(
    settings: &mut ProductServiceSettings,
    runtime_context: Option<&InstalledRuntimeContext>,
) {
    let Some(runtime_context) = runtime_context else {
        return;
    };

    settings.runtime_home = Some(runtime_context.runtime_home.clone());
    if settings.config_dir.is_none() {
        settings.config_dir = runtime_context
            .manifest
            .config_root
            .as_ref()
            .map(|path| path_to_string(path));
    }
    if settings.config_file.is_none() {
        settings.config_file = runtime_context
            .manifest
            .config_file
            .as_ref()
            .map(|path| path_to_string(path));
    }
    if settings.admin_site_dir.is_none() {
        settings.admin_site_dir = runtime_context.manifest.admin_site_dist_dir.clone();
    }
    if settings.portal_site_dir.is_none() {
        settings.portal_site_dir = runtime_context.manifest.portal_site_dist_dir.clone();
    }

    if env::var_os(SDKWORK_ROUTER_INSTALL_MODE).is_none() {
        if let Some(install_mode) = runtime_context.manifest.install_mode.as_deref() {
            env::set_var(SDKWORK_ROUTER_INSTALL_MODE, install_mode);
        }
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn load_router_env_defaults(settings: &ProductServiceSettings) -> anyhow::Result<()> {
    let config_root = settings
        .config_dir
        .as_deref()
        .map(PathBuf::from)
        .or_else(|| {
            settings
                .config_file
                .as_deref()
                .and_then(|value| Path::new(value).parent().map(Path::to_path_buf))
        });

    let Some(config_root) = config_root else {
        return Ok(());
    };

    let env_file = config_root.join("router.env");
    if !env_file.is_file() {
        return Ok(());
    }

    for (key, value) in read_env_file_pairs(&env_file)? {
        if env::var_os(&key).is_none() {
            env::set_var(key, value);
        }
    }

    Ok(())
}

fn read_env_file_pairs(env_file: &Path) -> anyhow::Result<Vec<(String, String)>> {
    let contents = fs::read_to_string(env_file)
        .with_context(|| format!("failed to read runtime env file {}", env_file.display()))?;
    let mut pairs = Vec::new();

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        pairs.push((key.to_owned(), unquote_env_value(value.trim())));
    }

    Ok(pairs)
}

fn unquote_env_value(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        return value[1..value.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
    }
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        return value[1..value.len() - 1].to_owned();
    }

    value.to_owned()
}

fn resolve_database_operation_contract(
    database_url: &str,
    operation: RuntimeOperationKind,
) -> DatabaseOperationContract {
    let normalized = database_url.trim().to_ascii_lowercase();
    if normalized.starts_with("postgres://") || normalized.starts_with("postgresql://") {
        return DatabaseOperationContract {
            kind: "postgresql".to_owned(),
            strategy: match operation {
                RuntimeOperationKind::Backup => "pg_dump-custom".to_owned(),
                RuntimeOperationKind::Restore => "pg_restore-custom".to_owned(),
            },
            dump_file: Some("database/postgresql.dump".to_owned()),
            supported: true,
        };
    }

    if normalized.starts_with("sqlite:") {
        return DatabaseOperationContract {
            kind: "sqlite".to_owned(),
            strategy: "filesystem-snapshot".to_owned(),
            dump_file: None,
            supported: true,
        };
    }

    let kind = normalized
        .split(':')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    DatabaseOperationContract {
        kind: kind.to_owned(),
        strategy: match operation {
            RuntimeOperationKind::Backup => "manual-provider-backup".to_owned(),
            RuntimeOperationKind::Restore => "manual-provider-restore".to_owned(),
        },
        dump_file: None,
        supported: false,
    }
}

fn execute_runtime_operation(
    operation: RuntimeOperationKind,
    settings: &ProductServiceSettings,
    config: &StandaloneConfig,
    runtime_context: Option<&InstalledRuntimeContext>,
) -> anyhow::Result<()> {
    let runtime_context = runtime_context.with_context(|| {
        format!(
            "{} requires --runtime-home that resolves to an installed current control home",
            operation.display_name()
        )
    })?;

    match operation {
        RuntimeOperationKind::Backup => execute_backup_operation(settings, config, runtime_context),
        RuntimeOperationKind::Restore => {
            execute_restore_operation(settings, config, runtime_context)
        }
    }
}

fn execute_backup_operation(
    settings: &ProductServiceSettings,
    config: &StandaloneConfig,
    runtime_context: &InstalledRuntimeContext,
) -> anyhow::Result<()> {
    let backup_output = settings
        .backup_output
        .as_deref()
        .context("backup requires --backup-output")?;
    let backup_output = absolutize_path(backup_output)?;
    let config_root = required_manifest_path(&runtime_context.manifest.config_root, "configRoot")?;
    let mutable_data_root = required_manifest_path(
        &runtime_context.manifest.mutable_data_root,
        "mutableDataRoot",
    )?;
    let log_root = required_manifest_path(&runtime_context.manifest.log_root, "logRoot")?;
    let run_root = required_manifest_path(&runtime_context.manifest.run_root, "runRoot")?;

    ensure_runtime_stopped(&run_root)?;
    ensure_backup_output_path_is_safe(
        &backup_output,
        [
            config_root.as_path(),
            mutable_data_root.as_path(),
            log_root.as_path(),
            run_root.as_path(),
        ],
    )?;
    prepare_output_directory(&backup_output, settings.force)?;

    let control_dir = backup_output.join("control");
    let config_dir = backup_output.join("config");
    let data_dir = backup_output.join("data");
    fs::create_dir_all(&control_dir)?;
    copy_file(
        &runtime_context.manifest_path,
        &control_dir.join("release-manifest.json"),
    )?;
    copy_dir_recursive(&config_root, &config_dir)?;
    if mutable_data_root.is_dir() {
        copy_dir_recursive(&mutable_data_root, &data_dir)?;
    } else {
        fs::create_dir_all(&data_dir)?;
    }

    let database_contract =
        resolve_database_operation_contract(&config.database_url, RuntimeOperationKind::Backup);
    if !database_contract.supported {
        anyhow::bail!(
            "backup does not support database scheme {} automatically; perform a provider-native export and store it beside {}",
            database_contract.kind,
            backup_output.display()
        );
    }

    if database_contract.kind == "postgresql" {
        let dump_file = backup_output.join(
            database_contract
                .dump_file
                .as_deref()
                .expect("postgresql backup dump path should exist"),
        );
        dump_postgresql_database(&config.database_url, &dump_file)?;
    }

    let bundle_manifest = BackupBundleManifest {
        format_version: 1,
        created_at: chrono_like_timestamp()?,
        runtime_home: runtime_context.runtime_home.clone(),
        install_mode: runtime_context.manifest.install_mode.clone(),
        release_version: runtime_context.manifest.release_version.clone(),
        product_root: runtime_context.manifest.product_root.clone(),
        config_root: config_root.clone(),
        config_file: runtime_context.manifest.config_file.clone(),
        mutable_data_root: mutable_data_root.clone(),
        log_root: log_root.clone(),
        run_root: run_root.clone(),
        database: BackupDatabaseContract {
            kind: database_contract.kind.clone(),
            strategy: database_contract.strategy.clone(),
            dump_file: database_contract.dump_file.clone(),
        },
    };
    write_json_file(
        &backup_output.join("backup-manifest.json"),
        &bundle_manifest,
    )?;

    println!(
        "router-product-service backup completed at {}",
        backup_output.display()
    );
    Ok(())
}

fn execute_restore_operation(
    settings: &ProductServiceSettings,
    config: &StandaloneConfig,
    runtime_context: &InstalledRuntimeContext,
) -> anyhow::Result<()> {
    if !settings.force {
        anyhow::bail!("restore requires --force");
    }

    let restore_source = settings
        .restore_source
        .as_deref()
        .context("restore requires --restore-source")?;
    let restore_source = absolutize_path(restore_source)?;
    let bundle_manifest = read_backup_bundle_manifest(&restore_source)?;
    let control_manifest_path = restore_source.join("control").join("release-manifest.json");
    if !control_manifest_path.is_file() {
        anyhow::bail!(
            "restore source {} is missing control/release-manifest.json",
            restore_source.display()
        );
    }

    let config_root = required_manifest_path(&runtime_context.manifest.config_root, "configRoot")?;
    let mutable_data_root = required_manifest_path(
        &runtime_context.manifest.mutable_data_root,
        "mutableDataRoot",
    )?;
    let run_root = required_manifest_path(&runtime_context.manifest.run_root, "runRoot")?;
    ensure_runtime_stopped(&run_root)?;

    replace_dir_from_snapshot(&restore_source.join("config"), &config_root, true)?;
    replace_dir_from_snapshot(&restore_source.join("data"), &mutable_data_root, true)?;

    if bundle_manifest.database.kind == "postgresql" {
        let dump_file = restore_source.join(
            bundle_manifest
                .database
                .dump_file
                .as_deref()
                .unwrap_or("database/postgresql.dump"),
        );
        if !dump_file.is_file() {
            anyhow::bail!(
                "restore source {} is missing PostgreSQL dump {}",
                restore_source.display(),
                dump_file.display()
            );
        }

        let restored_config =
            reload_config_from_runtime_roots(settings, &runtime_context.manifest)?;
        restore_postgresql_database(&restored_config.database_url, &dump_file)?;
    } else if bundle_manifest.database.kind != "sqlite" {
        anyhow::bail!(
            "restore does not support database scheme {} automatically; apply the provider-native restore before starting the runtime",
            bundle_manifest.database.kind
        );
    }

    println!(
        "router-product-service restore completed from {}",
        restore_source.display()
    );
    let _ = config;
    Ok(())
}

fn required_manifest_path(value: &Option<PathBuf>, field_name: &str) -> anyhow::Result<PathBuf> {
    value
        .clone()
        .with_context(|| format!("installed runtime manifest is missing {field_name}"))
}

fn ensure_runtime_stopped(run_root: &Path) -> anyhow::Result<()> {
    let pid_file = run_root.join("router-product-service.pid");
    if !pid_file.is_file() {
        return Ok(());
    }

    let pid_contents = fs::read_to_string(&pid_file)
        .with_context(|| format!("failed to read {}", pid_file.display()))?;
    let pid = pid_contents.trim().parse::<u32>().with_context(|| {
        format!(
            "invalid router-product-service pid file {}",
            pid_file.display()
        )
    })?;

    if process_is_running(pid)? {
        anyhow::bail!(
            "router-product-service is still running (pid {pid}); stop the service before continuing"
        );
    }

    Ok(())
}

fn process_is_running(pid: u32) -> anyhow::Result<bool> {
    if cfg!(windows) {
        let status = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!(
                    "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}"
                ),
            ])
            .status()
            .context("failed to probe running process via powershell.exe")?;
        return Ok(status.success());
    }

    let status = Command::new("sh")
        .args(["-c", &format!("kill -0 {pid}")])
        .status()
        .context("failed to probe running process via kill -0")?;
    Ok(status.success())
}

fn ensure_backup_output_path_is_safe<'a>(
    backup_output: &Path,
    protected_roots: impl IntoIterator<Item = &'a Path>,
) -> anyhow::Result<()> {
    for protected_root in protected_roots {
        if backup_output.starts_with(protected_root) {
            anyhow::bail!(
                "backup output {} must be outside {}",
                backup_output.display(),
                protected_root.display()
            );
        }
    }

    Ok(())
}

fn prepare_output_directory(output_dir: &Path, force: bool) -> anyhow::Result<()> {
    if output_dir.exists() {
        if !force {
            anyhow::bail!(
                "backup output {} already exists; rerun with --force to replace it",
                output_dir.display()
            );
        }

        remove_path(output_dir)?;
    }

    fs::create_dir_all(output_dir)?;
    Ok(())
}

fn remove_path(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> anyhow::Result<()> {
    let parent = destination
        .parent()
        .context("destination file is missing parent")?;
    fs::create_dir_all(parent)?;
    fs::copy(source, destination).with_context(|| {
        format!(
            "failed to copy file from {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> anyhow::Result<()> {
    if !source.is_dir() {
        anyhow::bail!("source directory does not exist: {}", source.display());
    }

    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            copy_file(&source_path, &destination_path)?;
        } else {
            anyhow::bail!(
                "unsupported non-file entry {} while copying {}",
                source_path.display(),
                source.display()
            );
        }
    }

    Ok(())
}

fn replace_dir_from_snapshot(
    snapshot_dir: &Path,
    destination_dir: &Path,
    force: bool,
) -> anyhow::Result<()> {
    if destination_dir.exists() {
        if !force {
            anyhow::bail!(
                "destination {} already exists; rerun with --force to replace it",
                destination_dir.display()
            );
        }
        remove_path(destination_dir)?;
    }

    if snapshot_dir.is_dir() {
        copy_dir_recursive(snapshot_dir, destination_dir)?;
    } else {
        fs::create_dir_all(destination_dir)?;
    }

    Ok(())
}

fn read_backup_bundle_manifest(source_root: &Path) -> anyhow::Result<BackupBundleManifest> {
    let manifest_path = source_root.join("backup-manifest.json");
    let contents = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))
}

fn dump_postgresql_database(database_url: &str, dump_file: &Path) -> anyhow::Result<()> {
    let parent = dump_file
        .parent()
        .context("dump file is missing parent directory")?;
    fs::create_dir_all(parent)?;
    let status = Command::new("pg_dump")
        .args(["--format=custom", "--file"])
        .arg(dump_file)
        .arg(database_url)
        .status()
        .context(
            "failed to start pg_dump; ensure PostgreSQL client tools are installed and on PATH",
        )?;
    if !status.success() {
        anyhow::bail!(
            "pg_dump failed for PostgreSQL backup to {}",
            dump_file.display()
        );
    }
    Ok(())
}

fn restore_postgresql_database(database_url: &str, dump_file: &Path) -> anyhow::Result<()> {
    let status = Command::new("pg_restore")
        .args([
            "--clean",
            "--if-exists",
            "--no-owner",
            "--no-privileges",
            "--dbname",
        ])
        .arg(database_url)
        .arg(dump_file)
        .status()
        .context(
            "failed to start pg_restore; ensure PostgreSQL client tools are installed and on PATH",
        )?;
    if !status.success() {
        anyhow::bail!(
            "pg_restore failed for PostgreSQL dump {}",
            dump_file.display()
        );
    }
    Ok(())
}

fn reload_config_from_runtime_roots(
    settings: &ProductServiceSettings,
    manifest: &InstalledRuntimeManifest,
) -> anyhow::Result<StandaloneConfig> {
    let config_root = settings
        .config_dir
        .as_deref()
        .map(PathBuf::from)
        .or_else(|| manifest.config_root.clone())
        .context("runtime config root is unavailable for restore")?;
    let config_file = settings.config_file.clone().or_else(|| {
        manifest
            .config_file
            .as_ref()
            .map(|path| path_to_string(path))
    });

    let mut env_pairs: HashMap<String, String> =
        read_env_file_pairs(&config_root.join("router.env"))?
            .into_iter()
            .collect();
    env_pairs.insert(SDKWORK_CONFIG_DIR.to_owned(), path_to_string(&config_root));
    if let Some(config_file) = config_file {
        env_pairs.insert(SDKWORK_CONFIG_FILE.to_owned(), config_file);
    }

    let (loader, config) =
        StandaloneConfigLoader::from_local_root_and_pairs(&config_root, env_pairs.clone())?;
    let overrides = build_loader_setting_overrides(settings);
    if overrides.is_empty() {
        Ok(config)
    } else {
        let (_loader, config) = loader.with_overrides(overrides)?;
        Ok(config)
    }
}

fn build_loader_setting_overrides(settings: &ProductServiceSettings) -> Vec<(String, String)> {
    let mut overrides = Vec::new();
    if let Some(database_url) = settings.database_url.as_deref() {
        overrides.push((SDKWORK_DATABASE_URL.to_owned(), database_url.to_owned()));
    }
    if let Some(gateway_bind) = settings.gateway_bind.as_deref() {
        overrides.push((SDKWORK_GATEWAY_BIND.to_owned(), gateway_bind.to_owned()));
    }
    if let Some(admin_bind) = settings.admin_bind.as_deref() {
        overrides.push((SDKWORK_ADMIN_BIND.to_owned(), admin_bind.to_owned()));
    }
    if let Some(portal_bind) = settings.portal_bind.as_deref() {
        overrides.push((SDKWORK_PORTAL_BIND.to_owned(), portal_bind.to_owned()));
    }
    overrides
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .context("json output path is missing parent directory")?;
    fs::create_dir_all(parent)?;
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn chrono_like_timestamp() -> anyhow::Result<String> {
    let output = if cfg!(windows) {
        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "(Get-Date).ToUniversalTime().ToString('o')",
            ])
            .output()
            .context("failed to render UTC timestamp via powershell.exe")?
    } else {
        Command::new("date")
            .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
            .output()
            .context("failed to render UTC timestamp via date -u")?
    };
    if !output.status.success() {
        anyhow::bail!("failed to render UTC timestamp");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn build_runtime_options(
    settings: &ProductServiceSettings,
    config: &StandaloneConfig,
) -> anyhow::Result<RouterProductRuntimeOptions> {
    let mut options = RouterProductRuntimeOptions::server(resolve_site_dirs(settings)?)
        .with_public_web_bind(resolve_effective_public_web_bind(settings, config));

    if let Some(node_id_prefix) = settings.node_id_prefix.as_deref() {
        options = options.with_node_id_prefix(node_id_prefix);
    }
    if let Some(admin_upstream) = settings.admin_upstream.as_deref() {
        options = options.with_admin_upstream(admin_upstream);
    }
    if let Some(portal_upstream) = settings.portal_upstream.as_deref() {
        options = options.with_portal_upstream(portal_upstream);
    }
    if let Some(gateway_upstream) = settings.gateway_upstream.as_deref() {
        options = options.with_gateway_upstream(gateway_upstream);
    }
    if let Some(roles) = settings.roles.clone() {
        options = options.with_roles(roles);
    }

    Ok(options)
}

fn load_runtime_config(
    settings: &ProductServiceSettings,
    cli: &RouterProductServiceCli,
) -> anyhow::Result<(StandaloneConfigLoader, StandaloneConfig)> {
    apply_loader_discovery_env_overrides(settings);
    let (loader, config) = StandaloneConfigLoader::from_env()?;
    let cli_overrides = build_loader_cli_overrides(cli);
    if cli_overrides.is_empty() {
        Ok((loader, config))
    } else {
        loader.with_overrides(cli_overrides)
    }
}

fn apply_loader_discovery_env_overrides(settings: &ProductServiceSettings) {
    for (key, value) in [
        (SDKWORK_CONFIG_DIR, settings.config_dir.as_deref()),
        (SDKWORK_CONFIG_FILE, settings.config_file.as_deref()),
    ] {
        if let Some(value) = value {
            env::set_var(key, value);
        }
    }
}

fn build_loader_cli_overrides(cli: &RouterProductServiceCli) -> Vec<(String, String)> {
    let mut overrides = Vec::new();
    if let Some(database_url) = cli.database_url.as_deref() {
        overrides.push((SDKWORK_DATABASE_URL.to_owned(), database_url.to_owned()));
    }
    if let Some(gateway_bind) = cli.gateway_bind.as_deref() {
        overrides.push((SDKWORK_GATEWAY_BIND.to_owned(), gateway_bind.to_owned()));
    }
    if let Some(admin_bind) = cli.admin_bind.as_deref() {
        overrides.push((SDKWORK_ADMIN_BIND.to_owned(), admin_bind.to_owned()));
    }
    if let Some(portal_bind) = cli.portal_bind.as_deref() {
        overrides.push((SDKWORK_PORTAL_BIND.to_owned(), portal_bind.to_owned()));
    }
    overrides
}

fn validate_runtime_config_for_install_mode(
    config: &StandaloneConfig,
    effective_public_web_bind: &str,
    install_mode: Option<&str>,
) -> anyhow::Result<()> {
    config.validate_security_posture()?;
    validate_public_web_bind_security_posture(config, effective_public_web_bind)?;

    let normalized_install_mode = install_mode
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());

    if matches!(normalized_install_mode.as_deref(), Some("system"))
        && !config.allow_insecure_dev_defaults
        && config
            .database_url
            .to_ascii_lowercase()
            .starts_with("sqlite:")
    {
        anyhow::bail!(
            "system install mode refuses SQLite database_url {}; configure PostgreSQL, MySQL, or another networked production database, or set SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS=true for explicit development-only override",
            config.database_url,
        );
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeOperationKind {
    Backup,
    Restore,
}

impl RuntimeOperationKind {
    fn dry_run_mode(self) -> &'static str {
        match self {
            Self::Backup => "backup-dry-run",
            Self::Restore => "restore-dry-run",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Backup => "backup",
            Self::Restore => "restore",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct InstalledRuntimeManifest {
    install_mode: Option<String>,
    product_root: Option<PathBuf>,
    control_root: Option<PathBuf>,
    release_version: Option<String>,
    releases_root: Option<PathBuf>,
    release_root: Option<PathBuf>,
    bootstrap_data_root: Option<PathBuf>,
    deployment_asset_root: Option<PathBuf>,
    release_payload_manifest: Option<PathBuf>,
    release_payload_readme_file: Option<PathBuf>,
    admin_site_dist_dir: Option<PathBuf>,
    portal_site_dist_dir: Option<PathBuf>,
    router_binary: Option<PathBuf>,
    config_root: Option<PathBuf>,
    config_file: Option<PathBuf>,
    mutable_data_root: Option<PathBuf>,
    log_root: Option<PathBuf>,
    run_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct InstalledRuntimeContext {
    runtime_home: PathBuf,
    manifest_path: PathBuf,
    manifest: InstalledRuntimeManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupBundleManifest {
    format_version: u32,
    created_at: String,
    runtime_home: PathBuf,
    install_mode: Option<String>,
    release_version: Option<String>,
    product_root: Option<PathBuf>,
    config_root: PathBuf,
    config_file: Option<PathBuf>,
    mutable_data_root: PathBuf,
    log_root: PathBuf,
    run_root: PathBuf,
    database: BackupDatabaseContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupDatabaseContract {
    kind: String,
    strategy: String,
    dump_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DatabaseOperationContract {
    kind: String,
    strategy: String,
    dump_file: Option<String>,
    supported: bool,
}

fn resolve_effective_public_web_bind<'a>(
    settings: &'a ProductServiceSettings,
    config: &'a StandaloneConfig,
) -> &'a str {
    settings
        .public_web_bind
        .as_deref()
        .or(config.public_web_bind.as_deref())
        .unwrap_or("127.0.0.1:3001")
}

fn validate_public_web_bind_security_posture(
    config: &StandaloneConfig,
    effective_public_web_bind: &str,
) -> anyhow::Result<()> {
    if bind_is_loopback(effective_public_web_bind) {
        return Ok(());
    }

    let mut externally_exposed = config.clone();
    externally_exposed.gateway_bind = effective_public_web_bind.to_owned();
    externally_exposed.validate_security_posture().map_err(|error| {
        anyhow::anyhow!(
            "refusing public_web_bind {} while the public product entrypoint would expose development defaults: {}",
            effective_public_web_bind,
            error
        )
    })
}

fn bind_is_loopback(bind: &str) -> bool {
    let normalized = bind
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let authority = normalized.split('/').next().unwrap_or(normalized);
    let host = if authority.starts_with('[') {
        authority
            .split(']')
            .next()
            .map(|value| format!("{value}]"))
            .unwrap_or_else(|| authority.to_owned())
    } else {
        authority.split(':').next().unwrap_or(authority).to_owned()
    };

    matches!(host.as_str(), "127.0.0.1" | "localhost" | "[::1]" | "::1")
}

fn parse_roles(value: &str) -> anyhow::Result<Vec<ProductRuntimeRole>> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ProductRuntimeRole::parse)
        .collect()
}

fn collect_env_values<I, K, V>(pairs: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    pairs
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect()
}

fn resolve_string_option(
    cli_value: Option<&str>,
    env_values: &HashMap<String, String>,
    env_key: &str,
) -> Option<String> {
    cli_value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            env_values
                .get(env_key)
                .map(String::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
}

fn resolve_path_option(
    cli_value: Option<&PathBuf>,
    env_values: &HashMap<String, String>,
    env_key: &str,
) -> Option<PathBuf> {
    cli_value.cloned().or_else(|| {
        env_values
            .get(env_key)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    })
}

fn resolve_site_dirs(settings: &ProductServiceSettings) -> anyhow::Result<ProductSiteDirs> {
    if let (Some(admin_site_dir), Some(portal_site_dir)) = (
        settings.admin_site_dir.clone(),
        settings.portal_site_dir.clone(),
    ) {
        return Ok(ProductSiteDirs::new(admin_site_dir, portal_site_dir));
    }

    let ProductSiteDirs {
        admin_site_dir: default_admin_site_dir,
        portal_site_dir: default_portal_site_dir,
    } = resolve_default_site_dirs()?;

    Ok(ProductSiteDirs::new(
        settings
            .admin_site_dir
            .clone()
            .unwrap_or(default_admin_site_dir),
        settings
            .portal_site_dir
            .clone()
            .unwrap_or(default_portal_site_dir),
    ))
}

fn resolve_default_site_dirs() -> anyhow::Result<ProductSiteDirs> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf);

    resolve_default_site_dirs_for_paths(env::current_exe().ok(), workspace_root.as_deref())
}

fn resolve_default_site_dirs_for_paths(
    current_exe: Option<PathBuf>,
    workspace_root: Option<&Path>,
) -> anyhow::Result<ProductSiteDirs> {
    let mut attempted_layouts = Vec::new();

    if let Some(current_exe) = current_exe {
        if let Some(release_layout) = release_runtime_site_dirs(&current_exe) {
            if site_dirs_exist(&release_layout) {
                return Ok(release_layout);
            }
            attempted_layouts.push(describe_site_dirs("release_runtime_home", &release_layout));
        }
    }

    if let Some(workspace_root) = workspace_root {
        let workspace_layout = ProductSiteDirs::from_workspace_root(workspace_root);
        if site_dirs_exist(&workspace_layout) {
            return Ok(workspace_layout);
        }
        attempted_layouts.push(describe_site_dirs("workspace_root", &workspace_layout));
    }

    let attempted_layouts = if attempted_layouts.is_empty() {
        "none".to_owned()
    } else {
        attempted_layouts.join("; ")
    };

    anyhow::bail!(
        "unable to resolve default site directories; set {SDKWORK_ADMIN_SITE_DIR} and {SDKWORK_PORTAL_SITE_DIR} or pass --admin-site-dir/--portal-site-dir. attempted layouts: {attempted_layouts}"
    );
}

fn release_runtime_site_dirs(current_exe: &Path) -> Option<ProductSiteDirs> {
    let runtime_home = current_exe.parent()?.parent()?;
    Some(ProductSiteDirs::new(
        runtime_home.join("sites").join("admin").join("dist"),
        runtime_home.join("sites").join("portal").join("dist"),
    ))
}

fn site_dirs_exist(site_dirs: &ProductSiteDirs) -> bool {
    site_dirs.admin_site_dir.is_dir() && site_dirs.portal_site_dir.is_dir()
}

fn describe_site_dirs(label: &str, site_dirs: &ProductSiteDirs) -> String {
    format!(
        "{label}: admin={}, portal={}",
        site_dirs.admin_site_dir.display(),
        site_dirs.portal_site_dir.display()
    )
}

fn render_service_plan(
    settings: &ProductServiceSettings,
    config: &StandaloneConfig,
) -> anyhow::Result<String> {
    match settings.plan_format {
        PlanFormat::Text => render_service_plan_text(settings, config),
        PlanFormat::Json => render_service_plan_json(settings, config),
    }
}

fn render_service_plan_text(
    settings: &ProductServiceSettings,
    config: &StandaloneConfig,
) -> anyhow::Result<String> {
    if let Some(operation) = settings.operation_kind() {
        let database_contract =
            resolve_database_operation_contract(&config.database_url, operation);
        let path_label = match operation {
            RuntimeOperationKind::Backup => "backup_output",
            RuntimeOperationKind::Restore => "restore_source",
        };
        let path_value = match operation {
            RuntimeOperationKind::Backup => settings
                .backup_output
                .as_deref()
                .map(path_to_string)
                .context("backup requires --backup-output")?,
            RuntimeOperationKind::Restore => settings
                .restore_source
                .as_deref()
                .map(path_to_string)
                .context("restore requires --restore-source")?,
        };
        let runtime_home = settings
            .runtime_home
            .as_deref()
            .map(path_to_string)
            .context("backup and restore require --runtime-home")?;

        let mut lines = vec![
            format!(
                "router-product-service {} dry run",
                operation.display_name()
            ),
            format!("runtime_home={runtime_home}"),
            format!("{path_label}={path_value}"),
            format!("database.kind={}", database_contract.kind),
            format!("database.strategy={}", database_contract.strategy),
        ];
        if let Some(dump_file) = database_contract.dump_file {
            lines.push(format!("database.dump_file={dump_file}"));
        }
        lines.push(String::new());
        return Ok(lines.join("\n"));
    }

    let site_dirs = resolve_site_dirs(settings)?;
    let roles = settings
        .roles
        .clone()
        .unwrap_or_else(|| {
            vec![
                ProductRuntimeRole::Web,
                ProductRuntimeRole::Gateway,
                ProductRuntimeRole::Admin,
                ProductRuntimeRole::Portal,
            ]
        })
        .into_iter()
        .map(ProductRuntimeRole::as_str)
        .collect::<Vec<_>>()
        .join(",");

    let mut lines = vec![
        "router-product-service dry run".to_owned(),
        format!("roles={roles}"),
        format!(
            "public_web_bind={}",
            resolve_effective_public_web_bind(settings, config)
        ),
        format!("gateway_bind={}", config.gateway_bind),
        format!("admin_bind={}", config.admin_bind),
        format!("portal_bind={}", config.portal_bind),
        format!("database_url={}", config.database_url),
        format!("admin_site_dir={}", site_dirs.admin_site_dir.display()),
        format!("portal_site_dir={}", site_dirs.portal_site_dir.display()),
    ];

    if let Some(config_dir) = settings.config_dir.as_deref() {
        lines.push(format!("config_dir={config_dir}"));
    }
    if let Some(config_file) = settings.config_file.as_deref() {
        lines.push(format!("config_file={config_file}"));
    }
    if let Some(node_id_prefix) = settings.node_id_prefix.as_deref() {
        lines.push(format!("node_id_prefix={node_id_prefix}"));
    }
    if let Some(gateway_upstream) = settings.gateway_upstream.as_deref() {
        lines.push(format!("gateway_upstream={gateway_upstream}"));
    }
    if let Some(admin_upstream) = settings.admin_upstream.as_deref() {
        lines.push(format!("admin_upstream={admin_upstream}"));
    }
    if let Some(portal_upstream) = settings.portal_upstream.as_deref() {
        lines.push(format!("portal_upstream={portal_upstream}"));
    }

    lines.push(String::new());
    Ok(lines.join("\n"))
}

fn render_service_plan_json(
    settings: &ProductServiceSettings,
    config: &StandaloneConfig,
) -> anyhow::Result<String> {
    if let Some(operation) = settings.operation_kind() {
        let database_contract =
            resolve_database_operation_contract(&config.database_url, operation);
        let operation_path = match operation {
            RuntimeOperationKind::Backup => settings
                .backup_output
                .as_deref()
                .map(path_to_string)
                .context("backup requires --backup-output")?,
            RuntimeOperationKind::Restore => settings
                .restore_source
                .as_deref()
                .map(path_to_string)
                .context("restore requires --restore-source")?,
        };
        let runtime_home = settings
            .runtime_home
            .as_deref()
            .map(path_to_string)
            .context("backup and restore require --runtime-home")?;

        let mut payload = json!({
            "mode": operation.dry_run_mode(),
            "plan_format": settings.plan_format.as_str(),
            "runtime_home": runtime_home,
            "database": {
                "kind": database_contract.kind,
                "strategy": database_contract.strategy,
                "dump_file": database_contract.dump_file,
            }
        });
        match operation {
            RuntimeOperationKind::Backup => payload["backup_output"] = json!(operation_path),
            RuntimeOperationKind::Restore => payload["restore_source"] = json!(operation_path),
        }

        return Ok(serde_json::to_string_pretty(&payload)
            .expect("operation plan json serialization should not fail"));
    }

    let site_dirs = resolve_site_dirs(settings)?;
    let roles = settings
        .roles
        .clone()
        .unwrap_or_else(|| {
            vec![
                ProductRuntimeRole::Web,
                ProductRuntimeRole::Gateway,
                ProductRuntimeRole::Admin,
                ProductRuntimeRole::Portal,
            ]
        })
        .into_iter()
        .map(ProductRuntimeRole::as_str)
        .collect::<Vec<_>>();

    Ok(serde_json::to_string_pretty(&json!({
        "mode": "dry-run",
        "plan_format": settings.plan_format.as_str(),
        "roles": roles,
        "public_web_bind": resolve_effective_public_web_bind(settings, config),
        "database_url": config.database_url,
        "config_dir": settings.config_dir,
        "config_file": settings.config_file,
        "node_id_prefix": settings.node_id_prefix,
        "binds": {
            "gateway": config.gateway_bind,
            "admin": config.admin_bind,
            "portal": config.portal_bind,
        },
        "site_dirs": {
            "admin": site_dirs.admin_site_dir,
            "portal": site_dirs.portal_site_dir,
        },
        "upstreams": {
            "gateway": settings.gateway_upstream,
            "admin": settings.admin_upstream,
            "portal": settings.portal_upstream,
        }
    }))
    .expect("service plan json serialization should not fail"))
}

fn print_runtime_summary(runtime: &RouterProductRuntime) {
    if let Some(bind) = runtime.public_bind_addr() {
        println!("router-product-service public web listening on {bind}");
    }
    if let Some(bind) = runtime.gateway_bind_addr() {
        println!("router-product-service gateway listening on {bind}");
    }
    if let Some(bind) = runtime.admin_bind_addr() {
        println!("router-product-service admin listening on {bind}");
    }
    if let Some(bind) = runtime.portal_bind_addr() {
        println!("router-product-service portal listening on {bind}");
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use clap::Parser;
    use serde_json::Value;

    use super::{
        build_loader_cli_overrides, render_service_plan, resolve_default_site_dirs_for_paths,
        resolve_effective_public_web_bind, resolve_service_settings,
        validate_runtime_config_for_install_mode, PlanFormat, ProductRuntimeRole,
        ProductServiceSettings, RouterProductServiceCli, StandaloneConfig, StandaloneConfigLoader,
    };

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("router-product-service-tests-{name}-{suffix}"))
    }

    #[test]
    fn resolve_service_settings_parses_full_cli_cluster_overrides() {
        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--config-dir",
            "D:/router/config",
            "--config-file",
            "cluster/router.yaml",
            "--database-url",
            "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router",
            "--bind",
            "0.0.0.0:3301",
            "--roles",
            "web,gateway",
            "--node-id-prefix",
            "edge-a",
            "--gateway-bind",
            "127.0.0.1:9080",
            "--admin-upstream",
            "10.0.0.12:8081",
            "--portal-upstream",
            "10.0.0.13:8082",
            "--admin-site-dir",
            "D:/sites/admin",
            "--portal-site-dir",
            "D:/sites/portal",
            "--plan-format",
            "json",
            "--dry-run",
        ])
        .expect("cli should parse");

        let settings = resolve_service_settings(&cli, Vec::<(String, String)>::new())
            .expect("settings should resolve");

        assert_eq!(settings.config_dir, Some("D:/router/config".to_owned()));
        assert_eq!(settings.config_file, Some("cluster/router.yaml".to_owned()));
        assert_eq!(
            settings.database_url,
            Some("postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router".to_owned())
        );
        assert_eq!(settings.public_web_bind, Some("0.0.0.0:3301".to_owned()));
        assert_eq!(settings.node_id_prefix, Some("edge-a".to_owned()));
        assert_eq!(settings.gateway_bind, Some("127.0.0.1:9080".to_owned()));
        assert_eq!(settings.admin_upstream, Some("10.0.0.12:8081".to_owned()));
        assert_eq!(settings.portal_upstream, Some("10.0.0.13:8082".to_owned()));
        assert_eq!(
            settings.roles,
            Some(vec![ProductRuntimeRole::Web, ProductRuntimeRole::Gateway])
        );
        assert_eq!(
            settings.admin_site_dir,
            Some(PathBuf::from("D:/sites/admin"))
        );
        assert_eq!(
            settings.portal_site_dir,
            Some(PathBuf::from("D:/sites/portal"))
        );
        assert_eq!(settings.plan_format.as_str(), "json");
        assert!(settings.dry_run);
    }

    #[test]
    fn resolve_service_settings_prefers_cli_values_over_env() {
        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--bind",
            "0.0.0.0:4301",
            "--roles",
            "portal",
            "--gateway-upstream",
            "10.0.0.22:8080",
        ])
        .expect("cli should parse");

        let settings = resolve_service_settings(
            &cli,
            [
                ("SDKWORK_DATABASE_URL", "sqlite:///tmp/router.db"),
                ("SDKWORK_WEB_BIND", "0.0.0.0:3001"),
                ("SDKWORK_ROUTER_ROLES", "web,gateway,admin"),
                ("SDKWORK_GATEWAY_PROXY_TARGET", "10.0.0.21:8080"),
                ("SDKWORK_ADMIN_PROXY_TARGET", "10.0.0.31:8081"),
            ],
        )
        .expect("settings should resolve");

        assert_eq!(settings.public_web_bind, Some("0.0.0.0:4301".to_owned()));
        assert_eq!(settings.gateway_upstream, Some("10.0.0.22:8080".to_owned()));
        assert_eq!(settings.admin_upstream, Some("10.0.0.31:8081".to_owned()));
        assert_eq!(
            settings.database_url,
            Some("sqlite:///tmp/router.db".to_owned())
        );
        assert_eq!(settings.roles, Some(vec![ProductRuntimeRole::Portal]));
    }

    #[test]
    fn resolve_service_settings_parses_backup_and_restore_operation_fields() {
        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--config-dir",
            "D:/router/config",
            "--runtime-home",
            "D:/router/current",
            "--backup-output",
            "D:/router/backups/2026-04-19",
            "--force",
            "--dry-run",
            "--plan-format",
            "json",
        ])
        .expect("backup cli should parse");

        let settings = resolve_service_settings(&cli, Vec::<(String, String)>::new())
            .expect("backup settings should resolve");

        assert_eq!(settings.config_dir, Some("D:/router/config".to_owned()));
        assert_eq!(
            settings.runtime_home,
            Some(PathBuf::from("D:/router/current"))
        );
        assert_eq!(
            settings.backup_output,
            Some(PathBuf::from("D:/router/backups/2026-04-19"))
        );
        assert_eq!(settings.restore_source, None);
        assert!(settings.force);
        assert!(settings.dry_run);

        let restore_cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--config-dir",
            "D:/router/config",
            "--runtime-home",
            "D:/router/current",
            "--restore-source",
            "D:/router/backups/2026-04-19",
            "--force",
            "--dry-run",
            "--plan-format",
            "json",
        ])
        .expect("restore cli should parse");

        let restore_settings =
            resolve_service_settings(&restore_cli, Vec::<(String, String)>::new())
                .expect("restore settings should resolve");

        assert_eq!(
            restore_settings.restore_source,
            Some(PathBuf::from("D:/router/backups/2026-04-19"))
        );
        assert_eq!(restore_settings.backup_output, None);
        assert!(restore_settings.force);
    }

    #[test]
    fn explicit_cli_database_url_overrides_loaded_config_file() {
        let root = unique_temp_dir("cli-database-url");
        fs::create_dir_all(&root).expect("config root should exist");
        fs::write(
            root.join("router.yaml"),
            r#"
database_url: "sqlite://router.db"
"#,
        )
        .expect("router.yaml should be written");

        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--database-url",
            "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router",
        ])
        .expect("cli should parse");

        let (loader, file_config) = StandaloneConfigLoader::from_local_root_and_pairs(
            &root,
            std::iter::empty::<(&str, &str)>(),
        )
        .expect("file-backed config should load");
        let expected_file_database_url = sqlite_url_for(root.join("router.db"));
        assert_eq!(file_config.database_url, expected_file_database_url);

        let (loader, overridden) = loader
            .with_overrides(build_loader_cli_overrides(&cli))
            .expect("cli overrides should reload config");

        assert_eq!(
            overridden.database_url,
            "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
        );
        assert_eq!(
            loader
                .reload()
                .expect("reloaded config should preserve cli overrides")
                .database_url,
            "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
        );
    }

    #[test]
    fn resolve_effective_public_web_bind_defaults_to_loopback_when_cli_and_config_are_absent() {
        let settings = ProductServiceSettings::default();
        let config = StandaloneConfig::default();

        assert_eq!(
            resolve_effective_public_web_bind(&settings, &config),
            "127.0.0.1:3001"
        );
    }

    #[test]
    fn system_install_mode_rejects_sqlite_without_explicit_dev_override() {
        let error = validate_runtime_config_for_install_mode(
            &StandaloneConfig {
                database_url: "sqlite:///tmp/router.db".to_owned(),
                ..StandaloneConfig::default()
            },
            "127.0.0.1:3001",
            Some("system"),
        )
        .expect_err("system installs should reject sqlite by default");

        assert!(error.to_string().contains("SQLite"));
        assert!(error
            .to_string()
            .contains("SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS"));
    }

    #[test]
    fn system_install_mode_allows_postgres_placeholders_for_validation() {
        validate_runtime_config_for_install_mode(
            &StandaloneConfig {
                database_url: "postgresql://sdkwork:change-me@127.0.0.1:5432/sdkwork_api_router"
                    .to_owned(),
                admin_jwt_signing_secret: "rotated-admin-jwt-secret".to_owned(),
                portal_jwt_signing_secret: "rotated-portal-jwt-secret".to_owned(),
                credential_master_key: "rotated-credential-master-key".to_owned(),
                metrics_bearer_token: "rotated-metrics-bearer-token".to_owned(),
                ..StandaloneConfig::default()
            },
            "0.0.0.0:3001",
            Some("system"),
        )
        .expect("system installs should accept PostgreSQL validation placeholders");
    }

    #[test]
    fn system_install_mode_rejects_non_loopback_public_web_bind_with_dev_defaults() {
        let error = validate_runtime_config_for_install_mode(
            &StandaloneConfig::default(),
            "0.0.0.0:3001",
            Some("system"),
        )
        .expect_err("system installs should reject public non-loopback binds with dev defaults");

        assert!(error.to_string().contains("0.0.0.0:3001"));
        assert!(error
            .to_string()
            .contains("SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS"));
    }

    fn sqlite_url_for(path: impl AsRef<Path>) -> String {
        let normalized = path.as_ref().to_string_lossy().replace('\\', "/");
        if normalized.starts_with('/') {
            format!("sqlite://{normalized}")
        } else {
            format!("sqlite:///{normalized}")
        }
    }

    #[test]
    fn render_service_plan_reports_cluster_shape() {
        let settings = ProductServiceSettings {
            config_dir: Some("D:/router/config".to_owned()),
            config_file: Some("cluster/router.yaml".to_owned()),
            database_url: Some(
                "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router".to_owned(),
            ),
            public_web_bind: Some("0.0.0.0:3301".to_owned()),
            roles: Some(vec![ProductRuntimeRole::Web, ProductRuntimeRole::Gateway]),
            node_id_prefix: Some("edge-a".to_owned()),
            gateway_bind: Some("127.0.0.1:9080".to_owned()),
            admin_bind: None,
            portal_bind: None,
            admin_upstream: Some("10.0.0.12:8081".to_owned()),
            portal_upstream: Some("10.0.0.13:8082".to_owned()),
            gateway_upstream: None,
            admin_site_dir: Some(PathBuf::from("D:/sites/admin")),
            portal_site_dir: Some(PathBuf::from("D:/sites/portal")),
            plan_format: PlanFormat::Text,
            dry_run: true,
            ..ProductServiceSettings::default()
        };
        let config = StandaloneConfig {
            public_web_bind: Some("0.0.0.0:3301".to_owned()),
            gateway_bind: "127.0.0.1:9080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            portal_bind: "127.0.0.1:8082".to_owned(),
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
                .to_owned(),
            ..StandaloneConfig::default()
        };

        let plan = render_service_plan(&settings, &config).expect("plan should render");

        assert!(plan.contains("router-product-service dry run"));
        assert!(plan.contains("roles=web,gateway"));
        assert!(plan.contains("public_web_bind=0.0.0.0:3301"));
        assert!(plan.contains(
            "database_url=postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
        ));
        assert!(plan.contains("admin_upstream=10.0.0.12:8081"));
        assert!(plan.contains("portal_upstream=10.0.0.13:8082"));
    }

    #[test]
    fn render_service_plan_supports_json_cluster_shape() {
        let settings = ProductServiceSettings {
            config_dir: Some("D:/router/config".to_owned()),
            config_file: Some("cluster/router.yaml".to_owned()),
            database_url: Some(
                "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router".to_owned(),
            ),
            public_web_bind: Some("0.0.0.0:3301".to_owned()),
            roles: Some(vec![ProductRuntimeRole::Web, ProductRuntimeRole::Gateway]),
            node_id_prefix: Some("edge-a".to_owned()),
            gateway_bind: Some("127.0.0.1:9080".to_owned()),
            admin_bind: None,
            portal_bind: None,
            admin_upstream: Some("10.0.0.12:8081".to_owned()),
            portal_upstream: Some("10.0.0.13:8082".to_owned()),
            gateway_upstream: None,
            admin_site_dir: Some(PathBuf::from("D:/sites/admin")),
            portal_site_dir: Some(PathBuf::from("D:/sites/portal")),
            plan_format: PlanFormat::Json,
            dry_run: true,
            ..ProductServiceSettings::default()
        };
        let config = StandaloneConfig {
            public_web_bind: Some("0.0.0.0:3301".to_owned()),
            gateway_bind: "127.0.0.1:9080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            portal_bind: "127.0.0.1:8082".to_owned(),
            database_url: "postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router"
                .to_owned(),
            ..StandaloneConfig::default()
        };

        let plan = render_service_plan(&settings, &config).expect("plan should render");
        let parsed: Value = serde_json::from_str(&plan).expect("plan should be valid json");

        assert_eq!(parsed["mode"], "dry-run");
        assert_eq!(parsed["plan_format"], "json");
        assert_eq!(parsed["roles"], serde_json::json!(["web", "gateway"]));
        assert_eq!(parsed["public_web_bind"], "0.0.0.0:3301");
        assert_eq!(parsed["binds"]["gateway"], "127.0.0.1:9080");
        assert_eq!(parsed["binds"]["admin"], "127.0.0.1:8081");
        assert_eq!(parsed["binds"]["portal"], "127.0.0.1:8082");
        assert_eq!(
            parsed["site_dirs"]["admin"],
            serde_json::Value::String("D:/sites/admin".to_owned())
        );
        assert_eq!(
            parsed["upstreams"]["admin"],
            serde_json::Value::String("10.0.0.12:8081".to_owned())
        );
        assert_eq!(
            parsed["upstreams"]["portal"],
            serde_json::Value::String("10.0.0.13:8082".to_owned())
        );
    }

    #[test]
    fn render_service_plan_supports_json_backup_contract() {
        let settings = ProductServiceSettings {
            config_dir: Some("D:/router/config".to_owned()),
            runtime_home: Some(PathBuf::from("D:/router/current")),
            backup_output: Some(PathBuf::from("D:/router/backups/2026-04-19")),
            plan_format: PlanFormat::Json,
            dry_run: true,
            force: true,
            ..ProductServiceSettings::default()
        };
        let config = StandaloneConfig {
            database_url: "postgresql://sdkwork:secret@127.0.0.1:5432/sdkwork_api_router"
                .to_owned(),
            ..StandaloneConfig::default()
        };

        let plan = render_service_plan(&settings, &config).expect("backup plan should render");
        let parsed: Value = serde_json::from_str(&plan).expect("backup plan should be valid json");

        assert_eq!(parsed["mode"], "backup-dry-run");
        assert_eq!(parsed["plan_format"], "json");
        assert_eq!(parsed["runtime_home"], "D:/router/current");
        assert_eq!(parsed["backup_output"], "D:/router/backups/2026-04-19");
        assert_eq!(parsed["database"]["kind"], "postgresql");
        assert_eq!(parsed["database"]["strategy"], "pg_dump-custom");
        assert_eq!(parsed["database"]["dump_file"], "database/postgresql.dump");
    }

    #[test]
    fn render_service_plan_supports_json_restore_contract() {
        let settings = ProductServiceSettings {
            config_dir: Some("D:/router/config".to_owned()),
            runtime_home: Some(PathBuf::from("D:/router/current")),
            restore_source: Some(PathBuf::from("D:/router/backups/2026-04-19")),
            plan_format: PlanFormat::Json,
            dry_run: true,
            force: true,
            ..ProductServiceSettings::default()
        };
        let config = StandaloneConfig {
            database_url: "postgresql://sdkwork:secret@127.0.0.1:5432/sdkwork_api_router"
                .to_owned(),
            ..StandaloneConfig::default()
        };

        let plan = render_service_plan(&settings, &config).expect("restore plan should render");
        let parsed: Value = serde_json::from_str(&plan).expect("restore plan should be valid json");

        assert_eq!(parsed["mode"], "restore-dry-run");
        assert_eq!(parsed["plan_format"], "json");
        assert_eq!(parsed["runtime_home"], "D:/router/current");
        assert_eq!(parsed["restore_source"], "D:/router/backups/2026-04-19");
        assert_eq!(parsed["database"]["kind"], "postgresql");
        assert_eq!(parsed["database"]["strategy"], "pg_restore-custom");
        assert_eq!(parsed["database"]["dump_file"], "database/postgresql.dump");
    }

    #[test]
    fn render_service_plan_uses_config_file_public_web_bind_when_cli_and_env_are_absent() {
        let settings = ProductServiceSettings {
            plan_format: PlanFormat::Text,
            dry_run: true,
            ..ProductServiceSettings::default()
        };
        let config = StandaloneConfig {
            public_web_bind: Some("127.0.0.1:3001".to_owned()),
            ..StandaloneConfig::default()
        };

        let plan = render_service_plan(&settings, &config).expect("plan should render");

        assert!(plan.contains("public_web_bind=127.0.0.1:3001"));
    }

    #[test]
    fn resolve_default_site_dirs_prefers_release_runtime_home_layout() {
        let runtime_home = unique_temp_dir("runtime-home");
        let binary_dir = runtime_home.join("bin");
        fs::create_dir_all(runtime_home.join("sites").join("admin").join("dist"))
            .expect("admin release site dir should exist");
        fs::create_dir_all(runtime_home.join("sites").join("portal").join("dist"))
            .expect("portal release site dir should exist");
        fs::create_dir_all(&binary_dir).expect("binary dir should exist");

        let site_dirs = resolve_default_site_dirs_for_paths(
            Some(binary_dir.join("router-product-service.exe")),
            None::<&Path>,
        )
        .expect("release layout should resolve");

        assert_eq!(
            site_dirs.admin_site_dir,
            runtime_home.join("sites").join("admin").join("dist")
        );
        assert_eq!(
            site_dirs.portal_site_dir,
            runtime_home.join("sites").join("portal").join("dist")
        );
    }

    #[test]
    fn resolve_default_site_dirs_falls_back_to_workspace_root_when_release_layout_is_missing() {
        let binary_root = unique_temp_dir("binary-root");
        let workspace_root = unique_temp_dir("workspace-root");
        fs::create_dir_all(binary_root.join("bin")).expect("binary dir should exist");
        fs::create_dir_all(
            workspace_root
                .join("apps")
                .join("sdkwork-router-admin")
                .join("dist"),
        )
        .expect("admin workspace site dir should exist");
        fs::create_dir_all(
            workspace_root
                .join("apps")
                .join("sdkwork-router-portal")
                .join("dist"),
        )
        .expect("portal workspace site dir should exist");

        let site_dirs = resolve_default_site_dirs_for_paths(
            Some(binary_root.join("bin").join("router-product-service.exe")),
            Some(&workspace_root),
        )
        .expect("workspace layout should resolve");

        assert_eq!(
            site_dirs.admin_site_dir,
            workspace_root
                .join("apps")
                .join("sdkwork-router-admin")
                .join("dist")
        );
        assert_eq!(
            site_dirs.portal_site_dir,
            workspace_root
                .join("apps")
                .join("sdkwork-router-portal")
                .join("dist")
        );
    }

    #[test]
    fn resolve_default_site_dirs_errors_when_no_supported_layout_exists() {
        let binary_root = unique_temp_dir("missing-layout");
        fs::create_dir_all(binary_root.join("bin")).expect("binary dir should exist");

        let error = resolve_default_site_dirs_for_paths(
            Some(binary_root.join("bin").join("router-product-service.exe")),
            None::<&Path>,
        )
        .expect_err("missing site layouts should error");

        assert!(error.to_string().contains("SDKWORK_ADMIN_SITE_DIR"));
        assert!(error.to_string().contains("SDKWORK_PORTAL_SITE_DIR"));
    }
}
