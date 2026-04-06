use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountCommerceReconciliationStateRecord, AccountHoldAllocationRecord,
    AccountHoldRecord, AccountLedgerAllocationRecord, AccountLedgerEntryRecord, AccountRecord,
    AccountType, BillingEventRecord, LedgerEntry, PricingPlanRecord, PricingRateRecord,
    QuotaPolicy, RequestSettlementRecord,
};
use sdkwork_api_domain_catalog::{
    Channel, ChannelModelRecord, ModelCatalogEntry, ModelPriceRecord, ProxyProvider,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentEventRecord, ProjectMembershipRecord,
};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::{
    AdminUserRecord, ApiKeyGroupRecord, CanonicalApiKeyRecord, GatewayApiKeyRecord,
    IdentityBindingRecord, IdentityUserRecord, PortalUserRecord,
};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobCallbackRecord, AsyncJobRecord,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CouponCodeRecord, CouponRedemptionRecord, CouponReservationRecord,
    CouponRollbackRecord, CouponTemplateRecord, MarketingCampaignRecord,
    MarketingOutboxEventRecord,
};
use sdkwork_api_domain_rate_limit::{
    RateLimitCheckResult, RateLimitPolicy, RateLimitWindowSnapshot,
};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, ProviderHealthSnapshot,
    RoutingDecisionLog, RoutingPolicy, RoutingProfileRecord,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestMeterMetricRecord, UsageRecord};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_secret_core::SecretEnvelope;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageDialect {
    Sqlite,
    Postgres,
    Mysql,
    Libsql,
}

impl StorageDialect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Libsql => "libsql",
        }
    }
}

#[async_trait]
pub trait StorageDriverFactory<T>: Send + Sync {
    fn dialect(&self) -> StorageDialect;

    fn driver_name(&self) -> &'static str;

