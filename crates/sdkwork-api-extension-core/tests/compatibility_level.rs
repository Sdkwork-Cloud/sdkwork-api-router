use sdkwork_api_extension_core::CompatibilityLevel;

#[test]
fn compatibility_levels_cover_gateway_truth_model() {
    let values = [
        CompatibilityLevel::Native,
        CompatibilityLevel::Relay,
        CompatibilityLevel::Translated,
        CompatibilityLevel::Emulated,
        CompatibilityLevel::Unsupported,
    ];

    assert_eq!(values.len(), 5);
}
