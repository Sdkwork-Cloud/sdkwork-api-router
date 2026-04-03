import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal theme provider delegates mode and brand selection to shared sdkwork theme primitives', () => {
  const theme = read('src/theme.css');
  const themeManager = read('packages/sdkwork-router-portal-core/src/application/providers/ThemeManager.tsx');
  const store = read('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');

  assert.match(themeManager, /SdkworkThemeProvider/);
  assert.match(themeManager, /createSdkworkTheme/);
  assert.match(themeManager, /themeMode/);
  assert.match(themeManager, /themeColor/);
  assert.match(themeManager, /data-theme/);
  assert.match(themeManager, /classList\.toggle\('dark', resolvedColorMode === 'dark'\)/);
  assert.doesNotMatch(themeManager, /classList\.add\('dark'\)/);
  assert.doesNotMatch(themeManager, /classList\.remove\('dark'\)/);
  assert.doesNotMatch(themeManager, /data-theme-mode/);
  assert.match(store, /themeMode/);
  assert.match(store, /themeColor/);
  assert.match(store, /tech-blue/);
  assert.match(store, /lobster/);
  assert.match(store, /green-tech/);
  assert.match(store, /zinc/);
  assert.match(store, /violet/);
  assert.match(store, /rose/);
  assert.match(theme, /\[data-theme="tech-blue"\]/);
  assert.match(theme, /\[data-theme="lobster"\]/);
  assert.match(theme, /\[data-theme="green-tech"\]/);
  assert.match(theme, /\[data-theme="zinc"\]/);
  assert.match(theme, /\[data-theme="violet"\]/);
  assert.match(theme, /\[data-theme="rose"\]/);
  assert.match(theme, /:root\.dark/);
});

test('portal preferences persist under a dedicated shell storage key and flow through the shared settings center', () => {
  const preferences = read('packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts');
  const store = read('packages/sdkwork-router-portal-core/src/store/usePortalShellStore.ts');
  const settingsCenter = read('packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx');

  assert.match(preferences, /sdkwork-router-portal\.preferences\.v1/);
  assert.match(store, /persist/);
  assert.match(preferences, /PORTAL_COLLAPSED_SIDEBAR_WIDTH = 72/);
  assert.match(preferences, /PORTAL_DEFAULT_SIDEBAR_WIDTH = 252/);
  assert.match(preferences, /PORTAL_MIN_SIDEBAR_WIDTH = 220/);
  assert.match(preferences, /PORTAL_MAX_SIDEBAR_WIDTH = 360/);
  assert.match(store, /sidebarCollapsePreference/);
  assert.match(store, /resolveAutoSidebarCollapsed/);
  assert.match(settingsCenter, /motion\.(div|section)/);
  assert.match(settingsCenter, /from 'motion\/react'/);
  assert.match(settingsCenter, /Search settings/);
  assert.match(settingsCenter, /appearance/);
  assert.match(settingsCenter, /navigation/);
  assert.match(settingsCenter, /workspace/);
  assert.match(settingsCenter, /Theme mode/);
  assert.match(settingsCenter, /Theme color/);
  assert.match(settingsCenter, /Sidebar navigation/);
  assert.match(settingsCenter, /max-h-\[calc\(100dvh-2rem\)\]/);
  assert.match(settingsCenter, /scrollbar-hide flex-1 overflow-x-hidden overflow-y-auto/);
  assert.match(settingsCenter, /mx-auto w-full max-w-5xl p-8 md:p-12/);
  assert.doesNotMatch(settingsCenter, /portalx-/);
  assert.doesNotMatch(settingsCenter, /portal-shell-/);
});

