import { lazy, Suspense } from 'react';
import {
  Navigate,
  Route,
  Routes,
  useLocation,
  useNavigate,
  useSearchParams,
} from 'react-router-dom';
import { usePortalI18n } from 'sdkwork-router-portal-commons';
import type {
  PortalDashboardSummary,
  PortalRouteKey,
  PortalTopLevelRouteKey,
  PortalWorkspaceSummary,
} from 'sdkwork-router-portal-types';

import { MainLayout } from '../layouts/MainLayout';
import { PortalSiteLayout } from '../layouts/PortalSiteLayout';
import { resolvePortalPath } from './routeManifest';
import { PORTAL_ROUTE_PATHS, toRouteElementPath } from './routePaths';

const PortalAuthPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-auth')).AuthPage,
}));
const PortalConsoleRoute = lazy(async () => ({
  default: (await import('sdkwork-router-portal-console')).PortalConsoleRoute,
}));
const PortalApiReferencePage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-api-reference')).PortalApiReferencePage,
}));
const PortalDocsPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-docs')).PortalDocsPage,
}));
const PortalDownloadsPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-downloads')).PortalDownloadsPage,
}));
const PortalHomePage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-home')).PortalHomePage,
}));
const PortalModelsPage = lazy(async () => ({
  default: (await import('sdkwork-router-portal-models')).PortalModelsPage,
}));

function PortalBootScreen({ status }: { status: string }) {
  const { t } = usePortalI18n();

  return (
    <section className="grid min-h-screen place-items-center px-6 py-10">
      <div className="grid w-[min(560px,100%)] gap-4 rounded-[32px] border border-[color:var(--portal-contrast-border)] [background:var(--portal-surface-contrast)] p-8 shadow-[var(--portal-shadow-strong)]">
        <p className="text-xs font-semibold uppercase tracking-[0.24em] text-[var(--portal-text-muted-on-contrast)]">
          {t('Portal Bootstrap')}
        </p>
        <h1 className="text-4xl font-semibold tracking-tight text-[var(--portal-text-on-contrast)]">
          {t('Restoring workspace access')}
        </h1>
        <p className="text-sm leading-6 text-[var(--portal-text-muted-on-contrast)]">{t(status)}</p>
      </div>
    </section>
  );
}

function resolveRedirectTarget(rawTarget: string | null): string {
  if (!rawTarget || !rawTarget.startsWith('/')) {
    return PORTAL_ROUTE_PATHS.dashboard;
  }

  if (
    rawTarget === '/auth' ||
    rawTarget === PORTAL_ROUTE_PATHS.login ||
    rawTarget === PORTAL_ROUTE_PATHS.register ||
    rawTarget === PORTAL_ROUTE_PATHS['forgot-password']
  ) {
    return PORTAL_ROUTE_PATHS.dashboard;
  }

  return rawTarget;
}

function buildAuthHref(pathname: string, redirectTarget?: string): string {
  const params = new URLSearchParams();

  if (redirectTarget && redirectTarget !== PORTAL_ROUTE_PATHS.dashboard) {
    params.set('redirect', redirectTarget);
  }

  const query = params.toString();
  return query ? `${pathname}?${query}` : pathname;
}

function isPublicPortalPath(pathname: string): boolean {
  return (
    pathname === PORTAL_ROUTE_PATHS.home ||
    pathname === PORTAL_ROUTE_PATHS.models ||
    pathname === PORTAL_ROUTE_PATHS['api-reference'] ||
    pathname === PORTAL_ROUTE_PATHS.docs ||
    pathname === PORTAL_ROUTE_PATHS.downloads
  );
}

