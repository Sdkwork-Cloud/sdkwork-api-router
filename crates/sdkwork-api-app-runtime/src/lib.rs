use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::Router;
use futures_util::{StreamExt, TryStreamExt, stream};
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sdkwork_api_app_credential::{CredentialSecretManager, resolve_credential_secret_with_manager};
use sdkwork_api_app_extension::{
    ExtensionDiscoveryPolicy, start_provider_health_snapshot_supervision,
};
use sdkwork_api_app_gateway::{
    ConfiguredExtensionHostReloadScope, reload_extension_host_with_policy,
    reload_extension_host_with_scope, start_configured_extension_hot_reload_supervision,
};
use sdkwork_api_cache_core::{CacheDriverFactory, CacheDriverRegistry, CacheRuntimeStores};
use sdkwork_api_cache_memory::MemoryCacheStore;
use sdkwork_api_cache_redis::RedisCacheStore;
use sdkwork_api_config::{
    CacheBackendKind, StandaloneConfig, StandaloneConfigLoader, StandaloneConfigWatchState,
    StandaloneRuntimeDynamicConfig,
};
use sdkwork_api_storage_core::{
    AdminStore, CommercialKernelStore, ExtensionRuntimeRolloutParticipantRecord,
    ExtensionRuntimeRolloutRecord, Reloadable, ServiceRuntimeNodeRecord,
    StandaloneConfigRolloutParticipantRecord, StandaloneConfigRolloutRecord, StorageDialect,
    StorageDriverFactory, StorageDriverRegistry,
};
use sdkwork_api_storage_postgres::{PostgresAdminStore, run_migrations as run_postgres_migrations};
use sdkwork_api_storage_sqlite::{SqliteAdminStore, run_migrations as run_sqlite_migrations};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

const CONFIG_RELOAD_POLL_INTERVAL_SECS: u64 = 1;
const EXTENSION_RUNTIME_ROLLOUT_POLL_INTERVAL_MS: u64 = 250;
const EXTENSION_RUNTIME_ROLLOUT_NODE_FRESHNESS_WINDOW_MS: u64 = 15_000;
const DEFAULT_EXTENSION_RUNTIME_ROLLOUT_TIMEOUT_SECS: u64 = 30;
const STANDALONE_CONFIG_ROLLOUT_NODE_FRESHNESS_WINDOW_MS: u64 = 15_000;
const DEFAULT_STANDALONE_CONFIG_ROLLOUT_TIMEOUT_SECS: u64 = 30;

pub struct StandaloneListenerHost {
    inner: Arc<StandaloneListenerHostInner>,
    exit_rx: mpsc::UnboundedReceiver<ListenerServerExit>,
}

#[derive(Clone)]
pub struct StandaloneListenerHandle {
    inner: Arc<StandaloneListenerHostInner>,
}

struct StandaloneListenerHostInner {
    router: Router,
    active: Mutex<Option<RunningStandaloneListenerServer>>,
    exit_tx: mpsc::UnboundedSender<ListenerServerExit>,
    next_generation: AtomicU64,
}

struct RunningStandaloneListenerServer {
    bind: String,
    generation: u64,
    shutdown_requested: Arc<AtomicBool>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    join_handle: JoinHandle<()>,
}

struct ListenerServerExit {
    generation: u64,
    bind: String,
    shutdown_requested: bool,
    result: io::Result<()>,
}

struct PreparedStandaloneListenerRebind {
    inner: Arc<StandaloneListenerHostInner>,
    bind: String,
    listener: TcpListener,
}

impl StandaloneListenerHost {
    pub async fn bind(bind: impl Into<String>, router: Router) -> Result<Self> {
        let bind = bind.into();
        let listener = TcpListener::bind(&bind)
            .await
            .with_context(|| format!("failed to bind standalone listener to {bind}"))?;
        let actual_bind = listener
            .local_addr()
            .with_context(|| format!("failed to resolve standalone listener bind for {bind}"))?
            .to_string();
        let (exit_tx, exit_rx) = mpsc::unbounded_channel();
        let inner = Arc::new(StandaloneListenerHostInner {
            router,
            active: Mutex::new(None),
            exit_tx,
            next_generation: AtomicU64::new(1),
        });
        inner.activate_prebound(actual_bind, listener);

        Ok(Self { inner, exit_rx })
    }

    pub fn reload_handle(&self) -> StandaloneListenerHandle {
        StandaloneListenerHandle {
            inner: self.inner.clone(),
        }
    }

    pub fn current_bind(&self) -> Option<String> {
        self.inner.current_bind()
    }

    pub async fn shutdown(mut self) -> Result<()> {
        let Some(active) = self.inner.take_active_server() else {
            return Ok(());
        };
        let generation = active.generation;
        let bind = active.bind.clone();
        active.request_shutdown();

        while let Some(exit) = self.exit_rx.recv().await {
            if exit.generation != generation {
                continue;
            }

            return exit
                .result
                .with_context(|| format!("listener shutdown failed for bind {bind}"));
        }

        anyhow::bail!("listener host closed before shutdown completed for bind {bind}");
    }

    pub async fn wait(mut self) -> Result<()> {
        while let Some(exit) = self.exit_rx.recv().await {
            if exit.shutdown_requested {
                if let Err(error) = exit.result {
                    eprintln!(
                        "standalone listener shutdown completed with error: bind={} error={error}",
                        exit.bind
                    );
                }
                continue;
            }

            return match exit.result {
                Ok(()) => anyhow::bail!(
                    "standalone listener exited unexpectedly without a shutdown request: bind={}",
                    exit.bind
                ),
                Err(error) => Err(anyhow::Error::new(error).context(format!(
                    "standalone listener exited unexpectedly: bind={}",
                    exit.bind
                ))),
            };
        }

        anyhow::bail!("standalone listener host closed unexpectedly");
    }
}

impl StandaloneListenerHandle {
    pub fn current_bind(&self) -> Option<String> {
        self.inner.current_bind()
    }

    pub async fn rebind(&self, bind: impl Into<String>) -> Result<()> {
        if let Some(prepared) = self.prepare_rebind(bind).await? {
            prepared.activate();
        }
        Ok(())
    }

    async fn prepare_rebind(
        &self,
        bind: impl Into<String>,
    ) -> Result<Option<PreparedStandaloneListenerRebind>> {
        let bind = bind.into();
        if self.current_bind().as_deref() == Some(bind.as_str()) {
            return Ok(None);
        }

        let listener = TcpListener::bind(&bind)
            .await
            .with_context(|| format!("failed to bind replacement standalone listener to {bind}"))?;
        let actual_bind = listener
            .local_addr()
            .with_context(|| {
                format!("failed to resolve replacement standalone listener bind for {bind}")
            })?
            .to_string();

        Ok(Some(PreparedStandaloneListenerRebind {
            inner: self.inner.clone(),
            bind: actual_bind,
            listener,
        }))
    }
}

impl StandaloneListenerHostInner {
    fn activate_prebound(&self, bind: String, listener: TcpListener) {
        let next_server = self.spawn_server(bind, listener);
        let previous = {
            let mut active = self.active.lock().unwrap();
            active.replace(next_server)
        };

        if let Some(previous) = previous {
            previous.request_shutdown();
        }
    }

    fn spawn_server(&self, bind: String, listener: TcpListener) -> RunningStandaloneListenerServer {
        let generation = self.next_generation.fetch_add(1, Ordering::SeqCst);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let shutdown_requested_for_task = shutdown_requested.clone();
        let exit_tx = self.exit_tx.clone();
        let router = self.router.clone();
        let bind_for_task = bind.clone();
        let join_handle = tokio::spawn(async move {
            let result = axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
            let _ = exit_tx.send(ListenerServerExit {
                generation,
                bind: bind_for_task,
                shutdown_requested: shutdown_requested_for_task.load(Ordering::SeqCst),
                result,
            });
        });

        RunningStandaloneListenerServer {
            bind,
            generation,
            shutdown_requested,
            shutdown_tx: Some(shutdown_tx),
            join_handle,
        }
    }

