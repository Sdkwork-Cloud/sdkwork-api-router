use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::configure_capability_catalog_cache_store;
use sdkwork_api_app_runtime::{
    build_admin_store_from_config, build_cache_runtime_from_config, resolve_service_runtime_node_id,
    start_extension_runtime_rollout_supervision, start_standalone_runtime_supervision,
    StandaloneListenerHost, StandaloneServiceKind, StandaloneServiceReloadHandles,
};
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_interface_admin::{admin_router_with_state_and_http_exposure, AdminApiState};
use sdkwork_api_observability::init_tracing;
use sdkwork_api_storage_core::Reloadable;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("admin-api-service");
    let (config_loader, config) = StandaloneConfigLoader::from_env()?;
    config.validate_security_posture()?;
    config.apply_to_process_env();
    let cache_runtime = build_cache_runtime_from_config(&config).await?;
    if config.cache_backend.supports_shared_cache_coherence() {
        configure_capability_catalog_cache_store(cache_runtime.cache_store());
    }
    let live_store = Reloadable::new(build_admin_store_from_config(&config).await?);
    let live_admin_jwt = Reloadable::new(config.admin_jwt_signing_secret.clone());
    let live_secret_manager =
        Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
            config.secret_backend,
            config.credential_master_key.clone(),
            config.credential_legacy_master_keys.clone(),
            config.secret_local_file.clone(),
            config.secret_keyring_service.clone(),
        ));
    let state = AdminApiState::with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
        live_store.clone(),
        live_secret_manager.clone(),
        live_admin_jwt.clone(),
    );
    let listener_host =
        StandaloneListenerHost::bind(
            config.admin_bind.clone(),
            admin_router_with_state_and_http_exposure(state, config.http_exposure_config()),
        )
        .await?;
    let node_id = resolve_service_runtime_node_id(StandaloneServiceKind::Admin);
    let _rollout_supervision = start_extension_runtime_rollout_supervision(
        StandaloneServiceKind::Admin,
        node_id.clone(),
        live_store.clone(),
    )?;
    let _runtime_supervision = start_standalone_runtime_supervision(
        StandaloneServiceKind::Admin,
        config_loader,
        config.clone(),
        StandaloneServiceReloadHandles::admin(live_store, live_admin_jwt)
            .with_secret_manager(live_secret_manager)
            .with_listener(listener_host.reload_handle())
            .with_node_id(node_id),
    );
    listener_host.wait().await?;
    Ok(())
}
