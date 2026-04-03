import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin shell entry and layout adopt shared sdkwork shell primitives instead of legacy adminx shell assets', () => {
  const shell = read('packages/sdkwork-router-admin-shell/src/index.ts');
  const appRoot = read('packages/sdkwork-router-admin-shell/src/application/app/AppRoot.tsx');
  const routes = read('packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx');
  const layout = read('packages/sdkwork-router-admin-shell/src/application/layouts/MainLayout.tsx');
  const header = read('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');
  const themeCss = read('src/theme.css');
  const packageJson = read('packages/sdkwork-router-admin-shell/package.json');

  assert.match(shell, /\.\/styles\/shell-host\.css/);
  assert.doesNotMatch(shell, /\.\/styles\/index\.css/);
  assert.match(appRoot, /AppRoutes/);
  assert.doesNotMatch(appRoot, /<MainLayout \/>/);
  assert.match(routes, /MainLayout/);
  assert.match(routes, /AdminLoginPage/);
  assert.match(layout, /relative flex h-screen flex-col overflow-hidden/);
  assert.match(layout, /\[background:var\(--admin-shell-background\)\]/);
  assert.match(layout, /<Sidebar \/>/);
  assert.match(layout, /<AppHeader \/>/);
  assert.match(layout, /admin-shell-content/);
  assert.match(layout, /bg-\[var\(--admin-content-background\)\]/);
  assert.match(header, /\[background:var\(--admin-header-background\)\]/);
  assert.match(themeCss, /@source "\.\/";/);
  assert.match(themeCss, /@source "\.\.\/packages";/);
  assert.match(themeCss, /--admin-sidebar-text:/);
  assert.match(themeCss, /--admin-sidebar-item-active:/);
  assert.match(themeCss, /--admin-sidebar-popover-background:/);
  assert.match(themeCss, /--admin-sidebar-edge-background:/);
  assert.doesNotMatch(layout, /admin-shell-auth-stage/);
  assert.doesNotMatch(layout, /admin-shell-auth-main/);
  assert.doesNotMatch(layout, /AppRoutes/);
  assert.doesNotMatch(layout, /isAdminAuthPath/);
  assert.doesNotMatch(layout, /authResolved/);
  assert.doesNotMatch(layout, /sessionUser/);
  assert.doesNotMatch(layout, /DesktopShellFrame/);
  assert.doesNotMatch(layout, /AdminShellBrandMark/);
  assert.doesNotMatch(layout, /SDKWork Router Admin/);
  assert.doesNotMatch(layout, /Control plane/);
  assert.doesNotMatch(layout, /adminx-shell/);
  assert.doesNotMatch(layout, /adminx-auth-stage/);
  assert.match(packageJson, /@sdkwork\/ui-pc-react/);
});

test('admin header adopts a claw-style shell strip while sidebar keeps the dark-rail interaction primitives', () => {
  const header = read('packages/sdkwork-router-admin-shell/src/components/AppHeader.tsx');
  const sidebar = read('packages/sdkwork-router-admin-shell/src/components/Sidebar.tsx');

  assert.match(header, /ShellStatus/);
  assert.match(header, /HeaderActionButton/);
  assert.match(header, /data-slot="app-header-leading"/);
  assert.match(header, /data-slot="app-header-brand"/);
  assert.match(header, /data-slot="app-header-trailing"/);
  assert.match(header, /dataSlot="app-header-search"/);
  assert.match(header, /dataSlot="app-header-refresh"/);
  assert.match(header, /t\('Router Admin'\)/);
  assert.match(header, /ROUTE_PATHS\.OVERVIEW/);
  assert.match(header, /32x32\.png/);
  assert.match(header, /import\.meta\.url/);
  assert.match(header, /Ctrl K/);
  assert.match(header, /\[background:var\(--admin-header-background\)\]/);
  assert.match(sidebar, /motion\/react/);
  assert.match(sidebar, /sidebar-edge-control/);
  assert.match(sidebar, /PanelLeftOpen/);
  assert.match(sidebar, /ChevronUp/);
  assert.match(sidebar, /\[background:var\(--admin-sidebar-background\)\]/);
  assert.match(sidebar, /text-\[var\(--admin-sidebar-text\)\]/);
  assert.match(sidebar, /text-\[var\(--admin-sidebar-text-muted\)\]/);
  assert.match(sidebar, /bg-\[var\(--admin-sidebar-item-active\)\]/);
  assert.match(sidebar, /border-\[var\(--admin-sidebar-divider\)\]/);
  assert.match(sidebar, /bg-\[var\(--admin-sidebar-popover-background\)\]/);
  assert.match(sidebar, /bg-\[var\(--admin-sidebar-edge-background\)\]/);
  assert.match(sidebar, /bg-primary-500/);
  assert.match(sidebar, /text-primary-400/);
  assert.match(sidebar, /bg-primary-500\/15/);
  assert.match(sidebar, /currentSidebarWidth = isSidebarCollapsed \? COLLAPSED_SIDEBAR_WIDTH : resolvedSidebarWidth/);
  assert.doesNotMatch(sidebar, /NavigationRail/);
  assert.doesNotMatch(sidebar, /DropdownMenu/);
  assert.doesNotMatch(sidebar, /AvatarFallback|<Avatar|import\s*\{\s*Avatar/);
  assert.doesNotMatch(sidebar, /text-zinc-/);
  assert.doesNotMatch(sidebar, /bg-zinc-/);
  assert.doesNotMatch(sidebar, /dark:bg-zinc-/);
  assert.doesNotMatch(sidebar, /border-white\//);
  assert.doesNotMatch(sidebar, /bg-white\/\[/);
  assert.doesNotMatch(sidebar, /AdminShellBrandMark/);
  assert.doesNotMatch(sidebar, /SDKWork Router Admin/);
  assert.doesNotMatch(sidebar, /t\('Control plane'\)/);
  assert.doesNotMatch(header, /Toolbar/);
  assert.doesNotMatch(header, /ToolbarGroup/);
  assert.doesNotMatch(header, /@sdkwork\/ui-pc-react\/components\/ui/);
  assert.doesNotMatch(header, /adminx-shell-header/);
  assert.doesNotMatch(sidebar, /adminx-shell-sidebar/);
});

test('theme manager keeps the root data-theme contract while resolving sdkwork theme tokens from app preferences', () => {
  const themeManager = read(
    'packages/sdkwork-router-admin-shell/src/application/providers/ThemeManager.tsx',
  );

  assert.match(themeManager, /createSdkworkTheme/);
  assert.match(themeManager, /export function useAdminShellTheme/);
  assert.match(themeManager, /root\.setAttribute\('data-theme', themeColor\)/);
  assert.match(themeManager, /data-sdk-color-mode/);
  assert.doesNotMatch(themeManager, /theme-light/);
  assert.doesNotMatch(themeManager, /theme-dark/);
});