    async fn build(&self, database_url: &str) -> Result<T>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponReservationCommand {
    pub template_to_persist: Option<CouponTemplateRecord>,
    pub campaign_to_persist: Option<MarketingCampaignRecord>,
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub reservation: CouponReservationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponReservationResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub reservation: CouponReservationRecord,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponConfirmationCommand {
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub expected_reservation: CouponReservationRecord,
    pub next_reservation: CouponReservationRecord,
    pub redemption: CouponRedemptionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponConfirmationResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub reservation: CouponReservationRecord,
    pub redemption: CouponRedemptionRecord,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponReleaseCommand {
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub expected_reservation: CouponReservationRecord,
    pub next_reservation: CouponReservationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponReleaseResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub reservation: CouponReservationRecord,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponRollbackCommand {
    pub expected_budget: CampaignBudgetRecord,
    pub next_budget: CampaignBudgetRecord,
    pub expected_code: CouponCodeRecord,
    pub next_code: CouponCodeRecord,
    pub expected_redemption: CouponRedemptionRecord,
    pub next_redemption: CouponRedemptionRecord,
    pub rollback: CouponRollbackRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicCouponRollbackResult {
    pub budget: CampaignBudgetRecord,
    pub code: CouponCodeRecord,
    pub redemption: CouponRedemptionRecord,
    pub rollback: CouponRollbackRecord,
    pub created: bool,
}

pub struct StorageDriverRegistry<T> {
    factories: HashMap<StorageDialect, Arc<dyn StorageDriverFactory<T>>>,
}

impl<T> Default for StorageDriverRegistry<T> {
    fn default() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
}

impl<T> StorageDriverRegistry<T>
where
    T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_factory<F>(mut self, factory: F) -> Self
    where
        F: StorageDriverFactory<T> + 'static,
    {
        self.register(factory);
        self
    }

    pub fn register<F>(&mut self, factory: F) -> Option<Arc<dyn StorageDriverFactory<T>>>
    where
        F: StorageDriverFactory<T> + 'static,
    {
        self.register_arc(Arc::new(factory))
    }

    pub fn register_arc(
        &mut self,
        factory: Arc<dyn StorageDriverFactory<T>>,
    ) -> Option<Arc<dyn StorageDriverFactory<T>>> {
        self.factories.insert(factory.dialect(), factory)
    }

    pub fn resolve(&self, dialect: StorageDialect) -> Option<Arc<dyn StorageDriverFactory<T>>> {
        self.factories.get(&dialect).cloned()
    }

    pub fn supports(&self, dialect: StorageDialect) -> bool {
        self.factories.contains_key(&dialect)
    }
}

pub struct Reloadable<T: Clone> {
    current: Arc<RwLock<T>>,
}

impl<T: Clone> Reloadable<T> {
    pub fn new(initial: T) -> Self {
        Self {
            current: Arc::new(RwLock::new(initial)),
        }
    }

    pub fn snapshot(&self) -> T {
        self.current
            .read()
            .expect("reloadable value lock poisoned")
            .clone()
    }

    pub fn replace(&self, next: T) {
        *self
            .current
            .write()
            .expect("reloadable value lock poisoned") = next;
    }
}

impl<T: Clone> Clone for Reloadable<T> {
    fn clone(&self) -> Self {
        Self {
            current: Arc::clone(&self.current),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRuntimeNodeRecord {
    pub node_id: String,
    pub service_kind: String,
    pub started_at_ms: u64,
    pub last_seen_at_ms: u64,
}

impl ServiceRuntimeNodeRecord {
    pub fn new(
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        started_at_ms: u64,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            started_at_ms,
            last_seen_at_ms: started_at_ms,
        }
    }

    pub fn with_last_seen_at_ms(mut self, last_seen_at_ms: u64) -> Self {
        self.last_seen_at_ms = last_seen_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionRuntimeRolloutRecord {
    pub rollout_id: String,
    pub scope: String,
    pub requested_extension_id: Option<String>,
    pub requested_instance_id: Option<String>,
    pub resolved_extension_id: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
}

impl ExtensionRuntimeRolloutRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rollout_id: impl Into<String>,
        scope: impl Into<String>,
        requested_extension_id: Option<String>,
        requested_instance_id: Option<String>,
        resolved_extension_id: Option<String>,
        created_by: impl Into<String>,
        created_at_ms: u64,
        deadline_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            scope: scope.into(),
            requested_extension_id,
            requested_instance_id,
            resolved_extension_id,
            created_by: created_by.into(),
            created_at_ms,
            deadline_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionRuntimeRolloutParticipantRecord {
    pub rollout_id: String,
    pub node_id: String,
    pub service_kind: String,
    pub status: String,
    pub message: Option<String>,
    pub updated_at_ms: u64,
}

impl ExtensionRuntimeRolloutParticipantRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        status: impl Into<String>,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            status: status.into(),
            message: None,
            updated_at_ms,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfigRolloutRecord {
    pub rollout_id: String,
    pub requested_service_kind: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
}

impl StandaloneConfigRolloutRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        requested_service_kind: Option<String>,
        created_by: impl Into<String>,
        created_at_ms: u64,
        deadline_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            requested_service_kind,
            created_by: created_by.into(),
            created_at_ms,
            deadline_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfigRolloutParticipantRecord {
    pub rollout_id: String,
    pub node_id: String,
    pub service_kind: String,
    pub status: String,
    pub message: Option<String>,
    pub updated_at_ms: u64,
}

impl StandaloneConfigRolloutParticipantRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        status: impl Into<String>,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            status: status.into(),
            message: None,
            updated_at_ms,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

macro_rules! define_admin_store_facet {
    ($trait_name:ident { $(async fn $name:ident(&self $(, $arg:ident : $ty:ty)*) -> $ret:ty;)+ }) => {
        #[async_trait]
        pub trait $trait_name: Send + Sync {
            $(
                async fn $name(&self, $( $arg : $ty ),*) -> Result<$ret>;
            )+
        }

        #[async_trait]
        impl<T> $trait_name for T
        where
            T: AdminStore + ?Sized,
        {
            $(
                async fn $name(&self, $( $arg : $ty ),*) -> Result<$ret> {
                    AdminStore::$name(self, $( $arg ),*).await
                }
            )+
        }
    };
}

define_admin_store_facet!(IdentityStore {
    async fn insert_portal_user(&self, user: &PortalUserRecord) -> PortalUserRecord;
    async fn list_portal_users(&self) -> Vec<PortalUserRecord>;
    async fn find_portal_user_by_email(&self, email: &str) -> Option<PortalUserRecord>;
    async fn find_portal_user_by_id(&self, user_id: &str) -> Option<PortalUserRecord>;
    async fn delete_portal_user(&self, user_id: &str) -> bool;
    async fn insert_admin_user(&self, user: &AdminUserRecord) -> AdminUserRecord;
    async fn list_admin_users(&self) -> Vec<AdminUserRecord>;
    async fn find_admin_user_by_email(&self, email: &str) -> Option<AdminUserRecord>;
    async fn find_admin_user_by_id(&self, user_id: &str) -> Option<AdminUserRecord>;
    async fn delete_admin_user(&self, user_id: &str) -> bool;
    async fn insert_gateway_api_key(&self, record: &GatewayApiKeyRecord) -> GatewayApiKeyRecord;
    async fn list_gateway_api_keys(&self) -> Vec<GatewayApiKeyRecord>;
    async fn find_gateway_api_key(&self, hashed_key: &str) -> Option<GatewayApiKeyRecord>;
    async fn delete_gateway_api_key(&self, hashed_key: &str) -> bool;
    async fn insert_api_key_group(&self, record: &ApiKeyGroupRecord) -> ApiKeyGroupRecord;
    async fn list_api_key_groups(&self) -> Vec<ApiKeyGroupRecord>;
    async fn find_api_key_group(&self, group_id: &str) -> Option<ApiKeyGroupRecord>;
    async fn delete_api_key_group(&self, group_id: &str) -> bool;
});

define_admin_store_facet!(TenantStore {
    async fn insert_tenant(&self, tenant: &Tenant) -> Tenant;
    async fn list_tenants(&self) -> Vec<Tenant>;
    async fn find_tenant(&self, tenant_id: &str) -> Option<Tenant>;
    async fn delete_tenant(&self, tenant_id: &str) -> bool;
    async fn insert_project(&self, project: &Project) -> Project;
    async fn list_projects(&self) -> Vec<Project>;
    async fn find_project(&self, project_id: &str) -> Option<Project>;
    async fn delete_project(&self, project_id: &str) -> bool;
    async fn upsert_project_membership(&self, membership: &ProjectMembershipRecord) -> ProjectMembershipRecord;
    async fn find_project_membership(&self, project_id: &str) -> Option<ProjectMembershipRecord>;
});

define_admin_store_facet!(CatalogStore {
    async fn insert_channel(&self, channel: &Channel) -> Channel;
    async fn list_channels(&self) -> Vec<Channel>;
    async fn delete_channel(&self, channel_id: &str) -> bool;
    async fn insert_provider(&self, provider: &ProxyProvider) -> ProxyProvider;
    async fn list_providers(&self) -> Vec<ProxyProvider>;
    async fn find_provider(&self, provider_id: &str) -> Option<ProxyProvider>;
    async fn delete_provider(&self, provider_id: &str) -> bool;
    async fn insert_model(&self, model: &ModelCatalogEntry) -> ModelCatalogEntry;
    async fn list_models(&self) -> Vec<ModelCatalogEntry>;
    async fn find_model(&self, external_name: &str) -> Option<ModelCatalogEntry>;
    async fn delete_model(&self, external_name: &str) -> bool;
    async fn delete_model_variant(&self, external_name: &str, provider_id: &str) -> bool;
    async fn insert_channel_model(&self, record: &ChannelModelRecord) -> ChannelModelRecord;
    async fn list_channel_models(&self) -> Vec<ChannelModelRecord>;
    async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> bool;
    async fn insert_model_price(&self, record: &ModelPriceRecord) -> ModelPriceRecord;
    async fn list_model_prices(&self) -> Vec<ModelPriceRecord>;
    async fn delete_model_price(&self, channel_id: &str, model_id: &str, proxy_provider_id: &str) -> bool;
});

define_admin_store_facet!(CredentialStore {
    async fn insert_credential(&self, credential: &UpstreamCredential) -> UpstreamCredential;
    async fn insert_encrypted_credential(&self, credential: &UpstreamCredential, envelope: &SecretEnvelope) -> UpstreamCredential;
    async fn list_credentials(&self) -> Vec<UpstreamCredential>;
    async fn find_credential(&self, tenant_id: &str, provider_id: &str, key_reference: &str) -> Option<UpstreamCredential>;
    async fn find_credential_envelope(&self, tenant_id: &str, provider_id: &str, key_reference: &str) -> Option<SecretEnvelope>;
    async fn find_provider_credential(&self, tenant_id: &str, provider_id: &str) -> Option<UpstreamCredential>;
    async fn delete_credential(&self, tenant_id: &str, provider_id: &str, key_reference: &str) -> bool;
});

define_admin_store_facet!(RoutingStore {
    async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> RoutingPolicy;
    async fn list_routing_policies(&self) -> Vec<RoutingPolicy>;
    async fn insert_routing_profile(&self, profile: &RoutingProfileRecord) -> RoutingProfileRecord;
    async fn list_routing_profiles(&self) -> Vec<RoutingProfileRecord>;
    async fn find_routing_profile(&self, profile_id: &str) -> Option<RoutingProfileRecord>;
    async fn insert_compiled_routing_snapshot(&self, snapshot: &CompiledRoutingSnapshotRecord) -> CompiledRoutingSnapshotRecord;
    async fn list_compiled_routing_snapshots(&self) -> Vec<CompiledRoutingSnapshotRecord>;
    async fn insert_project_routing_preferences(&self, preferences: &ProjectRoutingPreferences) -> ProjectRoutingPreferences;
    async fn find_project_routing_preferences(&self, project_id: &str) -> Option<ProjectRoutingPreferences>;
    async fn insert_routing_decision_log(&self, log: &RoutingDecisionLog) -> RoutingDecisionLog;
    async fn list_routing_decision_logs(&self) -> Vec<RoutingDecisionLog>;
    async fn insert_provider_health_snapshot(&self, snapshot: &ProviderHealthSnapshot) -> ProviderHealthSnapshot;
    async fn list_provider_health_snapshots(&self) -> Vec<ProviderHealthSnapshot>;
    async fn insert_rate_limit_policy(&self, policy: &RateLimitPolicy) -> RateLimitPolicy;
    async fn list_rate_limit_policies(&self) -> Vec<RateLimitPolicy>;
    async fn list_rate_limit_window_snapshots(&self) -> Vec<RateLimitWindowSnapshot>;
    async fn check_and_consume_rate_limit(&self, policy_id: &str, requested_requests: u64, limit_requests: u64, window_seconds: u64, now_ms: u64) -> RateLimitCheckResult;
});

define_admin_store_facet!(UsageStore {
    async fn insert_usage_record(&self, record: &UsageRecord) -> UsageRecord;
    async fn list_usage_records(&self) -> Vec<UsageRecord>;
});

define_admin_store_facet!(BillingStore {
    async fn insert_billing_event(&self, event: &BillingEventRecord) -> BillingEventRecord;
    async fn list_billing_events(&self) -> Vec<BillingEventRecord>;
    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> LedgerEntry;
    async fn list_ledger_entries(&self) -> Vec<LedgerEntry>;
    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> QuotaPolicy;
    async fn list_quota_policies(&self) -> Vec<QuotaPolicy>;
    async fn insert_coupon(&self, coupon: &CouponCampaign) -> CouponCampaign;
    async fn list_coupons(&self) -> Vec<CouponCampaign>;
    async fn find_coupon(&self, coupon_id: &str) -> Option<CouponCampaign>;
    async fn delete_coupon(&self, coupon_id: &str) -> bool;
    async fn insert_commerce_order(&self, order: &CommerceOrderRecord) -> CommerceOrderRecord;
    async fn list_commerce_orders(&self) -> Vec<CommerceOrderRecord>;
    async fn upsert_commerce_payment_event(&self, event: &CommercePaymentEventRecord) -> CommercePaymentEventRecord;
    async fn list_commerce_payment_events(&self) -> Vec<CommercePaymentEventRecord>;
    async fn find_commerce_payment_event_by_dedupe_key(&self, dedupe_key: &str) -> Option<CommercePaymentEventRecord>;
});

define_admin_store_facet!(JobStore {
    async fn insert_async_job(&self, record: &AsyncJobRecord) -> AsyncJobRecord;
    async fn list_async_jobs(&self) -> Vec<AsyncJobRecord>;
    async fn find_async_job(&self, job_id: &str) -> Option<AsyncJobRecord>;
    async fn insert_async_job_attempt(&self, record: &AsyncJobAttemptRecord) -> AsyncJobAttemptRecord;
    async fn list_async_job_attempts(&self, job_id: &str) -> Vec<AsyncJobAttemptRecord>;
    async fn insert_async_job_asset(&self, record: &AsyncJobAssetRecord) -> AsyncJobAssetRecord;
    async fn list_async_job_assets(&self, job_id: &str) -> Vec<AsyncJobAssetRecord>;
    async fn insert_async_job_callback(&self, record: &AsyncJobCallbackRecord) -> AsyncJobCallbackRecord;
    async fn list_async_job_callbacks(&self, job_id: &str) -> Vec<AsyncJobCallbackRecord>;
});

define_admin_store_facet!(ExtensionStore {
    async fn insert_extension_installation(&self, installation: &ExtensionInstallation) -> ExtensionInstallation;
    async fn list_extension_installations(&self) -> Vec<ExtensionInstallation>;
    async fn insert_extension_instance(&self, instance: &ExtensionInstance) -> ExtensionInstance;
    async fn list_extension_instances(&self) -> Vec<ExtensionInstance>;
    async fn upsert_service_runtime_node(&self, record: &ServiceRuntimeNodeRecord) -> ServiceRuntimeNodeRecord;
    async fn list_service_runtime_nodes(&self) -> Vec<ServiceRuntimeNodeRecord>;
    async fn insert_extension_runtime_rollout(&self, rollout: &ExtensionRuntimeRolloutRecord) -> ExtensionRuntimeRolloutRecord;
    async fn find_extension_runtime_rollout(&self, rollout_id: &str) -> Option<ExtensionRuntimeRolloutRecord>;
    async fn list_extension_runtime_rollouts(&self) -> Vec<ExtensionRuntimeRolloutRecord>;
    async fn insert_extension_runtime_rollout_participant(&self, participant: &ExtensionRuntimeRolloutParticipantRecord) -> ExtensionRuntimeRolloutParticipantRecord;
    async fn list_extension_runtime_rollout_participants(&self, rollout_id: &str) -> Vec<ExtensionRuntimeRolloutParticipantRecord>;
    async fn list_pending_extension_runtime_rollout_participants_for_node(&self, node_id: &str) -> Vec<ExtensionRuntimeRolloutParticipantRecord>;
    async fn transition_extension_runtime_rollout_participant(&self, rollout_id: &str, node_id: &str, from_status: &str, to_status: &str, message: Option<&str>, updated_at_ms: u64) -> bool;
    async fn insert_standalone_config_rollout(&self, rollout: &StandaloneConfigRolloutRecord) -> StandaloneConfigRolloutRecord;
    async fn find_standalone_config_rollout(&self, rollout_id: &str) -> Option<StandaloneConfigRolloutRecord>;
    async fn list_standalone_config_rollouts(&self) -> Vec<StandaloneConfigRolloutRecord>;
    async fn insert_standalone_config_rollout_participant(&self, participant: &StandaloneConfigRolloutParticipantRecord) -> StandaloneConfigRolloutParticipantRecord;
    async fn list_standalone_config_rollout_participants(&self, rollout_id: &str) -> Vec<StandaloneConfigRolloutParticipantRecord>;
    async fn list_pending_standalone_config_rollout_participants_for_node(&self, node_id: &str) -> Vec<StandaloneConfigRolloutParticipantRecord>;
    async fn transition_standalone_config_rollout_participant(&self, rollout_id: &str, node_id: &str, from_status: &str, to_status: &str, message: Option<&str>, updated_at_ms: u64) -> bool;
});

#[async_trait]
pub trait AdminStore: Send + Sync {
    fn dialect(&self) -> StorageDialect;

    async fn insert_channel(&self, channel: &Channel) -> Result<Channel>;
    async fn list_channels(&self) -> Result<Vec<Channel>>;
    async fn delete_channel(&self, channel_id: &str) -> Result<bool>;

    async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider>;
    async fn list_providers(&self) -> Result<Vec<ProxyProvider>>;
    async fn list_providers_for_model(&self, model: &str) -> Result<Vec<ProxyProvider>> {
        let model_provider_ids = self
            .list_models_for_external_name(model)
            .await?
            .into_iter()
            .map(|entry| entry.provider_id)
            .collect::<std::collections::HashSet<_>>();
        Ok(self
            .list_providers()
            .await?
            .into_iter()
            .filter(|provider| {
                model_provider_ids.is_empty() || model_provider_ids.contains(&provider.id)
            })
            .collect())
    }
    async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>>;
    async fn delete_provider(&self, provider_id: &str) -> Result<bool>;

    async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential>;
    async fn insert_encrypted_credential(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<UpstreamCredential>;
    async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>>;
    async fn list_credentials_for_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        Ok(self
            .list_credentials()
            .await?
            .into_iter()
            .filter(|credential| credential.tenant_id == tenant_id)
            .collect())
    }
    async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        Ok(self
            .list_credentials()
            .await?
            .into_iter()
            .filter(|credential| credential.provider_id == provider_id)
            .collect())
    }
    async fn find_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<UpstreamCredential>>;
    async fn find_credential_envelope(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<SecretEnvelope>>;
    async fn find_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<UpstreamCredential>>;
    async fn delete_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<bool>;

    async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry>;
    async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>>;
    async fn list_models_for_external_name(
        &self,
        external_name: &str,
    ) -> Result<Vec<ModelCatalogEntry>> {
        Ok(self
            .list_models()
            .await?
            .into_iter()
            .filter(|model| model.external_name == external_name)
            .collect())
    }
    async fn find_any_model(&self) -> Result<Option<ModelCatalogEntry>> {
        Ok(self.list_models().await?.into_iter().next())
    }
    async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>>;
    async fn delete_model(&self, external_name: &str) -> Result<bool>;
    async fn delete_model_variant(&self, external_name: &str, provider_id: &str) -> Result<bool>;
    async fn insert_channel_model(&self, record: &ChannelModelRecord)
        -> Result<ChannelModelRecord>;
    async fn list_channel_models(&self) -> Result<Vec<ChannelModelRecord>>;
    async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> Result<bool>;
    async fn insert_model_price(&self, record: &ModelPriceRecord) -> Result<ModelPriceRecord>;
    async fn list_model_prices(&self) -> Result<Vec<ModelPriceRecord>>;
    async fn delete_model_price(
        &self,
        channel_id: &str,
        model_id: &str,
        proxy_provider_id: &str,
    ) -> Result<bool>;

    async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy>;
    async fn list_routing_policies(&self) -> Result<Vec<RoutingPolicy>>;
    async fn insert_routing_profile(
        &self,
        profile: &RoutingProfileRecord,
    ) -> Result<RoutingProfileRecord>;
    async fn list_routing_profiles(&self) -> Result<Vec<RoutingProfileRecord>>;
    async fn find_routing_profile(&self, profile_id: &str) -> Result<Option<RoutingProfileRecord>>;
    async fn insert_compiled_routing_snapshot(
        &self,
        snapshot: &CompiledRoutingSnapshotRecord,
    ) -> Result<CompiledRoutingSnapshotRecord>;
    async fn list_compiled_routing_snapshots(&self) -> Result<Vec<CompiledRoutingSnapshotRecord>>;
    async fn insert_project_routing_preferences(
        &self,
        preferences: &ProjectRoutingPreferences,
    ) -> Result<ProjectRoutingPreferences>;
    async fn find_project_routing_preferences(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectRoutingPreferences>>;
    async fn insert_routing_decision_log(
        &self,
        log: &RoutingDecisionLog,
    ) -> Result<RoutingDecisionLog>;
    async fn list_routing_decision_logs(&self) -> Result<Vec<RoutingDecisionLog>>;
    async fn list_routing_decision_logs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RoutingDecisionLog>> {
        Ok(self
            .list_routing_decision_logs()
            .await?
            .into_iter()
            .filter(|log| log.project_id.as_deref() == Some(project_id))
            .collect())
    }
    async fn find_latest_routing_decision_log_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<RoutingDecisionLog>> {
        Ok(self
            .list_routing_decision_logs_for_project(project_id)
            .await?
            .into_iter()
            .next())
    }
    async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot>;
    async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>>;

    async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord>;
    async fn list_usage_records(&self) -> Result<Vec<UsageRecord>>;
    async fn list_usage_records_for_project(&self, project_id: &str) -> Result<Vec<UsageRecord>> {
        Ok(self
            .list_usage_records()
            .await?
            .into_iter()
            .filter(|record| record.project_id == project_id)
            .collect())
    }
    async fn find_latest_usage_record_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<UsageRecord>> {
        Ok(self
            .list_usage_records_for_project(project_id)
            .await?
            .into_iter()
            .next())
    }

    async fn insert_billing_event(&self, event: &BillingEventRecord) -> Result<BillingEventRecord>;
    async fn list_billing_events(&self) -> Result<Vec<BillingEventRecord>>;
    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry>;
    async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>>;
    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>> {
        Ok(self
            .list_ledger_entries()
            .await?
            .into_iter()
            .filter(|entry| entry.project_id == project_id)
            .collect())
    }
    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy>;
    async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>>;
    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>> {
        Ok(self
            .list_quota_policies()
            .await?
            .into_iter()
            .filter(|policy| policy.project_id == project_id)
            .collect())
    }

    async fn insert_rate_limit_policy(&self, policy: &RateLimitPolicy) -> Result<RateLimitPolicy>;
    async fn list_rate_limit_policies(&self) -> Result<Vec<RateLimitPolicy>>;
    async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>> {
        Ok(self
            .list_rate_limit_policies()
            .await?
            .into_iter()
            .filter(|policy| policy.project_id == project_id)
            .collect())
    }
    async fn list_rate_limit_window_snapshots(&self) -> Result<Vec<RateLimitWindowSnapshot>>;
    async fn list_rate_limit_window_snapshots_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitWindowSnapshot>> {
        Ok(self
            .list_rate_limit_window_snapshots()
            .await?
            .into_iter()
            .filter(|snapshot| snapshot.project_id == project_id)
            .collect())
    }
    async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult>;

    async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>>;
    async fn find_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>>;
    async fn delete_tenant(&self, tenant_id: &str) -> Result<bool>;

    async fn insert_project(&self, project: &Project) -> Result<Project>;
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn find_project(&self, project_id: &str) -> Result<Option<Project>>;
    async fn delete_project(&self, project_id: &str) -> Result<bool>;

    async fn insert_coupon(&self, coupon: &CouponCampaign) -> Result<CouponCampaign>;
    async fn list_coupons(&self) -> Result<Vec<CouponCampaign>>;
    async fn list_active_coupons(&self) -> Result<Vec<CouponCampaign>> {
        Ok(self
            .list_coupons()
            .await?
            .into_iter()
            .filter(CouponCampaign::is_compatibility_live)
            .collect())
    }
    async fn find_coupon(&self, coupon_id: &str) -> Result<Option<CouponCampaign>>;
    async fn delete_coupon(&self, coupon_id: &str) -> Result<bool>;

    async fn insert_coupon_template_record(
        &self,
        _record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_template_record",
        ))
    }
    async fn list_coupon_template_records(&self) -> Result<Vec<CouponTemplateRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_template_records",
        ))
    }
    async fn find_coupon_template_record(
        &self,
        _coupon_template_id: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_template_record",
        ))
    }
    async fn find_coupon_template_record_by_template_key(
        &self,
        template_key: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        Ok(AdminStore::list_coupon_template_records(self)
            .await?
            .into_iter()
            .find(|record| record.template_key == template_key))
    }
    async fn insert_marketing_campaign_record(
        &self,
        _record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_campaign_record",
        ))
    }
    async fn list_marketing_campaign_records(&self) -> Result<Vec<MarketingCampaignRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_campaign_records",
        ))
    }
    async fn list_marketing_campaign_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>> {
        Ok(AdminStore::list_marketing_campaign_records(self)
            .await?
            .into_iter()
            .filter(|record| record.coupon_template_id == coupon_template_id)
            .collect())
    }
    async fn insert_campaign_budget_record(
        &self,
        _record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_campaign_budget_record",
        ))
    }
    async fn list_campaign_budget_records(&self) -> Result<Vec<CampaignBudgetRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_campaign_budget_records",
        ))
    }
    async fn list_campaign_budget_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>> {
        Ok(AdminStore::list_campaign_budget_records(self)
            .await?
            .into_iter()
            .filter(|record| record.marketing_campaign_id == marketing_campaign_id)
            .collect())
    }
    async fn insert_coupon_code_record(
        &self,
        _record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_code_record",
        ))
    }
    async fn list_coupon_code_records(&self) -> Result<Vec<CouponCodeRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_code_records",
        ))
    }
    async fn find_coupon_code_record(
        &self,
        _coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_code_record",
        ))
    }
    async fn find_coupon_code_record_by_value(
        &self,
        code_value: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .find(|record| record.code_value == code_value))
    }
    async fn list_redeemable_coupon_code_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponCodeRecord>> {
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .filter(|record| record.is_redeemable_at(now_ms))
            .collect())
    }
    async fn insert_coupon_reservation_record(
        &self,
        _record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_reservation_record",
        ))
    }
    async fn list_coupon_reservation_records(&self) -> Result<Vec<CouponReservationRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_reservation_records",
        ))
    }
    async fn find_coupon_reservation_record(
        &self,
        _coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_reservation_record",
        ))
    }
    async fn list_active_coupon_reservation_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponReservationRecord>> {
        Ok(AdminStore::list_coupon_reservation_records(self)
            .await?
            .into_iter()
            .filter(|record| record.is_active_at(now_ms))
            .collect())
    }
    async fn insert_coupon_redemption_record(
        &self,
        _record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_redemption_record",
        ))
    }
    async fn list_coupon_redemption_records(&self) -> Result<Vec<CouponRedemptionRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_redemption_records",
        ))
    }
    async fn find_coupon_redemption_record(
        &self,
        _coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_redemption_record",
        ))
    }
    async fn insert_coupon_rollback_record(
        &self,
        _record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_rollback_record",
        ))
    }
    async fn list_coupon_rollback_records(&self) -> Result<Vec<CouponRollbackRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_rollback_records",
        ))
    }
    async fn insert_marketing_outbox_event_record(
        &self,
        _record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_outbox_event_record",
        ))
    }
    async fn list_marketing_outbox_event_records(&self) -> Result<Vec<MarketingOutboxEventRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_outbox_event_records",
        ))
    }
    async fn reserve_coupon_redemption_atomic(
        &self,
        _command: &AtomicCouponReservationCommand,
    ) -> Result<AtomicCouponReservationResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "reserve_coupon_redemption_atomic",
        ))
    }
    async fn confirm_coupon_redemption_atomic(
        &self,
        _command: &AtomicCouponConfirmationCommand,
    ) -> Result<AtomicCouponConfirmationResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "confirm_coupon_redemption_atomic",
        ))
    }
    async fn release_coupon_reservation_atomic(
        &self,
        _command: &AtomicCouponReleaseCommand,
    ) -> Result<AtomicCouponReleaseResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "release_coupon_reservation_atomic",
        ))
    }
    async fn rollback_coupon_redemption_atomic(
        &self,
        _command: &AtomicCouponRollbackCommand,
    ) -> Result<AtomicCouponRollbackResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "rollback_coupon_redemption_atomic",
        ))
    }

    async fn insert_commerce_order(
        &self,
        order: &CommerceOrderRecord,
    ) -> Result<CommerceOrderRecord>;
    async fn list_commerce_orders(&self) -> Result<Vec<CommerceOrderRecord>>;
    async fn list_recent_commerce_orders(
        &self,
        limit: usize,
    ) -> Result<Vec<CommerceOrderRecord>> {
        let mut orders = self.list_commerce_orders().await?;
        orders.sort_by(|left, right| {
            right
                .updated_at_ms
                .cmp(&left.updated_at_ms)
                .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
                .then_with(|| right.order_id.cmp(&left.order_id))
        });
        orders.truncate(limit);
        Ok(orders)
    }
    async fn upsert_commerce_payment_event(
        &self,
        event: &CommercePaymentEventRecord,
    ) -> Result<CommercePaymentEventRecord>;
    async fn list_commerce_payment_events(&self) -> Result<Vec<CommercePaymentEventRecord>>;
    async fn find_commerce_payment_event_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommercePaymentEventRecord>>;
    async fn list_commerce_orders_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        Ok(self
            .list_commerce_orders()
            .await?
            .into_iter()
            .filter(|order| order.project_id == project_id)
            .collect())
    }
    async fn list_commerce_orders_for_project_after(
        &self,
        project_id: &str,
        last_order_updated_at_ms: u64,
        last_order_created_at_ms: u64,
        last_order_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        Ok(self
            .list_commerce_orders_for_project(project_id)
            .await?
            .into_iter()
            .filter(|order| {
                order.updated_at_ms > last_order_updated_at_ms
                    || (order.updated_at_ms == last_order_updated_at_ms
                        && (order.created_at_ms > last_order_created_at_ms
                            || (order.created_at_ms == last_order_created_at_ms
                                && order.order_id.as_str() > last_order_id)))
            })
            .collect())
    }
    async fn list_commerce_payment_events_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommercePaymentEventRecord>> {
        Ok(self
            .list_commerce_payment_events()
            .await?
            .into_iter()
            .filter(|event| event.order_id == order_id)
            .collect())
    }
    async fn insert_async_job(&self, record: &AsyncJobRecord) -> Result<AsyncJobRecord>;
    async fn list_async_jobs(&self) -> Result<Vec<AsyncJobRecord>>;
    async fn find_async_job(&self, job_id: &str) -> Result<Option<AsyncJobRecord>>;
    async fn insert_async_job_attempt(
        &self,
        record: &AsyncJobAttemptRecord,
    ) -> Result<AsyncJobAttemptRecord>;
    async fn list_async_job_attempts(&self, job_id: &str) -> Result<Vec<AsyncJobAttemptRecord>>;
    async fn insert_async_job_asset(
        &self,
        record: &AsyncJobAssetRecord,
    ) -> Result<AsyncJobAssetRecord>;
    async fn list_async_job_assets(&self, job_id: &str) -> Result<Vec<AsyncJobAssetRecord>>;
    async fn insert_async_job_callback(
        &self,
        record: &AsyncJobCallbackRecord,
    ) -> Result<AsyncJobCallbackRecord>;
    async fn list_async_job_callbacks(&self, job_id: &str) -> Result<Vec<AsyncJobCallbackRecord>>;
    async fn upsert_project_membership(
        &self,
        membership: &ProjectMembershipRecord,
    ) -> Result<ProjectMembershipRecord>;
    async fn find_project_membership(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectMembershipRecord>>;

    async fn insert_portal_user(&self, user: &PortalUserRecord) -> Result<PortalUserRecord>;
    async fn list_portal_users(&self) -> Result<Vec<PortalUserRecord>>;
    async fn find_portal_user_by_email(&self, email: &str) -> Result<Option<PortalUserRecord>>;
    async fn find_portal_user_by_id(&self, user_id: &str) -> Result<Option<PortalUserRecord>>;
    async fn delete_portal_user(&self, user_id: &str) -> Result<bool>;
    async fn insert_admin_user(&self, user: &AdminUserRecord) -> Result<AdminUserRecord>;
    async fn list_admin_users(&self) -> Result<Vec<AdminUserRecord>>;
    async fn find_admin_user_by_email(&self, email: &str) -> Result<Option<AdminUserRecord>>;
    async fn find_admin_user_by_id(&self, user_id: &str) -> Result<Option<AdminUserRecord>>;
    async fn delete_admin_user(&self, user_id: &str) -> Result<bool>;

    async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord>;
    async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>>;
    async fn find_gateway_api_key(&self, hashed_key: &str) -> Result<Option<GatewayApiKeyRecord>>;
    async fn delete_gateway_api_key(&self, hashed_key: &str) -> Result<bool>;
    async fn insert_api_key_group(&self, record: &ApiKeyGroupRecord) -> Result<ApiKeyGroupRecord>;
    async fn list_api_key_groups(&self) -> Result<Vec<ApiKeyGroupRecord>>;
    async fn find_api_key_group(&self, group_id: &str) -> Result<Option<ApiKeyGroupRecord>>;
    async fn delete_api_key_group(&self, group_id: &str) -> Result<bool>;

    async fn insert_extension_installation(
        &self,
        installation: &ExtensionInstallation,
    ) -> Result<ExtensionInstallation>;
    async fn list_extension_installations(&self) -> Result<Vec<ExtensionInstallation>>;

    async fn insert_extension_instance(
        &self,
        instance: &ExtensionInstance,
    ) -> Result<ExtensionInstance>;
    async fn list_extension_instances(&self) -> Result<Vec<ExtensionInstance>>;

    async fn upsert_service_runtime_node(
        &self,
        record: &ServiceRuntimeNodeRecord,
    ) -> Result<ServiceRuntimeNodeRecord>;
    async fn list_service_runtime_nodes(&self) -> Result<Vec<ServiceRuntimeNodeRecord>>;

    async fn insert_extension_runtime_rollout(
        &self,
        rollout: &ExtensionRuntimeRolloutRecord,
    ) -> Result<ExtensionRuntimeRolloutRecord>;
    async fn find_extension_runtime_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<ExtensionRuntimeRolloutRecord>>;
    async fn list_extension_runtime_rollouts(&self) -> Result<Vec<ExtensionRuntimeRolloutRecord>>;

    async fn insert_extension_runtime_rollout_participant(
        &self,
        participant: &ExtensionRuntimeRolloutParticipantRecord,
    ) -> Result<ExtensionRuntimeRolloutParticipantRecord>;
    async fn list_extension_runtime_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>>;
    async fn list_pending_extension_runtime_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>>;
    async fn transition_extension_runtime_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool>;

    async fn insert_standalone_config_rollout(
        &self,
        rollout: &StandaloneConfigRolloutRecord,
    ) -> Result<StandaloneConfigRolloutRecord>;
    async fn find_standalone_config_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<StandaloneConfigRolloutRecord>>;
    async fn list_standalone_config_rollouts(&self) -> Result<Vec<StandaloneConfigRolloutRecord>>;

    async fn insert_standalone_config_rollout_participant(
        &self,
        participant: &StandaloneConfigRolloutParticipantRecord,
    ) -> Result<StandaloneConfigRolloutParticipantRecord>;
    async fn list_standalone_config_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>>;
    async fn list_pending_standalone_config_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>>;
    async fn transition_standalone_config_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool>;
}

