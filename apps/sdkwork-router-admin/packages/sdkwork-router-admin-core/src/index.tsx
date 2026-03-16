import { startTransition, useEffect, useState } from 'react';

import {
  adminBaseUrl,
  clearAdminSessionToken,
  createApiKey,
  deleteApiKey,
  deleteCredential,
  getAdminMe,
  getBillingSummary,
  getUsageSummary,
  listApiKeys,
  listChannels,
  listCoupons,
  listCredentials,
  listModels,
  listOperatorUsers,
  listPortalUsers,
  listProjects,
  listProviderHealthSnapshots,
  listProviders,
  reloadExtensionRuntimes,
  listRoutingDecisionLogs,
  listRuntimeStatuses,
  listTenants,
  listUsageRecords,
  loginAdminUser,
  persistAdminSessionToken,
  readAdminSessionToken,
  saveChannel,
  saveCoupon,
  saveCredential,
  saveModel,
  saveOperatorUser,
  savePortalUser,
  saveProject,
  saveProvider,
  saveTenant,
  deleteCoupon,
  deleteChannel,
  deleteModel,
  deleteOperatorUser,
  deletePortalUser,
  deleteProject,
  deleteProvider,
  deleteTenant,
  updateOperatorUserStatus,
  updateApiKeyStatus,
  updatePortalUserStatus,
} from 'sdkwork-router-admin-admin-api';
import { AdminLoginPage } from 'sdkwork-router-admin-auth';
import { CatalogPage } from 'sdkwork-router-admin-catalog';
import { InlineButton, Pill } from 'sdkwork-router-admin-commons';
import { CouponsPage } from 'sdkwork-router-admin-coupons';
import { OperationsPage } from 'sdkwork-router-admin-operations';
import { OverviewPage } from 'sdkwork-router-admin-overview';
import { TenantsPage } from 'sdkwork-router-admin-tenants';
import { TrafficPage } from 'sdkwork-router-admin-traffic';
import type {
  AdminAlert,
  AdminRouteKey,
  AdminSessionUser,
  AdminWorkspaceSnapshot,
  BillingSummary,
  CouponRecord,
  CreatedGatewayApiKey,
  ManagedUser,
  OperatorUserRecord,
  PortalUserRecord,
  RuntimeReloadReport,
  UsageSummary,
} from 'sdkwork-router-admin-types';
import { UsersPage } from 'sdkwork-router-admin-users';

import { adminRoutes } from './routes';

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

