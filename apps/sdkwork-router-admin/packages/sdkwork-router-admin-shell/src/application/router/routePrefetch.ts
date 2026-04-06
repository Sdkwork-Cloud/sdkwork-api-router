import {
  adminRouteManifest,
} from 'sdkwork-router-admin-core';
import type { AdminRouteModuleId } from 'sdkwork-router-admin-types';

const adminRouteModuleLoaders: Record<AdminRouteModuleId, () => Promise<unknown>> = {
  'sdkwork-router-admin-overview': () => import('sdkwork-router-admin-overview'),
  'sdkwork-router-admin-users': () => import('sdkwork-router-admin-users'),
  'sdkwork-router-admin-tenants': () => import('sdkwork-router-admin-tenants'),
  'sdkwork-router-admin-coupons': () => import('sdkwork-router-admin-coupons'),
  'sdkwork-router-admin-commercial': () => import('sdkwork-router-admin-commercial'),
  'sdkwork-router-admin-pricing': () => import('sdkwork-router-admin-pricing'),
  'sdkwork-router-admin-apirouter': () => import('sdkwork-router-admin-apirouter'),
  'sdkwork-router-admin-catalog': () => import('sdkwork-router-admin-catalog'),
  'sdkwork-router-admin-traffic': () => import('sdkwork-router-admin-traffic'),
  'sdkwork-router-admin-operations': () => import('sdkwork-router-admin-operations'),
  'sdkwork-router-admin-settings': () => import('sdkwork-router-admin-settings'),
};

export function loadAdminRouteModule(moduleId: AdminRouteModuleId) {
  return adminRouteModuleLoaders[moduleId]();
}

const sidebarRoutePrefetchers = adminRouteManifest
  .filter((route) => route.productModule.loading.prefetch === 'intent')
  .map((route) => [
    route.path,
    () => loadAdminRouteModule(route.moduleId),
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
