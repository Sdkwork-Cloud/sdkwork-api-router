import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal sidebar avoids zustand object selectors that can trigger React update loops', () => {
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');

  assert.match(navigationRail, /usePortalShellStore\(\(state\) => state\.hiddenSidebarItems\)/);
  assert.match(navigationRail, /usePortalShellStore\(\(state\) => state\.isSidebarCollapsed\)/);
  assert.match(navigationRail, /usePortalShellStore\(\(state\) => state\.sidebarWidth\)/);
  assert.match(navigationRail, /usePortalShellStore\(\(state\) => state\.toggleSidebar\)/);
  assert.match(navigationRail, /usePortalShellStore\(\(state\) => state\.setSidebarCollapsed\)/);
  assert.match(navigationRail, /usePortalShellStore\(\(state\) => state\.setSidebarWidth\)/);
  assert.doesNotMatch(navigationRail, /usePortalShellStore\(\(state\) => \(\{/);
});