fn unsupported_account_kernel_method(dialect: StorageDialect, method: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "storage dialect {} does not implement canonical account kernel method {} yet",
        dialect.as_str(),
        method
    )
}

fn unsupported_identity_kernel_method(dialect: StorageDialect, method: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "storage dialect {} does not implement canonical identity kernel method {} yet",
        dialect.as_str(),
        method
    )
}

fn unsupported_marketing_kernel_method(dialect: StorageDialect, method: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "storage dialect {} does not implement enterprise marketing kernel method {} yet",
        dialect.as_str(),
        method
    )
}

#[async_trait]
pub trait IdentityKernelStore: AdminStore {
    async fn insert_identity_user_record(
        &self,
        _record: &IdentityUserRecord,
    ) -> Result<IdentityUserRecord> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "insert_identity_user_record",
        ))
    }

    async fn list_identity_user_records(&self) -> Result<Vec<IdentityUserRecord>> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "list_identity_user_records",
        ))
    }

    async fn find_identity_user_record(&self, _user_id: u64) -> Result<Option<IdentityUserRecord>> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "find_identity_user_record",
        ))
    }

    async fn insert_canonical_api_key_record(
        &self,
        _record: &CanonicalApiKeyRecord,
    ) -> Result<CanonicalApiKeyRecord> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "insert_canonical_api_key_record",
        ))
    }

    async fn find_canonical_api_key_record_by_hash(
        &self,
        _key_hash: &str,
    ) -> Result<Option<CanonicalApiKeyRecord>> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "find_canonical_api_key_record_by_hash",
        ))
    }

    async fn insert_identity_binding_record(
        &self,
        _record: &IdentityBindingRecord,
    ) -> Result<IdentityBindingRecord> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "insert_identity_binding_record",
        ))
    }

    async fn find_identity_binding_record(
        &self,
        _binding_type: &str,
        _issuer: Option<&str>,
        _subject: Option<&str>,
    ) -> Result<Option<IdentityBindingRecord>> {
        Err(unsupported_identity_kernel_method(
            self.dialect(),
            "find_identity_binding_record",
        ))
    }
}

