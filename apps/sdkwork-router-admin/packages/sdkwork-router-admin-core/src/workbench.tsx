import {
  createContext,
  startTransition,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from 'react';

import {
  clearAdminSessionToken,
  getBillingEventSummary,
  getAdminMe,
  getBillingSummary,
  getUsageSummary,
  listApiKeys,
  listApiKeyGroups,
  listBillingEvents,
  listChannelModels,
  listChannels,
  listCompiledRoutingSnapshots,
  listCoupons,
  listCredentials,
  listModelPrices,
  listModels,
  listOperatorUsers,
  listPortalUsers,
  listProjects,
  listProviderHealthSnapshots,
  listProviders,
  listRateLimitPolicies,
  listRateLimitWindows,
  listRoutingProfiles,
  listRoutingDecisionLogs,
  listRuntimeStatuses,
  listTenants,
  listUsageRecords,
  loginAdminUser,
  persistAdminSessionToken,
  readAdminSessionToken,
} from 'sdkwork-router-admin-admin-api';
import type {
  AdminSessionUser,
  AdminWorkspaceSnapshot,
} from 'sdkwork-router-admin-types';
import {
  createWorkbenchActions,
  type WorkbenchActions,
} from './workbenchActions';
import { buildSnapshot, emptySnapshot } from './workbenchSnapshot';

interface AdminWorkbenchContextValue extends WorkbenchActions {
  authResolved: boolean;
  sessionUser: AdminSessionUser | null;
  snapshot: AdminWorkspaceSnapshot;
  status: string;
  loading: boolean;
  refreshWorkspace: (explicitSessionUser?: AdminSessionUser | null) => Promise<void>;
  handleLogin: (input: { email: string; password: string }) => Promise<void>;
  handleLogout: () => void;
}

const AdminWorkbenchContext = createContext<AdminWorkbenchContextValue | null>(null);

export function AdminWorkbenchProvider({ children }: { children: ReactNode }) {
  const [authResolved, setAuthResolved] = useState(false);
  const [sessionUser, setSessionUser] = useState<AdminSessionUser | null>(null);
  const [snapshot, setSnapshot] = useState<AdminWorkspaceSnapshot>(emptySnapshot);
  const [status, setStatus] = useState('Authenticate to open the super-admin workspace.');
  const [loading, setLoading] = useState(false);

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
        apiKeyGroups,
        routingProfiles,
        compiledRoutingSnapshots,
        rateLimitPolicies,
        rateLimitWindows,
        channels,
        providers,
        credentials,
        models,
        channelModels,
        modelPrices,
        usageRecords,
        usageSummary,
        billingEvents,
        billingEventSummary,
        billingSummary,
        routingLogs,
        providerHealth,
        runtimeStatuses,
      ] = await Promise.all([
        listCoupons(),
        listTenants(),
        listProjects(),
        listApiKeys(),
        listApiKeyGroups(),
        listRoutingProfiles(),
        listCompiledRoutingSnapshots(),
        listRateLimitPolicies(),
        listRateLimitWindows(),
        listChannels(),
        listProviders(),
        listCredentials(),
        listModels(),
        listChannelModels(),
        listModelPrices(),
        listUsageRecords(),
        getUsageSummary(),
        listBillingEvents(),
        getBillingEventSummary(),
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
        apiKeyGroups,
        routingProfiles,
        compiledRoutingSnapshots,
        rateLimitPolicies,
        rateLimitWindows,
        channels,
        providers,
        credentials,
        models,
        channelModels,
        modelPrices,
        usageRecords,
        usageSummary,
        billingEvents,
        billingEventSummary,
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
      setStatus(
        error instanceof Error ? error.message : 'Failed to refresh admin workspace.',
      );
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    const token = readAdminSessionToken();

    if (!token) {
      setAuthResolved(true);
      return;
    }

    let cancelled = false;

    void getAdminMe(token)
      .then(async (user) => {
        if (cancelled) {
          return;
        }

        setSessionUser(user);
        await refreshWorkspace(user);
      })
      .catch(() => {
        clearAdminSessionToken();
      })
      .finally(() => {
        if (!cancelled) {
          setAuthResolved(true);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

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
      setAuthResolved(true);
    }
  }

  function handleLogout() {
    clearAdminSessionToken();
    setSessionUser(null);
    setSnapshot(emptySnapshot);
    setStatus('Signed out of the super-admin workspace.');
    setAuthResolved(true);
  }
  const actions = createWorkbenchActions({
    refreshWorkspace: () => refreshWorkspace(),
    setStatus,
  });

  const value: AdminWorkbenchContextValue = {
    authResolved,
    sessionUser,
    snapshot,
    status,
    loading,
    refreshWorkspace,
    handleLogin,
    handleLogout,
    ...actions,
  };

  return (
    <AdminWorkbenchContext.Provider value={value}>
      {children}
    </AdminWorkbenchContext.Provider>
  );
}

export function useAdminWorkbench(): AdminWorkbenchContextValue {
  const context = useContext(AdminWorkbenchContext);

  if (!context) {
    throw new Error('useAdminWorkbench must be used within AdminWorkbenchProvider.');
  }

  return context;
}