    fn current_bind(&self) -> Option<String> {
        self.active
            .lock()
            .unwrap()
            .as_ref()
            .map(|server| server.bind.clone())
    }

    fn take_active_server(&self) -> Option<RunningStandaloneListenerServer> {
        self.active.lock().unwrap().take()
    }
}

impl RunningStandaloneListenerServer {
    fn request_shutdown(mut self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        std::mem::drop(self.join_handle);
    }
}

impl PreparedStandaloneListenerRebind {
    fn activate(self) {
        self.inner.activate_prebound(self.bind, self.listener);
    }
}

pub struct StandaloneRuntimeSupervision {
    join_handle: JoinHandle<()>,
}

impl Drop for StandaloneRuntimeSupervision {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandaloneServiceKind {
    Gateway,
    Admin,
    Portal,
}

impl StandaloneServiceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Admin => "admin",
            Self::Portal => "portal",
        }
    }

    fn supports_runtime_dynamic(self) -> bool {
        matches!(self, Self::Gateway | Self::Admin)
    }
}

pub struct StandaloneServiceReloadHandles {
    store: Reloadable<Arc<dyn AdminStore>>,
    payment_store: Option<Reloadable<Arc<dyn CommercialKernelStore>>>,
    coordination_store: Option<Reloadable<Arc<dyn AdminStore>>>,
    secret_manager: Option<Reloadable<CredentialSecretManager>>,
    admin_jwt_signing_secret: Option<Reloadable<String>>,
    portal_jwt_signing_secret: Option<Reloadable<String>>,
    listener: Option<StandaloneListenerHandle>,
    node_id: Option<String>,
}

impl StandaloneServiceReloadHandles {
    pub fn gateway(store: Reloadable<Arc<dyn AdminStore>>) -> Self {
        Self {
            store,
            payment_store: None,
            coordination_store: None,
            secret_manager: None,
            admin_jwt_signing_secret: None,
            portal_jwt_signing_secret: None,
            listener: None,
            node_id: None,
        }
    }

    pub fn admin(
        store: Reloadable<Arc<dyn AdminStore>>,
        admin_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store,
            payment_store: None,
            coordination_store: None,
            secret_manager: None,
            admin_jwt_signing_secret: Some(admin_jwt_signing_secret),
            portal_jwt_signing_secret: None,
            listener: None,
            node_id: None,
        }
    }

    pub fn portal(
        store: Reloadable<Arc<dyn AdminStore>>,
        portal_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store,
            payment_store: None,
            coordination_store: None,
            secret_manager: None,
            admin_jwt_signing_secret: None,
            portal_jwt_signing_secret: Some(portal_jwt_signing_secret),
            listener: None,
            node_id: None,
        }
    }

    pub fn with_coordination_store(mut self, coordination_store: Arc<dyn AdminStore>) -> Self {
        self.coordination_store = Some(Reloadable::new(coordination_store));
        self
    }

    pub fn with_payment_store(
        mut self,
        payment_store: Reloadable<Arc<dyn CommercialKernelStore>>,
    ) -> Self {
        self.payment_store = Some(payment_store);
        self
    }

    pub fn with_secret_manager(
        mut self,
        secret_manager: Reloadable<CredentialSecretManager>,
    ) -> Self {
        self.secret_manager = Some(secret_manager);
        self
    }

    pub fn with_listener(mut self, listener: StandaloneListenerHandle) -> Self {
        self.listener = Some(listener);
        self
    }

    pub fn with_node_id(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }
}

struct StandaloneRuntimeState {
    current_config: StandaloneConfig,
    current_dynamic: StandaloneRuntimeDynamicConfig,
    current_store: Arc<dyn AdminStore>,
    snapshot_supervision: AbortOnDropHandle,
    extension_hot_reload_supervision: AbortOnDropHandle,
    previous_watch_state: Option<StandaloneConfigWatchState>,
    pending_restart_required: Option<PendingStandaloneRuntimeRestartRequired>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingStandaloneRuntimeRestartRequired {
    watch_state: StandaloneConfigWatchState,
    message: String,
}

pub struct StandaloneAdminPaymentStoreHandles {
    pub admin_store: Arc<dyn AdminStore>,
    pub payment_store: Arc<dyn CommercialKernelStore>,
}

struct SqliteAdminPaymentStoreFactory;

#[async_trait]
impl StorageDriverFactory<StandaloneAdminPaymentStoreHandles> for SqliteAdminPaymentStoreFactory {
    fn dialect(&self) -> StorageDialect {
        StorageDialect::Sqlite
    }

    fn driver_name(&self) -> &'static str {
        "sqlite-admin-payment-store"
    }

    async fn build(&self, database_url: &str) -> Result<StandaloneAdminPaymentStoreHandles> {
        let pool = run_sqlite_migrations(database_url).await?;
        let store = Arc::new(SqliteAdminStore::new(pool));
        Ok(StandaloneAdminPaymentStoreHandles {
            admin_store: store.clone(),
            payment_store: store,
        })
    }
}

struct PostgresAdminPaymentStoreFactory;

#[async_trait]
impl StorageDriverFactory<StandaloneAdminPaymentStoreHandles> for PostgresAdminPaymentStoreFactory {
    fn dialect(&self) -> StorageDialect {
        StorageDialect::Postgres
    }

    fn driver_name(&self) -> &'static str {
        "postgres-admin-payment-store"
    }

    async fn build(&self, database_url: &str) -> Result<StandaloneAdminPaymentStoreHandles> {
        let pool = run_postgres_migrations(database_url).await?;
        let store = Arc::new(PostgresAdminStore::new(pool));
        Ok(StandaloneAdminPaymentStoreHandles {
            admin_store: store.clone(),
            payment_store: store,
        })
    }
}

struct MemoryCacheStoreFactory;

#[async_trait]
impl CacheDriverFactory for MemoryCacheStoreFactory {
    fn backend_kind(&self) -> CacheBackendKind {
        CacheBackendKind::Memory
    }

    fn driver_name(&self) -> &'static str {
        "memory-cache-store"
    }

    async fn build(&self, _cache_url: Option<&str>) -> Result<CacheRuntimeStores> {
        let store = Arc::new(MemoryCacheStore::default());
        Ok(CacheRuntimeStores::new(store.clone(), store))
    }
}

struct RedisCacheStoreFactory;

#[async_trait]
impl CacheDriverFactory for RedisCacheStoreFactory {
    fn backend_kind(&self) -> CacheBackendKind {
        CacheBackendKind::Redis
    }

    fn driver_name(&self) -> &'static str {
        "redis-cache-store"
    }

    async fn build(&self, cache_url: Option<&str>) -> Result<CacheRuntimeStores> {
        let cache_url =
            cache_url.ok_or_else(|| anyhow::anyhow!("redis cache backend requires cache_url"))?;
        let store = Arc::new(RedisCacheStore::connect(cache_url).await?);
        Ok(CacheRuntimeStores::new(store.clone(), store))
    }
}

const STANDALONE_SUPPORTED_STORAGE_DIALECTS: [StorageDialect; 2] =
    [StorageDialect::Sqlite, StorageDialect::Postgres];

fn standalone_admin_payment_store_registry()
-> StorageDriverRegistry<StandaloneAdminPaymentStoreHandles> {
    StorageDriverRegistry::new()
        .with_factory(SqliteAdminPaymentStoreFactory)
        .with_factory(PostgresAdminPaymentStoreFactory)
}