test('portal theme contract keeps shell, content, and chart surfaces on shared token families only', () => {
  const theme = read('src/theme.css');
  const desktopShell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');

  assert.match(theme, /--portal-shell-background/);
  assert.match(theme, /--portal-content-background/);
  assert.match(theme, /--portal-surface-background/);
  assert.match(theme, /--portal-surface-elevated/);
  assert.match(theme, /--portal-sidebar-background/);
  assert.match(theme, /:root\s*\{[\s\S]*--portal-sidebar-background:\s*#[0-9a-fA-F]{6};/);
  assert.match(theme, /:root\.dark\s*\{[\s\S]*--portal-sidebar-background:\s*#[0-9a-fA-F]{6};/);
  assert.match(theme, /--portal-border-color/);
  assert.match(theme, /--portal-text-primary/);
  assert.match(theme, /--portal-text-secondary/);
  assert.match(theme, /--portal-chart-grid/);
  assert.match(theme, /--portal-chart-tooltip-background/);
  assert.match(desktopShell, /\[background:var\(--portal-shell-background\)\]/);
  assert.match(desktopShell, /bg-\[var\(--portal-content-background\)\]/);
  assert.doesNotMatch(theme, /\.portal-shell-backdrop/);
  assert.doesNotMatch(theme, /\.portal-content-shell/);
  assert.doesNotMatch(theme, /\.portalx-auth-shell/);
  assert.doesNotMatch(theme, /\.portalx-insight-card/);
  assert.doesNotMatch(theme, /\.portalx-fact-list/);
  assert.doesNotMatch(theme, /\.portalx-summary-card/);
  assert.doesNotMatch(theme, /\.portal-shell-info-card/);
  assert.doesNotMatch(theme, /\.portalx-search-input/);
});

test('portal shell composes the shared desktop frame with a custom claw-style sidebar without forcing a global max-width content shell', () => {
  const layout = read('packages/sdkwork-router-portal-core/src/application/layouts/MainLayout.tsx');
  const desktopShell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const app = read('packages/sdkwork-router-portal-core/src/application/app/PortalProductApp.tsx');

  assert.match(layout, /PortalDesktopShell/);
  assert.match(layout, /PortalSettingsCenter/);
  assert.match(desktopShell, /DesktopShellFrame/);
  assert.match(desktopShell, /PortalNavigationRail/);
  assert.match(desktopShell, /sidebar=\{/);
  assert.match(desktopShell, /sidebarWidth=\{currentSidebarWidth\}/);
  assert.match(desktopShell, /brandMark=\{/);
  assert.match(desktopShell, /title=\{t\('SDKWork Router'\)\}/);
  assert.doesNotMatch(desktopShell, /navigation=\{/);
  assert.match(desktopShell, /WindowControls/);
  assert.match(desktopShell, /font-sans/);
  assert.match(desktopShell, /min-h-full w-full flex-col gap-6 px-4 py-5 md:px-6 xl:px-8/);
  assert.doesNotMatch(desktopShell, /portal-shell-backdrop/);
  assert.doesNotMatch(desktopShell, /portal-content-shell/);
  assert.doesNotMatch(desktopShell, /max-w-\[/);
  assert.match(navigationRail, /usePortalAuthStore/);
  assert.match(navigationRail, /workspace \?\? storedWorkspace/);
  assert.match(navigationRail, /t\((route|item)\.label(Key)?\)/);
  assert.match(navigationRail, /group\.section/);
  assert.match(navigationRail, /data-slot="sidebar-edge-control"/);
  assert.match(navigationRail, /data-slot="sidebar-resize-handle"/);
  assert.match(navigationRail, /data-slot="sidebar-user-control"/);
  assert.doesNotMatch(navigationRail, /<NavigationRail|NavigationRail\s*\}\s*from/);
  assert.doesNotMatch(navigationRail, /Developer portal/);
  assert.doesNotMatch(navigationRail, /SDKWork Router/);
  assert.doesNotMatch(layout, /ShellStatus/);
  assert.doesNotMatch(appRoutes, /pulseDetail|pulseStatus|pulseTitle|pulseTone/);
  assert.doesNotMatch(app, /buildWorkspacePulse|pulseDetail|pulseStatus|pulseTitle|pulseTone/);
  assert.doesNotMatch(navigationRail, /portalx-/);
  assert.doesNotMatch(navigationRail, /portal-shell-/);
});

test('portal auth page mirrors claw-style surfaces while honoring shared theme mode and accent color tokens', () => {
  const authPage = read('packages/sdkwork-router-portal-auth/src/pages/AuthPage.tsx');

  assert.match(authPage, /bg-zinc-50/);
  assert.match(authPage, /dark:bg-zinc-950/);
  assert.match(authPage, /bg-zinc-900/);
  assert.match(authPage, /dark:bg-black/);
  assert.match(authPage, /from-primary-600\/20/);
  assert.match(authPage, /text-primary-600/);
  assert.match(authPage, /hover:text-primary-500/);
  assert.match(authPage, /dark:border-zinc-800/);
  assert.match(authPage, /dark:bg-zinc-900/);
  assert.doesNotMatch(authPage, /AuthShell/);
  assert.doesNotMatch(authPage, /portalx-auth-hero/);
});

test('portal theme substrate keeps claw-style scrollbar and color-scheme behavior while staying scoped to the portal app', () => {
  const theme = read('src/theme.css');
  const desktopShell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');

  assert.match(desktopShell, /font-sans/);
  assert.match(theme, /font-family:\s*ui-sans-serif,\s*system-ui/s);
  assert.match(theme, /--scrollbar-size: 10px/);
  assert.match(theme, /--scrollbar-track:/);
  assert.match(theme, /--scrollbar-thumb:/);
  assert.match(theme, /scrollbar-width: thin/);
  assert.match(theme, /scrollbar-color: var\(--scrollbar-thumb\) var\(--scrollbar-track\)/);
  assert.match(theme, /:root\.dark\s*\{[^}]*color-scheme:\s*dark/s);
  assert.match(theme, /@source "\.\/";/);
  assert.match(theme, /@source "\.\.\/packages";/);
  assert.doesNotMatch(theme, /@source "\.\.\/\.\.\/\.\.\/\.\.\//);
  assert.doesNotMatch(theme, /Space Grotesk/);
  assert.doesNotMatch(theme, /Avenir Next/);
});

test('portal desktop chrome uses shared titlebar controls and removes the legacy shell component set', () => {
  const windowControls = read('packages/sdkwork-router-portal-core/src/components/WindowControls.tsx');
  const navigationRail = read('packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx');

  assert.match(windowControls, /hover:bg-zinc-950\/\[0\.06\]/);
  assert.match(windowControls, /hover:bg-rose-500 hover:text-white/);
  assert.match(navigationRail, /border-zinc-900\/90 bg-zinc-950 \[background:var\(--portal-sidebar-background\)\]/);
  assert.doesNotMatch(navigationRail, /linear-gradient/);
  assert.doesNotMatch(navigationRail, /radial-gradient/);
  assert.match(navigationRail, /data-slot="sidebar-edge-control"/);
  assert.match(navigationRail, /data-slot="sidebar-resize-handle"/);
  assert.match(navigationRail, /data-slot="sidebar-user-control"/);
  assert.doesNotMatch(navigationRail, /Developer portal/);
  assert.doesNotMatch(navigationRail, /SDKWork Router/);
  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-portal-core', 'src', 'components', 'Sidebar.tsx')),
    false,
  );
  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-portal-core', 'src', 'components', 'ConfigCenter.tsx')),
    false,
  );
  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-portal-core', 'src', 'components', 'AppHeader.tsx')),
    false,
  );
  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-portal-core', 'src', 'components', 'SidebarProfileDock.tsx')),
    false,
  );
});
