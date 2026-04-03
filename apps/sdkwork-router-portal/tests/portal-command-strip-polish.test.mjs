import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell replaces the mission strip with a claw-style grouped sidebar', () => {
  const desktopShell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');
  const sidebarPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'Sidebar.tsx',
  );
  const profileDockPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'SidebarProfileDock.tsx',
  );
  const shellStatusPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'ShellStatus.tsx',
  );

  assert.match(desktopShell, /PortalNavigationRail/);
  assert.match(desktopShell, /sidebar=\{/);
  assert.match(desktopShell, /sidebarWidth=\{currentSidebarWidth\}/);
  assert.doesNotMatch(desktopShell, /navigation=\{/);
  assert.match(navigationRail, /PanelLeftOpen/);
  assert.match(navigationRail, /PanelLeftClose/);
  assert.match(navigationRail, /data-slot="sidebar-edge-control"/);
  assert.match(navigationRail, /data-slot="sidebar-resize-handle"/);
  assert.match(navigationRail, /data-slot="sidebar-user-control"/);
  assert.match(navigationRail, /Sign out/);
  assert.doesNotMatch(navigationRail, /Active workspace/);
  assert.doesNotMatch(navigationRail, /Mission strip/);
  assert.doesNotMatch(navigationRail, /<NavigationRail|NavigationRail\s*\}\s*from/);
  assert.doesNotMatch(navigationRail, /Developer portal/);
  assert.doesNotMatch(navigationRail, /SDKWork Router/);
  assert.equal(existsSync(sidebarPath), false);
  assert.equal(existsSync(profileDockPath), false);
  assert.equal(existsSync(shellStatusPath), false);
});
