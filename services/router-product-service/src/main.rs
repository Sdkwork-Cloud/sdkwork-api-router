use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

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
const SDKWORK_USER_CENTER_MODE: &str = "SDKWORK_USER_CENTER_MODE";
const SDKWORK_USER_CENTER_APP_API_BASE_URL: &str = "SDKWORK_USER_CENTER_APP_API_BASE_URL";
const SDKWORK_USER_CENTER_EXTERNAL_BASE_URL: &str = "SDKWORK_USER_CENTER_EXTERNAL_BASE_URL";
const SDKWORK_USER_CENTER_PROVIDER_KEY: &str = "SDKWORK_USER_CENTER_PROVIDER_KEY";
const SDKWORK_USER_CENTER_APP_ID: &str = "SDKWORK_USER_CENTER_APP_ID";
const SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH: &str = "SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH";
const SDKWORK_USER_CENTER_SQLITE_PATH: &str = "SDKWORK_USER_CENTER_SQLITE_PATH";
const SDKWORK_USER_CENTER_DATABASE_URL: &str = "SDKWORK_USER_CENTER_DATABASE_URL";
const SDKWORK_USER_CENTER_SCHEMA_NAME: &str = "SDKWORK_USER_CENTER_SCHEMA_NAME";
const SDKWORK_USER_CENTER_TABLE_PREFIX: &str = "SDKWORK_USER_CENTER_TABLE_PREFIX";
const SDKWORK_USER_CENTER_SECRET_ID: &str = "SDKWORK_USER_CENTER_SECRET_ID";
const SDKWORK_USER_CENTER_SHARED_SECRET: &str = "SDKWORK_USER_CENTER_SHARED_SECRET";
const SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS: &str =
    "SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS";
const SDKWORK_USER_CENTER_AUTHORIZATION_HEADER_NAME: &str =
    "SDKWORK_USER_CENTER_AUTHORIZATION_HEADER_NAME";
const SDKWORK_USER_CENTER_ACCESS_TOKEN_HEADER_NAME: &str =
    "SDKWORK_USER_CENTER_ACCESS_TOKEN_HEADER_NAME";
const SDKWORK_USER_CENTER_REFRESH_TOKEN_HEADER_NAME: &str =
    "SDKWORK_USER_CENTER_REFRESH_TOKEN_HEADER_NAME";
const SDKWORK_USER_CENTER_SESSION_HEADER_NAME: &str = "SDKWORK_USER_CENTER_SESSION_HEADER_NAME";
const SDKWORK_USER_CENTER_AUTHORIZATION_SCHEME: &str = "SDKWORK_USER_CENTER_AUTHORIZATION_SCHEME";
const SDKWORK_USER_CENTER_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN: &str =
    "SDKWORK_USER_CENTER_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN";
const INSTALLED_RUNTIME_NAME: &str = "sdkwork-api-router";
const INSTALLED_RUNTIME_LAYOUT_VERSION: u32 = 2;
const ROUTER_PRODUCT_SERVICE_BINARY_STEM: &str = "router-product-service";
const USER_CENTER_DEFAULT_LOCAL_API_BASE_PATH: &str = "/api/app/v1/user-center";
const USER_CENTER_DEFAULT_SQLITE_PATH: &str = "./data/user-center.db";
const USER_CENTER_DEFAULT_TABLE_PREFIX: &str = "rp_uc_";
const USER_CENTER_DEFAULT_APP_ID: &str = "sdkwork-api-router";
const USER_CENTER_DEFAULT_PROVIDER_KEY: &str = "sdkwork-api-router-local";
const USER_CENTER_DEFAULT_AUTHORIZATION_HEADER_NAME: &str = "Authorization";
const USER_CENTER_DEFAULT_ACCESS_TOKEN_HEADER_NAME: &str = "Access-Token";
const USER_CENTER_DEFAULT_REFRESH_TOKEN_HEADER_NAME: &str = "Refresh-Token";
const USER_CENTER_DEFAULT_SESSION_HEADER_NAME: &str = "x-sdkwork-user-center-session-id";
const USER_CENTER_DEFAULT_AUTHORIZATION_SCHEME: &str = "Bearer";
const USER_CENTER_DEFAULT_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN: bool = true;
const USER_CENTER_DEFAULT_HANDSHAKE_FRESHNESS_WINDOW_MS: u64 = 30_000;
const USER_CENTER_HANDSHAKE_MODE_DISABLED: &str = "disabled";
const USER_CENTER_HANDSHAKE_MODE_PROVIDER_SHARED_SECRET: &str = "provider-shared-secret";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum UserCenterMode {
    #[default]
    BuiltinLocal,
    SdkworkCloudAppApi,
    ExternalUserCenter,
}

impl UserCenterMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::BuiltinLocal => "builtin-local",
            Self::SdkworkCloudAppApi => "sdkwork-cloud-app-api",
            Self::ExternalUserCenter => "external-user-center",
        }
    }

    fn from_raw(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" => None,
            "builtin-local" => Some(Self::BuiltinLocal),
            "sdkwork-cloud-app-api" => Some(Self::SdkworkCloudAppApi),
            "external-user-center" => Some(Self::ExternalUserCenter),
            _ => None,
        }
    }

    fn handshake_mode(self) -> &'static str {
        match self {
            Self::BuiltinLocal => USER_CENTER_HANDSHAKE_MODE_DISABLED,
            Self::SdkworkCloudAppApi | Self::ExternalUserCenter => {
                USER_CENTER_HANDSHAKE_MODE_PROVIDER_SHARED_SECRET
            }
        }
    }

    fn handshake_required(self) -> bool {
        !matches!(self, Self::BuiltinLocal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct UserCenterServerContract {
    mode: String,
    active_integration_kind: String,
    app_id: String,
    provider_key: String,
    authority_scope: String,
    session_transport: String,
    validation_strategy: String,
    local_api_base_path: String,
    sqlite_path: String,
    database_url: Option<String>,
    schema_name: Option<String>,
    table_prefix: String,
    app_api_base_url: Option<String>,
    external_base_url: Option<String>,
    secret_id: Option<String>,
    shared_secret_configured: bool,
    auth_token_header_name: String,
    access_token_header_name: String,
    refresh_token_header_name: String,
    session_header_name: String,
    authorization_scheme: String,
    allow_authorization_fallback_to_access_token: bool,
    handshake_mode: String,
    handshake_required: bool,
    handshake_freshness_window_ms: u64,
    protected_tokens: Vec<String>,
    #[serde(skip_serializing)]
    shared_secret: Option<String>,
}

impl Default for UserCenterServerContract {
    fn default() -> Self {
        Self {
            mode: UserCenterMode::BuiltinLocal.as_str().to_owned(),
            active_integration_kind: UserCenterMode::BuiltinLocal.as_str().to_owned(),
            app_id: USER_CENTER_DEFAULT_APP_ID.to_owned(),
            provider_key: USER_CENTER_DEFAULT_PROVIDER_KEY.to_owned(),
            authority_scope: "application".to_owned(),
            session_transport: "header".to_owned(),
            validation_strategy: "auth-access-token".to_owned(),
            local_api_base_path: USER_CENTER_DEFAULT_LOCAL_API_BASE_PATH.to_owned(),
            sqlite_path: USER_CENTER_DEFAULT_SQLITE_PATH.to_owned(),
            database_url: None,
            schema_name: None,
            table_prefix: USER_CENTER_DEFAULT_TABLE_PREFIX.to_owned(),
            app_api_base_url: None,
            external_base_url: None,
            secret_id: None,
            shared_secret_configured: false,
            auth_token_header_name: USER_CENTER_DEFAULT_AUTHORIZATION_HEADER_NAME.to_owned(),
            access_token_header_name: USER_CENTER_DEFAULT_ACCESS_TOKEN_HEADER_NAME.to_owned(),
            refresh_token_header_name: USER_CENTER_DEFAULT_REFRESH_TOKEN_HEADER_NAME.to_owned(),
            session_header_name: USER_CENTER_DEFAULT_SESSION_HEADER_NAME.to_owned(),
            authorization_scheme: USER_CENTER_DEFAULT_AUTHORIZATION_SCHEME.to_owned(),
            allow_authorization_fallback_to_access_token:
                USER_CENTER_DEFAULT_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN,
            handshake_mode: USER_CENTER_HANDSHAKE_MODE_DISABLED.to_owned(),
            handshake_required: false,
            handshake_freshness_window_ms: USER_CENTER_DEFAULT_HANDSHAKE_FRESHNESS_WINDOW_MS,
            protected_tokens: vec![
                "AuthToken".to_owned(),
                "AccessToken".to_owned(),
                "RefreshToken".to_owned(),
            ],
            shared_secret: None,
        }
    }
}

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
    #[arg(long = "user-center-mode", value_name = "MODE")]
    user_center_mode: Option<String>,
    #[arg(long = "user-center-app-api-base-url", value_name = "URL")]
    user_center_app_api_base_url: Option<String>,
    #[arg(long = "user-center-external-base-url", value_name = "URL")]
    user_center_external_base_url: Option<String>,
    #[arg(long = "user-center-provider-key", value_name = "VALUE")]
    user_center_provider_key: Option<String>,
    #[arg(long = "user-center-app-id", value_name = "VALUE")]
    user_center_app_id: Option<String>,
    #[arg(long = "user-center-local-api-base-path", value_name = "PATH")]
    user_center_local_api_base_path: Option<String>,
    #[arg(long = "user-center-sqlite-path", value_name = "PATH")]
    user_center_sqlite_path: Option<String>,
    #[arg(long = "user-center-database-url", value_name = "URL")]
    user_center_database_url: Option<String>,
    #[arg(long = "user-center-schema-name", value_name = "VALUE")]
    user_center_schema_name: Option<String>,
    #[arg(long = "user-center-table-prefix", value_name = "VALUE")]
    user_center_table_prefix: Option<String>,
    #[arg(long = "user-center-secret-id", value_name = "VALUE")]
    user_center_secret_id: Option<String>,
    #[arg(long = "user-center-shared-secret", value_name = "VALUE")]
    user_center_shared_secret: Option<String>,
    #[arg(long = "user-center-handshake-freshness-window-ms", value_name = "MS")]
    user_center_handshake_freshness_window_ms: Option<String>,
    #[arg(long = "user-center-authorization-header-name", value_name = "VALUE")]
    user_center_authorization_header_name: Option<String>,
    #[arg(long = "user-center-access-token-header-name", value_name = "VALUE")]
    user_center_access_token_header_name: Option<String>,
    #[arg(long = "user-center-refresh-token-header-name", value_name = "VALUE")]
    user_center_refresh_token_header_name: Option<String>,
    #[arg(long = "user-center-session-header-name", value_name = "VALUE")]
    user_center_session_header_name: Option<String>,
    #[arg(long = "user-center-authorization-scheme", value_name = "VALUE")]
    user_center_authorization_scheme: Option<String>,
    #[arg(
        long = "user-center-allow-authorization-fallback-to-access-token",
        value_name = "BOOL"
    )]
    user_center_allow_authorization_fallback_to_access_token: Option<String>,
    #[arg(
        long = "backup-output",
        value_name = "DIR",
        conflicts_with_all = ["restore_source", "support_bundle_output"]
    )]
    backup_output: Option<PathBuf>,
    #[arg(
        long = "restore-source",
        value_name = "DIR",
        conflicts_with_all = ["backup_output", "support_bundle_output"]
    )]
    restore_source: Option<PathBuf>,
    #[arg(
        long = "support-bundle-output",
        value_name = "DIR",
        conflicts_with_all = ["backup_output", "restore_source"]
    )]
    support_bundle_output: Option<PathBuf>,
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
    user_center: UserCenterServerContract,
    backup_output: Option<PathBuf>,
    restore_source: Option<PathBuf>,
    support_bundle_output: Option<PathBuf>,
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
        } else if self.support_bundle_output.is_some() {
            Some(RuntimeOperationKind::SupportBundle)
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = RouterProductServiceCli::parse();
    let (settings, runtime_context) =
        resolve_runtime_contextual_service_settings(&cli, env::vars())?;
    apply_user_center_env_overrides(&settings.user_center);
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

fn resolve_service_settings_from_env_values(
    cli: &RouterProductServiceCli,
    env_values: &HashMap<String, String>,
) -> anyhow::Result<ProductServiceSettings> {
    if (cli.backup_output.is_some()
        || cli.restore_source.is_some()
        || cli.support_bundle_output.is_some())
        && cli.runtime_home.is_none()
    {
        anyhow::bail!("runtime operations require --runtime-home");
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
        user_center: resolve_user_center_contract(cli, env_values)?,
        backup_output: cli.backup_output.clone(),
        restore_source: cli.restore_source.clone(),
        support_bundle_output: cli.support_bundle_output.clone(),
        force: cli.force,
        plan_format: cli.plan_format,
        dry_run: cli.dry_run,
    })
}

