import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('public site navigation and modules use shared i18n labels plus route-aware CTA wiring', () => {
  const topNavigation = read('packages/sdkwork-router-portal-core/src/components/PortalTopNavigation.tsx');
  const siteLayout = read('packages/sdkwork-router-portal-core/src/application/layouts/PortalSiteLayout.tsx');
  const desktopShell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');
  const homePage = read('packages/sdkwork-router-portal-home/src/index.tsx');
  const docsPage = read('packages/sdkwork-router-portal-docs/src/index.tsx');
  const docsRegistry = read('packages/sdkwork-router-portal-docs/src/registry.ts');
  const downloadsPage = read('packages/sdkwork-router-portal-downloads/src/index.tsx');
  const runtimeModeTitleCount = (downloadsPage.match(/title=\{t\('Runtime modes'\)\}/g) ?? []).length;

  assert.match(topNavigation, /t\('Top navigation'\)/);
  assert.match(topNavigation, /labelKey:\s*'Home'/);
  assert.match(topNavigation, /labelKey:\s*'Console'/);
  assert.match(topNavigation, /labelKey:\s*'Models'/);
  assert.match(topNavigation, /labelKey:\s*'API Reference'/);
  assert.match(topNavigation, /labelKey:\s*'Docs'/);
  assert.match(topNavigation, /key:\s*'models'[\s\S]*key:\s*'api-reference'[\s\S]*key:\s*'docs'/);
  assert.match(topNavigation, /href:\s*'\/api-reference'/);
  assert.match(topNavigation, /key:\s*'downloads'/);
  assert.match(topNavigation, /labelKey:\s*'Download Center'/);
  assert.match(topNavigation, /href:\s*'\/downloads'/);
  assert.match(topNavigation, /href:\s*'\/console\/dashboard'/);
  assert.doesNotMatch(topNavigation, /href:\s*'\/console'/);
  assert.doesNotMatch(topNavigation, /labelKey:\s*'Software Downloads'/);
  assert.match(siteLayout, /Download App/);
  assert.match(siteLayout, /navigate\('\/downloads'\)/);
  assert.match(siteLayout, /data-slot="portal-header-download-action"/);
  assert.match(desktopShell, /Download App/);
  assert.match(desktopShell, /navigate\('\/downloads'\)/);
  assert.match(desktopShell, /data-slot="portal-header-download-action"/);
  assert.match(topNavigation, /t\(item\.labelKey\)/);
  assert.doesNotMatch(topNavigation, /en:\s*'Home'/);
  assert.doesNotMatch(topNavigation, /zh:\s*'首页'/);

  assert.match(docsPage, /useNavigate/);
  assert.match(docsPage, /useSearchParams/);
  assert.match(docsPage, /searchParams\.get\('group'\)/);
  assert.match(docsPage, /setSearchParams/);
  assert.match(docsPage, /navigate\(/);
  assert.match(docsRegistry, /primaryAction/);
  assert.match(docsRegistry, /secondaryAction/);
  assert.match(docsRegistry, /href:\s*'\/console\/dashboard'/);

  assert.match(homePage, /Enter console/);
  assert.match(homePage, /href:\s*'\/console\/dashboard'/);
  assert.match(homePage, /navigate\('\/console\/dashboard'\)/);
  assert.match(homePage, /Explore models/);
  assert.match(homePage, /Read docs/);
  assert.match(homePage, /Download software/);
  assert.doesNotMatch(homePage, /\{t\('Open'\)\}/);

  assert.match(downloadsPage, /useNavigate/);
  assert.match(downloadsPage, /Install targets/);
  assert.match(downloadsPage, /Launch posture/);
  assert.match(downloadsPage, /Open install guide/);
  assert.match(downloadsPage, /Open console/);
  assert.match(downloadsPage, /href:\s*'\/console\/dashboard'/);
  assert.match(downloadsPage, /navigate\('\/console\/dashboard'\)/);
  assert.match(downloadsPage, /Runtime modes/);
  assert.match(downloadsPage, /pnpm product:start/);
  assert.match(downloadsPage, /pnpm product:service/);
  assert.match(downloadsPage, /pnpm server:start/);
  assert.equal(runtimeModeTitleCount, 1);
});
