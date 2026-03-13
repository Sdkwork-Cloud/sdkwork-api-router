use anyhow::Result;
use sdkwork_api_domain_catalog::{
    Channel, ModelCapability, ModelCatalogEntry, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_storage_core::AdminStore;

pub fn service_name() -> &'static str {
    "catalog-service"
}

pub fn create_channel(id: &str, name: &str) -> Result<Channel> {
    Ok(Channel::new(id, name))
}

pub fn create_provider(id: &str, channel_id: &str, display_name: &str) -> Result<ProxyProvider> {
    Ok(ProxyProvider::new(
        id,
        channel_id,
        channel_id,
        "http://localhost",
        display_name,
    ))
}

pub async fn persist_channel(store: &dyn AdminStore, id: &str, name: &str) -> Result<Channel> {
    let channel = create_channel(id, name)?;
    store.insert_channel(&channel).await
}

pub async fn list_channels(store: &dyn AdminStore) -> Result<Vec<Channel>> {
    store.list_channels().await
}

pub fn create_provider_with_config(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    create_provider_with_extension_id(id, channel_id, adapter_kind, None, base_url, display_name)
}

pub fn create_provider_with_extension_id(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    extension_id: Option<&str>,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    let provider = ProxyProvider::new(id, channel_id, adapter_kind, base_url, display_name);
    Ok(match extension_id {
        Some(extension_id) => provider.with_extension_id(extension_id),
        None => provider,
    })
}

pub fn create_provider_with_bindings(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    create_provider_with_bindings_and_extension_id(
        id,
        channel_id,
        adapter_kind,
        None,
        base_url,
        display_name,
        channel_bindings,
    )
}

pub fn create_provider_with_bindings_and_extension_id(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    extension_id: Option<&str>,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    let mut provider = create_provider_with_extension_id(
        id,
        channel_id,
        adapter_kind,
        extension_id,
        base_url,
        display_name,
    )?;
    for binding in channel_bindings {
        provider = provider.with_channel_binding(binding.clone());
    }
    Ok(provider)
}

pub async fn persist_provider(
    store: &dyn AdminStore,
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    let provider = create_provider_with_extension_id(
        id,
        channel_id,
        adapter_kind,
        None,
        base_url,
        display_name,
    )?;
    store.insert_provider(&provider).await
}

pub async fn persist_provider_with_bindings(
    store: &dyn AdminStore,
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    persist_provider_with_bindings_and_extension_id(
        store,
        id,
        channel_id,
        adapter_kind,
        None,
        base_url,
        display_name,
        channel_bindings,
    )
    .await
}

pub async fn persist_provider_with_bindings_and_extension_id(
    store: &dyn AdminStore,
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    extension_id: Option<&str>,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    let provider = create_provider_with_bindings_and_extension_id(
        id,
        channel_id,
        adapter_kind,
        extension_id,
        base_url,
        display_name,
        channel_bindings,
    )?;
    store.insert_provider(&provider).await
}

pub async fn list_providers(store: &dyn AdminStore) -> Result<Vec<ProxyProvider>> {
    store.list_providers().await
}

pub fn create_model(external_name: &str, provider_id: &str) -> Result<ModelCatalogEntry> {
    Ok(ModelCatalogEntry::new(external_name, provider_id))
}

pub fn create_model_with_metadata(
    external_name: &str,
    provider_id: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
) -> Result<ModelCatalogEntry> {
    let mut model = create_model(external_name, provider_id)?.with_streaming(streaming);
    for capability in capabilities {
        model = model.with_capability(capability.clone());
    }
    if let Some(context_window) = context_window {
        model = model.with_context_window(context_window);
    }
    Ok(model)
}

pub async fn persist_model(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
) -> Result<ModelCatalogEntry> {
    let model = create_model(external_name, provider_id)?;
    store.insert_model(&model).await
}

pub async fn persist_model_with_metadata(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
) -> Result<ModelCatalogEntry> {
    let model = create_model_with_metadata(
        external_name,
        provider_id,
        capabilities,
        streaming,
        context_window,
    )?;
    store.insert_model(&model).await
}

pub async fn list_model_entries(store: &dyn AdminStore) -> Result<Vec<ModelCatalogEntry>> {
    store.list_models().await
}