fn supported_storage_dialects_summary() -> String {
    STANDALONE_SUPPORTED_STORAGE_DIALECTS
        .iter()
        .map(|dialect| dialect.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn standalone_cache_driver_registry() -> CacheDriverRegistry {
    CacheDriverRegistry::new()
        .with_factory(MemoryCacheStoreFactory)
        .with_factory(RedisCacheStoreFactory)
}

pub async fn build_admin_store_from_config(
    config: &StandaloneConfig,
) -> Result<Arc<dyn AdminStore>> {
    Ok(build_admin_payment_store_handles_from_config(config)
        .await?
        .admin_store)
}

pub async fn build_admin_payment_store_handles_from_config(
    config: &StandaloneConfig,
) -> Result<StandaloneAdminPaymentStoreHandles> {
    let supported_dialects = supported_storage_dialects_summary();
    let Some(dialect) = config.storage_dialect() else {
        anyhow::bail!(
            "standalone runtime supervision received unsupported database URL scheme for {} (supported dialects: {})",
            config.database_url,
            supported_dialects
        );
    };

    let registry = standalone_admin_payment_store_registry();
    let Some(driver) = registry.resolve(dialect) else {
        anyhow::bail!(
            "standalone runtime supervision does not yet support storage dialect: {} (supported dialects: {})",
            dialect.as_str(),
            supported_dialects
        );
    };

    driver.build(&config.database_url).await.with_context(|| {
        format!(
            "failed to initialize standalone admin/payment store with driver {}",
            driver.driver_name()
        )
    })
}

pub async fn build_cache_runtime_from_config(
    config: &StandaloneConfig,
) -> Result<CacheRuntimeStores> {
    let registry = standalone_cache_driver_registry();
    let Some(driver) = registry.resolve(config.cache_backend) else {
        anyhow::bail!(
            "standalone runtime does not yet support cache backend: {}",
            config.cache_backend.as_str()
        );
    };

    driver
        .build(config.cache_url.as_deref())
        .await
        .with_context(|| {
            format!(
                "failed to initialize standalone cache runtime with driver {}",
                driver.driver_name()
            )
        })
}

fn build_secret_manager_from_config(config: &StandaloneConfig) -> CredentialSecretManager {
    CredentialSecretManager::new_with_legacy_master_keys(
        config.secret_backend,
        config.credential_master_key.clone(),
        config.credential_legacy_master_keys.clone(),
        config.secret_local_file.clone(),
        config.secret_keyring_service.clone(),
    )
}

async fn validate_secret_manager_for_store(
    store: &dyn AdminStore,
    manager: &CredentialSecretManager,
) -> Result<()> {
    let credentials = store.list_credentials().await?;
    stream::iter(credentials.into_iter().map(|credential| async move {
        let tenant_id = credential.tenant_id.clone();
        let provider_id = credential.provider_id.clone();
        let key_reference = credential.key_reference.clone();

        resolve_credential_secret_with_manager(
            store,
            manager,
            &tenant_id,
            &provider_id,
            &key_reference,
        )
        .await
        .with_context(|| {
            format!(
                "credential validation failed for tenant={} provider={} key_reference={}",
                tenant_id, provider_id, key_reference
            )
        })
    }))
    .buffer_unordered(8)
    .try_for_each(|_| async { Ok(()) })
    .await
}

static NEXT_EXTENSION_RUNTIME_ROLLOUT_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_STANDALONE_CONFIG_ROLLOUT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateExtensionRuntimeRolloutRequest {
    pub scope: ConfiguredExtensionHostReloadScope,
    pub requested_extension_id: Option<String>,
    pub requested_instance_id: Option<String>,
    pub resolved_extension_id: Option<String>,
    pub timeout_secs: u64,
}

impl CreateExtensionRuntimeRolloutRequest {
    pub fn new(scope: ConfiguredExtensionHostReloadScope, timeout_secs: u64) -> Self {
        let (requested_extension_id, requested_instance_id, resolved_extension_id) =
            rollout_request_fields_from_scope(&scope);

        Self {
            scope,
            requested_extension_id,
            requested_instance_id,
            resolved_extension_id,
            timeout_secs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionRuntimeRolloutDetails {
    pub rollout_id: String,
    pub status: String,
    pub scope: String,
    pub requested_extension_id: Option<String>,
    pub requested_instance_id: Option<String>,
    pub resolved_extension_id: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
    pub participant_count: usize,
    pub participants: Vec<ExtensionRuntimeRolloutParticipantRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateStandaloneConfigRolloutRequest {
    pub requested_service_kind: Option<String>,
    pub timeout_secs: u64,
}

impl CreateStandaloneConfigRolloutRequest {
    pub fn new(requested_service_kind: Option<String>, timeout_secs: u64) -> Self {
        Self {
            requested_service_kind,
            timeout_secs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfigRolloutDetails {
    pub rollout_id: String,
    pub status: String,
    pub requested_service_kind: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
    pub participant_count: usize,
    pub participants: Vec<StandaloneConfigRolloutParticipantRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StandaloneRuntimeReloadOutcome {
    NoChange { message: String },
    Applied { message: String },
    RestartRequired { message: String },
}

impl StandaloneRuntimeReloadOutcome {
    fn no_change() -> Self {
        Self::NoChange {
            message: "no effective config changes detected".to_owned(),
        }
    }

    fn applied(message: String) -> Self {
        Self::Applied { message }
    }

    fn restart_required(message: String) -> Self {
        Self::RestartRequired { message }
    }

    fn message(&self) -> &str {
        match self {
            Self::NoChange { message }
            | Self::Applied { message }
            | Self::RestartRequired { message } => message,
        }
    }

    fn requires_restart(&self) -> bool {
        matches!(self, Self::RestartRequired { .. })
    }
}

pub async fn create_extension_runtime_rollout(
    store: &dyn AdminStore,
    created_by: &str,
    scope: ConfiguredExtensionHostReloadScope,
    timeout_secs: u64,
) -> Result<ExtensionRuntimeRolloutDetails> {
    create_extension_runtime_rollout_with_request(
        store,
        created_by,
        CreateExtensionRuntimeRolloutRequest::new(scope, timeout_secs),
    )
    .await
}

pub async fn create_extension_runtime_rollout_with_request(
    store: &dyn AdminStore,
    created_by: &str,
    request: CreateExtensionRuntimeRolloutRequest,
) -> Result<ExtensionRuntimeRolloutDetails> {
    let now_ms = unix_timestamp_ms();
    let active_nodes = resolve_active_extension_runtime_rollout_nodes(store, now_ms).await?;
    if active_nodes.is_empty() {
        anyhow::bail!("no active gateway or admin nodes available for extension runtime rollout");
    }

    let timeout_secs = normalize_extension_runtime_rollout_timeout_secs(request.timeout_secs);
    let rollout = ExtensionRuntimeRolloutRecord::new(
        next_extension_runtime_rollout_id(now_ms),
        rollout_scope_name(&request.scope),
        request.requested_extension_id,
        request.requested_instance_id,
        request.resolved_extension_id,
        created_by,
        now_ms,
        now_ms.saturating_add(timeout_secs.saturating_mul(1_000)),
    );
    store.insert_extension_runtime_rollout(&rollout).await?;

    let mut participants = Vec::with_capacity(active_nodes.len());
    for node in active_nodes {
        let participant = ExtensionRuntimeRolloutParticipantRecord::new(
            rollout.rollout_id.clone(),
            node.node_id,
            node.service_kind,
            "pending",
            now_ms,
        );
        store
            .insert_extension_runtime_rollout_participant(&participant)
            .await?;
        participants.push(participant);
    }

    Ok(build_extension_runtime_rollout_details(
        rollout,
        participants,
        now_ms,
    ))
}

pub async fn list_extension_runtime_rollouts(
    store: &dyn AdminStore,
) -> Result<Vec<ExtensionRuntimeRolloutDetails>> {
    let now_ms = unix_timestamp_ms();
    let rollouts = store.list_extension_runtime_rollouts().await?;
    let mut details = Vec::with_capacity(rollouts.len());

    for rollout in rollouts {
        let participants = store
            .list_extension_runtime_rollout_participants(&rollout.rollout_id)
            .await?;
        details.push(build_extension_runtime_rollout_details(
            rollout,
            participants,
            now_ms,
        ));
    }

    Ok(details)
}

pub async fn find_extension_runtime_rollout(
    store: &dyn AdminStore,
    rollout_id: &str,
) -> Result<Option<ExtensionRuntimeRolloutDetails>> {
    let Some(rollout) = store.find_extension_runtime_rollout(rollout_id).await? else {
        return Ok(None);
    };
    let participants = store
        .list_extension_runtime_rollout_participants(&rollout.rollout_id)
        .await?;

    Ok(Some(build_extension_runtime_rollout_details(
        rollout,
        participants,
        unix_timestamp_ms(),
    )))
}

pub async fn create_standalone_config_rollout(
    store: &dyn AdminStore,
    created_by: &str,
    request: CreateStandaloneConfigRolloutRequest,
) -> Result<StandaloneConfigRolloutDetails> {
    let now_ms = unix_timestamp_ms();
    let active_nodes = resolve_active_standalone_config_rollout_nodes(
        store,
        &request.requested_service_kind,
        now_ms,
    )
    .await?;
    if active_nodes.is_empty() {
        anyhow::bail!("no active standalone nodes available for standalone config rollout");
    }

    let timeout_secs = normalize_standalone_config_rollout_timeout_secs(request.timeout_secs);
    let rollout = StandaloneConfigRolloutRecord::new(
        next_standalone_config_rollout_id(now_ms),
        request.requested_service_kind,
        created_by,
        now_ms,
        now_ms.saturating_add(timeout_secs.saturating_mul(1_000)),
    );
    store.insert_standalone_config_rollout(&rollout).await?;

    let mut participants = Vec::with_capacity(active_nodes.len());
    for node in active_nodes {
        let participant = StandaloneConfigRolloutParticipantRecord::new(
            rollout.rollout_id.clone(),
            node.node_id,
            node.service_kind,
            "pending",
            now_ms,
        );
        store
            .insert_standalone_config_rollout_participant(&participant)
            .await?;
        participants.push(participant);
    }

    Ok(build_standalone_config_rollout_details(
        rollout,
        participants,
        now_ms,
    ))
}

pub async fn list_standalone_config_rollouts(
    store: &dyn AdminStore,
) -> Result<Vec<StandaloneConfigRolloutDetails>> {
    let now_ms = unix_timestamp_ms();
    let rollouts = store.list_standalone_config_rollouts().await?;
    let mut details = Vec::with_capacity(rollouts.len());

    for rollout in rollouts {
        let participants = store
            .list_standalone_config_rollout_participants(&rollout.rollout_id)
            .await?;
        details.push(build_standalone_config_rollout_details(
            rollout,
            participants,
            now_ms,
        ));
    }

    Ok(details)
}

pub async fn find_standalone_config_rollout(
    store: &dyn AdminStore,
    rollout_id: &str,
) -> Result<Option<StandaloneConfigRolloutDetails>> {
    let Some(rollout) = store.find_standalone_config_rollout(rollout_id).await? else {
        return Ok(None);
    };
    let participants = store
        .list_standalone_config_rollout_participants(&rollout.rollout_id)
        .await?;

    Ok(Some(build_standalone_config_rollout_details(
        rollout,
        participants,
        unix_timestamp_ms(),
    )))
}

pub fn start_extension_runtime_rollout_supervision(
    service_kind: StandaloneServiceKind,
    node_id: impl Into<String>,
    live_store: Reloadable<Arc<dyn AdminStore>>,
) -> Result<JoinHandle<()>> {
    if !service_kind.supports_runtime_dynamic() {
        anyhow::bail!(
            "standalone service does not participate in extension runtime rollouts: {}",
            service_kind.as_str()
        );
    }

    let node_id = node_id.into();
    let started_at_ms = unix_timestamp_ms();

    Ok(tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(
            EXTENSION_RUNTIME_ROLLOUT_POLL_INTERVAL_MS,
        ));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            let store = live_store.snapshot();
            let heartbeat = ServiceRuntimeNodeRecord::new(
                node_id.clone(),
                service_kind.as_str(),
                started_at_ms,
            )
            .with_last_seen_at_ms(unix_timestamp_ms());
            if let Err(error) = store.upsert_service_runtime_node(&heartbeat).await {
                eprintln!(
                    "extension runtime rollout heartbeat failed: service={} node_id={} error={error}",
                    service_kind.as_str(),
                    node_id
                );
                continue;
            }

            if let Err(error) =
                process_extension_runtime_rollout_work(store.as_ref(), service_kind, &node_id).await
            {
                eprintln!(
                    "extension runtime rollout processing failed: service={} node_id={} error={error}",
                    service_kind.as_str(),
                    node_id
                );
            }
        }
    }))
}

pub fn resolve_service_runtime_node_id(service_kind: StandaloneServiceKind) -> String {
    if let Ok(node_id) = std::env::var("SDKWORK_SERVICE_INSTANCE_ID") {
        let node_id = node_id.trim();
        if !node_id.is_empty() {
            return node_id.to_owned();
        }
    }

    format!(
        "{}-{}-{}",
        service_kind.as_str(),
        std::process::id(),
        unix_timestamp_ms()
    )
}

async fn process_extension_runtime_rollout_work(
    store: &dyn AdminStore,
    service_kind: StandaloneServiceKind,
    node_id: &str,
) -> Result<()> {
    let participants = store
        .list_pending_extension_runtime_rollout_participants_for_node(node_id)
        .await?;

    for participant in participants {
        if participant.service_kind != service_kind.as_str() {
            continue;
        }

        let Some(rollout) = store
            .find_extension_runtime_rollout(&participant.rollout_id)
            .await?
        else {
            continue;
        };

        if rollout.deadline_at_ms <= unix_timestamp_ms() {
            continue;
        }

        let applying_at_ms = unix_timestamp_ms();
        if !store
            .transition_extension_runtime_rollout_participant(
                &participant.rollout_id,
                node_id,
                "pending",
                "applying",
                None,
                applying_at_ms,
            )
            .await?
        {
            continue;
        }

        let completed_at_ms = unix_timestamp_ms();
        match apply_extension_runtime_rollout(&rollout) {
            Ok(()) => {
                store
                    .transition_extension_runtime_rollout_participant(
                        &participant.rollout_id,
                        node_id,
                        "applying",
                        "succeeded",
                        None,
                        completed_at_ms,
                    )
                    .await?;
            }
            Err(error) => {
                let message = error.to_string();
                store
                    .transition_extension_runtime_rollout_participant(
                        &participant.rollout_id,
                        node_id,
                        "applying",
                        "failed",
                        Some(message.as_str()),
                        completed_at_ms,
                    )
                    .await?;
            }
        }

        return Ok(());
    }

    Ok(())
}

fn apply_extension_runtime_rollout(rollout: &ExtensionRuntimeRolloutRecord) -> Result<()> {
    let scope = rollout_gateway_scope(rollout)?;
    reload_extension_host_with_scope(&scope)
        .map(|_| ())
        .with_context(|| {
            format!(
                "failed to apply extension runtime rollout {}",
                rollout.rollout_id
            )
        })
}

async fn resolve_active_extension_runtime_rollout_nodes(
    store: &dyn AdminStore,
    now_ms: u64,
) -> Result<Vec<ServiceRuntimeNodeRecord>> {
    let active_after_ms = now_ms.saturating_sub(EXTENSION_RUNTIME_ROLLOUT_NODE_FRESHNESS_WINDOW_MS);

    Ok(store
        .list_service_runtime_nodes()
        .await?
        .into_iter()
        .filter(|node| matches!(node.service_kind.as_str(), "gateway" | "admin"))
        .filter(|node| node.last_seen_at_ms >= active_after_ms)
        .collect())
}

fn build_extension_runtime_rollout_details(
    rollout: ExtensionRuntimeRolloutRecord,
    participants: Vec<ExtensionRuntimeRolloutParticipantRecord>,
    now_ms: u64,
) -> ExtensionRuntimeRolloutDetails {
    let status = aggregate_extension_runtime_rollout_status(&rollout, &participants, now_ms);

    ExtensionRuntimeRolloutDetails {
        rollout_id: rollout.rollout_id,
        status,
        scope: rollout.scope,
        requested_extension_id: rollout.requested_extension_id,
        requested_instance_id: rollout.requested_instance_id,
        resolved_extension_id: rollout.resolved_extension_id,
        created_by: rollout.created_by,
        created_at_ms: rollout.created_at_ms,
        deadline_at_ms: rollout.deadline_at_ms,
        participant_count: participants.len(),
        participants,
    }
}

fn aggregate_extension_runtime_rollout_status(
    rollout: &ExtensionRuntimeRolloutRecord,
    participants: &[ExtensionRuntimeRolloutParticipantRecord],
    now_ms: u64,
) -> String {
    if !participants.is_empty()
        && participants
            .iter()
            .all(|participant| participant.status == "succeeded")
    {
        return "succeeded".to_owned();
    }

    if participants
        .iter()
        .any(|participant| participant.status == "failed")
    {
        return "failed".to_owned();
    }

    if rollout.deadline_at_ms <= now_ms
        && participants
            .iter()
            .any(|participant| matches!(participant.status.as_str(), "pending" | "applying"))
    {
        return "timed_out".to_owned();
    }

    if participants
        .iter()
        .any(|participant| participant.status == "applying")
    {
        return "applying".to_owned();
    }

    "pending".to_owned()
}

async fn reload_standalone_runtime_config_pass(
    service_kind: StandaloneServiceKind,
    config_loader: &StandaloneConfigLoader,
    reload_handles: &StandaloneServiceReloadHandles,
    state: &mut StandaloneRuntimeState,
    force_reload: bool,
) -> Result<StandaloneRuntimeReloadOutcome> {
    let next_watch_state = config_loader.watch_state()?;
    if let Some(pending_restart) = state.pending_restart_required.as_ref() {
        if pending_restart.watch_state == next_watch_state {
            return Ok(StandaloneRuntimeReloadOutcome::restart_required(
                pending_restart.message.clone(),
            ));
        }
    }
    if !force_reload && state.previous_watch_state.as_ref() == Some(&next_watch_state) {
        return Ok(StandaloneRuntimeReloadOutcome::no_change());
    }

    let next_config = config_loader.reload()?;
    let restart_required_changes =
        restart_required_changed_fields(service_kind, &state.current_config, &next_config);
    let restart_required_message = (!restart_required_changes.is_empty()).then(|| {
        format!(
            "restart required for {}",
            restart_required_changes.join(", ")
        )
    });

    let next_dynamic = next_config.runtime_dynamic_config();
    let bind_changed = service_bind(service_kind, &state.current_config)
        != service_bind(service_kind, &next_config);
    let database_changed = state.current_config.database_url != next_config.database_url;
    let admin_jwt_changed = service_kind == StandaloneServiceKind::Admin
        && state.current_config.admin_jwt_signing_secret != next_config.admin_jwt_signing_secret;
    let portal_jwt_changed = service_kind == StandaloneServiceKind::Portal
        && state.current_config.portal_jwt_signing_secret != next_config.portal_jwt_signing_secret;
    let secret_manager_changed = service_kind != StandaloneServiceKind::Portal
        && secret_manager_config_changed(&state.current_config, &next_config);
    let dynamic_changed =
        service_kind.supports_runtime_dynamic() && next_dynamic != state.current_dynamic;

    if !database_changed
        && !admin_jwt_changed
        && !portal_jwt_changed
        && !secret_manager_changed
        && !bind_changed
        && !dynamic_changed
    {
        state.previous_watch_state = Some(next_watch_state.clone());
        update_pending_restart_required(
            state,
            next_watch_state,
            restart_required_message.as_deref(),
        );
        return Ok(match restart_required_message {
            Some(message) => StandaloneRuntimeReloadOutcome::restart_required(message),
            None => StandaloneRuntimeReloadOutcome::no_change(),
        });
    }

    let prepared_store_handles = if database_changed {
        Some(build_admin_payment_store_handles_from_config(&next_config).await?)
    } else {
        None
    };

    let prepared_listener = if bind_changed {
        let listener = reload_handles.listener.as_ref().with_context(|| {
            format!(
                "runtime config reload failed because no listener handle is configured for bind change: service={} bind={}",
                service_kind.as_str(),
                service_bind(service_kind, &next_config),
            )
        })?;

        listener
            .prepare_rebind(service_bind(service_kind, &next_config))
            .await?
    } else {
        None
    };

    let prepared_secret_manager = if secret_manager_changed {
        let _ = reload_handles.secret_manager.as_ref().with_context(|| {
            format!(
                "runtime config reload failed because no secret manager handle is configured for secret-manager change: service={}",
                service_kind.as_str(),
            )
        })?;

        let next_secret_manager = build_secret_manager_from_config(&next_config);
        let validation_store = prepared_store_handles
            .as_ref()
            .map(|handles| handles.admin_store.as_ref())
            .unwrap_or(state.current_store.as_ref());
        validate_secret_manager_for_store(validation_store, &next_secret_manager).await?;
        Some(next_secret_manager)
    } else {
        None
    };

    let extension_policy_changed = service_kind.supports_runtime_dynamic()
        && extension_runtime_policy_changed(&state.current_dynamic, &next_dynamic);
    if extension_policy_changed {
        let policy = extension_discovery_policy_from_config(&next_dynamic);
        reload_extension_host_with_policy(&policy)?;
    }

    if dynamic_changed {
        next_dynamic.apply_to_process_env();
    }

    if let Some(next_store_handles) = prepared_store_handles {
        state.current_store = next_store_handles.admin_store.clone();
        reload_handles.store.replace(next_store_handles.admin_store);
        if let Some(live_payment_store) = reload_handles.payment_store.as_ref() {
            live_payment_store.replace(next_store_handles.payment_store);
        }
    }

    if admin_jwt_changed {
        if let Some(live_jwt) = reload_handles.admin_jwt_signing_secret.as_ref() {
            live_jwt.replace(next_config.admin_jwt_signing_secret.clone());
        }
    }

    if portal_jwt_changed {
        if let Some(live_jwt) = reload_handles.portal_jwt_signing_secret.as_ref() {
            live_jwt.replace(next_config.portal_jwt_signing_secret.clone());
        }
    }

    if let Some(next_secret_manager) = prepared_secret_manager {
        if let Some(live_secret_manager) = reload_handles.secret_manager.as_ref() {
            live_secret_manager.replace(next_secret_manager);
        }
    }

    if let Some(prepared_listener) = prepared_listener {
        prepared_listener.activate();
    }

    if service_kind.supports_runtime_dynamic()
        && (database_changed
            || state.current_dynamic.runtime_snapshot_interval_secs
                != next_dynamic.runtime_snapshot_interval_secs)
    {
        state
            .snapshot_supervision
            .replace(start_provider_health_snapshot_supervision(
                state.current_store.clone(),
                next_dynamic.runtime_snapshot_interval_secs,
            ));
    }

    if service_kind.supports_runtime_dynamic()
        && (extension_policy_changed
            || state.current_dynamic.extension_hot_reload_interval_secs
                != next_dynamic.extension_hot_reload_interval_secs)
    {
        state.extension_hot_reload_supervision.replace(
            start_configured_extension_hot_reload_supervision(
                next_dynamic.extension_hot_reload_interval_secs,
            ),
        );
    }

    eprintln!(
        "runtime config reload applied: service={} bind_changed={} database_changed={} admin_jwt_changed={} portal_jwt_changed={} secret_manager_changed={} extension_policy_changed={} runtime_snapshot_interval_secs={} extension_hot_reload_interval_secs={} native_dynamic_shutdown_drain_timeout_ms={}",
        service_kind.as_str(),
        bind_changed,
        database_changed,
        admin_jwt_changed,
        portal_jwt_changed,
        secret_manager_changed,
        extension_policy_changed,
        next_dynamic.runtime_snapshot_interval_secs,
        next_dynamic.extension_hot_reload_interval_secs,
        next_dynamic.native_dynamic_shutdown_drain_timeout_ms
    );

    let applied_message = format!(
        "runtime config reload applied: bind_changed={bind_changed} database_changed={database_changed} admin_jwt_changed={admin_jwt_changed} portal_jwt_changed={portal_jwt_changed} secret_manager_changed={secret_manager_changed} extension_policy_changed={extension_policy_changed}"
    );
    let applied_config =
        merge_applied_service_config(service_kind, &state.current_config, &next_config);
    state.current_config = applied_config.clone();
    state.current_dynamic = applied_config.runtime_dynamic_config();
    state.previous_watch_state = Some(next_watch_state.clone());
    update_pending_restart_required(state, next_watch_state, restart_required_message.as_deref());

    Ok(match restart_required_message {
        Some(message) => StandaloneRuntimeReloadOutcome::restart_required(format!(
            "{applied_message}; {message}"
        )),
        None => StandaloneRuntimeReloadOutcome::applied(applied_message),
    })
}

async fn process_standalone_config_rollout_work(
    coordination_store: &dyn AdminStore,
    service_kind: StandaloneServiceKind,
    node_id: &str,
    config_loader: &StandaloneConfigLoader,
    reload_handles: &StandaloneServiceReloadHandles,
    state: &mut StandaloneRuntimeState,
) -> Result<()> {
    let participants = coordination_store
        .list_pending_standalone_config_rollout_participants_for_node(node_id)
        .await?;

    for participant in participants {
        if participant.service_kind != service_kind.as_str() {
            continue;
        }

        let Some(rollout) = coordination_store
            .find_standalone_config_rollout(&participant.rollout_id)
            .await?
        else {
            continue;
        };

        if rollout.deadline_at_ms <= unix_timestamp_ms() {
            continue;
        }

        let applying_at_ms = unix_timestamp_ms();
        if !coordination_store
            .transition_standalone_config_rollout_participant(
                &participant.rollout_id,
                node_id,
                "pending",
                "applying",
                None,
                applying_at_ms,
            )
            .await?
        {
            continue;
        }

        let completed_at_ms = unix_timestamp_ms();
        match reload_standalone_runtime_config_pass(
            service_kind,
            config_loader,
            reload_handles,
            state,
            true,
        )
        .await
        {
            Ok(outcome) => {
                let next_status = if outcome.requires_restart() {
                    "failed"
                } else {
                    "succeeded"
                };
                coordination_store
                    .transition_standalone_config_rollout_participant(
                        &participant.rollout_id,
                        node_id,
                        "applying",
                        next_status,
                        Some(outcome.message()),
                        completed_at_ms,
                    )
                    .await?;
            }
            Err(error) => {
                let message = error.to_string();
                coordination_store
                    .transition_standalone_config_rollout_participant(
                        &participant.rollout_id,
                        node_id,
                        "applying",
                        "failed",
                        Some(message.as_str()),
                        completed_at_ms,
                    )
                    .await?;
            }
        }

        return Ok(());
    }

    Ok(())
}

async fn resolve_active_standalone_config_rollout_nodes(
    store: &dyn AdminStore,
    requested_service_kind: &Option<String>,
    now_ms: u64,
) -> Result<Vec<ServiceRuntimeNodeRecord>> {
    let active_after_ms = now_ms.saturating_sub(STANDALONE_CONFIG_ROLLOUT_NODE_FRESHNESS_WINDOW_MS);

    Ok(store
        .list_service_runtime_nodes()
        .await?
        .into_iter()
        .filter(|node| matches!(node.service_kind.as_str(), "gateway" | "admin" | "portal"))
        .filter(|node| node.last_seen_at_ms >= active_after_ms)
        .filter(|node| match requested_service_kind.as_deref() {
            Some(requested_service_kind) => node.service_kind == requested_service_kind,
            None => true,
        })
        .collect())
}

fn build_standalone_config_rollout_details(
    rollout: StandaloneConfigRolloutRecord,
    participants: Vec<StandaloneConfigRolloutParticipantRecord>,
    now_ms: u64,
) -> StandaloneConfigRolloutDetails {
    let status = aggregate_standalone_config_rollout_status(&rollout, &participants, now_ms);

    StandaloneConfigRolloutDetails {
        rollout_id: rollout.rollout_id,
        status,
        requested_service_kind: rollout.requested_service_kind,
        created_by: rollout.created_by,
        created_at_ms: rollout.created_at_ms,
        deadline_at_ms: rollout.deadline_at_ms,
        participant_count: participants.len(),
        participants,
    }
}

fn aggregate_standalone_config_rollout_status(
    rollout: &StandaloneConfigRolloutRecord,
    participants: &[StandaloneConfigRolloutParticipantRecord],
    now_ms: u64,
) -> String {
    if !participants.is_empty()
        && participants
            .iter()
            .all(|participant| participant.status == "succeeded")
    {
        return "succeeded".to_owned();
    }

    if participants
        .iter()
        .any(|participant| participant.status == "failed")
    {
        return "failed".to_owned();
    }

    if rollout.deadline_at_ms <= now_ms
        && participants
            .iter()
            .any(|participant| matches!(participant.status.as_str(), "pending" | "applying"))
    {
        return "timed_out".to_owned();
    }

    if participants
        .iter()
        .any(|participant| participant.status == "applying")
    {
        return "applying".to_owned();
    }

    "pending".to_owned()
}

fn rollout_gateway_scope(
    rollout: &ExtensionRuntimeRolloutRecord,
) -> Result<ConfiguredExtensionHostReloadScope> {
    match rollout.scope.as_str() {
        "all" => Ok(ConfiguredExtensionHostReloadScope::All),
        "extension" => {
            let extension_id = rollout
                .resolved_extension_id
                .clone()
                .or_else(|| rollout.requested_extension_id.clone())
                .context("extension rollout is missing a resolved extension id")?;
            Ok(ConfiguredExtensionHostReloadScope::Extension { extension_id })
        }
        "instance" => {
            let instance_id = rollout
                .requested_instance_id
                .clone()
                .context("instance rollout is missing a requested instance id")?;
            Ok(ConfiguredExtensionHostReloadScope::Instance { instance_id })
        }
        other => anyhow::bail!("unsupported extension runtime rollout scope: {other}"),
    }
}

fn rollout_request_fields_from_scope(
    scope: &ConfiguredExtensionHostReloadScope,
) -> (Option<String>, Option<String>, Option<String>) {
    match scope {
        ConfiguredExtensionHostReloadScope::All => (None, None, None),
        ConfiguredExtensionHostReloadScope::Extension { extension_id } => {
            (Some(extension_id.clone()), None, Some(extension_id.clone()))
        }
        ConfiguredExtensionHostReloadScope::Instance { instance_id } => {
            (None, Some(instance_id.clone()), None)
        }
    }
}

fn rollout_scope_name(scope: &ConfiguredExtensionHostReloadScope) -> &'static str {
    match scope {
        ConfiguredExtensionHostReloadScope::All => "all",
        ConfiguredExtensionHostReloadScope::Extension { .. } => "extension",
        ConfiguredExtensionHostReloadScope::Instance { .. } => "instance",
    }
}

fn next_extension_runtime_rollout_id(now_ms: u64) -> String {
    let sequence = NEXT_EXTENSION_RUNTIME_ROLLOUT_ID.fetch_add(1, Ordering::SeqCst);
    format!("runtime-rollout-{now_ms}-{sequence}")
}

fn normalize_extension_runtime_rollout_timeout_secs(timeout_secs: u64) -> u64 {
    if timeout_secs == 0 {
        DEFAULT_EXTENSION_RUNTIME_ROLLOUT_TIMEOUT_SECS
    } else {
        timeout_secs
    }
}

fn next_standalone_config_rollout_id(now_ms: u64) -> String {
    let sequence = NEXT_STANDALONE_CONFIG_ROLLOUT_ID.fetch_add(1, Ordering::SeqCst);
    format!("config-rollout-{now_ms}-{sequence}")
}

fn normalize_standalone_config_rollout_timeout_secs(timeout_secs: u64) -> u64 {
    if timeout_secs == 0 {
        DEFAULT_STANDALONE_CONFIG_ROLLOUT_TIMEOUT_SECS
    } else {
        timeout_secs
    }
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

pub fn start_standalone_runtime_supervision(
    service_kind: StandaloneServiceKind,
    config_loader: StandaloneConfigLoader,
    initial_config: StandaloneConfig,
    reload_handles: StandaloneServiceReloadHandles,
) -> StandaloneRuntimeSupervision {
    let initial_watch_state = match config_loader.watch_state() {
        Ok(state) => Some(state),
        Err(error) => {
            eprintln!("runtime config watch startup state capture failed: {error}");
            None
        }
    };
    let started_at_ms = unix_timestamp_ms();
    let coordination_store = reload_handles
        .coordination_store
        .clone()
        .unwrap_or_else(|| reload_handles.store.clone());

    StandaloneRuntimeSupervision {
        join_handle: tokio::spawn(async move {
            let current_dynamic = initial_config.runtime_dynamic_config();
            let current_store = reload_handles.store.snapshot();
            let snapshot_supervision =
                AbortOnDropHandle::new(if service_kind.supports_runtime_dynamic() {
                    start_provider_health_snapshot_supervision(
                        current_store.clone(),
                        current_dynamic.runtime_snapshot_interval_secs,
                    )
                } else {
                    None
                });
            let extension_hot_reload_supervision =
                AbortOnDropHandle::new(if service_kind.supports_runtime_dynamic() {
                    start_configured_extension_hot_reload_supervision(
                        current_dynamic.extension_hot_reload_interval_secs,
                    )
                } else {
                    None
                });
            let mut state = StandaloneRuntimeState {
                current_config: initial_config,
                current_dynamic,
                current_store,
                snapshot_supervision,
                extension_hot_reload_supervision,
                previous_watch_state: initial_watch_state,
                pending_restart_required: None,
            };

            let mut interval =
                tokio::time::interval(Duration::from_secs(CONFIG_RELOAD_POLL_INTERVAL_SECS));
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
            interval.tick().await;

            loop {
                interval.tick().await;

                if let Some(node_id) = reload_handles.node_id.as_deref() {
                    let coordination_store = coordination_store.snapshot();
                    let heartbeat = ServiceRuntimeNodeRecord::new(
                        node_id,
                        service_kind.as_str(),
                        started_at_ms,
                    )
                    .with_last_seen_at_ms(unix_timestamp_ms());
                    if let Err(error) = coordination_store
                        .upsert_service_runtime_node(&heartbeat)
                        .await
                    {
                        eprintln!(
                            "standalone config rollout heartbeat failed: service={} node_id={} error={error}",
                            service_kind.as_str(),
                            node_id,
                        );
                    }

                    if let Err(error) = process_standalone_config_rollout_work(
                        coordination_store.as_ref(),
                        service_kind,
                        node_id,
                        &config_loader,
                        &reload_handles,
                        &mut state,
                    )
                    .await
                    {
                        eprintln!(
                            "standalone config rollout processing failed: service={} node_id={} error={error}",
                            service_kind.as_str(),
                            node_id,
                        );
                    }
                }

                if let Err(error) = reload_standalone_runtime_config_pass(
                    service_kind,
                    &config_loader,
                    &reload_handles,
                    &mut state,
                    false,
                )
                .await
                {
                    eprintln!("runtime config reload failed: {error}");
                }
            }
        }),
    }
}

struct AbortOnDropHandle(Option<JoinHandle<()>>);

impl AbortOnDropHandle {
    fn new(handle: Option<JoinHandle<()>>) -> Self {
        Self(handle)
    }

    fn replace(&mut self, handle: Option<JoinHandle<()>>) {
        self.abort();
        self.0 = handle;
    }

    fn abort(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
        }
    }
}

impl Drop for AbortOnDropHandle {
    fn drop(&mut self) {
        self.abort();
    }
}

fn restart_required_changed_fields(
    service_kind: StandaloneServiceKind,
    current: &StandaloneConfig,
    next: &StandaloneConfig,
) -> Vec<&'static str> {
    current
        .non_reloadable_changed_fields(next)
        .into_iter()
        .filter(|field| service_relevant_field(service_kind, field))
        .filter(|field| !service_reloadable_field(service_kind, field))
        .collect()
}

fn service_relevant_field(service_kind: StandaloneServiceKind, field: &str) -> bool {
    match field {
        "gateway_bind" => service_kind == StandaloneServiceKind::Gateway,
        "admin_bind" => service_kind == StandaloneServiceKind::Admin,
        "portal_bind" => service_kind == StandaloneServiceKind::Portal,
        "database_url" => true,
        "cache_backend" | "cache_url" => service_kind != StandaloneServiceKind::Portal,
        "admin_jwt_signing_secret" => service_kind == StandaloneServiceKind::Admin,
        "portal_jwt_signing_secret" => service_kind == StandaloneServiceKind::Portal,
        "secret_backend"
        | "credential_master_key"
        | "credential_legacy_master_keys"
        | "secret_local_file"
        | "secret_keyring_service" => service_kind != StandaloneServiceKind::Portal,
        _ => false,
    }
}

fn service_reloadable_field(service_kind: StandaloneServiceKind, field: &str) -> bool {
    match field {
        "gateway_bind" => service_kind == StandaloneServiceKind::Gateway,
        "admin_bind" => service_kind == StandaloneServiceKind::Admin,
        "portal_bind" => service_kind == StandaloneServiceKind::Portal,
        "database_url" => true,
        "admin_jwt_signing_secret" => service_kind == StandaloneServiceKind::Admin,
        "portal_jwt_signing_secret" => service_kind == StandaloneServiceKind::Portal,
        "secret_backend"
        | "credential_master_key"
        | "credential_legacy_master_keys"
        | "secret_local_file"
        | "secret_keyring_service" => service_kind != StandaloneServiceKind::Portal,
        _ => false,
    }
}

fn secret_manager_config_changed(current: &StandaloneConfig, next: &StandaloneConfig) -> bool {
    current.secret_backend != next.secret_backend
        || current.credential_master_key != next.credential_master_key
        || current.credential_legacy_master_keys != next.credential_legacy_master_keys
        || current.secret_local_file != next.secret_local_file
        || current.secret_keyring_service != next.secret_keyring_service
}

fn service_bind(service_kind: StandaloneServiceKind, config: &StandaloneConfig) -> &str {
    match service_kind {
        StandaloneServiceKind::Gateway => &config.gateway_bind,
        StandaloneServiceKind::Admin => &config.admin_bind,
        StandaloneServiceKind::Portal => &config.portal_bind,
    }
}

fn merge_applied_service_config(
    service_kind: StandaloneServiceKind,
    current: &StandaloneConfig,
    next: &StandaloneConfig,
) -> StandaloneConfig {
    let mut applied = current.clone();

    match service_kind {
        StandaloneServiceKind::Gateway => {
            applied.gateway_bind = next.gateway_bind.clone();
        }
        StandaloneServiceKind::Admin => {
            applied.admin_bind = next.admin_bind.clone();
            applied.admin_jwt_signing_secret = next.admin_jwt_signing_secret.clone();
        }
        StandaloneServiceKind::Portal => {
            applied.portal_bind = next.portal_bind.clone();
            applied.portal_jwt_signing_secret = next.portal_jwt_signing_secret.clone();
        }
    }

    applied.database_url = next.database_url.clone();

    if service_kind.supports_runtime_dynamic() {
        applied.extension_paths = next.extension_paths.clone();
        applied.enable_connector_extensions = next.enable_connector_extensions;
        applied.enable_native_dynamic_extensions = next.enable_native_dynamic_extensions;
        applied.extension_hot_reload_interval_secs = next.extension_hot_reload_interval_secs;
        applied.extension_trusted_signers = next.extension_trusted_signers.clone();
        applied.require_signed_connector_extensions = next.require_signed_connector_extensions;
        applied.require_signed_native_dynamic_extensions =
            next.require_signed_native_dynamic_extensions;
        applied.native_dynamic_shutdown_drain_timeout_ms =
            next.native_dynamic_shutdown_drain_timeout_ms;
        applied.runtime_snapshot_interval_secs = next.runtime_snapshot_interval_secs;
    }

    if service_kind != StandaloneServiceKind::Portal {
        applied.secret_backend = next.secret_backend;
        applied.credential_master_key = next.credential_master_key.clone();
        applied.credential_legacy_master_keys = next.credential_legacy_master_keys.clone();
        applied.secret_local_file = next.secret_local_file.clone();
        applied.secret_keyring_service = next.secret_keyring_service.clone();
    }

    applied
}

fn update_pending_restart_required(
    state: &mut StandaloneRuntimeState,
    watch_state: StandaloneConfigWatchState,
    message: Option<&str>,
) {
    match message {
        Some(message) => {
            eprintln!("runtime config reload requires restart: {message}");
            state.pending_restart_required = Some(PendingStandaloneRuntimeRestartRequired {
                watch_state,
                message: message.to_owned(),
            });
        }
        None => {
            state.pending_restart_required = None;
        }
    }
}

fn extension_runtime_policy_changed(
    current: &StandaloneRuntimeDynamicConfig,
    next: &StandaloneRuntimeDynamicConfig,
) -> bool {
    current.extension_paths != next.extension_paths
        || current.enable_connector_extensions != next.enable_connector_extensions
        || current.enable_native_dynamic_extensions != next.enable_native_dynamic_extensions
        || current.extension_trusted_signers != next.extension_trusted_signers
        || current.require_signed_connector_extensions != next.require_signed_connector_extensions
        || current.require_signed_native_dynamic_extensions
            != next.require_signed_native_dynamic_extensions
}

fn extension_discovery_policy_from_config(
    config: &StandaloneRuntimeDynamicConfig,
) -> ExtensionDiscoveryPolicy {
    let mut policy = ExtensionDiscoveryPolicy::new(
        config
            .extension_paths
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>(),
    )
    .with_connector_extensions(config.enable_connector_extensions)
    .with_native_dynamic_extensions(config.enable_native_dynamic_extensions)
    .with_required_signatures_for_connector_extensions(config.require_signed_connector_extensions)
    .with_required_signatures_for_native_dynamic_extensions(
        config.require_signed_native_dynamic_extensions,
    );
    for (publisher, public_key) in &config.extension_trusted_signers {
        policy = policy.with_trusted_signer(publisher.clone(), public_key.clone());
    }
    policy
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_api_app_credential::persist_credential_with_secret_and_manager;
    use sdkwork_api_config::CacheBackendKind;
    use sdkwork_api_storage_sqlite::{SqliteAdminStore, run_migrations};
    use std::io::Write;

    #[tokio::test]
    async fn validate_secret_manager_for_store_checks_multiple_credentials() {
        let pool = run_migrations("sqlite::memory:").await.unwrap();
        let store = SqliteAdminStore::new(pool);
        let manager = CredentialSecretManager::database_encrypted("runtime-test-master-key");

        persist_credential_with_secret_and_manager(
            &store,
            &manager,
            "tenant-1",
            "provider-a",
            "cred-a",
            "secret-a",
        )
        .await
        .unwrap();
        persist_credential_with_secret_and_manager(
            &store,
            &manager,
            "tenant-2",
            "provider-b",
            "cred-b",
            "secret-b",
        )
        .await
        .unwrap();

        validate_secret_manager_for_store(&store, &manager)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn build_cache_runtime_from_config_returns_memory_cache_runtime() {
        let config = StandaloneConfig::default();

        let stores = build_cache_runtime_from_config(&config).await.unwrap();
        stores
            .cache_store()
            .put("routing", "selection", b"provider-a".to_vec(), None, &[])
            .await
            .unwrap();
        let cached = stores
            .cache_store()
            .get("routing", "selection")
            .await
            .unwrap()
            .expect("cached entry");

        assert_eq!(cached.value(), b"provider-a");
    }

    #[tokio::test]
    async fn build_cache_runtime_from_config_builds_redis_cache_runtime() {
        let mut config = StandaloneConfig::default();
        config.cache_backend = CacheBackendKind::Redis;
        let server = MinimalRedisPingServer::start();
        config.cache_url = Some(server.url_with_db(4));

        build_cache_runtime_from_config(&config).await.unwrap();
    }

    #[tokio::test]
    async fn build_admin_store_from_config_surfaces_supported_dialects_for_mysql() {
        let mut config = StandaloneConfig::default();
        config.database_url = "mysql://router:secret@localhost:3306/router".to_owned();

        let error = match build_admin_store_from_config(&config).await {
            Ok(_) => panic!("mysql should remain unsupported until a real driver ships"),
            Err(error) => error.to_string(),
        };

        assert!(error.contains("mysql"));
        assert!(error.contains("supported dialects"));
        assert!(error.contains("sqlite"));
        assert!(error.contains("postgres"));
    }

    #[tokio::test]
    async fn build_admin_store_from_config_surfaces_supported_dialects_for_libsql() {
        let mut config = StandaloneConfig::default();
        config.database_url = "libsql://router.example.com".to_owned();

        let error = match build_admin_store_from_config(&config).await {
            Ok(_) => panic!("libsql should remain unsupported until a real driver ships"),
            Err(error) => error.to_string(),
        };

        assert!(error.contains("libsql"));
        assert!(error.contains("supported dialects"));
        assert!(error.contains("sqlite"));
        assert!(error.contains("postgres"));
    }

    #[test]
    fn restart_required_changed_fields_include_cache_backend_for_gateway_runtime() {
        let current = StandaloneConfig::default();
        let next = StandaloneConfig {
            cache_backend: CacheBackendKind::Redis,
            cache_url: Some("redis://127.0.0.1:6379/8".to_owned()),
            ..current.clone()
        };

        let changed =
            restart_required_changed_fields(StandaloneServiceKind::Gateway, &current, &next);

        assert!(changed.contains(&"cache_backend"));
        assert!(changed.contains(&"cache_url"));
    }

    #[test]
    fn merge_applied_service_config_keeps_gateway_cache_backend_on_restart_required_changes() {
        let current = StandaloneConfig::default();
        let next = StandaloneConfig {
            gateway_bind: "127.0.0.1:19090".to_owned(),
            cache_backend: CacheBackendKind::Redis,
            cache_url: Some("redis://127.0.0.1:6379/9".to_owned()),
            ..current.clone()
        };

        let applied = merge_applied_service_config(StandaloneServiceKind::Gateway, &current, &next);

        assert_eq!(applied.gateway_bind, "127.0.0.1:19090");
        assert_eq!(applied.cache_backend, CacheBackendKind::Memory);
        assert_eq!(applied.cache_url, None);
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
                                            stream.write_all(b"+PONG\r\n").unwrap();
                                            stream.flush().unwrap();
                                        }
                                        "GET" => {
                                            stream.write_all(b"$-1\r\n").unwrap();
                                            stream.flush().unwrap();
                                        }
                                        "AUTH" | "SELECT" => {
                                            stream.write_all(b"+OK\r\n").unwrap();
                                            stream.flush().unwrap();
                                        }
                                        other => {
                                            panic!("unexpected minimal redis command: {other}")
                                        }
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
                                        break;
                                    }
                                    Err(error) => {
                                        panic!("minimal redis server read failed: {error}")
                                    }
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
}
