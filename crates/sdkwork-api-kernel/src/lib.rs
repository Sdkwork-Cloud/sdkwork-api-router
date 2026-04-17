use std::sync::OnceLock;

pub fn workspace_name() -> &'static str {
    "sdkwork-api-router"
}

pub fn ensure_reqwest_rustls_provider() {
    static INSTALL_ONCE: OnceLock<()> = OnceLock::new();
    INSTALL_ONCE.get_or_init(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}
