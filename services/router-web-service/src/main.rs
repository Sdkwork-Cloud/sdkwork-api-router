use std::{env, path::PathBuf};

use sdkwork_api_observability::init_tracing;
use sdkwork_api_runtime_host::{serve_public_web, RuntimeHostConfig};

fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_path_or(name: &str, default: &str) -> PathBuf {
    env::var(name)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default))
}

fn main() -> anyhow::Result<()> {
    init_tracing("router-web-service");

    let config = RuntimeHostConfig::new(
        env_or("SDKWORK_WEB_BIND", "0.0.0.0:3001"),
        env_path_or("SDKWORK_ADMIN_SITE_DIR", "apps/sdkwork-router-admin/dist"),
        env_path_or("SDKWORK_PORTAL_SITE_DIR", "apps/sdkwork-router-portal/dist"),
        env_or("SDKWORK_ADMIN_PROXY_TARGET", "127.0.0.1:8081"),
        env_or("SDKWORK_PORTAL_PROXY_TARGET", "127.0.0.1:8082"),
        env_or("SDKWORK_GATEWAY_PROXY_TARGET", "127.0.0.1:8080"),
    );

    serve_public_web(config)
}
