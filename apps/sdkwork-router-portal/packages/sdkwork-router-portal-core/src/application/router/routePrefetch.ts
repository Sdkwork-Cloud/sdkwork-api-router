import type { PortalRouteModuleId } from 'sdkwork-router-portal-types';

import { portalRouteManifest } from './routeManifest';

const portalRouteModuleLoaders: Record<PortalRouteModuleId, () => Promise<unknown>> = {
  'sdkwork-router-portal-gateway': () => import('sdkwork-router-portal-gateway'),
  'sdkwork-router-portal-dashboard': () => import('sdkwork-router-portal-dashboard'),
  'sdkwork-router-portal-routing': () => import('sdkwork-router-portal-routing'),
  'sdkwork-router-portal-api-keys': () => import('sdkwork-router-portal-api-keys'),
  'sdkwork-router-portal-usage': () => import('sdkwork-router-portal-usage'),
  'sdkwork-router-portal-user': () => import('sdkwork-router-portal-user'),
  'sdkwork-router-portal-credits': () => import('sdkwork-router-portal-credits'),
  'sdkwork-router-portal-recharge': () => import('sdkwork-router-portal-recharge'),
  'sdkwork-router-portal-billing': () => import('sdkwork-router-portal-billing'),
  'sdkwork-router-portal-account': () => import('sdkwork-router-portal-account'),
};

export function loadPortalRouteModule(moduleId: PortalRouteModuleId) {
  return portalRouteModuleLoaders[moduleId]();
}

const sidebarRoutePrefetchers = portalRouteManifest
  .filter((route) => route.productModule.loading.prefetch === 'intent')
  .map((route) => [
    route.path,
    () => loadPortalRouteModule(route.moduleId),
  ]) as readonly SidebarRoutePrefetcher[];

type SidebarRoutePrefetcher = readonly [string, () => Promise<unknown>];
type ScheduledPrefetchHandle = unknown;

function normalizeRoutePath(pathname: string) {
  return pathname.split(/[?#]/, 1)[0] || pathname;
}

function resolveSidebarRoutePrefetcher(
  routePrefetchers: readonly SidebarRoutePrefetcher[],
  pathname: string,
) {
  const normalizedPath = normalizeRoutePath(pathname);
  return routePrefetchers.find(([prefix]) => (
    normalizedPath === prefix || normalizedPath.startsWith(`${prefix}/`)
  ));
}

export function createSidebarRoutePrefetchController(input: {
  routePrefetchers: readonly SidebarRoutePrefetcher[];
  scheduleDelayMs?: number;
  schedule?: (callback: () => void, delayMs: number) => ScheduledPrefetchHandle;
  clearScheduled?: (handle: ScheduledPrefetchHandle) => void;
}) {
  const {
    routePrefetchers,
    scheduleDelayMs = 120,
    schedule = (callback, delayMs) => window.setTimeout(callback, delayMs),
    clearScheduled = (handle) => window.clearTimeout(handle as number),
  } = input;

  const prefetchedSidebarRoutes = new Map<string, Promise<unknown>>();
  const scheduledSidebarRoutes = new Map<string, ScheduledPrefetchHandle>();

  const prefetch = (pathname: string) => {
    const match = resolveSidebarRoutePrefetcher(routePrefetchers, pathname);
    if (!match) {
      return;
    }

    const [routePrefix, loadRoute] = match;
    if (prefetchedSidebarRoutes.has(routePrefix)) {
      return;
    }

    const pending = loadRoute().catch((error) => {
      prefetchedSidebarRoutes.delete(routePrefix);
      throw error;
    });

    prefetchedSidebarRoutes.set(routePrefix, pending);
  };

  const cancel = (pathname: string) => {
    const match = resolveSidebarRoutePrefetcher(routePrefetchers, pathname);
    if (!match) {
      return;
    }

    const [routePrefix] = match;
    const scheduled = scheduledSidebarRoutes.get(routePrefix);
    if (!scheduled) {
      return;
    }

    clearScheduled(scheduled);
    scheduledSidebarRoutes.delete(routePrefix);
  };

  const queue = (pathname: string) => {
    const match = resolveSidebarRoutePrefetcher(routePrefetchers, pathname);
    if (!match) {
      return;
    }

    const [routePrefix] = match;
    if (prefetchedSidebarRoutes.has(routePrefix) || scheduledSidebarRoutes.has(routePrefix)) {
      return;
    }

    const handle = schedule(() => {
      scheduledSidebarRoutes.delete(routePrefix);
      prefetch(pathname);
    }, scheduleDelayMs);

    scheduledSidebarRoutes.set(routePrefix, handle);
  };

  return {
    prefetch,
    schedule: queue,
    cancel,
  };
}

const sidebarRoutePrefetchController = createSidebarRoutePrefetchController({
  routePrefetchers: sidebarRoutePrefetchers,
});

export function prefetchSidebarRoute(pathname: string) {
  sidebarRoutePrefetchController.prefetch(pathname);
}

export function scheduleSidebarRoutePrefetch(pathname: string) {
  sidebarRoutePrefetchController.schedule(pathname);
}

export function cancelSidebarRoutePrefetch(pathname: string) {
  sidebarRoutePrefetchController.cancel(pathname);
}