#[async_trait]
pub trait AccountKernelStore: AdminStore {
    async fn insert_account_record(&self, _record: &AccountRecord) -> Result<AccountRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_record",
        ))
    }

    async fn list_account_records(&self) -> Result<Vec<AccountRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_records",
        ))
    }

    async fn find_account_record(&self, _account_id: u64) -> Result<Option<AccountRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "find_account_record",
        ))
    }

    async fn find_account_record_by_owner(
        &self,
        _tenant_id: u64,
        _organization_id: u64,
        _user_id: u64,
        _account_type: AccountType,
    ) -> Result<Option<AccountRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "find_account_record_by_owner",
        ))
    }

    async fn insert_account_benefit_lot(
        &self,
        _record: &AccountBenefitLotRecord,
    ) -> Result<AccountBenefitLotRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_benefit_lot",
        ))
    }

    async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_benefit_lots",
        ))
    }

    async fn insert_account_hold(&self, _record: &AccountHoldRecord) -> Result<AccountHoldRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_hold",
        ))
    }

    async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_holds",
        ))
    }

    async fn insert_account_hold_allocation(
        &self,
        _record: &AccountHoldAllocationRecord,
    ) -> Result<AccountHoldAllocationRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_hold_allocation",
        ))
    }

    async fn list_account_hold_allocations(&self) -> Result<Vec<AccountHoldAllocationRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_hold_allocations",
        ))
    }

    async fn insert_account_ledger_entry_record(
        &self,
        _record: &AccountLedgerEntryRecord,
    ) -> Result<AccountLedgerEntryRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_ledger_entry_record",
        ))
    }

    async fn list_account_ledger_entry_records(&self) -> Result<Vec<AccountLedgerEntryRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_ledger_entry_records",
        ))
    }

    async fn insert_account_ledger_allocation(
        &self,
        _record: &AccountLedgerAllocationRecord,
    ) -> Result<AccountLedgerAllocationRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_ledger_allocation",
        ))
    }

    async fn list_account_ledger_allocations(&self) -> Result<Vec<AccountLedgerAllocationRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_account_ledger_allocations",
        ))
    }

    async fn insert_account_commerce_reconciliation_state(
        &self,
        _record: &AccountCommerceReconciliationStateRecord,
    ) -> Result<AccountCommerceReconciliationStateRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_account_commerce_reconciliation_state",
        ))
    }

    async fn find_account_commerce_reconciliation_state(
        &self,
        _account_id: u64,
        _project_id: &str,
    ) -> Result<Option<AccountCommerceReconciliationStateRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "find_account_commerce_reconciliation_state",
        ))
    }

    async fn insert_request_meter_fact(
        &self,
        _record: &RequestMeterFactRecord,
    ) -> Result<RequestMeterFactRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_request_meter_fact",
        ))
    }

    async fn list_request_meter_facts(&self) -> Result<Vec<RequestMeterFactRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_request_meter_facts",
        ))
    }

    async fn insert_request_meter_metric(
        &self,
        _record: &RequestMeterMetricRecord,
    ) -> Result<RequestMeterMetricRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_request_meter_metric",
        ))
    }

    async fn list_request_meter_metrics(&self) -> Result<Vec<RequestMeterMetricRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_request_meter_metrics",
        ))
    }

    async fn insert_pricing_plan_record(
        &self,
        _record: &PricingPlanRecord,
    ) -> Result<PricingPlanRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_pricing_plan_record",
        ))
    }

    async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_pricing_plan_records",
        ))
    }

    async fn insert_pricing_rate_record(
        &self,
        _record: &PricingRateRecord,
    ) -> Result<PricingRateRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_pricing_rate_record",
        ))
    }

    async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_pricing_rate_records",
        ))
    }

    async fn insert_request_settlement_record(
        &self,
        _record: &RequestSettlementRecord,
    ) -> Result<RequestSettlementRecord> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "insert_request_settlement_record",
        ))
    }

    async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>> {
        Err(unsupported_account_kernel_method(
            self.dialect(),
            "list_request_settlement_records",
        ))
    }
}

