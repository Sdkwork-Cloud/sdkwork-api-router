use sdkwork_api_domain_catalog::{
    ModelCapability, ModelVariant, ProviderChannelBinding, ProxyProvider,
};

#[test]
fn provider_can_bind_to_multiple_channels() {
    let provider = ProxyProvider::new(
        "provider-openrouter-main",
        "openrouter",
        "openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter Main",
    )
    .with_channel_binding(ProviderChannelBinding::new(
        "provider-openrouter-main",
        "openai",
    ));

    assert_eq!(provider.channel_id, "openrouter");
    assert_eq!(provider.channel_bindings.len(), 2);
    assert_eq!(
        provider.channel_bindings[0].provider_id,
        "provider-openrouter-main"
    );
    assert_eq!(provider.channel_bindings[0].channel_id, "openrouter");
    assert!(provider.channel_bindings[0].is_primary);
    assert_eq!(provider.channel_bindings[1].channel_id, "openai");
    assert!(!provider.channel_bindings[1].is_primary);
}

#[test]
fn model_variant_tracks_capabilities_and_streaming() {
    let model = ModelVariant::new("gpt-4.1", "provider-openai-official")
        .with_capability(ModelCapability::Responses)
        .with_capability(ModelCapability::ChatCompletions)
        .with_streaming(true)
        .with_context_window(128_000);

    assert!(model.streaming);
    assert_eq!(model.capabilities.len(), 2);
    assert_eq!(model.context_window, Some(128_000));
}
