export type AdminRouteKey =
  | 'overview'
  | 'users'
  | 'tenants'
  | 'coupons'
  | 'api-keys'
  | 'rate-limits'
  | 'route-config'
  | 'model-mapping'
  | 'usage-records'
  | 'catalog'
  | 'traffic'
  | 'operations'
  | 'settings';

export type ThemeMode = 'light' | 'dark' | 'system';
export type ThemeColor =
  | 'tech-blue'
  | 'lobster'
  | 'green-tech'
  | 'zinc'
  | 'violet'
  | 'rose';

export type AdminSidebarItemKey = AdminRouteKey;

export interface AdminThemePreference {
  mode: ThemeMode;
  color: ThemeColor;
}

export type AdminDataSource = 'live';

export interface AdminSessionUser {
  id: string;
  email: string;
  display_name: string;
  active: boolean;
  created_at_ms: number;
}

export interface AdminAuthSession {
  token: string;
  claims: {
    sub: string;
    iss: string;
    aud: string;
    exp: number;
    iat: number;
  };
  user: AdminSessionUser;
}

export interface AdminRouteDefinition {
  key: AdminRouteKey;
  label: string;
  eyebrow: string;
  detail: string;
  group?: string;
}

export type AdminRouteModuleId =
  | 'sdkwork-router-admin-overview'
  | 'sdkwork-router-admin-users'
  | 'sdkwork-router-admin-tenants'
  | 'sdkwork-router-admin-coupons'
  | 'sdkwork-router-admin-apirouter'
  | 'sdkwork-router-admin-catalog'
  | 'sdkwork-router-admin-traffic'
  | 'sdkwork-router-admin-operations'
  | 'sdkwork-router-admin-settings';

export interface AdminModuleLoadingPolicy {
  strategy: 'lazy';
  prefetch: 'none' | 'intent';
  chunkGroup?: string;
}

export interface AdminModuleNavigationDescriptor {
  group: string;
  order: number;
  sidebar: boolean;
}

export interface AdminProductModuleManifest {
  moduleId: AdminRouteModuleId;
  pluginId: AdminRouteModuleId;
  pluginKind: 'admin-module';
  packageName: AdminRouteModuleId;
  displayName: string;
  routeKeys: AdminRouteKey[];
  capabilityTags: string[];
  requiredPermissions: string[];
  navigation: AdminModuleNavigationDescriptor;
  loading: AdminModuleLoadingPolicy;
}

export interface AdminRouteManifestEntry extends AdminRouteDefinition {
  path: string;
  moduleId: AdminRouteModuleId;
  prefetchGroup?: string;
  productModule: AdminProductModuleManifest;
}

export interface ManagedUser {
  id: string;
  email: string;
  display_name: string;
  role: 'operator' | 'portal';
  active: boolean;
  workspace_tenant_id?: string;
  workspace_project_id?: string;
  request_count: number;
  usage_units: number;
  total_tokens: number;
  source: AdminDataSource;
}

export interface OperatorUserRecord {
  id: string;
  email: string;
  display_name: string;
  active: boolean;
  created_at_ms: number;
}

export interface PortalUserRecord {
  id: string;
  email: string;
  display_name: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
  created_at_ms: number;
}

export interface CouponRecord {
  id: string;
  code: string;
  discount_label: string;
  audience: string;
  remaining: number;
  active: boolean;
  note: string;
  expires_on: string;
}

export interface TenantRecord {
  id: string;
  name: string;
}

export interface ProjectRecord {
  tenant_id: string;
  id: string;
  name: string;
}

export interface GatewayApiKeyRecord {
  tenant_id: string;
  project_id: string;
  environment: string;
  hashed_key: string;
  api_key_group_id?: string | null;
  raw_key?: string | null;
  label: string;
  notes?: string | null;
  created_at_ms: number;
  last_used_at_ms?: number | null;
  expires_at_ms?: number | null;
  active: boolean;
}

export interface ApiKeyGroupRecord {
  group_id: string;
  tenant_id: string;
  project_id: string;
  environment: string;
  name: string;
  slug: string;
  description?: string | null;
  color?: string | null;
  default_capability_scope?: string | null;
  default_routing_profile_id?: string | null;
  default_accounting_mode?: string | null;
  active: boolean;
  created_at_ms: number;
  updated_at_ms: number;
}

export type BillingAccountingMode = 'platform_credit' | 'byok' | 'passthrough';