#[async_trait]
pub trait MarketingStore: AdminStore {
    async fn insert_coupon_template_record(
        &self,
        _record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_template_record",
        ))
    }

    async fn list_coupon_template_records(&self) -> Result<Vec<CouponTemplateRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_template_records",
        ))
    }

    async fn find_coupon_template_record(
        &self,
        _coupon_template_id: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_template_record",
        ))
    }

    async fn find_coupon_template_record_by_template_key(
        &self,
        template_key: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        Ok(AdminStore::list_coupon_template_records(self)
            .await?
            .into_iter()
            .find(|record| record.template_key == template_key))
    }

    async fn insert_marketing_campaign_record(
        &self,
        _record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_campaign_record",
        ))
    }

    async fn list_marketing_campaign_records(&self) -> Result<Vec<MarketingCampaignRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_campaign_records",
        ))
    }

    async fn list_marketing_campaign_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>> {
        Ok(AdminStore::list_marketing_campaign_records(self)
            .await?
            .into_iter()
            .filter(|record| record.coupon_template_id == coupon_template_id)
            .collect())
    }

    async fn insert_campaign_budget_record(
        &self,
        _record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_campaign_budget_record",
        ))
    }

    async fn list_campaign_budget_records(&self) -> Result<Vec<CampaignBudgetRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_campaign_budget_records",
        ))
    }

    async fn list_campaign_budget_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>> {
        Ok(AdminStore::list_campaign_budget_records(self)
            .await?
            .into_iter()
            .filter(|record| record.marketing_campaign_id == marketing_campaign_id)
            .collect())
    }

    async fn insert_coupon_code_record(
        &self,
        _record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_code_record",
        ))
    }

    async fn list_coupon_code_records(&self) -> Result<Vec<CouponCodeRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_code_records",
        ))
    }

    async fn find_coupon_code_record(
        &self,
        _coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_code_record",
        ))
    }

    async fn find_coupon_code_record_by_value(
        &self,
        code_value: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .find(|record| record.code_value == code_value))
    }

    async fn list_redeemable_coupon_code_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponCodeRecord>> {
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .filter(|record| record.is_redeemable_at(now_ms))
            .collect())
    }

    async fn insert_coupon_reservation_record(
        &self,
        _record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_reservation_record",
        ))
    }

    async fn list_coupon_reservation_records(&self) -> Result<Vec<CouponReservationRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_reservation_records",
        ))
    }

    async fn find_coupon_reservation_record(
        &self,
        _coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_reservation_record",
        ))
    }

    async fn list_active_coupon_reservation_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponReservationRecord>> {
        Ok(AdminStore::list_coupon_reservation_records(self)
            .await?
            .into_iter()
            .filter(|record| record.is_active_at(now_ms))
            .collect())
    }

    async fn insert_coupon_redemption_record(
        &self,
        _record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_redemption_record",
        ))
    }

    async fn list_coupon_redemption_records(&self) -> Result<Vec<CouponRedemptionRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_redemption_records",
        ))
    }

    async fn find_coupon_redemption_record(
        &self,
        _coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_redemption_record",
        ))
    }

    async fn insert_coupon_rollback_record(
        &self,
        _record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_rollback_record",
        ))
    }

    async fn list_coupon_rollback_records(&self) -> Result<Vec<CouponRollbackRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_rollback_records",
        ))
    }

    async fn insert_marketing_outbox_event_record(
        &self,
        _record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_outbox_event_record",
        ))
    }

    async fn list_marketing_outbox_event_records(&self) -> Result<Vec<MarketingOutboxEventRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_outbox_event_records",
        ))
    }
}