fn resolve_user_center_contract(
    cli: &RouterProductServiceCli,
    env_values: &HashMap<String, String>,
) -> anyhow::Result<UserCenterServerContract> {
    let requested_mode = resolve_string_option(
        cli.user_center_mode.as_deref(),
        env_values,
        SDKWORK_USER_CENTER_MODE,
    );
    let app_api_base_url = resolve_string_option(
        cli.user_center_app_api_base_url.as_deref(),
        env_values,
        SDKWORK_USER_CENTER_APP_API_BASE_URL,
    );
    let external_base_url = resolve_string_option(
        cli.user_center_external_base_url.as_deref(),
        env_values,
        SDKWORK_USER_CENTER_EXTERNAL_BASE_URL,
    );
    let mode = resolve_user_center_mode(
        requested_mode.as_deref(),
        app_api_base_url.as_deref(),
        external_base_url.as_deref(),
    )?;
    let provider_key = resolve_string_option(
        cli.user_center_provider_key.as_deref(),
        env_values,
        SDKWORK_USER_CENTER_PROVIDER_KEY,
    )
    .or_else(|| match mode {
        UserCenterMode::BuiltinLocal => Some(USER_CENTER_DEFAULT_PROVIDER_KEY.to_owned()),
        UserCenterMode::SdkworkCloudAppApi | UserCenterMode::ExternalUserCenter => None,
    })
    .unwrap_or_default();
    let secret_id = resolve_string_option(
        cli.user_center_secret_id.as_deref(),
        env_values,
        SDKWORK_USER_CENTER_SECRET_ID,
    );
    let shared_secret = resolve_string_option(
        cli.user_center_shared_secret.as_deref(),
        env_values,
        SDKWORK_USER_CENTER_SHARED_SECRET,
    );
    let contract = UserCenterServerContract {
        mode: mode.as_str().to_owned(),
        active_integration_kind: mode.as_str().to_owned(),
        app_id: resolve_string_option(
            cli.user_center_app_id.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_APP_ID,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_APP_ID.to_owned()),
        provider_key,
        authority_scope: "application".to_owned(),
        session_transport: "header".to_owned(),
        validation_strategy: "auth-access-token".to_owned(),
        local_api_base_path: resolve_string_option(
            cli.user_center_local_api_base_path.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_LOCAL_API_BASE_PATH.to_owned()),
        sqlite_path: resolve_string_option(
            cli.user_center_sqlite_path.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_SQLITE_PATH,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_SQLITE_PATH.to_owned()),
        database_url: resolve_string_option(
            cli.user_center_database_url.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_DATABASE_URL,
        ),
        schema_name: resolve_string_option(
            cli.user_center_schema_name.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_SCHEMA_NAME,
        ),
        table_prefix: resolve_string_option(
            cli.user_center_table_prefix.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_TABLE_PREFIX,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_TABLE_PREFIX.to_owned()),
        app_api_base_url,
        external_base_url,
        secret_id,
        shared_secret_configured: shared_secret.is_some(),
        auth_token_header_name: resolve_string_option(
            cli.user_center_authorization_header_name.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_AUTHORIZATION_HEADER_NAME,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_AUTHORIZATION_HEADER_NAME.to_owned()),
        access_token_header_name: resolve_string_option(
            cli.user_center_access_token_header_name.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_ACCESS_TOKEN_HEADER_NAME,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_ACCESS_TOKEN_HEADER_NAME.to_owned()),
        refresh_token_header_name: resolve_string_option(
            cli.user_center_refresh_token_header_name.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_REFRESH_TOKEN_HEADER_NAME,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_REFRESH_TOKEN_HEADER_NAME.to_owned()),
        session_header_name: resolve_string_option(
            cli.user_center_session_header_name.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_SESSION_HEADER_NAME,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_SESSION_HEADER_NAME.to_owned()),
        authorization_scheme: resolve_string_option(
            cli.user_center_authorization_scheme.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_AUTHORIZATION_SCHEME,
        )
        .unwrap_or_else(|| USER_CENTER_DEFAULT_AUTHORIZATION_SCHEME.to_owned()),
        allow_authorization_fallback_to_access_token: resolve_bool_option(
            cli.user_center_allow_authorization_fallback_to_access_token
                .as_deref(),
            env_values,
            SDKWORK_USER_CENTER_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN,
        )?
        .unwrap_or(USER_CENTER_DEFAULT_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN),
        handshake_mode: mode.handshake_mode().to_owned(),
        handshake_required: mode.handshake_required(),
        handshake_freshness_window_ms: resolve_u64_option(
            cli.user_center_handshake_freshness_window_ms.as_deref(),
            env_values,
            SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS,
        )?
        .unwrap_or(USER_CENTER_DEFAULT_HANDSHAKE_FRESHNESS_WINDOW_MS),
        protected_tokens: vec![
            "AuthToken".to_owned(),
            "AccessToken".to_owned(),
            "RefreshToken".to_owned(),
        ],
        shared_secret,
    };

    validate_user_center_contract(&contract)?;
    Ok(contract)
}

fn resolve_user_center_mode(
    requested_mode: Option<&str>,
    app_api_base_url: Option<&str>,
    external_base_url: Option<&str>,
) -> anyhow::Result<UserCenterMode> {
    if let Some(requested_mode) = requested_mode {
        if let Some(mode) = UserCenterMode::from_raw(requested_mode) {
            return Ok(mode);
        }

        anyhow::bail!(
            "{SDKWORK_USER_CENTER_MODE} must be one of: builtin-local, sdkwork-cloud-app-api, external-user-center"
        );
    }

    match (
        app_api_base_url
            .map(str::trim)
            .filter(|value| !value.is_empty()),
        external_base_url
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    ) {
        (Some(_), None) => Ok(UserCenterMode::SdkworkCloudAppApi),
        (None, Some(_)) => Ok(UserCenterMode::ExternalUserCenter),
        _ => Ok(UserCenterMode::BuiltinLocal),
    }
}

fn validate_user_center_contract(contract: &UserCenterServerContract) -> anyhow::Result<()> {
    let mode = UserCenterMode::from_raw(&contract.mode).unwrap_or_default();
    match mode {
        UserCenterMode::BuiltinLocal => Ok(()),
        UserCenterMode::SdkworkCloudAppApi => {
            require_user_center_value(
                contract.app_api_base_url.as_deref(),
                SDKWORK_USER_CENTER_APP_API_BASE_URL,
            )?;
            require_user_center_value(
                Some(contract.provider_key.as_str()),
                SDKWORK_USER_CENTER_PROVIDER_KEY,
            )?;
            require_user_center_value(
                contract.secret_id.as_deref(),
                SDKWORK_USER_CENTER_SECRET_ID,
            )?;
            require_user_center_value(
                contract.shared_secret.as_deref(),
                SDKWORK_USER_CENTER_SHARED_SECRET,
            )?;
            Ok(())
        }
        UserCenterMode::ExternalUserCenter => {
            require_user_center_value(
                contract.external_base_url.as_deref(),
                SDKWORK_USER_CENTER_EXTERNAL_BASE_URL,
            )?;
            require_user_center_value(
                Some(contract.provider_key.as_str()),
                SDKWORK_USER_CENTER_PROVIDER_KEY,
            )?;
            require_user_center_value(
                contract.secret_id.as_deref(),
                SDKWORK_USER_CENTER_SECRET_ID,
            )?;
            require_user_center_value(
                contract.shared_secret.as_deref(),
                SDKWORK_USER_CENTER_SHARED_SECRET,
            )?;
            Ok(())
        }
    }
}

fn require_user_center_value(value: Option<&str>, env_name: &str) -> anyhow::Result<()> {
    if value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        return Ok(());
    }

    anyhow::bail!("{env_name} is required for the selected user-center mode");
}

