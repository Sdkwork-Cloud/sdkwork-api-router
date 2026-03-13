use sdkwork_api_app_catalog::create_provider_with_config;

#[test]
fn creates_provider_for_channel() {
    let provider = create_provider_with_config(
        "provider-openai-official",
        "openai",
        "openai",
        "https://api.openai.com",
        "OpenAI Official",
    )
    .unwrap();
    assert_eq!(provider.channel_id, "openai");
    assert_eq!(provider.adapter_kind, "openai");
    assert_eq!(provider.base_url, "https://api.openai.com");
}