pub type MarketingKernelTransactionFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[async_trait]
pub trait MarketingKernelTransaction: Send {
    async fn upsert_coupon_template_record(
        &mut self,
        record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord>;

    async fn upsert_marketing_campaign_record(
        &mut self,
        record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord>;

    async fn find_coupon_reservation_record(
        &mut self,
        coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>>;

    async fn find_coupon_code_record(
        &mut self,
        coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>>;

    async fn find_campaign_budget_record(
        &mut self,
        campaign_budget_id: &str,
    ) -> Result<Option<CampaignBudgetRecord>>;

    async fn find_coupon_redemption_record(
        &mut self,
        coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>>;

    async fn find_coupon_rollback_record(
        &mut self,
        coupon_rollback_id: &str,
    ) -> Result<Option<CouponRollbackRecord>>;

    async fn list_marketing_campaign_records_for_template(
        &mut self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>>;

    async fn list_campaign_budget_records_for_campaign(
        &mut self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>>;

    async fn upsert_coupon_reservation_record(
        &mut self,
        record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord>;

    async fn upsert_coupon_code_record(
        &mut self,
        record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord>;

    async fn upsert_campaign_budget_record(
        &mut self,
        record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord>;

    async fn upsert_coupon_redemption_record(
        &mut self,
        record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord>;

    async fn upsert_coupon_rollback_record(
        &mut self,
        record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord>;

    async fn upsert_marketing_outbox_event_record(
        &mut self,
        record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord>;
}

pub trait MarketingKernelTransactionExecutor: MarketingStore {
    fn with_marketing_kernel_transaction<'a, T, F>(
        &'a self,
        operation: F,
    ) -> MarketingKernelTransactionFuture<'a, T>
    where
        T: Send + 'a,
        F: for<'tx> FnOnce(
                &'tx mut dyn MarketingKernelTransaction,
            ) -> MarketingKernelTransactionFuture<'tx, T>
            + Send
            + 'a;
}

pub async fn execute_atomic_coupon_reservation<E>(
    store: &E,
    command: &AtomicCouponReservationCommand,
) -> Result<AtomicCouponReservationResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                if let Some(existing_reservation) = tx
                    .find_coupon_reservation_record(&command.reservation.coupon_reservation_id)
                    .await?
                {
                    if existing_reservation != command.reservation {
                        return Err(marketing_kernel_conflicting_replay(
                            "coupon reservation",
                            &command.reservation.coupon_reservation_id,
                        ));
                    }
                    let budget =
                        load_campaign_budget_snapshot(tx, &command.next_budget.campaign_budget_id)
                            .await?;
                    let code =
                        load_coupon_code_snapshot(tx, &command.next_code.coupon_code_id).await?;
                    ensure_marketing_record_matches(
                        "campaign budget",
                        &command.next_budget.campaign_budget_id,
                        Some(budget.clone()),
                        &command.next_budget,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon code",
                        &command.next_code.coupon_code_id,
                        Some(code.clone()),
                        &command.next_code,
                    )?;
                    return Ok(AtomicCouponReservationResult {
                        budget,
                        code,
                        reservation: existing_reservation,
                        created: false,
                    });
                }

                if let Some(template) = command.template_to_persist.as_ref() {
                    tx.upsert_coupon_template_record(template).await?;
                }
                if let Some(campaign) = command.campaign_to_persist.as_ref() {
                    tx.upsert_marketing_campaign_record(campaign).await?;
                }
                if command.template_to_persist.is_some() || command.campaign_to_persist.is_some() {
                    if tx
                        .find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?
                        .is_none()
                    {
                        tx.upsert_campaign_budget_record(&command.expected_budget)
                            .await?;
                    }
                    if tx
                        .find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?
                        .is_none()
                    {
                        tx.upsert_coupon_code_record(&command.expected_code).await?;
                    }
                }

                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    tx.find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?,
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    tx.find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?,
                    &command.expected_code,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let reservation = tx
                    .upsert_coupon_reservation_record(&command.reservation)
                    .await?;
                Ok(AtomicCouponReservationResult {
                    budget,
                    code,
                    reservation,
                    created: true,
                })
            })
        })
        .await
}

pub async fn execute_atomic_coupon_confirmation<E>(
    store: &E,
    command: &AtomicCouponConfirmationCommand,
) -> Result<AtomicCouponConfirmationResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                if let Some(existing_redemption) = tx
                    .find_coupon_redemption_record(&command.redemption.coupon_redemption_id)
                    .await?
                {
                    if existing_redemption != command.redemption {
                        return Err(marketing_kernel_conflicting_replay(
                            "coupon redemption",
                            &command.redemption.coupon_redemption_id,
                        ));
                    }
                    let budget =
                        load_campaign_budget_snapshot(tx, &command.next_budget.campaign_budget_id)
                            .await?;
                    let code =
                        load_coupon_code_snapshot(tx, &command.next_code.coupon_code_id).await?;
                    let reservation = load_coupon_reservation_snapshot(
                        tx,
                        &command.next_reservation.coupon_reservation_id,
                    )
                    .await?;
                    ensure_marketing_record_matches(
                        "campaign budget",
                        &command.next_budget.campaign_budget_id,
                        Some(budget.clone()),
                        &command.next_budget,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon code",
                        &command.next_code.coupon_code_id,
                        Some(code.clone()),
                        &command.next_code,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon reservation",
                        &command.next_reservation.coupon_reservation_id,
                        Some(reservation.clone()),
                        &command.next_reservation,
                    )?;
                    return Ok(AtomicCouponConfirmationResult {
                        budget,
                        code,
                        reservation,
                        redemption: existing_redemption,
                        created: false,
                    });
                }

                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    tx.find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?,
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    tx.find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?,
                    &command.expected_code,
                )?;
                ensure_marketing_record_matches(
                    "coupon reservation",
                    &command.expected_reservation.coupon_reservation_id,
                    tx.find_coupon_reservation_record(
                        &command.expected_reservation.coupon_reservation_id,
                    )
                    .await?,
                    &command.expected_reservation,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let reservation = tx
                    .upsert_coupon_reservation_record(&command.next_reservation)
                    .await?;
                let redemption = tx
                    .upsert_coupon_redemption_record(&command.redemption)
                    .await?;
                Ok(AtomicCouponConfirmationResult {
                    budget,
                    code,
                    reservation,
                    redemption,
                    created: true,
                })
            })
        })
        .await
}

pub async fn execute_atomic_coupon_release<E>(
    store: &E,
    command: &AtomicCouponReleaseCommand,
) -> Result<AtomicCouponReleaseResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                let current_reservation = load_coupon_reservation_snapshot(
                    tx,
                    &command.expected_reservation.coupon_reservation_id,
                )
                .await?;
                let budget =
                    load_campaign_budget_snapshot(tx, &command.expected_budget.campaign_budget_id)
                        .await?;
                let code =
                    load_coupon_code_snapshot(tx, &command.expected_code.coupon_code_id).await?;

