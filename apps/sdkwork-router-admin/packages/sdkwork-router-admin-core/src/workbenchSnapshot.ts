import { adminBaseUrl } from 'sdkwork-router-admin-admin-api';
import type {
  AdminAlert,
  AdminSessionUser,
  AdminWorkspaceSnapshot,
  BillingEventSummary,
  BillingSummary,
  CouponRecord,
  ManagedUser,
  OperatorUserRecord,
  PortalUserRecord,
  UsageSummary,
} from 'sdkwork-router-admin-types';

const emptyUsageSummary: UsageSummary = {
  total_requests: 0,
  project_count: 0,
  model_count: 0,
  provider_count: 0,
  projects: [],
  providers: [],
  models: [],
};

const emptyBillingSummary: BillingSummary = {
  total_entries: 0,
  project_count: 0,
  total_units: 0,
  total_amount: 0,
  active_quota_policy_count: 0,
  exhausted_project_count: 0,
  projects: [],
};

const emptyBillingEventSummary: BillingEventSummary = {
  total_events: 0,
  project_count: 0,
  group_count: 0,
  capability_count: 0,
  total_request_count: 0,
  total_units: 0,
  total_input_tokens: 0,
  total_output_tokens: 0,
  total_tokens: 0,
  total_image_count: 0,
  total_audio_seconds: 0,
  total_video_seconds: 0,
  total_music_seconds: 0,
  total_upstream_cost: 0,
  total_customer_charge: 0,
  projects: [],
  groups: [],
  capabilities: [],
  accounting_modes: [],
};

export const emptySnapshot: AdminWorkspaceSnapshot = {
  sessionUser: null,
  operatorUsers: [],
  portalUsers: [],
  coupons: [],
  couponTemplates: [],
  marketingCampaigns: [],
  campaignBudgets: [],
  couponCodes: [],
  couponReservations: [],
  couponRedemptions: [],
  couponRollbacks: [],
  tenants: [],
  projects: [],
  apiKeys: [],
  apiKeyGroups: [],
  routingProfiles: [],
  compiledRoutingSnapshots: [],
  rateLimitPolicies: [],
  rateLimitWindows: [],
  channels: [],
  providers: [],
  credentials: [],
  models: [],
  channelModels: [],
  modelPrices: [],
  usageRecords: [],
  usageSummary: emptyUsageSummary,
  billingEvents: [],
  billingEventSummary: emptyBillingEventSummary,
  billingSummary: emptyBillingSummary,
  commerceOrders: [],
  commercePaymentEvents: [],
  commercialAccounts: [],
  commercialAccountHolds: [],
  commercialAccountLedger: [],
  commercialRequestSettlements: [],
  commercialPricingPlans: [],
  commercialPricingRates: [],
  routingLogs: [],
  providerHealth: [],
  runtimeStatuses: [],
  overviewMetrics: [],
  alerts: [],
};

export function buildManagedUsers(
  operatorDirectory: OperatorUserRecord[],
  portalDirectory: PortalUserRecord[],
  usageRecords: AdminWorkspaceSnapshot['usageRecords'],
  usageSummary: UsageSummary,
  billingSummary: BillingSummary,
): { operatorUsers: ManagedUser[]; portalUsers: ManagedUser[] } {
  const requestsByProject = new Map(
    usageSummary.projects.map((project) => [project.project_id, project.request_count]),
  );
  const unitsByProject = new Map(
    billingSummary.projects.map((project) => [project.project_id, project.used_units]),
  );
  const tokensByProject = new Map<string, number>();

  for (const record of usageRecords) {
    tokensByProject.set(
      record.project_id,
      (tokensByProject.get(record.project_id) ?? 0) + record.total_tokens,
    );
  }

  const operatorUsers = operatorDirectory.map<ManagedUser>((user) => ({
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    role: 'operator',
    active: user.active,
    request_count: 0,
    usage_units: 0,
    total_tokens: 0,
    source: 'live',
  }));

  const portalUsers = portalDirectory.map<ManagedUser>((user) => ({
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    role: 'portal',
    active: user.active,
    workspace_tenant_id: user.workspace_tenant_id,
    workspace_project_id: user.workspace_project_id,
    request_count: requestsByProject.get(user.workspace_project_id) ?? 0,
    usage_units: unitsByProject.get(user.workspace_project_id) ?? 0,
    total_tokens: tokensByProject.get(user.workspace_project_id) ?? 0,
    source: 'live',
  }));

  return { operatorUsers, portalUsers };
}

