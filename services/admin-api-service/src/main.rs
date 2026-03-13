use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_interface_admin::admin_router_with_pool_and_master_key;
use sdkwork_api_storage_sqlite::run_migrations;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = StandaloneConfig::from_env()?;
    match config.storage_dialect().map(|dialect| dialect.as_str()) {
        Some("sqlite") => {}
        Some(other) => anyhow::bail!(
            "admin-api-service startup currently supports sqlite only; requested storage dialect: {other}"
        ),
        None => anyhow::bail!("admin-api-service received unsupported database URL scheme"),
    }

    let pool = run_migrations(&config.database_url).await?;
    let listener = TcpListener::bind(&config.admin_bind).await?;
    axum::serve(
        listener,
        admin_router_with_pool_and_master_key(pool, config.credential_master_key),
    )
    .await?;
    Ok(())
}