                if current_reservation == command.next_reservation
                    && budget == command.next_budget
                    && code == command.next_code
                {
                    return Ok(AtomicCouponReleaseResult {
                        budget,
                        code,
                        reservation: current_reservation,
                        created: false,
                    });
                }

                ensure_marketing_record_matches(
                    "coupon reservation",
                    &command.expected_reservation.coupon_reservation_id,
                    Some(current_reservation),
                    &command.expected_reservation,
                )?;
                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    Some(budget),
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    Some(code),
                    &command.expected_code,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let reservation = tx
                    .upsert_coupon_reservation_record(&command.next_reservation)
                    .await?;
                Ok(AtomicCouponReleaseResult {
                    budget,
                    code,
                    reservation,
                    created: true,
                })
            })
        })
        .await
}

pub async fn execute_atomic_coupon_rollback<E>(
    store: &E,
    command: &AtomicCouponRollbackCommand,
) -> Result<AtomicCouponRollbackResult>
where
    E: MarketingKernelTransactionExecutor + ?Sized,
{
    let command = command.clone();
    store
        .with_marketing_kernel_transaction(move |tx| {
            Box::pin(async move {
                if let Some(existing_rollback) = tx
                    .find_coupon_rollback_record(&command.rollback.coupon_rollback_id)
                    .await?
                {
                    if existing_rollback != command.rollback {
                        return Err(marketing_kernel_conflicting_replay(
                            "coupon rollback",
                            &command.rollback.coupon_rollback_id,
                        ));
                    }
                    let budget =
                        load_campaign_budget_snapshot(tx, &command.next_budget.campaign_budget_id)
                            .await?;
                    let code =
                        load_coupon_code_snapshot(tx, &command.next_code.coupon_code_id).await?;
                    let redemption = load_coupon_redemption_snapshot(
                        tx,
                        &command.next_redemption.coupon_redemption_id,
                    )
                    .await?;
                    ensure_marketing_record_matches(
                        "campaign budget",
                        &command.next_budget.campaign_budget_id,
                        Some(budget.clone()),
                        &command.next_budget,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon code",
                        &command.next_code.coupon_code_id,
                        Some(code.clone()),
                        &command.next_code,
                    )?;
                    ensure_marketing_record_matches(
                        "coupon redemption",
                        &command.next_redemption.coupon_redemption_id,
                        Some(redemption.clone()),
                        &command.next_redemption,
                    )?;
                    return Ok(AtomicCouponRollbackResult {
                        budget,
                        code,
                        redemption,
                        rollback: existing_rollback,
                        created: false,
                    });
                }

                ensure_marketing_record_matches(
                    "campaign budget",
                    &command.expected_budget.campaign_budget_id,
                    tx.find_campaign_budget_record(&command.expected_budget.campaign_budget_id)
                        .await?,
                    &command.expected_budget,
                )?;
                ensure_marketing_record_matches(
                    "coupon code",
                    &command.expected_code.coupon_code_id,
                    tx.find_coupon_code_record(&command.expected_code.coupon_code_id)
                        .await?,
                    &command.expected_code,
                )?;
                ensure_marketing_record_matches(
                    "coupon redemption",
                    &command.expected_redemption.coupon_redemption_id,
                    tx.find_coupon_redemption_record(
                        &command.expected_redemption.coupon_redemption_id,
                    )
                    .await?,
                    &command.expected_redemption,
                )?;

                let budget = tx
                    .upsert_campaign_budget_record(&command.next_budget)
                    .await?;
                let code = tx.upsert_coupon_code_record(&command.next_code).await?;
                let redemption = tx
                    .upsert_coupon_redemption_record(&command.next_redemption)
                    .await?;
                let rollback = tx.upsert_coupon_rollback_record(&command.rollback).await?;
                Ok(AtomicCouponRollbackResult {
                    budget,
                    code,
                    redemption,
                    rollback,
                    created: true,
                })
            })
        })
        .await
}