function normalizeRoute(hash: string): AdminRouteKey {
  const candidate = hash.replace(/^#/, '') as AdminRouteKey;
  return adminRoutes.some((route) => route.key === candidate) ? candidate : 'overview';
}

function writeRoute(route: AdminRouteKey): void {
  window.location.hash = route;
}

function buildManagedUsers(
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

function buildOverviewMetrics(snapshot: Omit<AdminWorkspaceSnapshot, 'overviewMetrics' | 'alerts'>) {
  const coveredProviders = new Set(snapshot.credentials.map((credential) => credential.provider_id));

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

function buildAlerts(snapshot: Omit<AdminWorkspaceSnapshot, 'overviewMetrics' | 'alerts'>): AdminAlert[] {
  const alerts: AdminAlert[] = [];
  const coveredProviders = new Set(snapshot.credentials.map((credential) => credential.provider_id));
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

function buildSnapshot(
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
  const { operatorDirectory: _operatorDirectory, portalDirectory: _portalDirectory, ...workspaceData } = liveData;

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

const emptySnapshot: AdminWorkspaceSnapshot = {
  sessionUser: null,
  operatorUsers: [],
  portalUsers: [],
  coupons: [],
  tenants: [],
  projects: [],
  apiKeys: [],
  channels: [],
  providers: [],
  credentials: [],
  models: [],
  usageRecords: [],
  usageSummary: emptyUsageSummary,
  billingSummary: emptyBillingSummary,
  routingLogs: [],
  providerHealth: [],
  runtimeStatuses: [],
  overviewMetrics: [],
  alerts: [],
};

export function AdminWorkbenchApp() {
  const [route, setRoute] = useState<AdminRouteKey>(() => normalizeRoute(window.location.hash));
  const [sessionUser, setSessionUser] = useState<AdminSessionUser | null>(null);
  const [snapshot, setSnapshot] = useState<AdminWorkspaceSnapshot>(emptySnapshot);
  const [status, setStatus] = useState('Authenticate to open the super-admin workspace.');
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState('');

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(normalizeRoute(window.location.hash));
    };

    window.addEventListener('hashchange', handleHashChange);
    if (!window.location.hash) {
      writeRoute('overview');
    }

    return () => {
      window.removeEventListener('hashchange', handleHashChange);
    };
  }, []);

  useEffect(() => {
    const token = readAdminSessionToken();
    if (!token) {
      return;
    }

    let cancelled = false;

    void getAdminMe(token)
      .then((user) => {
        if (cancelled) {
          return;
        }
        setSessionUser(user);
        void refreshWorkspace(user);
      })
      .catch(() => {
        clearAdminSessionToken();
      });

    return () => {
      cancelled = true;
    };
  }, []);

  function navigate(nextRoute: AdminRouteKey) {
    writeRoute(nextRoute);
  }

  async function refreshWorkspace(explicitSessionUser = sessionUser) {
    if (!explicitSessionUser) {
      return;
    }

    setLoading(true);
    setStatus('Refreshing live admin data...');

    try {
      const [operatorDirectory, portalDirectory] = await Promise.all([
        listOperatorUsers(),
        listPortalUsers(),
      ]);

      const [
        coupons,
        tenants,
        projects,
        apiKeys,
        channels,
        providers,
        credentials,
        models,
        usageRecords,
        usageSummary,
        billingSummary,
        routingLogs,
        providerHealth,
        runtimeStatuses,
      ] = await Promise.all([
        listCoupons(),
        listTenants(),
        listProjects(),
        listApiKeys(),
        listChannels(),
        listProviders(),
        listCredentials(),
        listModels(),
        listUsageRecords(),
        getUsageSummary(),
        getBillingSummary(),
        listRoutingDecisionLogs(),
        listProviderHealthSnapshots(),
        listRuntimeStatuses(),
      ]);

      const nextSnapshot = buildSnapshot(explicitSessionUser, coupons, {
        operatorDirectory,
        portalDirectory,
        tenants,
        projects,
        apiKeys,
        channels,
        providers,
        credentials,
        models,
        usageRecords,
        usageSummary,
        billingSummary,
        routingLogs,
        providerHealth,
        runtimeStatuses,
      });

      startTransition(() => {
        setSnapshot(nextSnapshot);
        setStatus('Live control-plane data synchronized.');
      });
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to refresh admin workspace.');
    } finally {
      setLoading(false);
    }
  }

  async function handleLogin(input: { email: string; password: string }) {
    setLoading(true);
    setStatus('Establishing operator session...');

    try {
      const session = await loginAdminUser(input);
      persistAdminSessionToken(session.token);
      setSessionUser(session.user);
      setStatus('Operator session established. Loading super-admin workspace...');
      await refreshWorkspace(session.user);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Login failed.');
    } finally {
      setLoading(false);
    }
  }

  function handleLogout() {
    clearAdminSessionToken();
    setSessionUser(null);
    setSnapshot(emptySnapshot);
    setStatus('Signed out of the super-admin workspace.');
  }

  async function handleSaveOperatorUser(input: {
    id?: string;
    email: string;
    display_name: string;
    password?: string;
    active: boolean;
  }) {
    setStatus(input.id ? 'Updating operator identity...' : 'Provisioning operator identity...');
    try {
      await saveOperatorUser(input);
      await refreshWorkspace();
      setStatus('Operator user saved.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to save operator user.');
    }
  }

  async function handleSavePortalUser(input: {
    id?: string;
    email: string;
    display_name: string;
    password?: string;
    workspace_tenant_id: string;
    workspace_project_id: string;
    active: boolean;
  }) {
    setStatus(input.id ? 'Updating portal identity...' : 'Provisioning portal identity...');
    try {
      await savePortalUser(input);
      await refreshWorkspace();
      setStatus('Portal user saved.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to save portal user.');
    }
  }

  async function handleToggleOperatorUser(userId: string, active: boolean) {
    setStatus(active ? 'Re-activating operator access...' : 'Disabling operator access...');
    try {
      await updateOperatorUserStatus(userId, active);
      await refreshWorkspace();
      setStatus('Operator access updated.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to update operator access.');
    }
  }

  async function handleTogglePortalUser(userId: string, active: boolean) {
    setStatus(active ? 'Re-activating portal access...' : 'Disabling portal access...');
    try {
      await updatePortalUserStatus(userId, active);
      await refreshWorkspace();
      setStatus('Portal access updated.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to update portal access.');
    }
  }

  async function handleDeleteOperatorUser(userId: string) {
    setStatus('Deleting operator identity...');
    try {
      await deleteOperatorUser(userId);
      await refreshWorkspace();
      setStatus('Operator user deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete operator user.');
    }
  }

  async function handleDeletePortalUser(userId: string) {
    setStatus('Deleting portal identity...');
    try {
      await deletePortalUser(userId);
      await refreshWorkspace();
      setStatus('Portal user deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete portal user.');
    }
  }

  async function handleSaveCoupon(coupon: CouponRecord) {
    setStatus(coupon.id ? 'Saving coupon campaign...' : 'Creating coupon campaign...');
    try {
      await saveCoupon(coupon);
      await refreshWorkspace();
      setStatus('Coupon campaign saved.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to save coupon.');
    }
  }

  async function handleToggleCoupon(coupon: CouponRecord) {
    setStatus(coupon.active ? 'Archiving coupon campaign...' : 'Restoring coupon campaign...');
    try {
      await saveCoupon({ ...coupon, active: !coupon.active });
      await refreshWorkspace();
      setStatus('Coupon campaign status updated.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to update coupon.');
    }
  }

  async function handleDeleteCoupon(couponId: string) {
    setStatus('Deleting coupon campaign...');
    try {
      await deleteCoupon(couponId);
      await refreshWorkspace();
      setStatus('Coupon campaign deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete coupon.');
    }
  }

  async function handleSaveTenant(input: { id: string; name: string }) {
    await saveTenant(input);
    await refreshWorkspace();
  }

  async function handleDeleteTenant(tenantId: string) {
    setStatus('Deleting tenant...');
    try {
      await deleteTenant(tenantId);
      await refreshWorkspace();
      setStatus('Tenant deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete tenant.');
    }
  }

  async function handleSaveProject(input: { tenant_id: string; id: string; name: string }) {
    await saveProject(input);
    await refreshWorkspace();
  }

  async function handleCreateApiKey(input: {
    tenant_id: string;
    project_id: string;
    environment: string;
  }): Promise<CreatedGatewayApiKey> {
    setStatus('Issuing gateway key...');
    try {
      const created = await createApiKey(input);
      await refreshWorkspace();
      setStatus(`Gateway key issued for ${created.project_id} (${created.environment}).`);
      return created;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to issue gateway key.');
      throw error;
    }
  }

  async function handleUpdateApiKeyStatus(hashedKey: string, active: boolean) {
    setStatus(active ? 'Restoring gateway key...' : 'Revoking gateway key...');
    try {
      await updateApiKeyStatus(hashedKey, active);
      await refreshWorkspace();
      setStatus(active ? 'Gateway key restored.' : 'Gateway key revoked.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to update gateway key.');
    }
  }

  async function handleDeleteApiKey(hashedKey: string) {
    setStatus('Deleting gateway key...');
    try {
      await deleteApiKey(hashedKey);
      await refreshWorkspace();
      setStatus('Gateway key deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete gateway key.');
    }
  }

  async function handleReloadRuntimes(input?: {
    extension_id?: string;
    instance_id?: string;
  }): Promise<RuntimeReloadReport> {
    setStatus('Reloading extension runtimes...');
    try {
      const report = await reloadExtensionRuntimes(input);
      await refreshWorkspace();
      setStatus(`Runtime reload finished. Active runtimes: ${report.active_runtime_count}.`);
      return report;
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to reload runtimes.');
      throw error;
    }
  }

  async function handleDeleteProject(projectId: string) {
    setStatus('Deleting project...');
    try {
      await deleteProject(projectId);
      await refreshWorkspace();
      setStatus('Project deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete project.');
    }
  }

  async function handleSaveChannel(input: { id: string; name: string }) {
    await saveChannel(input);
    await refreshWorkspace();
  }

  async function handleDeleteChannel(channelId: string) {
    setStatus('Deleting channel...');
    try {
      await deleteChannel(channelId);
      await refreshWorkspace();
      setStatus('Channel deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete channel.');
    }
  }

  async function handleSaveProvider(input: {
    id: string;
    channel_id: string;
    extension_id?: string;
    adapter_kind: string;
    base_url: string;
    display_name: string;
    channel_bindings: Array<{ channel_id: string; is_primary: boolean }>;
  }) {
    await saveProvider(input);
    await refreshWorkspace();
  }

  async function handleDeleteProvider(providerId: string) {
    setStatus('Deleting provider...');
    try {
      await deleteProvider(providerId);
      await refreshWorkspace();
      setStatus('Provider deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete provider.');
    }
  }

  async function handleSaveModel(input: {
    external_name: string;
    provider_id: string;
    capabilities: string[];
    streaming: boolean;
    context_window?: number;
  }) {
    await saveModel(input);
    await refreshWorkspace();
  }

  async function handleSaveCredential(input: {
    tenant_id: string;
    provider_id: string;
    key_reference: string;
    secret_value: string;
  }) {
    setStatus('Saving provider credential...');
    try {
      await saveCredential(input);
      await refreshWorkspace();
      setStatus('Provider credential saved.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to save provider credential.');
    }
  }

  async function handleDeleteCredential(
    tenantId: string,
    providerId: string,
    keyReference: string,
  ) {
    setStatus('Deleting provider credential...');
    try {
      await deleteCredential(tenantId, providerId, keyReference);
      await refreshWorkspace();
      setStatus('Provider credential deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete provider credential.');
    }
  }

  async function handleDeleteModel(externalName: string, providerId: string) {
    setStatus('Deleting model...');
    try {
      await deleteModel(externalName, providerId);
      await refreshWorkspace();
      setStatus('Model deleted.');
    } catch (error) {
      setStatus(error instanceof Error ? error.message : 'Failed to delete model.');
    }
  }

  const visibleRoutes = adminRoutes.filter((routeItem) => {
    const value = `${routeItem.label} ${routeItem.detail}`.toLowerCase();
    return value.includes(search.toLowerCase());
  });

  if (!sessionUser) {
    return <AdminLoginPage status={status} loading={loading} onLogin={handleLogin} />;
  }

  return (
    <div className="adminx-app">
      <aside className="adminx-sidebar">
        <div className="adminx-brand">
          <p className="adminx-eyebrow">SDKWork Router Admin</p>
          <h1>Super Admin</h1>
          <p>Independent operator workspace for governance, growth, and runtime operations.</p>
        </div>

        <nav className="adminx-nav">
          {visibleRoutes.map((routeItem) => (
            <button
              key={routeItem.key}
              className={route === routeItem.key ? 'is-active' : undefined}
              onClick={() => navigate(routeItem.key)}
              type="button"
            >
              <strong>{routeItem.label}</strong>
              <span>{routeItem.detail}</span>
            </button>
          ))}
        </nav>

        <div className="adminx-sidebar-card">
          <span>{sessionUser.display_name}</span>
          <span>{sessionUser.email}</span>
          <Pill tone="live">live operator session</Pill>
        </div>

        <InlineButton onClick={handleLogout}>Logout</InlineButton>
      </aside>

      <main className="adminx-main">
        <header className="adminx-topbar">
          <label className="adminx-search">
            <span>Search modules</span>
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="users, coupons, routing..."
            />
          </label>
          <div className="adminx-topbar-actions">
            <Pill tone="live">{status}</Pill>
            <InlineButton onClick={() => void refreshWorkspace()} tone="primary">
              {loading ? 'Refreshing...' : 'Refresh'}
            </InlineButton>
          </div>
        </header>

        {route === 'overview' ? <OverviewPage snapshot={snapshot} onNavigate={navigate} /> : null}
        {route === 'users' ? (
          <UsersPage
            snapshot={snapshot}
            onSaveOperatorUser={handleSaveOperatorUser}
            onSavePortalUser={handleSavePortalUser}
            onToggleOperatorUser={handleToggleOperatorUser}
            onTogglePortalUser={handleTogglePortalUser}
            onDeleteOperatorUser={handleDeleteOperatorUser}
            onDeletePortalUser={handleDeletePortalUser}
          />
        ) : null}
        {route === 'tenants' ? (
          <TenantsPage
            snapshot={snapshot}
            onSaveTenant={handleSaveTenant}
            onSaveProject={handleSaveProject}
            onCreateApiKey={handleCreateApiKey}
            onUpdateApiKeyStatus={handleUpdateApiKeyStatus}
            onDeleteApiKey={handleDeleteApiKey}
            onDeleteTenant={handleDeleteTenant}
            onDeleteProject={handleDeleteProject}
          />
        ) : null}
        {route === 'coupons' ? (
          <CouponsPage
            snapshot={snapshot}
            onSaveCoupon={handleSaveCoupon}
            onToggleCoupon={handleToggleCoupon}
            onDeleteCoupon={handleDeleteCoupon}
          />
        ) : null}
        {route === 'catalog' ? (
          <CatalogPage
            snapshot={snapshot}
            onSaveChannel={handleSaveChannel}
            onSaveProvider={handleSaveProvider}
            onSaveCredential={handleSaveCredential}
            onSaveModel={handleSaveModel}
            onDeleteChannel={handleDeleteChannel}
            onDeleteProvider={handleDeleteProvider}
            onDeleteCredential={handleDeleteCredential}
            onDeleteModel={handleDeleteModel}
          />
        ) : null}
        {route === 'traffic' ? <TrafficPage snapshot={snapshot} /> : null}
        {route === 'operations' ? (
          <OperationsPage
            snapshot={snapshot}
            onReloadRuntimes={handleReloadRuntimes}
          />
        ) : null}
      </main>
    </div>
  );
}
