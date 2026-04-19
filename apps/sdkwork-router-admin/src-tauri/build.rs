fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "runtime_base_url",
                "install_api_router_client_setup",
                "list_api_key_instances",
            ]),
        ),
    )
    .expect("failed to configure tauri build");
}