async fn load_campaign_budget_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    campaign_budget_id: &str,
) -> Result<CampaignBudgetRecord> {
    tx.find_campaign_budget_record(campaign_budget_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("campaign budget", campaign_budget_id))
}

async fn load_coupon_code_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    coupon_code_id: &str,
) -> Result<CouponCodeRecord> {
    tx.find_coupon_code_record(coupon_code_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("coupon code", coupon_code_id))
}

async fn load_coupon_reservation_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    coupon_reservation_id: &str,
) -> Result<CouponReservationRecord> {
    tx.find_coupon_reservation_record(coupon_reservation_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("coupon reservation", coupon_reservation_id))
}

async fn load_coupon_redemption_snapshot(
    tx: &mut dyn MarketingKernelTransaction,
    coupon_redemption_id: &str,
) -> Result<CouponRedemptionRecord> {
    tx.find_coupon_redemption_record(coupon_redemption_id)
        .await?
        .ok_or_else(|| marketing_kernel_missing_record("coupon redemption", coupon_redemption_id))
}

fn ensure_marketing_record_matches<T: PartialEq>(
    record_type: &str,
    record_id: &str,
    actual: Option<T>,
    expected: &T,
) -> Result<()> {
    match actual {
        Some(actual) if actual == *expected => Ok(()),
        Some(_) => Err(marketing_kernel_snapshot_changed(record_type, record_id)),
        None => Err(marketing_kernel_missing_record(record_type, record_id)),
    }
}

fn marketing_kernel_snapshot_changed(record_type: &str, record_id: &str) -> anyhow::Error {
    anyhow::anyhow!("{record_type} {record_id} changed concurrently")
}

fn marketing_kernel_missing_record(record_type: &str, record_id: &str) -> anyhow::Error {
    anyhow::anyhow!("{record_type} {record_id} is missing")
}

fn marketing_kernel_conflicting_replay(record_type: &str, record_id: &str) -> anyhow::Error {
    anyhow::anyhow!("{record_type} {record_id} already exists with different state")
}

pub type AccountKernelTransactionFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[async_trait]
pub trait AccountKernelTransaction: Send {
    async fn find_account_record(&mut self, account_id: u64) -> Result<Option<AccountRecord>>;

    async fn find_account_benefit_lot(
        &mut self,
        lot_id: u64,
    ) -> Result<Option<AccountBenefitLotRecord>>;

    async fn list_account_benefit_lots_for_account(
        &mut self,
        account_id: u64,
    ) -> Result<Vec<AccountBenefitLotRecord>>;

    async fn find_account_hold_by_request_id(
        &mut self,
        request_id: u64,
    ) -> Result<Option<AccountHoldRecord>>;

    async fn list_account_hold_allocations_for_hold(
        &mut self,
        hold_id: u64,
    ) -> Result<Vec<AccountHoldAllocationRecord>>;

    async fn find_request_settlement_by_request_id(
        &mut self,
        request_id: u64,
    ) -> Result<Option<RequestSettlementRecord>>;

    async fn find_request_settlement_record(
        &mut self,
        request_settlement_id: u64,
    ) -> Result<Option<RequestSettlementRecord>>;

    async fn find_account_ledger_entry_record(
        &mut self,
        ledger_entry_id: u64,
    ) -> Result<Option<AccountLedgerEntryRecord>>;

    async fn list_account_ledger_allocations_for_entry(
        &mut self,
        ledger_entry_id: u64,
    ) -> Result<Vec<AccountLedgerAllocationRecord>>;

    async fn upsert_account_benefit_lot(
        &mut self,
        record: &AccountBenefitLotRecord,
    ) -> Result<AccountBenefitLotRecord>;

    async fn upsert_account_hold(
        &mut self,
        record: &AccountHoldRecord,
    ) -> Result<AccountHoldRecord>;

    async fn upsert_account_hold_allocation(
        &mut self,
        record: &AccountHoldAllocationRecord,
    ) -> Result<AccountHoldAllocationRecord>;

    async fn upsert_request_settlement_record(
        &mut self,
        record: &RequestSettlementRecord,
    ) -> Result<RequestSettlementRecord>;

    async fn upsert_account_ledger_entry_record(
        &mut self,
        record: &AccountLedgerEntryRecord,
    ) -> Result<AccountLedgerEntryRecord>;

    async fn upsert_account_ledger_allocation(
        &mut self,
        record: &AccountLedgerAllocationRecord,
    ) -> Result<AccountLedgerAllocationRecord>;
}

pub trait AccountKernelTransactionExecutor: AccountKernelStore {
    fn with_account_kernel_transaction<'a, T, F>(
        &'a self,
        operation: F,
    ) -> AccountKernelTransactionFuture<'a, T>
    where
        T: Send + 'a,
        F: for<'tx> FnOnce(
                &'tx mut dyn AccountKernelTransaction,
            ) -> AccountKernelTransactionFuture<'tx, T>
            + Send
            + 'a;
}