export interface RoutingProfileRecord {
  profile_id: string;
  tenant_id: string;
  project_id: string;
  name: string;
  slug: string;
  description?: string | null;
  active: boolean;
  strategy: string;
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy: boolean;
  preferred_region?: string | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface CompiledRoutingSnapshotRecord {
  snapshot_id: string;
  tenant_id?: string | null;
  project_id?: string | null;
  api_key_group_id?: string | null;
  capability: string;
  route_key: string;
  matched_policy_id?: string | null;
  project_routing_preferences_project_id?: string | null;
  applied_routing_profile_id?: string | null;
  strategy: string;
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy: boolean;
  preferred_region?: string | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface RateLimitPolicyRecord {
  policy_id: string;
  project_id: string;
  api_key_hash?: string | null;
  route_key?: string | null;
  model_name?: string | null;
  requests_per_window: number;
  window_seconds: number;
  burst_requests: number;
  limit_requests: number;
  enabled: boolean;
  notes?: string | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface RateLimitWindowRecord {
  policy_id: string;
  project_id: string;
  api_key_hash?: string | null;
  route_key?: string | null;
  model_name?: string | null;
  requests_per_window: number;
  window_seconds: number;
  burst_requests: number;
  limit_requests: number;
  request_count: number;
  remaining_requests: number;
  window_start_ms: number;
  window_end_ms: number;
  updated_at_ms: number;
  enabled: boolean;
  exceeded: boolean;
}

export interface CreatedGatewayApiKey {
  plaintext: string;
  hashed: string;
  tenant_id: string;
  project_id: string;
  environment: string;
  api_key_group_id?: string | null;
  label: string;
  notes?: string | null;
  created_at_ms: number;
  expires_at_ms?: number | null;
}

export interface ChannelRecord {
  id: string;
  name: string;
}

export interface ProviderChannelBinding {
  channel_id: string;
  is_primary: boolean;
}

export interface ProxyProviderRecord {
  id: string;
  channel_id: string;
  extension_id?: string | null;
  adapter_kind: string;
  base_url: string;
  display_name: string;
  channel_bindings: ProviderChannelBinding[];
}

export interface ModelCatalogRecord {
  external_name: string;
  provider_id: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number | null;
}

export interface ChannelModelRecord {
  channel_id: string;
  model_id: string;
  model_display_name: string;
  capabilities: string[];
  streaming: boolean;
  context_window?: number | null;
  description?: string | null;
}

export interface ModelPriceRecord {
  channel_id: string;
  model_id: string;
  proxy_provider_id: string;
  currency_code: string;
  price_unit: string;
  input_price: number;
  output_price: number;
  cache_read_price: number;
  cache_write_price: number;
  request_price: number;
  is_active: boolean;
}

export interface CredentialRecord {
  tenant_id: string;
  provider_id: string;
  key_reference: string;
  secret_backend: string;
  secret_local_file?: string | null;
  secret_keyring_service?: string | null;
  secret_master_key_id?: string | null;
}

export interface UsageRecord {
  project_id: string;
  model: string;
  provider: string;
  units: number;
  amount: number;
  api_key_hash?: string | null;
  channel_id?: string | null;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  latency_ms?: number | null;
  reference_amount?: number | null;
  created_at_ms: number;
}

export interface UsageSummary {
  total_requests: number;
  project_count: number;
  model_count: number;
  provider_count: number;
  projects: Array<{ project_id: string; request_count: number }>;
  providers: Array<{ provider: string; request_count: number; project_count: number }>;
  models: Array<{ model: string; request_count: number; provider_count: number }>;
}

export interface BillingSummary {
  total_entries: number;
  project_count: number;
  total_units: number;
  total_amount: number;
  active_quota_policy_count: number;
  exhausted_project_count: number;
  projects: Array<{
    project_id: string;
    entry_count: number;
    used_units: number;
    booked_amount: number;
    quota_policy_id?: string | null;
    quota_limit_units?: number | null;
    remaining_units?: number | null;
    exhausted: boolean;
  }>;
}

export interface BillingEventRecord {
  event_id: string;
  tenant_id: string;
  project_id: string;
  api_key_group_id?: string | null;
  capability: string;
  route_key: string;
  usage_model: string;
  provider_id: string;
  accounting_mode: BillingAccountingMode;
  operation_kind: string;
  modality: string;
  api_key_hash?: string | null;
  channel_id?: string | null;
  reference_id?: string | null;
  latency_ms?: number | null;
  units: number;
  request_count: number;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  cache_read_tokens: number;
  cache_write_tokens: number;
  image_count: number;
  audio_seconds: number;
  video_seconds: number;
  music_seconds: number;
  upstream_cost: number;
  customer_charge: number;
  applied_routing_profile_id?: string | null;
  compiled_routing_snapshot_id?: string | null;
  fallback_reason?: string | null;
  created_at_ms: number;
}

export interface BillingEventProjectSummary {
  project_id: string;
  event_count: number;
  request_count: number;
  total_units: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  total_image_count: number;
  total_audio_seconds: number;
  total_video_seconds: number;
  total_music_seconds: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingEventGroupSummary {
  api_key_group_id?: string | null;
  project_count: number;
  event_count: number;
  request_count: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingEventCapabilitySummary {
  capability: string;
  event_count: number;
  request_count: number;
  total_tokens: number;
  image_count: number;
  audio_seconds: number;
  video_seconds: number;
  music_seconds: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingEventAccountingModeSummary {
  accounting_mode: BillingAccountingMode;
  event_count: number;
  request_count: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingEventSummary {
  total_events: number;
  project_count: number;
  group_count: number;
  capability_count: number;
  total_request_count: number;
  total_units: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  total_image_count: number;
  total_audio_seconds: number;
  total_video_seconds: number;
  total_music_seconds: number;
  total_upstream_cost: number;
  total_customer_charge: number;
  projects: BillingEventProjectSummary[];
  groups: BillingEventGroupSummary[];
  capabilities: BillingEventCapabilitySummary[];
  accounting_modes: BillingEventAccountingModeSummary[];
}

export interface RoutingDecisionLogRecord {
  decision_id: string;
  decision_source: string;
  capability: string;
  route_key: string;
  selected_provider_id: string;
  strategy?: string | null;
  selection_reason?: string | null;
  compiled_routing_snapshot_id?: string | null;
  fallback_reason?: string | null;
  requested_region?: string | null;
  selection_seed?: number | null;
  slo_applied: boolean;
  slo_degraded: boolean;
  created_at_ms: number;
}

export interface ProviderHealthSnapshot {
  provider_id: string;
  status: string;
  healthy: boolean;
  message?: string | null;
  observed_at_ms: number;
}

export interface RuntimeStatusRecord {
  runtime: string;
  extension_id: string;
  instance_id?: string | null;
  display_name: string;
  running: boolean;
  healthy: boolean;
  message?: string | null;
}

export interface RuntimeReloadReport {
  scope: string;
  requested_extension_id?: string | null;
  requested_instance_id?: string | null;
  resolved_extension_id?: string | null;
  discovered_package_count: number;
  loadable_package_count: number;
  active_runtime_count: number;
  reloaded_at_ms: number;
  runtime_statuses: RuntimeStatusRecord[];
}

export interface OverviewMetric {
  label: string;
  value: string;
  detail: string;
}

export interface AdminAlert {
  id: string;
  title: string;
  detail: string;
  severity: 'high' | 'medium' | 'low';
}

export interface AdminWorkspaceSnapshot {
  sessionUser: AdminSessionUser | null;
  operatorUsers: ManagedUser[];
  portalUsers: ManagedUser[];
  coupons: CouponRecord[];
  tenants: TenantRecord[];
  projects: ProjectRecord[];
  apiKeys: GatewayApiKeyRecord[];
  apiKeyGroups: ApiKeyGroupRecord[];
  routingProfiles: RoutingProfileRecord[];
  compiledRoutingSnapshots: CompiledRoutingSnapshotRecord[];
  rateLimitPolicies: RateLimitPolicyRecord[];
  rateLimitWindows: RateLimitWindowRecord[];
  channels: ChannelRecord[];
  providers: ProxyProviderRecord[];
  credentials: CredentialRecord[];
  models: ModelCatalogRecord[];
  channelModels: ChannelModelRecord[];
  modelPrices: ModelPriceRecord[];
  usageRecords: UsageRecord[];
  usageSummary: UsageSummary;
  billingEvents: BillingEventRecord[];
  billingEventSummary: BillingEventSummary;
  billingSummary: BillingSummary;
  routingLogs: RoutingDecisionLogRecord[];
  providerHealth: ProviderHealthSnapshot[];
  runtimeStatuses: RuntimeStatusRecord[];
  overviewMetrics: OverviewMetric[];
  alerts: AdminAlert[];
}

export interface AdminPageProps {
  snapshot: AdminWorkspaceSnapshot;
}
