import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal sidebar route warming mirrors claw-studio prefetch orchestration', () => {
  const prefetch = read(
    'packages/sdkwork-router-portal-core/src/application/router/routePrefetch.ts',
  );
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');

  assert.match(prefetch, /createSidebarRoutePrefetchController/);
  assert.match(prefetch, /normalizeRoutePath/);
  assert.match(prefetch, /scheduleDelayMs = 120/);
  assert.match(prefetch, /prefetchedSidebarRoutes/);
  assert.match(prefetch, /scheduledSidebarRoutes/);
  assert.match(prefetch, /sdkwork-router-portal-gateway/);
  assert.match(prefetch, /sdkwork-router-portal-dashboard/);
  assert.match(prefetch, /sdkwork-router-portal-routing/);
  assert.match(prefetch, /sdkwork-router-portal-api-keys/);
  assert.match(prefetch, /sdkwork-router-portal-usage/);
  assert.match(prefetch, /sdkwork-router-portal-user/);
  assert.match(prefetch, /sdkwork-router-portal-credits/);
  assert.match(prefetch, /sdkwork-router-portal-billing/);
  assert.match(prefetch, /sdkwork-router-portal-account/);
  assert.match(prefetch, /export function prefetchSidebarRoute/);
  assert.match(prefetch, /export function scheduleSidebarRoutePrefetch/);
  assert.match(prefetch, /export function cancelSidebarRoutePrefetch/);

  assert.match(navigationRail, /onPointerDown=\{\(\) => prefetchSidebarRoute/);
  assert.match(navigationRail, /onMouseEnter=\{\(\) => scheduleSidebarRoutePrefetch/);
  assert.match(navigationRail, /onMouseLeave=\{\(\) => cancelSidebarRoutePrefetch/);
  assert.match(navigationRail, /onFocus=\{\(\) => scheduleSidebarRoutePrefetch/);
  assert.match(navigationRail, /onBlur=\{\(\) => cancelSidebarRoutePrefetch/);
});