export function buildOverviewMetrics(
  snapshot: Omit<AdminWorkspaceSnapshot, 'overviewMetrics' | 'alerts'>,
) {
  const coveredProviders = new Set(
    snapshot.credentials.map((credential) => credential.provider_id),
  );

  return [
    {
      label: 'Admin API base',
      value: adminBaseUrl(),
      detail: 'Independent admin project talking to the operator control plane.',
    },
    {
      label: 'Managed users',
      value: String(snapshot.operatorUsers.length + snapshot.portalUsers.length),
      detail: 'Combined operator and portal inventory.',
    },
    {
      label: 'Active models',
      value: String(snapshot.models.length),
      detail: 'Models currently exposed through the routing catalog.',
    },
    {
      label: 'Credential coverage',
      value: `${coveredProviders.size}/${snapshot.providers.length}`,
      detail: 'Providers currently backed by at least one upstream credential.',
    },
    {
      label: 'Request volume',
      value: String(snapshot.usageSummary.total_requests),
      detail: 'Total requests recorded by the usage summary.',
    },
  ];
}

export function buildAlerts(
  snapshot: Omit<AdminWorkspaceSnapshot, 'overviewMetrics' | 'alerts'>,
): AdminAlert[] {
  const alerts: AdminAlert[] = [];
  const coveredProviders = new Set(
    snapshot.credentials.map((credential) => credential.provider_id),
  );
  const providersWithoutCredential = snapshot.providers.filter(
    (provider) => !coveredProviders.has(provider.id),
  );

  if (!snapshot.models.length) {
    alerts.push({
      id: 'no-models',
      title: 'No model catalog entries',
      detail: 'The routing layer has no published models. Create or upsert models in Catalog.',
      severity: 'high',
    });
  }

  if (snapshot.billingSummary.exhausted_project_count > 0) {
    alerts.push({
      id: 'quota-exhausted',
      title: 'Projects with exhausted quota',
      detail: `${snapshot.billingSummary.exhausted_project_count} projects have exhausted their quota posture.`,
      severity: 'high',
    });
  }

  if (snapshot.runtimeStatuses.some((runtime) => !runtime.healthy)) {
    alerts.push({
      id: 'runtime-risk',
      title: 'Runtime health degradation detected',
      detail: 'One or more managed runtimes are unhealthy. Review the Operations module.',
      severity: 'medium',
    });
  }

  if (providersWithoutCredential.length > 0) {
    alerts.push({
      id: 'credential-gap',
      title: 'Provider credentials are missing',
      detail: `${providersWithoutCredential.length} providers have no credential coverage. Rotate or create credentials in Catalog before routing live traffic.`,
      severity: 'medium',
    });
  }

  alerts.push({
    id: 'coupon-repository',
    title: 'Coupon campaigns are live-backed',
    detail: 'Coupon operations now persist through the admin control plane instead of local workspace state.',
    severity: 'low',
  });

  return alerts;
}

export function buildSnapshot(
  sessionUser: AdminSessionUser,
  coupons: CouponRecord[],
  liveData: Omit<
    AdminWorkspaceSnapshot,
    'sessionUser' | 'operatorUsers' | 'portalUsers' | 'coupons' | 'overviewMetrics' | 'alerts'
  > & {
    operatorDirectory: OperatorUserRecord[];
    portalDirectory: PortalUserRecord[];
  },
): AdminWorkspaceSnapshot {
  const { operatorUsers, portalUsers } = buildManagedUsers(
    liveData.operatorDirectory,
    liveData.portalDirectory,
    liveData.usageRecords,
    liveData.usageSummary,
    liveData.billingSummary,
  );
  const {
    operatorDirectory: _operatorDirectory,
    portalDirectory: _portalDirectory,
    ...workspaceData
  } = liveData;

  const base = {
    sessionUser,
    operatorUsers,
    portalUsers,
    coupons,
    ...workspaceData,
  };

  return {
    ...base,
    overviewMetrics: buildOverviewMetrics(base),
    alerts: buildAlerts(base),
  };
}
