use sdkwork_api_app_runtime::{
    build_admin_store_from_config, build_cache_runtime_from_config, resolve_service_runtime_node_id,
    start_standalone_runtime_supervision, StandaloneListenerHost, StandaloneServiceKind,
    StandaloneServiceReloadHandles,
};
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_interface_portal::{portal_router_with_state, PortalApiState};
use sdkwork_api_observability::init_tracing;
use sdkwork_api_storage_core::Reloadable;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("portal-api-service");
    let (config_loader, config) = StandaloneConfigLoader::from_env()?;
    config.apply_to_process_env();
    let _cache_runtime = build_cache_runtime_from_config(&config).await?;
    let live_store = Reloadable::new(build_admin_store_from_config(&config).await?);
    let live_portal_jwt = Reloadable::new(config.portal_jwt_signing_secret.clone());
    let state = PortalApiState::with_live_store_and_jwt_secret_handle(
        live_store.clone(),
        live_portal_jwt.clone(),
    );
    let listener_host =
        StandaloneListenerHost::bind(config.portal_bind.clone(), portal_router_with_state(state))
            .await?;
    let node_id = resolve_service_runtime_node_id(StandaloneServiceKind::Portal);
    let _runtime_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Portal,
        config_loader,
        config.clone(),
        StandaloneServiceReloadHandles::portal(live_store, live_portal_jwt)
            .with_listener(listener_host.reload_handle())
            .with_node_id(node_id),
    );
    listener_host.wait().await?;
    Ok(())
}
