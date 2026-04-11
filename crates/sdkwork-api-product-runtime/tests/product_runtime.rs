use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use reqwest::Client;
use sdkwork_api_config::{CacheBackendKind, StandaloneConfigLoader};
use sdkwork_api_product_runtime::{
    ProductRuntimeRole, ProductSiteDirs, RouterProductRuntime, RouterProductRuntimeOptions,
};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn desktop_product_runtime_serves_static_sites_and_all_api_health_routes() {
    let config_root = temp_root("desktop-runtime-config");
    let admin_site_dir = temp_root("desktop-admin-site");
    let portal_site_dir = temp_root("desktop-portal-site");
    fs::write(
        admin_site_dir.join("index.html"),
        "<!doctype html><html><body>admin desktop site</body></html>",
    )
    .unwrap();
    fs::write(
        portal_site_dir.join("index.html"),
        "<!doctype html><html><body>portal desktop site</body></html>",
    )
    .unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP", "true")],
    )
    .unwrap();

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        )),
    )
    .await
    .unwrap();

    let base_url = runtime.public_base_url().unwrap().to_owned();
    let snapshot = runtime.snapshot();
    let client = http_client();

    assert_eq!(snapshot.mode, "desktop");
    assert_eq!(
        snapshot.roles,
        vec![
            "web".to_owned(),
            "gateway".to_owned(),
            "admin".to_owned(),
            "portal".to_owned()
        ]
    );
    assert_eq!(snapshot.public_base_url.as_deref(), Some(base_url.as_str()));
    assert!(snapshot
        .public_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .gateway_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .admin_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(snapshot
        .portal_bind_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));

    assert_eq!(
        client
            .get(format!("{base_url}/api/admin/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert_eq!(
        client
            .get(format!("{base_url}/api/portal/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert_eq!(
        client
            .get(format!("{base_url}/api/v1/health"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "ok"
    );
    assert!(client
        .get(format!("{base_url}/admin/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
        .contains("admin desktop site"));
    assert!(client
        .get(format!("{base_url}/portal/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
        .contains("portal desktop site"));
}

#[tokio::test]
async fn desktop_product_runtime_rejects_local_dev_defaults_without_explicit_dev_mode() {
    let config_root = temp_root("desktop-runtime-security");
    let admin_site_dir = temp_root("desktop-security-admin-site");
    let portal_site_dir = temp_root("desktop-security-portal-site");
    fs::write(admin_site_dir.join("index.html"), "admin").unwrap();
    fs::write(portal_site_dir.join("index.html"), "portal").unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();

    let error = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        )),
    )
    .await
    .err()
    .expect("runtime should reject insecure local-dev startup defaults");

    assert!(
        error
            .to_string()
            .contains("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP"),
        "{error}"
    );
}

#[tokio::test]
async fn desktop_product_runtime_does_not_bootstrap_default_users_without_explicit_dev_mode() {
    let config_root = temp_root("desktop-runtime-no-default-users");
    let admin_site_dir = temp_root("desktop-no-default-admin-site");
    let portal_site_dir = temp_root("desktop-no-default-portal-site");
    fs::write(admin_site_dir.join("index.html"), "admin").unwrap();
    fs::write(portal_site_dir.join("index.html"), "portal").unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [
            (
                "SDKWORK_ADMIN_JWT_SIGNING_SECRET",
                "prod-admin-jwt-secret-1234567890",
            ),
            (
                "SDKWORK_PORTAL_JWT_SIGNING_SECRET",
                "prod-portal-jwt-secret-1234567890",
            ),
            (
                "SDKWORK_CREDENTIAL_MASTER_KEY",
                "prod-master-key-1234567890",
            ),
        ],
    )
    .unwrap();

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        )),
    )
    .await
    .unwrap();

    let base_url = runtime.public_base_url().unwrap().to_owned();
    let client = http_client();

    let admin_response = client
        .post(format!("{base_url}/api/admin/auth/login"))
        .header("content-type", "application/json")
        .body(r#"{"email":"admin@sdkwork.local","password":"ChangeMe123!"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(admin_response.status(), reqwest::StatusCode::UNAUTHORIZED);

    let portal_response = client
        .post(format!("{base_url}/api/portal/auth/login"))
        .header("content-type", "application/json")
        .body(r#"{"email":"portal@sdkwork.local","password":"ChangeMe123!"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(portal_response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn server_product_runtime_rejects_web_role_without_required_api_upstreams() {
    let config_root = temp_root("server-runtime-config");
    let admin_site_dir = temp_root("server-admin-site");
    let portal_site_dir = temp_root("server-portal-site");
    fs::write(admin_site_dir.join("index.html"), "admin").unwrap();
    fs::write(portal_site_dir.join("index.html"), "portal").unwrap();

    let (loader, config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP", "true")],
    )
    .unwrap();

    let error = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::server(ProductSiteDirs::new(
            &admin_site_dir,
            &portal_site_dir,
        ))
        .with_roles([ProductRuntimeRole::Web]),
    )
    .await
    .err()
    .expect("web-only server runtime without API upstreams should fail");

    assert!(error.to_string().contains("gateway upstream"));
}

#[tokio::test]
async fn product_runtime_supports_redis_cache_backend_during_startup() {
    let config_root = temp_root("runtime-cache-backend");
    let (loader, mut config) = StandaloneConfigLoader::from_local_root_and_pairs(
        &config_root,
        [("SDKWORK_ALLOW_LOCAL_DEV_BOOTSTRAP", "true")],
    )
    .unwrap();
    let redis_server = MinimalRedisPingServer::start();
    config.cache_backend = CacheBackendKind::Redis;
    config.cache_url = Some(redis_server.url_with_db(0));

    let runtime = RouterProductRuntime::start(
        loader,
        config,
        RouterProductRuntimeOptions::desktop(ProductSiteDirs::new(
            config_root.join("unused-admin-site"),
            config_root.join("unused-portal-site"),
        ))
        .with_roles(Vec::<ProductRuntimeRole>::new()),
    )
    .await
    .unwrap();

    assert_eq!(runtime.snapshot().roles, Vec::<String>::new());
}

struct MinimalRedisPingServer {
    address: String,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MinimalRedisPingServer {
    fn start() -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop = stop.clone();
        let thread = std::thread::spawn(move || {
            while !thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        loop {
                            match read_minimal_resp_array(&mut stream) {
                                Ok(Some(command)) => match String::from_utf8_lossy(&command[0])
                                    .to_ascii_uppercase()
                                    .as_str()
                                {
                                    "PING" => {
                                        use std::io::Write;
                                        stream.write_all(b"+PONG\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    "AUTH" | "SELECT" => {
                                        use std::io::Write;
                                        stream.write_all(b"+OK\r\n").unwrap();
                                        stream.flush().unwrap();
                                    }
                                    other => panic!("unexpected minimal redis command: {other}"),
                                },
                                Ok(None) => break,
                                Err(error)
                                    if matches!(
                                        error.kind(),
                                        std::io::ErrorKind::UnexpectedEof
                                            | std::io::ErrorKind::ConnectionReset
                                            | std::io::ErrorKind::TimedOut
                                    ) =>
                                {
                                    break
                                }
                                Err(error) => panic!("minimal redis server read failed: {error}"),
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(error) => panic!("minimal redis accept failed: {error}"),
                }
            }
        });

        Self {
            address,
            stop,
            thread: Some(thread),
        }
    }

    fn url_with_db(&self, db: u32) -> String {
        format!("redis://{}/{db}", self.address)
    }
}

impl Drop for MinimalRedisPingServer {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = std::net::TcpStream::connect(&self.address);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

fn read_minimal_resp_array(
    stream: &mut std::net::TcpStream,
) -> std::io::Result<Option<Vec<Vec<u8>>>> {
    let mut marker = [0_u8; 1];
    match std::io::Read::read_exact(stream, &mut marker) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    assert_eq!(marker[0], b'*');
    let count = read_minimal_resp_line(stream)?.parse::<usize>().unwrap();
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bulk_marker = [0_u8; 1];
        std::io::Read::read_exact(stream, &mut bulk_marker)?;
        assert_eq!(bulk_marker[0], b'$');
        let length = read_minimal_resp_line(stream)?.parse::<usize>().unwrap();
        let mut value = vec![0_u8; length];
        std::io::Read::read_exact(stream, &mut value)?;
        let mut crlf = [0_u8; 2];
        std::io::Read::read_exact(stream, &mut crlf)?;
        values.push(value);
    }
    Ok(Some(values))
}

fn read_minimal_resp_line(stream: &mut std::net::TcpStream) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        std::io::Read::read_exact(stream, &mut byte)?;
        if byte[0] == b'\r' {
            let mut newline = [0_u8; 1];
            std::io::Read::read_exact(stream, &mut newline)?;
            assert_eq!(newline[0], b'\n');
            break;
        }
        bytes.push(byte[0]);
    }
    Ok(String::from_utf8(bytes).unwrap())
}

fn temp_root(label: &str) -> PathBuf {
    let unique = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sdkwork-product-runtime-tests-{label}-{unique}"));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn http_client() -> Client {
    Client::builder().build().unwrap()
}