fn apply_user_center_env_overrides(user_center: &UserCenterServerContract) {
    env::set_var(SDKWORK_USER_CENTER_MODE, user_center.mode.as_str());
    env::set_var(
        SDKWORK_USER_CENTER_PROVIDER_KEY,
        user_center.provider_key.as_str(),
    );
    env::set_var(SDKWORK_USER_CENTER_APP_ID, user_center.app_id.as_str());
    env::set_var(
        SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH,
        user_center.local_api_base_path.as_str(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_SQLITE_PATH,
        user_center.sqlite_path.as_str(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_TABLE_PREFIX,
        user_center.table_prefix.as_str(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS,
        user_center.handshake_freshness_window_ms.to_string(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_AUTHORIZATION_HEADER_NAME,
        user_center.auth_token_header_name.as_str(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_ACCESS_TOKEN_HEADER_NAME,
        user_center.access_token_header_name.as_str(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_REFRESH_TOKEN_HEADER_NAME,
        user_center.refresh_token_header_name.as_str(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_SESSION_HEADER_NAME,
        user_center.session_header_name.as_str(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_AUTHORIZATION_SCHEME,
        user_center.authorization_scheme.as_str(),
    );
    env::set_var(
        SDKWORK_USER_CENTER_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN,
        if user_center.allow_authorization_fallback_to_access_token {
            "true"
        } else {
            "false"
        },
    );

    set_optional_env(
        SDKWORK_USER_CENTER_APP_API_BASE_URL,
        user_center.app_api_base_url.as_deref(),
    );
    set_optional_env(
        SDKWORK_USER_CENTER_EXTERNAL_BASE_URL,
        user_center.external_base_url.as_deref(),
    );
    set_optional_env(
        SDKWORK_USER_CENTER_DATABASE_URL,
        user_center.database_url.as_deref(),
    );
    set_optional_env(
        SDKWORK_USER_CENTER_SCHEMA_NAME,
        user_center.schema_name.as_deref(),
    );
    set_optional_env(
        SDKWORK_USER_CENTER_SECRET_ID,
        user_center.secret_id.as_deref(),
    );
    set_optional_env(
        SDKWORK_USER_CENTER_SHARED_SECRET,
        user_center.shared_secret.as_deref(),
    );
}

fn set_optional_env(env_key: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        env::set_var(env_key, value);
    }
}

fn resolve_runtime_contextual_service_settings<I, K, V>(
    cli: &RouterProductServiceCli,
    env_pairs: I,
) -> anyhow::Result<(ProductServiceSettings, Option<InstalledRuntimeContext>)>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let mut env_values = collect_env_values(env_pairs);
    let mut settings = resolve_service_settings_from_env_values(cli, &env_values)?;
    let runtime_context = resolve_runtime_context(settings.runtime_home.as_deref())?;
    apply_runtime_context_defaults(&mut settings, runtime_context.as_ref());

    let mut merged_router_env = false;
    for (key, value) in load_router_env_default_pairs(&settings)? {
        if !env_values.contains_key(&key) {
            env_values.insert(key.clone(), value.clone());
            merged_router_env = true;
        }
        if env::var_os(&key).is_none() {
            env::set_var(&key, &value);
        }
    }

    if merged_router_env {
        settings = resolve_service_settings_from_env_values(cli, &env_values)?;
        apply_runtime_context_defaults(&mut settings, runtime_context.as_ref());
    }

    Ok((settings, runtime_context))
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
    validate_installed_runtime_manifest(&runtime_home, &manifest_path, &manifest)?;

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

fn load_router_env_default_pairs(
    settings: &ProductServiceSettings,
) -> anyhow::Result<Vec<(String, String)>> {
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
        return Ok(Vec::new());
    };

    let env_file = config_root.join("router.env");
    if !env_file.is_file() {
        return Ok(Vec::new());
    }

    Ok(read_env_file_pairs(&env_file)?
        .into_iter()
        .filter(|(key, _)| key != SDKWORK_CONFIG_DIR && key != SDKWORK_CONFIG_FILE)
        .collect())
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
    let kind = resolve_database_kind(database_url);
    if kind == "postgresql" {
        return DatabaseOperationContract {
            kind,
            strategy: match operation {
                RuntimeOperationKind::Backup => "pg_dump-custom".to_owned(),
                RuntimeOperationKind::Restore => "pg_restore-custom".to_owned(),
                RuntimeOperationKind::SupportBundle => "metadata-redacted".to_owned(),
            },
            dump_file: if matches!(operation, RuntimeOperationKind::SupportBundle) {
                None
            } else {
                Some("database/postgresql.dump".to_owned())
            },
            supported: !matches!(operation, RuntimeOperationKind::SupportBundle),
        };
    }

    if kind == "sqlite" {
        return DatabaseOperationContract {
            kind,
            strategy: match operation {
                RuntimeOperationKind::SupportBundle => "metadata-redacted".to_owned(),
                _ => "filesystem-snapshot".to_owned(),
            },
            dump_file: None,
            supported: !matches!(operation, RuntimeOperationKind::SupportBundle),
        };
    }

    DatabaseOperationContract {
        kind,
        strategy: match operation {
            RuntimeOperationKind::Backup => "manual-provider-backup".to_owned(),
            RuntimeOperationKind::Restore => "manual-provider-restore".to_owned(),
            RuntimeOperationKind::SupportBundle => "metadata-redacted".to_owned(),
        },
        dump_file: None,
        supported: false,
    }
}

fn resolve_database_kind(database_url: &str) -> String {
    let normalized = database_url.trim().to_ascii_lowercase();
    if normalized.starts_with("postgres://") || normalized.starts_with("postgresql://") {
        return "postgresql".to_owned();
    }
    if normalized.starts_with("sqlite:") {
        return "sqlite".to_owned();
    }

    normalized
        .split(':')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_owned()
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
        RuntimeOperationKind::SupportBundle => {
            execute_support_bundle_operation(settings, config, runtime_context)
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
    ensure_output_path_is_safe(
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
        format_version: 2,
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
        bundle: BackupBundleContentsContract {
            control_manifest_file: "control/release-manifest.json".to_owned(),
            config_snapshot_root: "config".to_owned(),
            mutable_data_snapshot_root: "data".to_owned(),
        },
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
    let control_manifest_path = resolve_backup_bundle_member_path(
        &restore_source,
        &bundle_manifest.bundle.control_manifest_file,
        "bundle.controlManifestFile",
    )?;
    if !control_manifest_path.is_file() {
        anyhow::bail!(
            "restore source {} is missing {}",
            restore_source.display(),
            control_manifest_path.display()
        );
    }
    let config_snapshot_root = resolve_backup_bundle_member_path(
        &restore_source,
        &bundle_manifest.bundle.config_snapshot_root,
        "bundle.configSnapshotRoot",
    )?;
    if !config_snapshot_root.is_dir() {
        anyhow::bail!(
            "restore source {} is missing config snapshot {}",
            restore_source.display(),
            config_snapshot_root.display()
        );
    }
    let data_snapshot_root = resolve_backup_bundle_member_path(
        &restore_source,
        &bundle_manifest.bundle.mutable_data_snapshot_root,
        "bundle.mutableDataSnapshotRoot",
    )?;
    if !data_snapshot_root.is_dir() {
        anyhow::bail!(
            "restore source {} is missing mutable data snapshot {}",
            restore_source.display(),
            data_snapshot_root.display()
        );
    }

    let config_root = required_manifest_path(&runtime_context.manifest.config_root, "configRoot")?;
    let mutable_data_root = required_manifest_path(
        &runtime_context.manifest.mutable_data_root,
        "mutableDataRoot",
    )?;
    let run_root = required_manifest_path(&runtime_context.manifest.run_root, "runRoot")?;
    ensure_runtime_stopped(&run_root)?;

    replace_dir_from_snapshot(&config_snapshot_root, &config_root, true)?;
    replace_dir_from_snapshot(&data_snapshot_root, &mutable_data_root, true)?;

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

fn execute_support_bundle_operation(
    settings: &ProductServiceSettings,
    config: &StandaloneConfig,
    runtime_context: &InstalledRuntimeContext,
) -> anyhow::Result<()> {
    let support_bundle_output = settings
        .support_bundle_output
        .as_deref()
        .context("support-bundle requires --support-bundle-output")?;
    let support_bundle_output = absolutize_path(support_bundle_output)?;
    let config_root = required_manifest_path(&runtime_context.manifest.config_root, "configRoot")?;
    let log_root = required_manifest_path(&runtime_context.manifest.log_root, "logRoot")?;
    let run_root = required_manifest_path(&runtime_context.manifest.run_root, "runRoot")?;

    ensure_output_path_is_safe(
        &support_bundle_output,
        [
            config_root.as_path(),
            log_root.as_path(),
            run_root.as_path(),
        ],
    )?;
    prepare_output_directory(&support_bundle_output, settings.force)?;

    let path_contract = standard_support_bundle_path_contract();
    let control_manifest_path = support_bundle_output.join(&path_contract.control_manifest_file);
    fs::create_dir_all(
        control_manifest_path
            .parent()
            .context("support bundle control manifest path is missing parent directory")?,
    )?;
    copy_file(&runtime_context.manifest_path, &control_manifest_path)?;

    write_support_bundle_directory_snapshot(
        &config_root,
        &support_bundle_output.join(&path_contract.config_snapshot_root),
        SupportBundleSnapshotKind::Config,
    )?;
    write_support_bundle_directory_snapshot(
        &log_root,
        &support_bundle_output.join(&path_contract.logs_snapshot_root),
        SupportBundleSnapshotKind::Logs,
    )?;
    write_support_bundle_directory_snapshot(
        &run_root,
        &support_bundle_output.join(&path_contract.runtime_snapshot_root),
        SupportBundleSnapshotKind::RuntimeState,
    )?;
    write_json_file(
        &support_bundle_output.join(&path_contract.process_state_file),
        &collect_support_bundle_process_state(&run_root)?,
    )?;

    let bundle_manifest = SupportBundleManifest {
        format_version: 2,
        created_at: chrono_like_timestamp()?,
        runtime_home: runtime_context.runtime_home.clone(),
        install_mode: runtime_context.manifest.install_mode.clone(),
        release_version: runtime_context.manifest.release_version.clone(),
        product_root: runtime_context.manifest.product_root.clone(),
        config_root,
        config_file: runtime_context.manifest.config_file.clone(),
        log_root,
        run_root,
        database: SupportBundleDatabaseContract {
            kind: resolve_database_kind(&config.database_url),
            strategy: "metadata-redacted".to_owned(),
        },
        bundle: SupportBundleContentsContract {
            includes_redacted_config: true,
            includes_logs: true,
            includes_runtime_state: true,
        },
        paths: path_contract,
    };
    write_json_file(
        &support_bundle_output.join("support-bundle-manifest.json"),
        &bundle_manifest,
    )?;

    println!(
        "router-product-service support bundle completed at {}",
        support_bundle_output.display()
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupportBundleSnapshotKind {
    Config,
    Logs,
    RuntimeState,
}

impl SupportBundleSnapshotKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Logs => "logs",
            Self::RuntimeState => "runtime-state",
        }
    }

    fn copies_text_files(self) -> bool {
        !matches!(self, Self::RuntimeState)
    }
}

fn write_support_bundle_directory_snapshot(
    source_root: &Path,
    destination_root: &Path,
    snapshot_kind: SupportBundleSnapshotKind,
) -> anyhow::Result<()> {
    fs::create_dir_all(destination_root)?;

    let mut inventory = SupportBundleDirectoryInventory {
        snapshot_kind: snapshot_kind.as_str().to_owned(),
        source_root: source_root.to_path_buf(),
        exists: source_root.is_dir(),
        entries: Vec::new(),
    };

    if source_root.is_dir() {
        collect_support_bundle_directory_entries(
            source_root,
            source_root,
            destination_root,
            snapshot_kind,
            &mut inventory.entries,
        )?;
    }

    write_json_file(&destination_root.join("inventory.json"), &inventory)
}

fn collect_support_bundle_directory_entries(
    source_root: &Path,
    current_source: &Path,
    destination_root: &Path,
    snapshot_kind: SupportBundleSnapshotKind,
    entries: &mut Vec<SupportBundleDirectoryEntry>,
) -> anyhow::Result<()> {
    let mut directory_entries = fs::read_dir(current_source)?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("failed to enumerate {}", current_source.display()))?;
    directory_entries.sort_by(|left, right| left.path().cmp(&right.path()));

    for entry in directory_entries {
        let source_path = entry.path();
        let relative_path = source_path
            .strip_prefix(source_root)
            .expect("snapshot entry must stay inside source root")
            .to_path_buf();
        let relative_path_string = path_to_portable_string(&relative_path);
        let destination_path = destination_root.join(&relative_path);
        let metadata = entry
            .metadata()
            .with_context(|| format!("failed to read metadata for {}", source_path.display()))?;
        let modified_unix_seconds = metadata
            .modified()
            .ok()
            .and_then(system_time_to_unix_seconds);
        let file_type = entry.file_type().with_context(|| {
            format!("failed to resolve entry type for {}", source_path.display())
        })?;

        if file_type.is_dir() {
            fs::create_dir_all(&destination_path)?;
            entries.push(SupportBundleDirectoryEntry {
                relative_path: relative_path_string,
                entry_kind: "directory".to_owned(),
                size_bytes: None,
                modified_unix_seconds,
                redacted_copy: false,
                omitted_reason: None,
            });
            collect_support_bundle_directory_entries(
                source_root,
                &source_path,
                destination_root,
                snapshot_kind,
                entries,
            )?;
            continue;
        }

        if file_type.is_file() {
            let (redacted_copy, omitted_reason) = maybe_copy_support_bundle_file(
                &source_path,
                &destination_path,
                &metadata,
                snapshot_kind,
            )?;
            entries.push(SupportBundleDirectoryEntry {
                relative_path: relative_path_string,
                entry_kind: "file".to_owned(),
                size_bytes: Some(metadata.len()),
                modified_unix_seconds,
                redacted_copy,
                omitted_reason,
            });
            continue;
        }

        entries.push(SupportBundleDirectoryEntry {
            relative_path: relative_path_string,
            entry_kind: "other".to_owned(),
            size_bytes: None,
            modified_unix_seconds,
            redacted_copy: false,
            omitted_reason: Some("unsupported-non-file-entry".to_owned()),
        });
    }

    Ok(())
}

fn maybe_copy_support_bundle_file(
    source_path: &Path,
    destination_path: &Path,
    metadata: &fs::Metadata,
    snapshot_kind: SupportBundleSnapshotKind,
) -> anyhow::Result<(bool, Option<String>)> {
    const MAX_TEXT_BYTES: u64 = 2 * 1024 * 1024;

    if !snapshot_kind.copies_text_files() {
        return Ok((false, Some("inventory-only".to_owned())));
    }

    if snapshot_kind == SupportBundleSnapshotKind::Config
        && should_omit_support_bundle_sensitive_config_file(source_path)
    {
        return Ok((false, Some("sensitive-file-omitted".to_owned())));
    }

    if metadata.len() > MAX_TEXT_BYTES {
        return Ok((false, Some("file-too-large".to_owned())));
    }

    let contents = match fs::read_to_string(source_path) {
        Ok(contents) => contents,
        Err(_) => return Ok((false, Some("non-utf8-or-binary".to_owned()))),
    };

    let redacted_contents = redact_support_bundle_text(&contents);
    let parent = destination_path
        .parent()
        .context("support-bundle destination file is missing parent directory")?;
    fs::create_dir_all(parent)?;
    fs::write(destination_path, redacted_contents)
        .with_context(|| format!("failed to write {}", destination_path.display()))?;
    Ok((true, None))
}

fn should_omit_support_bundle_sensitive_config_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    matches!(
        extension.as_str(),
        "pem" | "key" | "p12" | "pfx" | "der" | "crt" | "cer" | "db" | "sqlite" | "sqlite3"
    ) || file_name.contains("secret")
        || file_name.contains("credential")
        || file_name.contains("keystore")
        || file_name.contains("keyring")
}

fn redact_support_bundle_text(contents: &str) -> String {
    let mut lines = contents
        .lines()
        .map(redact_support_bundle_line)
        .collect::<Vec<_>>()
        .join("\n");
    if contents.ends_with('\n') {
        lines.push('\n');
    }
    lines
}

fn redact_support_bundle_line(line: &str) -> String {
    if let Some(redacted) = redact_support_bundle_header_line(line) {
        return redacted;
    }
    if let Some(redacted) = redact_support_bundle_key_value_line(line) {
        return redacted;
    }
    redact_connection_url_in_line(line)
}

fn redact_support_bundle_header_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let normalized = trimmed.to_ascii_lowercase();
    for header_name in [
        "authorization:",
        "proxy-authorization:",
        "x-api-key:",
        "api-key:",
        "set-cookie:",
    ] {
        if normalized.starts_with(header_name) {
            let indent = &line[..line.len() - trimmed.len()];
            let header = &trimmed[..header_name.len()];
            return Some(format!("{indent}{header} ***REDACTED***"));
        }
    }

    None
}

fn redact_support_bundle_key_value_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
        return None;
    }

    let indent = &line[..line.len() - trimmed.len()];

    if let Some(separator_index) = trimmed.find('=') {
        let key = trimmed[..separator_index].trim();
        if support_bundle_key_is_sensitive(key) {
            let prefix = &trimmed[..separator_index + 1];
            return Some(format!("{indent}{prefix}***REDACTED***"));
        }
    }

    if let Some(separator_index) = trimmed.find(':') {
        let raw_key = trimmed[..separator_index]
            .trim()
            .trim_start_matches('-')
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        if support_bundle_key_is_sensitive(raw_key) {
            let prefix = &trimmed[..separator_index + 1];
            let suffix = if trimmed.trim_end().ends_with(',') {
                ","
            } else {
                ""
            };
            return Some(format!("{indent}{prefix} \"***REDACTED***\"{suffix}"));
        }
    }

    None
}

fn support_bundle_key_is_sensitive(key: &str) -> bool {
    let normalized = key
        .trim()
        .to_ascii_lowercase()
        .replace(['-', '.', ' '], "_");

    normalized == "database_url"
        || normalized == "cache_url"
        || normalized.contains("password")
        || normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("api_key")
        || normalized.ends_with("_key")
        || normalized.contains("master_key")
}

fn redact_connection_url_in_line(line: &str) -> String {
    for scheme in [
        "postgresql://",
        "postgres://",
        "mysql://",
        "redis://",
        "amqp://",
    ] {
        let normalized = line.to_ascii_lowercase();
        let Some(start_index) = normalized.find(scheme) else {
            continue;
        };

        let suffix = &line[start_index..];
        let end_offset = suffix
            .find(|character: char| {
                character.is_whitespace() || matches!(character, '"' | '\'' | ',' | ')' | ']')
            })
            .unwrap_or(suffix.len());
        let candidate = &line[start_index..start_index + end_offset];
        let Some(redacted_candidate) = redact_connection_url_candidate(candidate) else {
            continue;
        };

        return format!(
            "{}{}{}",
            &line[..start_index],
            redacted_candidate,
            &line[start_index + end_offset..]
        );
    }

    line.to_owned()
}

fn redact_connection_url_candidate(candidate: &str) -> Option<String> {
    let scheme_index = candidate.find("://")?;
    let after_scheme_index = scheme_index + 3;
    let remainder = &candidate[after_scheme_index..];
    let at_index = remainder.find('@')?;

    Some(format!(
        "{}***REDACTED***@{}",
        &candidate[..after_scheme_index],
        &remainder[at_index + 1..]
    ))
}

fn collect_support_bundle_process_state(
    run_root: &Path,
) -> anyhow::Result<SupportBundleProcessState> {
    let pid_file = run_root.join("router-product-service.pid");
    let state_file = run_root.join("router-product-service.state");

    let mut pid = None;
    let mut running = false;
    let mut process_fingerprint = None;

    if pid_file.is_file() {
        let pid_contents = fs::read_to_string(&pid_file)
            .with_context(|| format!("failed to read {}", pid_file.display()))?;
        if let Ok(parsed_pid) = pid_contents.trim().parse::<u32>() {
            pid = Some(parsed_pid);
            running = process_is_running(parsed_pid)?;
            if running {
                process_fingerprint = process_start_fingerprint(parsed_pid)?;
            }
        }
    }

    Ok(SupportBundleProcessState {
        pid_file: "router-product-service.pid".to_owned(),
        state_file: state_file
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("router-product-service.state")
            .to_owned(),
        pid,
        running,
        process_fingerprint,
    })
}

fn standard_support_bundle_path_contract() -> SupportBundlePathContract {
    SupportBundlePathContract {
        control_manifest_file: "control/release-manifest.json".to_owned(),
        config_snapshot_root: "config".to_owned(),
        config_inventory_file: "config/inventory.json".to_owned(),
        logs_snapshot_root: "logs".to_owned(),
        logs_inventory_file: "logs/inventory.json".to_owned(),
        runtime_snapshot_root: "runtime".to_owned(),
        runtime_inventory_file: "runtime/inventory.json".to_owned(),
        process_state_file: "runtime/process-state.json".to_owned(),
    }
}

fn process_start_fingerprint(pid: u32) -> anyhow::Result<Option<String>> {
    let output = if cfg!(windows) {
        Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!(
                    "(Get-Process -Id {pid} -ErrorAction SilentlyContinue).StartTime.ToUniversalTime().ToString('o')"
                ),
            ])
            .output()
            .context("failed to probe process fingerprint via powershell.exe")?
    } else {
        Command::new("ps")
            .args(["-o", "lstart=", "-p", &pid.to_string()])
            .output()
            .context("failed to probe process fingerprint via ps")?
    };

    if !output.status.success() {
        return Ok(None);
    }

    let fingerprint = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if fingerprint.is_empty() {
        Ok(None)
    } else {
        Ok(Some(fingerprint))
    }
}

fn system_time_to_unix_seconds(value: std::time::SystemTime) -> Option<u64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn path_to_portable_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn required_manifest_path(value: &Option<PathBuf>, field_name: &str) -> anyhow::Result<PathBuf> {
    value
        .clone()
        .with_context(|| format!("installed runtime manifest is missing {field_name}"))
}

fn validate_installed_runtime_manifest(
    runtime_home: &Path,
    manifest_path: &Path,
    manifest: &InstalledRuntimeManifest,
) -> anyhow::Result<()> {
    let missing_generated_metadata = collect_missing_installed_runtime_generated_metadata(manifest);
    if !missing_generated_metadata.is_empty() {
        anyhow::bail!(
            "installed runtime manifest is missing required generated metadata fields {}: {}",
            missing_generated_metadata.join(", "),
            manifest_path.display()
        );
    }

    let runtime = manifest
        .runtime
        .as_deref()
        .map(str::trim)
        .context("installed runtime manifest is missing runtime")?;
    if runtime != INSTALLED_RUNTIME_NAME {
        anyhow::bail!(
            "installed runtime manifest runtime must equal {}: {}",
            INSTALLED_RUNTIME_NAME,
            manifest_path.display()
        );
    }

    if manifest.layout_version != Some(INSTALLED_RUNTIME_LAYOUT_VERSION) {
        anyhow::bail!(
            "installed runtime manifest layoutVersion must equal {}: {}",
            INSTALLED_RUNTIME_LAYOUT_VERSION,
            manifest_path.display()
        );
    }

    let release_version = required_manifest_string(
        manifest.release_version.as_deref(),
        "releaseVersion",
        manifest_path,
    )?;
    let _target = required_manifest_string(manifest.target.as_deref(), "target", manifest_path)?;
    let _installed_at = required_manifest_string(
        manifest.installed_at.as_deref(),
        "installedAt",
        manifest_path,
    )?;
    let installed_binaries = required_manifest_string_list(
        manifest.installed_binaries.as_ref(),
        "installedBinaries",
        manifest_path,
    )?;
    if !installed_binaries
        .iter()
        .any(|binary_name| binary_name == ROUTER_PRODUCT_SERVICE_BINARY_STEM)
    {
        anyhow::bail!(
            "installed runtime manifest installedBinaries must include {}: {}",
            ROUTER_PRODUCT_SERVICE_BINARY_STEM,
            manifest_path.display()
        );
    }

    let product_root = resolve_manifest_contract_path(
        manifest.product_root.as_ref(),
        "productRoot",
        manifest_path,
        runtime_home.parent().unwrap_or(runtime_home),
    )?;
    let control_root = resolve_manifest_contract_path(
        manifest.control_root.as_ref(),
        "controlRoot",
        manifest_path,
        &product_root,
    )?;
    let releases_root = resolve_manifest_contract_path(
        manifest.releases_root.as_ref(),
        "releasesRoot",
        manifest_path,
        &product_root,
    )?;
    let release_root = resolve_manifest_contract_path(
        manifest.release_root.as_ref(),
        "releaseRoot",
        manifest_path,
        &product_root,
    )?;
    let bootstrap_data_root = resolve_manifest_contract_path(
        manifest.bootstrap_data_root.as_ref(),
        "bootstrapDataRoot",
        manifest_path,
        &product_root,
    )?;
    let deployment_asset_root = resolve_manifest_contract_path(
        manifest.deployment_asset_root.as_ref(),
        "deploymentAssetRoot",
        manifest_path,
        &product_root,
    )?;
    let release_payload_manifest = resolve_manifest_contract_path(
        manifest.release_payload_manifest.as_ref(),
        "releasePayloadManifest",
        manifest_path,
        &product_root,
    )?;
    let release_payload_readme_file = resolve_manifest_contract_path(
        manifest.release_payload_readme_file.as_ref(),
        "releasePayloadReadmeFile",
        manifest_path,
        &product_root,
    )?;
    let admin_site_dist_dir = resolve_manifest_contract_path(
        manifest.admin_site_dist_dir.as_ref(),
        "adminSiteDistDir",
        manifest_path,
        &product_root,
    )?;
    let portal_site_dist_dir = resolve_manifest_contract_path(
        manifest.portal_site_dist_dir.as_ref(),
        "portalSiteDistDir",
        manifest_path,
        &product_root,
    )?;
    let router_binary = resolve_manifest_contract_path(
        manifest.router_binary.as_ref(),
        "routerBinary",
        manifest_path,
        &product_root,
    )?;

    for required_path in [
        &product_root,
        &control_root,
        &releases_root,
        &release_root,
        &bootstrap_data_root,
        &deployment_asset_root,
        &release_payload_manifest,
        &release_payload_readme_file,
        &admin_site_dist_dir,
        &portal_site_dist_dir,
        &router_binary,
    ] {
        if !required_path.exists() {
            anyhow::bail!(
                "installed runtime manifest declares a missing path: {}",
                required_path.display()
            );
        }
    }

    let manifest_control_root = manifest_path
        .parent()
        .context("installed runtime manifest path is missing parent directory")?;
    if !same_existing_path(&control_root, manifest_control_root)? {
        anyhow::bail!(
            "installed runtime manifest controlRoot does not match the manifest location: {}",
            manifest_path.display()
        );
    }
    if !same_existing_path(&control_root, runtime_home)? {
        anyhow::bail!(
            "installed runtime manifest controlRoot does not match the resolved runtime home: {}",
            manifest_path.display()
        );
    }

    assert_manifest_contract_path_within_root(
        manifest_path,
        "controlRoot",
        &control_root,
        &product_root,
        "current",
    )?;
    assert_manifest_contract_path_within_root(
        manifest_path,
        "releasesRoot",
        &releases_root,
        &product_root,
        "releases",
    )?;
    assert_manifest_contract_path_within_root(
        manifest_path,
        "releaseRoot",
        &release_root,
        &releases_root,
        &release_version,
    )?;
    assert_manifest_contract_path_within_root(
        manifest_path,
        "bootstrapDataRoot",
        &bootstrap_data_root,
        &release_root,
        "data",
    )?;
    assert_manifest_contract_path_within_root(
        manifest_path,
        "deploymentAssetRoot",
        &deployment_asset_root,
        &release_root,
        "deploy",
    )?;
    assert_manifest_contract_path_within_root(
        manifest_path,
        "releasePayloadManifest",
        &release_payload_manifest,
        &release_root,
        "release-manifest.json",
    )?;
    assert_manifest_contract_path_within_root(
        manifest_path,
        "releasePayloadReadmeFile",
        &release_payload_readme_file,
        &release_root,
        "README.txt",
    )?;
    assert_manifest_contract_path_within_root(
        manifest_path,
        "adminSiteDistDir",
        &admin_site_dist_dir,
        &release_root,
        "sites/admin/dist",
    )?;
    assert_manifest_contract_path_within_root(
        manifest_path,
        "portalSiteDistDir",
        &portal_site_dist_dir,
        &release_root,
        "sites/portal/dist",
    )?;

    let router_binary_relative_path =
        relative_existing_path_within_root(&router_binary, &release_root)?;
    let router_binary_matches_contract = match router_binary_relative_path.as_deref() {
        Some("bin/router-product-service") => true,
        Some(relative_path) if relative_path.starts_with("bin/router-product-service.") => {
            relative_path["bin/router-product-service.".len()..]
                .chars()
                .all(|character| {
                    character.is_ascii_alphanumeric() || character == '-' || character == '_'
                })
        }
        _ => false,
    };
    if !router_binary_matches_contract {
        anyhow::bail!(
            "installed runtime manifest routerBinary must resolve within the active release payload layout under bin/router-product-service*: {}",
            manifest_path.display()
        );
    }

    Ok(())
}

fn collect_missing_installed_runtime_generated_metadata(
    manifest: &InstalledRuntimeManifest,
) -> Vec<&'static str> {
    let mut missing_fields = Vec::new();

    if manifest
        .runtime
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        missing_fields.push("runtime");
    }
    if manifest.layout_version.is_none() {
        missing_fields.push("layoutVersion");
    }
    if manifest
        .release_version
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        missing_fields.push("releaseVersion");
    }
    if manifest
        .target
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        missing_fields.push("target");
    }
    let has_installed_binaries = manifest
        .installed_binaries
        .as_ref()
        .map(|values| values.iter().any(|value| !value.trim().is_empty()))
        .unwrap_or(false);
    if !has_installed_binaries {
        missing_fields.push("installedBinaries");
    }
    if manifest
        .installed_at
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        missing_fields.push("installedAt");
    }

    missing_fields
}

