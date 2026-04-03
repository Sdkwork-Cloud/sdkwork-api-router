import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal navigation rail follows claw-studio grouped business navigation and collapsed affordances', () => {
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');

  assert.match(navigationRail, /portalSidebarRoutes/);
  assert.match(navigationRail, /resolvePortalPath/);
  assert.match(navigationRail, /Operations/);
  assert.match(navigationRail, /Access/);
  assert.match(navigationRail, /Revenue/);
  assert.match(navigationRail, /mx-auto h-11 w-11 justify-center/);
  assert.match(navigationRail, /group relative flex items-center rounded-2xl/);
  assert.match(navigationRail, /data-slot="sidebar-edge-control"/);
  assert.match(navigationRail, /data-slot="sidebar-user-control"/);
  assert.doesNotMatch(navigationRail, /Active workspace/);
  assert.doesNotMatch(navigationRail, /Route signals/);
  assert.doesNotMatch(navigationRail, /<NavigationRail|NavigationRail\s*\}\s*from/);
  assert.doesNotMatch(navigationRail, /Developer portal/);
  assert.doesNotMatch(navigationRail, /SDKWork Router/);
  assert.match(routes, /Dashboard/);
  assert.match(routes, /Routing/);
  assert.match(routes, /API Keys/);
  assert.match(routes, /Redeem/);
  assert.match(routes, /key:\s*'gateway'[\s\S]*?sidebarVisible:\s*false/);
  assert.match(routes, /key:\s*'routing'[\s\S]*?sidebarVisible:\s*false/);
  assert.match(routes, /key:\s*'billing'[\s\S]*?sidebarVisible:\s*false/);
  assert.match(routes, /export const portalSidebarRoutes/);
  assert.match(routes, /group:\s*'operations'/);
  assert.match(routes, /group:\s*'access'/);
  assert.match(routes, /group:\s*'revenue'/);
  assert.match(routes, /key:\s*'credits'/);
  assert.match(routes, /key:\s*'account'/);
  assert.match(appRoutes, /'credits',/);
});

test('dashboard exposes module posture instead of a route-signal map', () => {
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(dashboardPage, /Module posture/);
  assert.doesNotMatch(dashboardPage, /Route signal map/);
});