export function AppRoutes({
  authenticated,
  bootStatus,
  bootstrapped,
  dashboardSnapshot,
  register,
  signIn,
  workspace,
}: {
  authenticated: boolean;
  bootStatus: string;
  bootstrapped: boolean;
  dashboardSnapshot: PortalDashboardSummary | null;
  register: (payload: { name: string; email: string; password: string }) => Promise<unknown>;
  signIn: (credentials: { email: string; password: string }) => Promise<unknown>;
  workspace: PortalWorkspaceSummary | null;
}) {
  const location = useLocation();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const redirectTarget = resolveRedirectTarget(searchParams.get('redirect'));
  const requestedTarget = `${location.pathname}${location.search}`;

  function navigateToRoute(routeKey: PortalRouteKey | PortalTopLevelRouteKey) {
    navigate(resolvePortalPath(routeKey));
  }

  function renderProtectedRoute(routeKey: PortalRouteKey) {
    switch (routeKey) {
      case 'gateway':
      case 'dashboard':
      case 'routing':
      case 'api-keys':
      case 'usage':
      case 'user':
      case 'credits':
      case 'recharge':
      case 'billing':
      case 'settlements':
      case 'account':
        return (
          <PortalConsoleRoute
            dashboardSnapshot={dashboardSnapshot}
            onNavigate={navigateToRoute}
            routeKey={routeKey}
            workspace={workspace}
          />
        );
      default:
        return null;
    }
  }

  if (!bootstrapped && !isPublicPortalPath(location.pathname)) {
    return <PortalBootScreen status={bootStatus} />;
  }

  return (
    <Suspense fallback={<PortalBootScreen status="Loading portal workspace..." />}>
      <Routes>
        <Route
          element={
            <PortalSiteLayout>
              <PortalHomePage />
            </PortalSiteLayout>
          }
          path=""
        />
        <Route
          element={
            <PortalSiteLayout>
              <PortalModelsPage />
            </PortalSiteLayout>
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.models)}
        />
        <Route
          element={
            <PortalSiteLayout>
              <PortalApiReferencePage />
            </PortalSiteLayout>
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS['api-reference'])}
        />
        <Route
          element={
            <PortalSiteLayout>
              <PortalDocsPage />
            </PortalSiteLayout>
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.docs)}
        />
        <Route
          element={
            <PortalSiteLayout>
              <PortalDownloadsPage />
            </PortalSiteLayout>
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.downloads)}
        />
        <Route
          element={
            <Navigate
              replace
              to={buildAuthHref(
                PORTAL_ROUTE_PATHS.login,
                searchParams.get('redirect') ?? undefined,
              )}
            />
          }
          path="auth"
        />
        <Route
          element={
            authenticated ? (
              <Navigate replace to={redirectTarget} />
            ) : (
              <PortalAuthPage register={register} signIn={signIn} />
            )
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.login)}
        />
        <Route
          element={
            authenticated ? (
              <Navigate replace to={redirectTarget} />
            ) : (
              <PortalAuthPage register={register} signIn={signIn} />
            )
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.register)}
        />
        <Route
          element={
            authenticated ? (
              <Navigate replace to={redirectTarget} />
            ) : (
              <PortalAuthPage register={register} signIn={signIn} />
            )
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS['forgot-password'])}
        />
        <Route
          element={
            <Navigate
              replace
              to={
                authenticated
                  ? PORTAL_ROUTE_PATHS.dashboard
                  : buildAuthHref(PORTAL_ROUTE_PATHS.login, PORTAL_ROUTE_PATHS.dashboard)
              }
            />
          }
          path={toRouteElementPath(PORTAL_ROUTE_PATHS.console)}
        />
        {(
          [
            'gateway',
            'dashboard',
            'routing',
            'api-keys',
            'usage',
            'user',
            'credits',
            'recharge',
            'billing',
            'settlements',
            'account',
          ] as PortalRouteKey[]
        ).map((routeKey) => (
          <Route
            element={
              authenticated ? (
                <MainLayout workspace={workspace}>
                  {renderProtectedRoute(routeKey)}
                </MainLayout>
              ) : (
                <Navigate
                  replace
                  to={buildAuthHref(PORTAL_ROUTE_PATHS.login, requestedTarget)}
                />
              )
            }
            key={routeKey}
            path={toRouteElementPath(resolvePortalPath(routeKey))}
          />
        ))}
        <Route
          element={
            <Navigate
              replace
              to={PORTAL_ROUTE_PATHS.home}
            />
          }
          path="*"
        />
      </Routes>
    </Suspense>
  );
}