fn required_manifest_string(
    value: Option<&str>,
    field_name: &str,
    manifest_path: &Path,
) -> anyhow::Result<String> {
    let normalized_value = value.map(str::trim).unwrap_or_default();
    if normalized_value.is_empty() {
        anyhow::bail!(
            "installed runtime manifest is missing {}: {}",
            field_name,
            manifest_path.display()
        );
    }

    Ok(normalized_value.to_owned())
}

fn required_manifest_string_list(
    value: Option<&Vec<String>>,
    field_name: &str,
    manifest_path: &Path,
) -> anyhow::Result<Vec<String>> {
    let Some(values) = value else {
        anyhow::bail!(
            "installed runtime manifest is missing {}: {}",
            field_name,
            manifest_path.display()
        );
    };

    let normalized_values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if normalized_values.is_empty() {
        anyhow::bail!(
            "installed runtime manifest is missing {}: {}",
            field_name,
            manifest_path.display()
        );
    }

    Ok(normalized_values)
}

fn resolve_manifest_contract_path(
    value: Option<&PathBuf>,
    field_name: &str,
    manifest_path: &Path,
    base_root: &Path,
) -> anyhow::Result<PathBuf> {
    let Some(candidate) = value else {
        anyhow::bail!(
            "installed runtime manifest is missing {}: {}",
            field_name,
            manifest_path.display()
        );
    };
    if candidate.as_os_str().is_empty() {
        anyhow::bail!(
            "installed runtime manifest is missing {}: {}",
            field_name,
            manifest_path.display()
        );
    }

    if candidate.is_absolute() {
        Ok(candidate.clone())
    } else {
        Ok(base_root.join(candidate))
    }
}

