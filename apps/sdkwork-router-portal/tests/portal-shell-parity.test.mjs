import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal shell adopts the shared desktop-shell composition with a claw-style sidebar slot', () => {
  const packageJson = read('packages/sdkwork-router-portal-core/package.json');
  const appProviders = read('packages/sdkwork-router-portal-core/src/application/providers/AppProviders.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const shell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');

  assert.match(packageJson, /@sdkwork\/ui-pc-react/);
  assert.match(appProviders, /BrowserRouter/);
  assert.match(appProviders, /basename="\s*\/portal\s*"/);
  assert.match(appRoutes, /Routes/);
  assert.match(appRoutes, /Route/);
  assert.match(appRoutes, /lazy\(async \(\) =>/);
  assert.match(appRoutes, /<Suspense fallback=\{<PortalBootScreen status="Loading portal workspace\.\.\." \/>}/);
  assert.match(layout, /PortalDesktopShell/);
  assert.match(layout, /PortalSettingsCenter/);
  assert.doesNotMatch(layout, /<Sidebar/);
  assert.doesNotMatch(layout, /<AppHeader/);
  assert.doesNotMatch(layout, /ConfigCenter/);
  assert.match(shell, /DesktopShellFrame/);
  assert.match(shell, /PortalNavigationRail/);
  assert.match(shell, /sidebar=\{/);
  assert.match(shell, /sidebarWidth=\{currentSidebarWidth\}/);
  assert.match(shell, /brandMark=\{/);
  assert.match(shell, /title=\{t\('SDKWork Router'\)\}/);
  assert.doesNotMatch(shell, /portal-shell-backdrop/);
  assert.doesNotMatch(shell, /portal-content-shell/);
  assert.doesNotMatch(shell, /navigation=\{/);
  assert.match(shell, /WindowControls|DesktopWindowControls/);
});

test('portal header uses a claw-style flat title bar with monochrome, theme-adaptive actions', () => {
  const desktopShell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');
  const siteLayout = read('packages/sdkwork-router-portal-core/src/application/layouts/PortalSiteLayout.tsx');
  const topNavigation = read('packages/sdkwork-router-portal-core/src/components/PortalTopNavigation.tsx');
  const brandMark = read('packages/sdkwork-router-portal-core/src/components/PortalBrandMark.tsx');

  assert.match(desktopShell, /centerMaxWidth="min\(96rem, calc\(100% - 8rem\)\)"/);
  assert.match(siteLayout, /centerMaxWidth="min\(96rem, calc\(100% - 8rem\)\)"/);
  assert.match(desktopShell, /centerShell:/);
  assert.match(siteLayout, /centerShell:/);
  assert.match(desktopShell, /size="default"/);
  assert.match(siteLayout, /size="default"/);
  assert.match(topNavigation, /w-full/);
  assert.match(topNavigation, /justify-start/);
  assert.match(topNavigation, /max-w-\[min\(100%,72rem\)\]/);
  assert.match(topNavigation, /inline-flex w-full min-w-0 items-center justify-start gap-1\.5/);
  assert.match(topNavigation, /text-\[13px\]/);
  assert.match(topNavigation, /inline-flex h-9 flex-none/);
  assert.match(topNavigation, /bg-zinc-950\/\[0\.08\] text-zinc-950/);
  assert.match(topNavigation, /dark:bg-white\/\[0\.08\] dark:text-white/);
  assert.match(topNavigation, /hover:bg-zinc-950\/\[0\.04\]/);
  assert.match(topNavigation, /dark:hover:bg-white\/\[0\.06\]/);
  assert.match(siteLayout, /data-slot="portal-header-download-action"/);
  assert.match(desktopShell, /data-slot="portal-header-download-action"/);
  assert.match(siteLayout, /navigate\('\/downloads'\)/);
  assert.match(desktopShell, /navigate\('\/downloads'\)/);
  assert.match(siteLayout, /Download App/);
  assert.match(desktopShell, /Download App/);
  assert.doesNotMatch(topNavigation, /bg-white\/90/);
  assert.doesNotMatch(topNavigation, /dark:bg-zinc-950\/90/);
  assert.doesNotMatch(topNavigation, /border-zinc-200\/80/);
  assert.doesNotMatch(topNavigation, /flex-1 items-center justify-center rounded-xl px-4/);
  assert.doesNotMatch(topNavigation, /bg-zinc-950 text-white dark:bg-white dark:text-zinc-950/);
  assert.doesNotMatch(topNavigation, /grid-cols-5/);
  assert.doesNotMatch(topNavigation, /dark:bg-white\/\[0\.05\]/);
  assert.doesNotMatch(topNavigation, /bg-white\/35/);
  assert.doesNotMatch(topNavigation, /backdrop-blur-sm/);
  assert.doesNotMatch(topNavigation, /border-zinc-200\/55/);
  assert.doesNotMatch(topNavigation, /overflow-x-auto/);
  assert.doesNotMatch(topNavigation, /min-w-max/);
  assert.match(topNavigation, /key:\s*'downloads'/);
  assert.match(topNavigation, /labelKey:\s*'Download Center'/);
  assert.match(siteLayout, /data-slot="portal-header-console-action"/);
  assert.match(siteLayout, /rounded-2xl border border-zinc-200\/80 bg-white\/88 px-4 text-zinc-700/);
  assert.match(siteLayout, /dark:border-zinc-800 dark:bg-zinc-950\/88 dark:text-zinc-200/);
  assert.doesNotMatch(brandMark, /linearGradient/);
  assert.match(brandMark, /bg-white\/92/);
  assert.match(brandMark, /dark:bg-zinc-950\/92/);
  assert.match(brandMark, /border-zinc-200\/80/);
  assert.match(brandMark, /dark:border-zinc-800/);
});

test('portal sidebar owns grouped claw-style navigation, collapse affordances, and route warming', () => {
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');

  assert.match(navigationRail, /portalSidebarRoutes/);
  assert.match(navigationRail, /resolvePortalPath/);
  assert.match(navigationRail, /hiddenSidebarItems/);
  assert.match(navigationRail, /toggleSidebar/);
  assert.match(navigationRail, /PanelLeftOpen/);
  assert.match(navigationRail, /PanelLeftClose/);
  assert.match(navigationRail, /data-slot="sidebar-edge-control"/);
  assert.match(navigationRail, /data-slot="sidebar-resize-handle"/);
  assert.match(navigationRail, /data-slot="sidebar-user-control"/);
  assert.match(navigationRail, /data-slot="sidebar-nav-list"/);
  assert.match(navigationRail, /data-slot="sidebar-nav-item"/);
  assert.match(navigationRail, /startSidebarResize/);
  assert.match(navigationRail, /scheduleSidebarRoutePrefetch/);
  assert.match(navigationRail, /prefetchSidebarRoute/);
  assert.match(navigationRail, /cancelSidebarRoutePrefetch/);
  assert.match(navigationRail, /group\.section/);
  assert.match(navigationRail, /User details/);
  assert.match(navigationRail, /Sign out|Logout/);
  assert.match(navigationRail, /border-zinc-900\/90 bg-zinc-950 \[background:var\(--portal-sidebar-background\)\]/);
  assert.doesNotMatch(navigationRail, /sidebar-group-badge/);
  assert.doesNotMatch(navigationRail, /portal-nav-item-indicator/);
  assert.doesNotMatch(navigationRail, /data-slot="sidebar-active-route-panel"/);
  assert.doesNotMatch(navigationRail, /linear-gradient/);
  assert.doesNotMatch(navigationRail, /radial-gradient/);
  assert.match(routes, /Dashboard/);
  assert.match(routes, /Routing/);
  assert.match(routes, /API Keys/);
  assert.match(routes, /Redeem/);
  assert.match(routes, /key:\s*'gateway'[\s\S]*?sidebarVisible:\s*false/);
  assert.match(routes, /key:\s*'routing'[\s\S]*?sidebarVisible:\s*false/);
  assert.match(routes, /key:\s*'user'[\s\S]*?sidebarVisible:\s*false/);
  assert.match(routes, /key:\s*'billing'[\s\S]*?sidebarVisible:\s*false/);
  assert.match(routes, /export const portalSidebarRoutes/);
  assert.match(routes, /group:\s*'operations'/);
  assert.match(routes, /group:\s*'access'/);
  assert.match(routes, /group:\s*'revenue'/);
  assert.match(routes, /key:\s*'credits'/);
  assert.match(routes, /key:\s*'account'/);
  assert.match(appRoutes, /case 'credits':/);
  assert.doesNotMatch(navigationRail, /<NavigationRail|NavigationRail\s*\}\s*from/);
  assert.doesNotMatch(navigationRail, /BrandMark/);
  assert.doesNotMatch(navigationRail, /workspaceIdentity/);
  assert.doesNotMatch(navigationRail, /Developer portal/);
  assert.doesNotMatch(navigationRail, /SDKWork Router/);
});

test('portal settings center replaces the old config-center module', () => {
  const settingsCenter = read('packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx');
  const legacyHeaderPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'AppHeader.tsx',
  );
  const legacySidebarPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'Sidebar.tsx',
  );
  const legacyProfileDockPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'SidebarProfileDock.tsx',
  );
  const legacyConfigCenterPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'ConfigCenter.tsx',
  );
  const windowControlsPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-core',
    'src',
    'components',
    'WindowControls.tsx',
  );
  const tauriConfigPath = path.join(appRoot, 'src-tauri', 'tauri.conf.json');
  const tauriCargoPath = path.join(appRoot, 'src-tauri', 'Cargo.toml');
  const tauriIconPath = path.join(appRoot, 'src-tauri', 'icons', 'icon.ico');

  assert.match(settingsCenter, /Dialog/);
  assert.match(settingsCenter, /SearchInput/);
  assert.match(settingsCenter, /PORTAL_LOCALE_OPTIONS/);
  assert.match(settingsCenter, /themeColor/);
  assert.match(settingsCenter, /hiddenSidebarItems/);
  assert.match(settingsCenter, /portalSidebarRoutes/);
  assert.equal(existsSync(windowControlsPath), true);
  assert.equal(existsSync(legacyHeaderPath), false);
  assert.equal(existsSync(legacySidebarPath), false);
  assert.equal(existsSync(legacyProfileDockPath), false);
  assert.equal(existsSync(legacyConfigCenterPath), false);
  assert.equal(existsSync(tauriConfigPath), true);
  assert.equal(existsSync(tauriCargoPath), true);
  assert.equal(existsSync(tauriIconPath), true);

  const tauriConfig = readFileSync(tauriConfigPath, 'utf8');
  assert.match(tauriConfig, /"decorations"\s*:\s*false/);
});
