import { create } from 'zustand';
import {
  clearPortalSessionToken,
  getPortalDashboard,
  getPortalMe,
  getPortalWorkspace,
  loginPortalUser,
  onPortalSessionExpired,
  persistPortalSessionToken,
  portalErrorMessage,
  PortalApiError,
  readPortalSessionToken,
  registerPortalUser,
} from 'sdkwork-router-portal-portal-api';
import type {
  PortalAuthSession,
  PortalDashboardSummary,
  PortalUserProfile,
  PortalWorkspaceSummary,
} from 'sdkwork-router-portal-types';
const DEFAULT_BOOTSTRAP_STATUS_KEY = 'Checking for an existing portal session token.';
const DEFAULT_DASHBOARD_STATUS_KEY = 'Sign in to load the current workspace status.';
const SYNC_SESSION_BOOTSTRAP_STATUS_KEY = 'Refreshing workspace identity and dashboard context.';
const SYNC_SESSION_DASHBOARD_STATUS_KEY = 'Refreshing workspace status after sign-in.';
const HYDRATE_BOOTSTRAP_STATUS_KEY = 'Refreshing workspace identity and navigation context.';
const HYDRATE_DASHBOARD_STATUS_KEY = 'Refreshing workspace status for the active project.';
const WORKSPACE_RESTORED_STATUS_KEY = 'Workspace identity restored.';
const SESSION_ENDED_STATUS_KEY = 'Your portal session ended. Sign in again to continue.';
const SESSION_INVALID_STATUS_KEY = 'The saved portal session is no longer valid. Please sign in again.';
const SESSION_EXPIRED_STATUS_KEY = 'Your portal session expired. Sign in again to continue.';
const DASHBOARD_SYNCED_STATUS_KEY = 'Workspace status is synced with the latest dashboard snapshot.';

interface PortalAuthState {
  isAuthenticated: boolean;
  isBootstrapping: boolean;
  sessionToken: string | null;
  user: PortalUserProfile | null;
  workspace: PortalWorkspaceSummary | null;
  dashboardSnapshot: PortalDashboardSummary | null;
  bootstrapStatus: string;
  dashboardStatus: string;
  signIn: (credentials: { email: string; password: string }) => Promise<PortalAuthSession>;
  register: (payload: { name: string; email: string; password: string }) => Promise<PortalAuthSession>;
  signOut: (message?: string) => Promise<void>;
  hydrate: () => Promise<void>;
  syncWorkspace: (token?: string) => Promise<PortalWorkspaceSummary | null>;
  syncDashboard: (token?: string) => Promise<PortalDashboardSummary | null>;
}

function signedOutState(message = DEFAULT_BOOTSTRAP_STATUS_KEY) {
  return {
    isAuthenticated: false,
    isBootstrapping: false,
    sessionToken: null,
    user: null,
    workspace: null,
    dashboardSnapshot: null,
    bootstrapStatus: message,
    dashboardStatus: DEFAULT_DASHBOARD_STATUS_KEY,
  } as const;
}

async function syncSessionState(
  session: PortalAuthSession,
  get: () => PortalAuthState,
  set: (
    partial:
      | Partial<PortalAuthState>
      | ((state: PortalAuthState) => Partial<PortalAuthState>),
  ) => void,
): Promise<void> {
  persistPortalSessionToken(session.token);

  set({
    isAuthenticated: true,
    isBootstrapping: true,
    sessionToken: session.token,
    user: session.user,
    bootstrapStatus: SYNC_SESSION_BOOTSTRAP_STATUS_KEY,
    dashboardStatus: SYNC_SESSION_DASHBOARD_STATUS_KEY,
  });

  await get().syncWorkspace(session.token);
  await get().syncDashboard(session.token);

  set({
    isAuthenticated: true,
    isBootstrapping: false,
    sessionToken: session.token,
    user: session.user,
    bootstrapStatus: WORKSPACE_RESTORED_STATUS_KEY,
  });
}

export const usePortalAuthStore = create<PortalAuthState>()(
  (set, get) => ({
    ...signedOutState(),
    signIn: async (credentials) => {
      const session = await loginPortalUser(credentials);
      await syncSessionState(session, get, set);
      return session;
    },
    register: async (payload) => {
      const session = await registerPortalUser({
        display_name: payload.name,
        email: payload.email,
        password: payload.password,
      });
      await syncSessionState(session, get, set);
      return session;
    },
    signOut: async (message) => {
      clearPortalSessionToken();
      set(signedOutState(message ?? SESSION_ENDED_STATUS_KEY));
    },
    hydrate: async () => {
      const persistedToken = readPortalSessionToken();

      if (!persistedToken) {
        set(signedOutState());
        return;
      }

      set({
        isBootstrapping: true,
        bootstrapStatus: HYDRATE_BOOTSTRAP_STATUS_KEY,
        dashboardStatus: HYDRATE_DASHBOARD_STATUS_KEY,
        sessionToken: persistedToken,
      });

      try {
        const [user, workspace] = await Promise.all([
          getPortalMe(persistedToken),
          getPortalWorkspace(persistedToken),
        ]);

        set({
          isAuthenticated: true,
          isBootstrapping: true,
          sessionToken: persistedToken,
          user,
          workspace,
          bootstrapStatus: WORKSPACE_RESTORED_STATUS_KEY,
        });

        await get().syncDashboard(persistedToken);

        set({
          isAuthenticated: true,
          isBootstrapping: false,
          sessionToken: persistedToken,
          user,
          workspace,
        });
      } catch (error) {
        const nextMessage =
          error instanceof PortalApiError && error.status === 401
            ? SESSION_INVALID_STATUS_KEY
            : portalErrorMessage(error);

        clearPortalSessionToken();
        set(signedOutState(nextMessage));
      }
    },
    syncWorkspace: async (token) => {
      const currentToken = token ?? get().sessionToken ?? readPortalSessionToken();

      if (!currentToken) {
        set({ workspace: null });
        return null;
      }

      try {
        const workspace = await getPortalWorkspace(currentToken);
        set({
          isAuthenticated: true,
          sessionToken: currentToken,
          workspace,
        });
        return workspace;
      } catch (error) {
        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          set(signedOutState(SESSION_EXPIRED_STATUS_KEY));
          return null;
        }

        set({
          bootstrapStatus: portalErrorMessage(error),
        });
        return null;
      }
    },
    syncDashboard: async (token) => {
      const currentToken = token ?? get().sessionToken ?? readPortalSessionToken();

      if (!currentToken) {
        set({
          dashboardSnapshot: null,
          dashboardStatus: DEFAULT_DASHBOARD_STATUS_KEY,
        });
        return null;
      }

      try {
        const dashboardSnapshot = await getPortalDashboard(currentToken);
        set({
          isAuthenticated: true,
          sessionToken: currentToken,
          dashboardSnapshot,
          dashboardStatus: DASHBOARD_SYNCED_STATUS_KEY,
        });
        return dashboardSnapshot;
      } catch (error) {
        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          set(signedOutState(SESSION_EXPIRED_STATUS_KEY));
          return null;
        }

        set({
          dashboardStatus: portalErrorMessage(error),
        });
        return null;
      }
    },
  }),
);

export function subscribeToPortalSessionExpiry() {
  return onPortalSessionExpired(() => {
    void usePortalAuthStore
      .getState()
      .signOut(SESSION_EXPIRED_STATUS_KEY);
  });
}