fn assert_manifest_contract_path_within_root(
    manifest_path: &Path,
    field_name: &str,
    field_path: &Path,
    root_path: &Path,
    expected_relative_path: &str,
) -> anyhow::Result<()> {
    let actual_relative_path = relative_existing_path_within_root(field_path, root_path)?;
    if actual_relative_path.as_deref() != Some(expected_relative_path) {
        anyhow::bail!(
            "installed runtime manifest {} must resolve within the active release payload layout at {}: {}",
            field_name,
            expected_relative_path,
            manifest_path.display()
        );
    }

    Ok(())
}

fn relative_existing_path_within_root(
    target_path: &Path,
    root_path: &Path,
) -> anyhow::Result<Option<String>> {
    let canonical_target_path = canonicalize_existing_path(target_path)?;
    let canonical_root_path = canonicalize_existing_path(root_path)?;
    let Ok(relative_path) = canonical_target_path.strip_prefix(&canonical_root_path) else {
        return Ok(None);
    };

    Ok(Some(normalize_relative_contract_path(relative_path)))
}

fn canonicalize_existing_path(path: &Path) -> anyhow::Result<PathBuf> {
    fs::canonicalize(path).with_context(|| format!("failed to canonicalize {}", path.display()))
}

fn same_existing_path(left_path: &Path, right_path: &Path) -> anyhow::Result<bool> {
    Ok(canonicalize_existing_path(left_path)? == canonicalize_existing_path(right_path)?)
}

fn normalize_relative_contract_path(path: &Path) -> String {
    let portable = path_to_portable_string(path);
    let normalized = portable.trim_start_matches("./").trim_matches('/');
    if normalized.is_empty() {
        ".".to_owned()
    } else {
        normalized.to_owned()
    }
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

fn ensure_output_path_is_safe<'a>(
    output_dir: &Path,
    protected_roots: impl IntoIterator<Item = &'a Path>,
) -> anyhow::Result<()> {
    for protected_root in protected_roots {
        if output_dir.starts_with(protected_root) {
            anyhow::bail!(
                "output directory {} must be outside {}",
                output_dir.display(),
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
                "output directory {} already exists; rerun with --force to replace it",
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
    let manifest: BackupBundleManifest = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    if manifest.format_version != 2 {
        anyhow::bail!(
            "backup bundle manifest {} has unsupported formatVersion {}",
            manifest_path.display(),
            manifest.format_version
        );
    }

    Ok(manifest)
}

fn resolve_backup_bundle_member_path(
    source_root: &Path,
    relative_path: &str,
    field_name: &str,
) -> anyhow::Result<PathBuf> {
    if relative_path.trim().is_empty() {
        anyhow::bail!("backup bundle manifest is missing {}", field_name);
    }

    let candidate = Path::new(relative_path);
    if candidate.is_absolute() {
        anyhow::bail!(
            "backup bundle manifest {} must be relative to the bundle root",
            field_name
        );
    }

    let mut normalized_relative_path = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(segment) => normalized_relative_path.push(segment),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!(
                    "backup bundle manifest {} must stay within the bundle root",
                    field_name
                );
            }
        }
    }

    if normalized_relative_path.as_os_str().is_empty() {
        anyhow::bail!("backup bundle manifest is missing {}", field_name);
    }

    Ok(source_root.join(normalized_relative_path))
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
    SupportBundle,
}

