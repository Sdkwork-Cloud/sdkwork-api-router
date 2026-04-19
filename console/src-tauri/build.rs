fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["runtime_base_url"])),
    )
    .expect("failed to configure tauri build");
}