impl RuntimeOperationKind {
    fn dry_run_mode(self) -> &'static str {
        match self {
            Self::Backup => "backup-dry-run",
            Self::Restore => "restore-dry-run",
            Self::SupportBundle => "support-bundle-dry-run",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Backup => "backup",
            Self::Restore => "restore",
            Self::SupportBundle => "support-bundle",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct InstalledRuntimeManifest {
    runtime: Option<String>,
    layout_version: Option<u32>,
    install_mode: Option<String>,
    product_root: Option<PathBuf>,
    control_root: Option<PathBuf>,
    release_version: Option<String>,
    releases_root: Option<PathBuf>,
    release_root: Option<PathBuf>,
    target: Option<String>,
    installed_binaries: Option<Vec<String>>,
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
    installed_at: Option<String>,
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
    bundle: BackupBundleContentsContract,
    database: BackupDatabaseContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupBundleContentsContract {
    control_manifest_file: String,
    config_snapshot_root: String,
    mutable_data_snapshot_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupDatabaseContract {
    kind: String,
    strategy: String,
    dump_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SupportBundleManifest {
    format_version: u32,
    created_at: String,
    runtime_home: PathBuf,
    install_mode: Option<String>,
    release_version: Option<String>,
    product_root: Option<PathBuf>,
    config_root: PathBuf,
    config_file: Option<PathBuf>,
    log_root: PathBuf,
    run_root: PathBuf,
    database: SupportBundleDatabaseContract,
    bundle: SupportBundleContentsContract,
    paths: SupportBundlePathContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SupportBundleDatabaseContract {
    kind: String,
    strategy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SupportBundleContentsContract {
    includes_redacted_config: bool,
    includes_logs: bool,
    includes_runtime_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SupportBundlePathContract {
    control_manifest_file: String,
    config_snapshot_root: String,
    config_inventory_file: String,
    logs_snapshot_root: String,
    logs_inventory_file: String,
    runtime_snapshot_root: String,
    runtime_inventory_file: String,
    process_state_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SupportBundleDirectoryInventory {
    snapshot_kind: String,
    source_root: PathBuf,
    exists: bool,
    entries: Vec<SupportBundleDirectoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SupportBundleDirectoryEntry {
    relative_path: String,
    entry_kind: String,
    size_bytes: Option<u64>,
    modified_unix_seconds: Option<u64>,
    redacted_copy: bool,
    omitted_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SupportBundleProcessState {
    pid_file: String,
    state_file: String,
    pid: Option<u32>,
    running: bool,
    process_fingerprint: Option<String>,
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

fn resolve_bool_option(
    cli_value: Option<&str>,
    env_values: &HashMap<String, String>,
    env_key: &str,
) -> anyhow::Result<Option<bool>> {
    let value = resolve_string_option(cli_value, env_values, env_key);
    match value.as_deref().map(|value| value.to_ascii_lowercase()) {
        None => Ok(None),
        Some(normalized) if matches!(normalized.as_str(), "1" | "true" | "yes" | "on") => {
            Ok(Some(true))
        }
        Some(normalized) if matches!(normalized.as_str(), "0" | "false" | "no" | "off") => {
            Ok(Some(false))
        }
        Some(_) => anyhow::bail!("{env_key} must be a boolean value"),
    }
}

fn resolve_u64_option(
    cli_value: Option<&str>,
    env_values: &HashMap<String, String>,
    env_key: &str,
) -> anyhow::Result<Option<u64>> {
    match resolve_string_option(cli_value, env_values, env_key) {
        None => Ok(None),
        Some(value) => value
            .parse::<u64>()
            .map(Some)
            .with_context(|| format!("{env_key} must be an unsigned integer")),
    }
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
            RuntimeOperationKind::SupportBundle => "support_bundle_output",
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
            RuntimeOperationKind::SupportBundle => settings
                .support_bundle_output
                .as_deref()
                .map(path_to_string)
                .context("support-bundle requires --support-bundle-output")?,
        };
        let runtime_home = settings
            .runtime_home
            .as_deref()
            .map(path_to_string)
            .with_context(|| format!("{} requires --runtime-home", operation.display_name()))?;

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
        if operation == RuntimeOperationKind::SupportBundle {
            lines.push("bundle.includes_redacted_config=true".to_owned());
            lines.push("bundle.includes_logs=true".to_owned());
            lines.push("bundle.includes_runtime_state=true".to_owned());
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
    lines.extend(render_user_center_plan_lines(&settings.user_center));

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
            RuntimeOperationKind::SupportBundle => settings
                .support_bundle_output
                .as_deref()
                .map(path_to_string)
                .context("support-bundle requires --support-bundle-output")?,
        };
        let runtime_home = settings
            .runtime_home
            .as_deref()
            .map(path_to_string)
            .with_context(|| format!("{} requires --runtime-home", operation.display_name()))?;

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
            RuntimeOperationKind::SupportBundle => {
                payload["support_bundle_output"] = json!(operation_path);
                payload["bundle"] = json!({
                    "includes_redacted_config": true,
                    "includes_logs": true,
                    "includes_runtime_state": true,
                });
            }
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
        },
        "user_center": &settings.user_center,
    }))
    .expect("service plan json serialization should not fail"))
}

fn render_user_center_plan_lines(user_center: &UserCenterServerContract) -> Vec<String> {
    let mut lines = vec![
        format!("user_center.mode={}", user_center.mode),
        format!(
            "user_center.active_integration_kind={}",
            user_center.active_integration_kind
        ),
        format!("user_center.app_id={}", user_center.app_id),
        format!("user_center.provider_key={}", user_center.provider_key),
        format!(
            "user_center.local_api_base_path={}",
            user_center.local_api_base_path
        ),
        format!("user_center.sqlite_path={}", user_center.sqlite_path),
        format!("user_center.table_prefix={}", user_center.table_prefix),
        format!(
            "user_center.auth_token_header_name={}",
            user_center.auth_token_header_name
        ),
        format!(
            "user_center.access_token_header_name={}",
            user_center.access_token_header_name
        ),
        format!(
            "user_center.refresh_token_header_name={}",
            user_center.refresh_token_header_name
        ),
        format!(
            "user_center.session_header_name={}",
            user_center.session_header_name
        ),
        format!(
            "user_center.authorization_scheme={}",
            user_center.authorization_scheme
        ),
        format!(
            "user_center.allow_authorization_fallback_to_access_token={}",
            user_center.allow_authorization_fallback_to_access_token
        ),
        format!("user_center.handshake_mode={}", user_center.handshake_mode),
        format!(
            "user_center.handshake_required={}",
            user_center.handshake_required
        ),
        format!(
            "user_center.handshake_freshness_window_ms={}",
            user_center.handshake_freshness_window_ms
        ),
        format!(
            "user_center.protected_tokens={}",
            user_center.protected_tokens.join(",")
        ),
        format!(
            "user_center.shared_secret_configured={}",
            user_center.shared_secret_configured
        ),
    ];

    if let Some(app_api_base_url) = user_center.app_api_base_url.as_deref() {
        lines.push(format!("user_center.app_api_base_url={app_api_base_url}"));
    }
    if let Some(external_base_url) = user_center.external_base_url.as_deref() {
        lines.push(format!("user_center.external_base_url={external_base_url}"));
    }
    if let Some(database_url) = user_center.database_url.as_deref() {
        lines.push(format!("user_center.database_url={database_url}"));
    }
    if let Some(schema_name) = user_center.schema_name.as_deref() {
        lines.push(format!("user_center.schema_name={schema_name}"));
    }
    if let Some(secret_id) = user_center.secret_id.as_deref() {
        lines.push(format!("user_center.secret_id={secret_id}"));
    }

    lines
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
        build_loader_cli_overrides, collect_env_values, execute_backup_operation,
        execute_restore_operation, execute_support_bundle_operation, render_service_plan,
        resolve_default_site_dirs_for_paths, resolve_effective_public_web_bind,
        resolve_runtime_context, resolve_runtime_contextual_service_settings,
        resolve_service_settings_from_env_values, validate_runtime_config_for_install_mode,
        InstalledRuntimeContext, InstalledRuntimeManifest, PlanFormat, ProductRuntimeRole,
        ProductServiceSettings, RouterProductServiceCli, StandaloneConfig, StandaloneConfigLoader,
        UserCenterMode, INSTALLED_RUNTIME_LAYOUT_VERSION, INSTALLED_RUNTIME_NAME,
        ROUTER_PRODUCT_SERVICE_BINARY_STEM,
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

        let settings = resolve_service_settings_from_env_values(
            &cli,
            &collect_env_values(Vec::<(String, String)>::new()),
        )
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

        let settings = resolve_service_settings_from_env_values(
            &cli,
            &collect_env_values([
                ("SDKWORK_DATABASE_URL", "sqlite:///tmp/router.db"),
                ("SDKWORK_WEB_BIND", "0.0.0.0:3001"),
                ("SDKWORK_ROUTER_ROLES", "web,gateway,admin"),
                ("SDKWORK_GATEWAY_PROXY_TARGET", "10.0.0.21:8080"),
                ("SDKWORK_ADMIN_PROXY_TARGET", "10.0.0.31:8081"),
            ]),
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
    fn user_center_mode_only_accepts_canonical_public_identifiers() {
        assert_eq!(
            UserCenterMode::from_raw("builtin-local"),
            Some(UserCenterMode::BuiltinLocal)
        );
        assert_eq!(
            UserCenterMode::from_raw("sdkwork-cloud-app-api"),
            Some(UserCenterMode::SdkworkCloudAppApi)
        );
        assert_eq!(
            UserCenterMode::from_raw("external-user-center"),
            Some(UserCenterMode::ExternalUserCenter)
        );
        assert_eq!(UserCenterMode::from_raw("spring-ai-plus-app-api"), None);
        assert_eq!(UserCenterMode::from_raw("sdkwork-app-api"), None);
        assert_eq!(UserCenterMode::from_raw("local"), None);
        assert_eq!(UserCenterMode::from_raw("local-native"), None);
        assert_eq!(UserCenterMode::from_raw("app-api"), None);
        assert_eq!(UserCenterMode::from_raw("app-api-hub"), None);
        assert_eq!(UserCenterMode::from_raw("external"), None);
        assert_eq!(UserCenterMode::from_raw("external-hub"), None);
        assert_eq!(
            UserCenterMode::SdkworkCloudAppApi.as_str(),
            "sdkwork-cloud-app-api"
        );
    }

    #[test]
    fn resolve_service_settings_defaults_user_center_to_builtin_local() {
        let cli = RouterProductServiceCli::try_parse_from(["router-product-service"])
            .expect("cli should parse");

        let settings = resolve_service_settings_from_env_values(
            &cli,
            &collect_env_values(Vec::<(String, String)>::new()),
        )
        .expect("settings should resolve");

        assert_eq!(settings.user_center.mode, "builtin-local");
        assert_eq!(
            settings.user_center.active_integration_kind,
            "builtin-local"
        );
        assert_eq!(
            settings.user_center.provider_key,
            "sdkwork-api-router-local"
        );
        assert_eq!(
            settings.user_center.local_api_base_path,
            "/api/app/v1/user-center"
        );
        assert_eq!(settings.user_center.sqlite_path, "./data/user-center.db");
        assert_eq!(settings.user_center.handshake_required, false);
        assert_eq!(settings.user_center.handshake_mode, "disabled");
    }

    #[test]
    fn resolve_service_settings_fails_closed_when_cloud_user_center_is_under_configured() {
        let cli = RouterProductServiceCli::try_parse_from(["router-product-service"])
            .expect("cli should parse");

        let error = resolve_service_settings_from_env_values(
            &cli,
            &collect_env_values([
                ("SDKWORK_USER_CENTER_MODE", "sdkwork-cloud-app-api"),
                (
                    "SDKWORK_USER_CENTER_APP_API_BASE_URL",
                    "https://cloud.example.test/app",
                ),
                ("SDKWORK_USER_CENTER_PROVIDER_KEY", "router-cloud"),
                ("SDKWORK_USER_CENTER_SECRET_ID", "secret-01"),
            ]),
        )
        .expect_err("cloud user-center without shared secret must fail closed");

        assert!(error
            .to_string()
            .contains("SDKWORK_USER_CENTER_SHARED_SECRET"));
    }

    #[test]
    fn resolve_service_settings_rejects_non_canonical_public_mode_aliases() {
        let cli = RouterProductServiceCli::try_parse_from(["router-product-service"])
            .expect("cli should parse");

        let error = resolve_service_settings_from_env_values(
            &cli,
            &collect_env_values([("SDKWORK_USER_CENTER_MODE", "app-api-hub")]),
        )
        .expect_err("legacy public mode aliases must fail closed");

        assert!(error
            .to_string()
            .contains("SDKWORK_USER_CENTER_MODE must be one of"));
    }

    #[test]
    fn resolve_service_settings_infers_canonical_cloud_user_center_mode_from_single_upstream_base_url(
    ) {
        let cli = RouterProductServiceCli::try_parse_from(["router-product-service"])
            .expect("cli should parse");

        let settings = resolve_service_settings_from_env_values(
            &cli,
            &collect_env_values([
                (
                    "SDKWORK_USER_CENTER_APP_API_BASE_URL",
                    "https://cloud.example.test/app",
                ),
                ("SDKWORK_USER_CENTER_PROVIDER_KEY", "router-cloud"),
                ("SDKWORK_USER_CENTER_SECRET_ID", "secret-01"),
                ("SDKWORK_USER_CENTER_SHARED_SECRET", "shared-01"),
            ]),
        )
        .expect("single remote base url should infer the canonical cloud mode");

        assert_eq!(settings.user_center.mode, "sdkwork-cloud-app-api");
        assert_eq!(
            settings.user_center.active_integration_kind,
            "sdkwork-cloud-app-api"
        );
    }

    #[test]
    fn render_service_plan_json_embeds_normalized_user_center_contract() {
        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--dry-run",
            "--plan-format",
            "json",
        ])
        .expect("cli should parse");

        let settings = resolve_service_settings_from_env_values(
            &cli,
            &collect_env_values([
                (
                    "SDKWORK_USER_CENTER_APP_API_BASE_URL",
                    "https://cloud.example.test/app",
                ),
                ("SDKWORK_USER_CENTER_PROVIDER_KEY", "router-cloud"),
                ("SDKWORK_USER_CENTER_SECRET_ID", "secret-01"),
                ("SDKWORK_USER_CENTER_SHARED_SECRET", "shared-01"),
            ]),
        )
        .expect("settings should resolve");
        let plan = render_service_plan(&settings, &StandaloneConfig::default())
            .expect("plan should render");
        let parsed: Value = serde_json::from_str(&plan).expect("plan should parse");

        assert_eq!(parsed["plan_format"], "json");
        assert_eq!(parsed["user_center"]["mode"], "sdkwork-cloud-app-api");
        assert_eq!(
            parsed["user_center"]["active_integration_kind"],
            "sdkwork-cloud-app-api"
        );
        assert_eq!(
            parsed["user_center"]["app_api_base_url"],
            "https://cloud.example.test/app"
        );
        assert_eq!(parsed["user_center"]["handshake_required"], true);
        assert_eq!(
            parsed["user_center"]["handshake_mode"],
            "provider-shared-secret"
        );
        assert_eq!(parsed["user_center"]["shared_secret_configured"], true);
    }

    #[test]
    fn resolve_runtime_contextual_service_settings_merges_router_env_defaults_before_materializing_settings(
    ) {
        let config_root = unique_temp_dir("router-env-defaults");
        fs::create_dir_all(&config_root).expect("config root should exist");
        fs::write(
            config_root.join("router.env"),
            "SDKWORK_ROUTER_ROLES=web,portal\nSDKWORK_ADMIN_PROXY_TARGET=10.0.0.21:8081\nSDKWORK_PORTAL_PROXY_TARGET=10.0.0.22:8082\n",
        )
        .expect("router.env should exist");

        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--config-dir",
            &config_root.to_string_lossy(),
        ])
        .expect("cli should parse");

        let (settings, _) =
            resolve_runtime_contextual_service_settings(&cli, Vec::<(String, String)>::new())
                .expect("settings should resolve");

        assert_eq!(
            settings.roles,
            Some(vec![ProductRuntimeRole::Web, ProductRuntimeRole::Portal])
        );
        assert_eq!(settings.admin_upstream, Some("10.0.0.21:8081".to_owned()));
        assert_eq!(settings.portal_upstream, Some("10.0.0.22:8082".to_owned()));

        let (env_preferred_settings, _) = resolve_runtime_contextual_service_settings(
            &cli,
            [("SDKWORK_ROUTER_ROLES", "gateway")],
        )
        .expect("env-preferred settings should resolve");
        assert_eq!(
            env_preferred_settings.roles,
            Some(vec![ProductRuntimeRole::Gateway])
        );
    }

    #[test]
    fn resolve_runtime_contextual_service_settings_reads_router_env_from_installed_runtime_home() {
        let (runtime_root, runtime_home, manifest) =
            create_valid_installed_runtime_manifest_fixture("runtime-home-router-env");
        let config_root = manifest
            .config_root
            .clone()
            .expect("config root should exist in manifest");
        fs::write(
            config_root.join("router.env"),
            "SDKWORK_ROUTER_NODE_ID_PREFIX=desktop\n",
        )
        .expect("router.env should exist");
        fs::write(
            runtime_home.join("release-manifest.json"),
            serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("release manifest should exist");

        let cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--runtime-home",
            &runtime_home.to_string_lossy(),
        ])
        .expect("cli should parse");

        let (settings, runtime_context) =
            resolve_runtime_contextual_service_settings(&cli, Vec::<(String, String)>::new())
                .expect("settings should resolve");

        assert_eq!(settings.node_id_prefix, Some("desktop".to_owned()));
        assert_eq!(
            settings.config_dir,
            Some(config_root.to_string_lossy().into_owned())
        );
        assert_eq!(
            runtime_context
                .expect("runtime context should resolve")
                .runtime_home,
            runtime_home
        );
        fs::remove_dir_all(runtime_root).expect("fixture should be removed");
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

        let settings = resolve_service_settings_from_env_values(
            &cli,
            &collect_env_values(Vec::<(String, String)>::new()),
        )
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

        let restore_settings = resolve_service_settings_from_env_values(
            &restore_cli,
            &collect_env_values(Vec::<(String, String)>::new()),
        )
        .expect("restore settings should resolve");

        assert_eq!(
            restore_settings.restore_source,
            Some(PathBuf::from("D:/router/backups/2026-04-19"))
        );
        assert_eq!(restore_settings.backup_output, None);
        assert!(restore_settings.force);

        let support_bundle_cli = RouterProductServiceCli::try_parse_from([
            "router-product-service",
            "--config-dir",
            "D:/router/config",
            "--runtime-home",
            "D:/router/current",
            "--support-bundle-output",
            "D:/router/support/2026-04-19",
            "--force",
            "--dry-run",
            "--plan-format",
            "json",
        ])
        .expect("support-bundle cli should parse");

        let support_bundle_settings = resolve_service_settings_from_env_values(
            &support_bundle_cli,
            &collect_env_values(Vec::<(String, String)>::new()),
        )
        .expect("support-bundle settings should resolve");

        assert_eq!(
            support_bundle_settings.support_bundle_output,
            Some(PathBuf::from("D:/router/support/2026-04-19"))
        );
        assert_eq!(support_bundle_settings.backup_output, None);
        assert_eq!(support_bundle_settings.restore_source, None);
        assert!(support_bundle_settings.force);
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
    fn system_install_mode_rejects_postgres_placeholders_for_validation() {
        let error = validate_runtime_config_for_install_mode(
            &StandaloneConfig {
                database_url:
                    "postgresql://sdkwork:replace-with-db-password@127.0.0.1:5432/sdkwork_api_router"
                    .to_owned(),
                admin_jwt_signing_secret: "replace-with-admin-jwt-secret".to_owned(),
                portal_jwt_signing_secret: "replace-with-portal-jwt-secret".to_owned(),
                credential_master_key: "replace-with-credential-master-key".to_owned(),
                metrics_bearer_token: "replace-with-metrics-token".to_owned(),
                ..StandaloneConfig::default()
            },
            "0.0.0.0:3001",
            Some("system"),
        )
        .expect_err("system installs must reject placeholder PostgreSQL validation values");

        assert!(error.to_string().contains("placeholder"));
        assert!(error.to_string().contains("database_url"));
    }

    #[test]
    fn system_install_mode_allows_postgres_placeholders_with_explicit_dev_override() {
        validate_runtime_config_for_install_mode(
            &StandaloneConfig {
                database_url:
                    "postgresql://sdkwork:replace-with-db-password@127.0.0.1:5432/sdkwork_api_router"
                        .to_owned(),
                admin_jwt_signing_secret: "replace-with-admin-jwt-secret".to_owned(),
                portal_jwt_signing_secret: "replace-with-portal-jwt-secret".to_owned(),
                credential_master_key: "replace-with-credential-master-key".to_owned(),
                metrics_bearer_token: "replace-with-metrics-token".to_owned(),
                allow_insecure_dev_defaults: true,
                ..StandaloneConfig::default()
            },
            "0.0.0.0:3001",
            Some("system"),
        )
        .expect("explicit development override should allow placeholder validation inputs");
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

    fn router_product_service_binary_name() -> &'static str {
        if cfg!(windows) {
            "router-product-service.exe"
        } else {
            "router-product-service"
        }
    }

    fn create_valid_installed_runtime_manifest_fixture(
        name: &str,
    ) -> (PathBuf, PathBuf, InstalledRuntimeManifest) {
        let product_root = unique_temp_dir(name);
        let runtime_home = product_root.join("current");
        let release_root = product_root.join("releases").join("1.2.3");
        let config_root = product_root.join("config");
        let data_root = product_root.join("data");
        let log_root = product_root.join("log");
        let run_root = product_root.join("run");
        let bootstrap_data_root = release_root.join("data");
        let deployment_asset_root = release_root.join("deploy");
        let release_payload_manifest = release_root.join("release-manifest.json");
        let release_payload_readme_file = release_root.join("README.txt");
        let admin_site_dist_dir = release_root.join("sites").join("admin").join("dist");
        let portal_site_dist_dir = release_root.join("sites").join("portal").join("dist");
        let router_binary = release_root
            .join("bin")
            .join(router_product_service_binary_name());

        fs::create_dir_all(&runtime_home).expect("runtime home should exist");
        fs::create_dir_all(&config_root).expect("config root should exist");
        fs::create_dir_all(&data_root).expect("data root should exist");
        fs::create_dir_all(&log_root).expect("log root should exist");
        fs::create_dir_all(&run_root).expect("run root should exist");
        fs::create_dir_all(&bootstrap_data_root).expect("bootstrap data root should exist");
        fs::create_dir_all(&deployment_asset_root).expect("deployment asset root should exist");
        fs::create_dir_all(&admin_site_dist_dir).expect("admin site dir should exist");
        fs::create_dir_all(&portal_site_dist_dir).expect("portal site dir should exist");
        fs::create_dir_all(
            router_binary
                .parent()
                .expect("router binary should have parent directory"),
        )
        .expect("router binary directory should exist");

        fs::write(config_root.join("router.yaml"), "portal_title: fixture\n")
            .expect("router.yaml should be written");
        fs::write(&release_payload_manifest, "{}\n")
            .expect("release payload manifest should be written");
        fs::write(&release_payload_readme_file, "release readme\n")
            .expect("release payload readme should be written");
        fs::write(
            admin_site_dist_dir.join("index.html"),
            "<html>admin</html>\n",
        )
        .expect("admin index should be written");
        fs::write(
            portal_site_dist_dir.join("index.html"),
            "<html>portal</html>\n",
        )
        .expect("portal index should be written");
        fs::write(&router_binary, "#!/usr/bin/env sh\nexit 0\n")
            .expect("router binary fixture should be written");

        let manifest = InstalledRuntimeManifest {
            runtime: Some(INSTALLED_RUNTIME_NAME.to_owned()),
            layout_version: Some(INSTALLED_RUNTIME_LAYOUT_VERSION),
            install_mode: Some("portable".to_owned()),
            product_root: Some(product_root.clone()),
            control_root: Some(runtime_home.clone()),
            release_version: Some("1.2.3".to_owned()),
            releases_root: Some(product_root.join("releases")),
            release_root: Some(release_root),
            target: Some("x86_64-unknown-linux-gnu".to_owned()),
            installed_binaries: Some(vec![ROUTER_PRODUCT_SERVICE_BINARY_STEM.to_owned()]),
            bootstrap_data_root: Some(bootstrap_data_root),
            deployment_asset_root: Some(deployment_asset_root),
            release_payload_manifest: Some(release_payload_manifest),
            release_payload_readme_file: Some(release_payload_readme_file),
            admin_site_dist_dir: Some(admin_site_dist_dir),
            portal_site_dist_dir: Some(portal_site_dist_dir),
            router_binary: Some(router_binary),
            config_root: Some(config_root.clone()),
            config_file: Some(config_root.join("router.yaml")),
            mutable_data_root: Some(data_root),
            log_root: Some(log_root),
            run_root: Some(run_root),
            installed_at: Some("2026-04-20T00:00:00.000Z".to_owned()),
            ..InstalledRuntimeManifest::default()
        };

        (product_root, runtime_home, manifest)
    }

    #[test]
    fn resolve_runtime_context_rejects_installed_runtime_manifests_missing_generated_contract_metadata(
    ) {
        let (product_root, runtime_home, mut manifest) =
            create_valid_installed_runtime_manifest_fixture("runtime-context-metadata");
        let manifest_path = runtime_home.join("release-manifest.json");
        manifest.runtime = None;
        manifest.layout_version = None;

        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("release manifest should be written");

        let error = resolve_runtime_context(Some(product_root.as_path()))
            .expect_err("runtime context should reject incomplete manifest metadata");

        assert!(error.to_string().contains("runtime"));
        assert!(error.to_string().contains("layoutVersion"));
        assert!(error.to_string().contains("installed runtime manifest"));

        fs::remove_dir_all(product_root).expect("fixture should be removed");
    }

    #[test]
    fn resolve_runtime_context_rejects_installed_runtime_manifests_with_release_payload_drift() {
        let (product_root, runtime_home, mut manifest) =
            create_valid_installed_runtime_manifest_fixture("runtime-context-drift");
        let manifest_path = runtime_home.join("release-manifest.json");

        manifest.admin_site_dist_dir = Some(product_root.join("broken-admin-site"));
        fs::create_dir_all(
            manifest
                .admin_site_dist_dir
                .as_ref()
                .expect("broken admin site dir should exist"),
        )
        .expect("broken admin site dir should be created");

        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("release manifest should be written");

        let error = resolve_runtime_context(Some(product_root.as_path()))
            .expect_err("runtime context should reject release payload drift");

        assert!(error.to_string().contains("adminSiteDistDir"));
        assert!(error.to_string().contains("release payload"));

        fs::remove_dir_all(product_root).expect("fixture should be removed");
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
    fn execute_backup_operation_writes_self_describing_backup_bundle_manifest() {
        let product_root = unique_temp_dir("backup-bundle-manifest");
        let runtime_home = product_root.join("current");
        let config_root = product_root.join("config");
        let data_root = product_root.join("data");
        let log_root = product_root.join("log");
        let run_root = product_root.join("run");
        let backup_output = product_root.join("backup");

        fs::create_dir_all(&runtime_home).expect("runtime home should exist");
        fs::create_dir_all(&config_root).expect("config root should exist");
        fs::create_dir_all(&data_root).expect("data root should exist");
        fs::create_dir_all(&log_root).expect("log root should exist");
        fs::create_dir_all(&run_root).expect("run root should exist");
        fs::write(config_root.join("router.yaml"), "portal_title: backup\n")
            .expect("router config should exist");
        fs::write(data_root.join("state.json"), "{\"hello\":\"world\"}\n")
            .expect("mutable data should exist");

        let manifest = InstalledRuntimeManifest {
            install_mode: Some("system".to_owned()),
            product_root: Some(product_root.clone()),
            control_root: Some(runtime_home.clone()),
            release_version: Some("0.1.0".to_owned()),
            config_root: Some(config_root.clone()),
            config_file: Some(config_root.join("router.yaml")),
            mutable_data_root: Some(data_root.clone()),
            log_root: Some(log_root.clone()),
            run_root: Some(run_root.clone()),
            ..InstalledRuntimeManifest::default()
        };
        let manifest_path = runtime_home.join("release-manifest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("release manifest should be written");
        let runtime_context = InstalledRuntimeContext {
            runtime_home: runtime_home.clone(),
            manifest_path,
            manifest,
        };

        let settings = ProductServiceSettings {
            runtime_home: Some(runtime_home),
            backup_output: Some(backup_output.clone()),
            force: true,
            ..ProductServiceSettings::default()
        };
        let config = StandaloneConfig {
            database_url: sqlite_url_for(data_root.join("router.db")),
            ..StandaloneConfig::default()
        };

        execute_backup_operation(&settings, &config, &runtime_context)
            .expect("backup operation should export");

        let backup_manifest: Value = serde_json::from_str(
            &fs::read_to_string(backup_output.join("backup-manifest.json"))
                .expect("backup manifest should exist"),
        )
        .expect("backup manifest should parse");

        assert_eq!(backup_manifest["formatVersion"], 2);
        assert_eq!(
            backup_manifest["bundle"]["controlManifestFile"],
            "control/release-manifest.json"
        );
        assert_eq!(backup_manifest["bundle"]["configSnapshotRoot"], "config");
        assert_eq!(backup_manifest["bundle"]["mutableDataSnapshotRoot"], "data");
    }

    #[test]
    fn execute_restore_operation_uses_manifest_declared_backup_bundle_paths() {
        let product_root = unique_temp_dir("restore-bundle-manifest");
        let runtime_home = product_root.join("current");
        let config_root = product_root.join("config");
        let data_root = product_root.join("data");
        let log_root = product_root.join("log");
        let run_root = product_root.join("run");
        let restore_source = product_root.join("backup-source");

        fs::create_dir_all(&runtime_home).expect("runtime home should exist");
        fs::create_dir_all(&config_root).expect("config root should exist");
        fs::create_dir_all(&data_root).expect("data root should exist");
        fs::create_dir_all(&log_root).expect("log root should exist");
        fs::create_dir_all(&run_root).expect("run root should exist");
        fs::write(config_root.join("router.yaml"), "portal_title: stale\n")
            .expect("stale router config should exist");
        fs::write(data_root.join("state.json"), "{\"version\":\"stale\"}\n")
            .expect("stale mutable data should exist");

        let manifest = InstalledRuntimeManifest {
            install_mode: Some("system".to_owned()),
            product_root: Some(product_root.clone()),
            control_root: Some(runtime_home.clone()),
            release_version: Some("0.1.0".to_owned()),
            config_root: Some(config_root.clone()),
            config_file: Some(config_root.join("router.yaml")),
            mutable_data_root: Some(data_root.clone()),
            log_root: Some(log_root.clone()),
            run_root: Some(run_root.clone()),
            ..InstalledRuntimeManifest::default()
        };
        let manifest_path = runtime_home.join("release-manifest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("release manifest should be written");
        let runtime_context = InstalledRuntimeContext {
            runtime_home: runtime_home.clone(),
            manifest_path,
            manifest,
        };

        let control_manifest_relative = PathBuf::from("bundle-state/control/runtime-manifest.json");
        let config_snapshot_relative = PathBuf::from("bundle-state/config-snapshot");
        let data_snapshot_relative = PathBuf::from("bundle-state/data-snapshot");
        let control_manifest_path = restore_source.join(&control_manifest_relative);
        let config_snapshot_root = restore_source.join(&config_snapshot_relative);
        let data_snapshot_root = restore_source.join(&data_snapshot_relative);
        fs::create_dir_all(
            control_manifest_path
                .parent()
                .expect("control manifest should have parent"),
        )
        .expect("control manifest parent should exist");
        fs::create_dir_all(&config_snapshot_root).expect("config snapshot should exist");
        fs::create_dir_all(&data_snapshot_root).expect("data snapshot should exist");
        fs::write(&control_manifest_path, "{}\n").expect("control manifest should exist");
        fs::write(
            config_snapshot_root.join("router.yaml"),
            "portal_title: restored\n",
        )
        .expect("restored router config should exist");
        fs::write(
            data_snapshot_root.join("state.json"),
            "{\"version\":\"restored\"}\n",
        )
        .expect("restored mutable data should exist");
        fs::write(
            restore_source.join("backup-manifest.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "formatVersion": 2,
                "createdAt": "2026-04-20T00:00:00Z",
                "runtimeHome": runtime_home,
                "installMode": "system",
                "releaseVersion": "0.1.0",
                "productRoot": product_root,
                "configRoot": config_root,
                "configFile": config_root.join("router.yaml"),
                "mutableDataRoot": data_root,
                "logRoot": log_root,
                "runRoot": run_root,
                "bundle": {
                    "controlManifestFile": control_manifest_relative,
                    "configSnapshotRoot": config_snapshot_relative,
                    "mutableDataSnapshotRoot": data_snapshot_relative,
                },
                "database": {
                    "kind": "sqlite",
                    "strategy": "filesystem-snapshot",
                    "dumpFile": null,
                },
            }))
            .expect("backup manifest should serialize"),
        )
        .expect("backup manifest should be written");

        let settings = ProductServiceSettings {
            runtime_home: Some(runtime_home),
            restore_source: Some(restore_source),
            force: true,
            ..ProductServiceSettings::default()
        };
        let config = StandaloneConfig {
            database_url: sqlite_url_for(data_root.join("router.db")),
            ..StandaloneConfig::default()
        };

        execute_restore_operation(&settings, &config, &runtime_context)
            .expect("restore operation should consume the manifest-declared bundle paths");

        assert_eq!(
            fs::read_to_string(config_root.join("router.yaml"))
                .expect("restored router config should exist"),
            "portal_title: restored\n"
        );
        assert_eq!(
            fs::read_to_string(data_root.join("state.json"))
                .expect("restored mutable data should exist"),
            "{\"version\":\"restored\"}\n"
        );
    }

    #[test]
    fn render_service_plan_supports_json_support_bundle_contract() {
        let settings = ProductServiceSettings {
            config_dir: Some("D:/router/config".to_owned()),
            runtime_home: Some(PathBuf::from("D:/router/current")),
            support_bundle_output: Some(PathBuf::from("D:/router/support/2026-04-19")),
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

        let plan =
            render_service_plan(&settings, &config).expect("support-bundle plan should render");
        let parsed: Value =
            serde_json::from_str(&plan).expect("support-bundle plan should be valid json");

        assert_eq!(parsed["mode"], "support-bundle-dry-run");
        assert_eq!(parsed["plan_format"], "json");
        assert_eq!(parsed["runtime_home"], "D:/router/current");
        assert_eq!(
            parsed["support_bundle_output"],
            "D:/router/support/2026-04-19"
        );
        assert_eq!(parsed["database"]["kind"], "postgresql");
        assert_eq!(parsed["database"]["strategy"], "metadata-redacted");
        assert_eq!(parsed["bundle"]["includes_logs"], true);
        assert_eq!(parsed["bundle"]["includes_runtime_state"], true);
        assert_eq!(parsed["bundle"]["includes_redacted_config"], true);
    }

    #[test]
    fn execute_support_bundle_operation_creates_redacted_operator_bundle() {
        let product_root = unique_temp_dir("support-bundle");
        let runtime_home = product_root.join("current");
        let config_root = product_root.join("config");
        let log_root = product_root.join("log");
        let run_root = product_root.join("run");
        let output_root = product_root.join("support");

        fs::create_dir_all(&runtime_home).expect("runtime home should exist");
        fs::create_dir_all(&config_root).expect("config root should exist");
        fs::create_dir_all(&log_root).expect("log root should exist");
        fs::create_dir_all(&run_root).expect("run root should exist");

        fs::write(
            config_root.join("router.yaml"),
            r#"
database_url: "postgresql://sdkwork:secret@127.0.0.1:5432/sdkwork_api_router"
metrics_bearer_token: "very-secret-token"
"#,
        )
        .expect("router.yaml should be written");
        fs::write(
            config_root.join("router.env"),
            "SDKWORK_DATABASE_URL=postgresql://sdkwork:secret@127.0.0.1:5432/sdkwork_api_router\nSDKWORK_ADMIN_JWT_SIGNING_SECRET=super-secret\n",
        )
        .expect("router.env should be written");
        fs::write(
            config_root.join("secrets.json"),
            "{\"key\":\"keep-me-out\"}\n",
        )
        .expect("secrets.json should be written");
        fs::write(
            log_root.join("router.log"),
            "authorization: Bearer abc123\nconnected to postgresql://sdkwork:secret@127.0.0.1:5432/sdkwork_api_router\n",
        )
        .expect("router.log should be written");
        fs::write(run_root.join("router-product-service.pid"), "999999\n")
            .expect("pid file should be written");
        fs::write(
            run_root.join("router-product-service.state"),
            "SDKWORK_ROUTER_MANAGED_PID=\"999999\"\n",
        )
        .expect("state file should be written");

        let manifest = InstalledRuntimeManifest {
            install_mode: Some("system".to_owned()),
            product_root: Some(product_root.clone()),
            control_root: Some(runtime_home.clone()),
            release_version: Some("0.1.0".to_owned()),
            config_root: Some(config_root.clone()),
            config_file: Some(config_root.join("router.yaml")),
            log_root: Some(log_root.clone()),
            run_root: Some(run_root.clone()),
            ..InstalledRuntimeManifest::default()
        };
        let manifest_path = runtime_home.join("release-manifest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("release manifest should be written");
        let runtime_context = InstalledRuntimeContext {
            runtime_home: runtime_home.clone(),
            manifest_path,
            manifest,
        };

        let settings = ProductServiceSettings {
            runtime_home: Some(runtime_home),
            support_bundle_output: Some(output_root.clone()),
            force: true,
            ..ProductServiceSettings::default()
        };
        let config = StandaloneConfig {
            database_url: "postgresql://sdkwork:secret@127.0.0.1:5432/sdkwork_api_router"
                .to_owned(),
            ..StandaloneConfig::default()
        };

        execute_support_bundle_operation(&settings, &config, &runtime_context)
            .expect("support bundle should export");

        assert!(output_root
            .join("control")
            .join("release-manifest.json")
            .is_file());
        assert!(output_root.join("config").join("inventory.json").is_file());
        assert!(output_root.join("logs").join("inventory.json").is_file());
        assert!(output_root.join("runtime").join("inventory.json").is_file());
        assert!(output_root
            .join("runtime")
            .join("process-state.json")
            .is_file());
        assert!(output_root.join("support-bundle-manifest.json").is_file());

        let support_bundle_manifest: Value = serde_json::from_str(
            &fs::read_to_string(output_root.join("support-bundle-manifest.json"))
                .expect("support bundle manifest should exist"),
        )
        .expect("support bundle manifest should parse");
        assert_eq!(support_bundle_manifest["formatVersion"], 2);
        assert_eq!(
            support_bundle_manifest["paths"]["controlManifestFile"],
            "control/release-manifest.json"
        );
        assert_eq!(
            support_bundle_manifest["paths"]["configSnapshotRoot"],
            "config"
        );
        assert_eq!(
            support_bundle_manifest["paths"]["configInventoryFile"],
            "config/inventory.json"
        );
        assert_eq!(support_bundle_manifest["paths"]["logsSnapshotRoot"], "logs");
        assert_eq!(
            support_bundle_manifest["paths"]["logsInventoryFile"],
            "logs/inventory.json"
        );
        assert_eq!(
            support_bundle_manifest["paths"]["runtimeSnapshotRoot"],
            "runtime"
        );
        assert_eq!(
            support_bundle_manifest["paths"]["runtimeInventoryFile"],
            "runtime/inventory.json"
        );
        assert_eq!(
            support_bundle_manifest["paths"]["processStateFile"],
            "runtime/process-state.json"
        );
        assert!(support_bundle_manifest.get("inventories").is_none());

        let redacted_router_yaml =
            fs::read_to_string(output_root.join("config").join("router.yaml"))
                .expect("redacted router.yaml should exist");
        assert!(redacted_router_yaml.contains("***REDACTED***"));
        assert!(!redacted_router_yaml.contains("very-secret-token"));
        assert!(!redacted_router_yaml.contains("postgresql://sdkwork:secret@"));

        let redacted_router_env = fs::read_to_string(output_root.join("config").join("router.env"))
            .expect("redacted router.env should exist");
        assert!(redacted_router_env.contains("***REDACTED***"));
        assert!(!redacted_router_env.contains("super-secret"));

        assert!(!output_root.join("config").join("secrets.json").exists());

        let redacted_log = fs::read_to_string(output_root.join("logs").join("router.log"))
            .expect("redacted router.log should exist");
        assert!(redacted_log.contains("***REDACTED***"));
        assert!(!redacted_log.contains("abc123"));
        assert!(!redacted_log.contains("postgresql://sdkwork:secret@"));

        let config_inventory: Value = serde_json::from_str(
            &fs::read_to_string(output_root.join("config").join("inventory.json"))
                .expect("config inventory should exist"),
        )
        .expect("config inventory should parse");
        assert_eq!(config_inventory["exists"], true);
        assert!(config_inventory["entries"]
            .as_array()
            .expect("config entries should be an array")
            .iter()
            .any(|entry| entry["relativePath"] == "secrets.json"
                && entry["omittedReason"] == "sensitive-file-omitted"));

        let process_state: Value = serde_json::from_str(
            &fs::read_to_string(output_root.join("runtime").join("process-state.json"))
                .expect("process state should exist"),
        )
        .expect("process state should parse");
        assert_eq!(process_state["pid"], 999999);
        assert_eq!(process_state["running"], false);
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
