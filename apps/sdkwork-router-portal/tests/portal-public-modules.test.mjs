import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('public modules expose a real landing page, grouped docs center, and software downloads guidance', () => {
  const homePage = read('packages/sdkwork-router-portal-home/src/index.tsx');
  const docsPage = read('packages/sdkwork-router-portal-docs/src/index.tsx');
  const downloadsPage = read('packages/sdkwork-router-portal-downloads/src/index.tsx');
  const modelsPage = read('packages/sdkwork-router-portal-models/src/index.tsx');
  const frameworkExports = read('packages/sdkwork-router-portal-commons/src/framework.tsx');
  const commonsPackage = read('packages/sdkwork-router-portal-commons/package.json');
  const docsRegistryPath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-docs',
    'src',
    'registry.ts',
  );

  assert.match(homePage, /Unified AI gateway workspace/);
  assert.match(homePage, /Enter console/);
  assert.match(homePage, /Explore models/);
  assert.match(homePage, /Documentation center|Docs/);
  assert.match(homePage, /Software Downloads/);
  assert.match(frameworkExports, /\.\/framework\/site/);
  assert.match(commonsPackage, /"\.\/framework\/site"/);
  assert.match(homePage, /framework\/site/);
  assert.match(modelsPage, /framework\/site/);
  assert.match(docsPage, /framework\/site/);
  assert.match(downloadsPage, /framework\/site/);
  assert.match(homePage, /PortalSiteHero/);
  assert.match(homePage, /PortalSitePanel/);
  assert.match(homePage, /PortalSiteMetricCard/);
  assert.match(homePage, /data-slot="portal-home-metrics"/);
  assert.match(homePage, /data-slot="portal-home-pathways"/);
  assert.match(homePage, /data-slot="portal-home-value-grid"/);
  assert.match(homePage, /data-slot="portal-home-launch-tracks"/);
  assert.match(homePage, /Product pathways/);
  assert.match(homePage, /Launch tracks/);
  assert.match(homePage, /Business-ready surfaces/);
  assert.match(homePage, /Operator-first onboarding/);
  assert.match(homePage, /Launch without context loss/);
  assert.match(homePage, /Platform teams/);
  assert.match(homePage, /Application teams/);
  assert.match(homePage, /Operations teams/);
  assert.match(homePage, /Start with product posture/);
  assert.match(homePage, /Map the model layer/);
  assert.match(homePage, /Follow implementation guides/);
  assert.match(homePage, /Install and launch runtime/);
  assert.match(modelsPage, /PortalSiteMetricCard/);
  assert.match(docsPage, /PortalSitePanel/);
  assert.match(downloadsPage, /PortalSiteHero/);
  assert.doesNotMatch(homePage, /framework\/layout/);
  assert.doesNotMatch(modelsPage, /framework\/layout/);
  assert.doesNotMatch(docsPage, /framework\/layout/);
  assert.doesNotMatch(downloadsPage, /framework\/layout/);

  assert.equal(existsSync(docsRegistryPath), true, 'missing docs registry');
  const docsRegistry = read('packages/sdkwork-router-portal-docs/src/registry.ts');
  assert.match(docsRegistry, /Quickstart/);
  assert.match(docsRegistry, /SDKs|Integration/);
  assert.match(docsRegistry, /Operations/);
  assert.match(docsRegistry, /Reference|API/);
  assert.match(docsRegistry, /Open quickstart/);
  assert.match(docsRegistry, /Open console/);
  assert.match(docsPage, /Documentation center/);
  assert.match(docsPage, /registry/);
  assert.match(docsPage, /useSearchParams/);

  assert.match(downloadsPage, /Software Downloads/);
  assert.match(downloadsPage, /Windows/);
  assert.match(downloadsPage, /macOS/);
  assert.match(downloadsPage, /Linux/);
  assert.match(downloadsPage, /System requirements/);
  assert.match(downloadsPage, /Installation steps/);
  assert.match(downloadsPage, /Open install guide/);
  assert.match(downloadsPage, /pnpm product:start/);
  assert.match(downloadsPage, /pnpm product:service/);
  assert.match(downloadsPage, /pnpm server:start/);
  assert.doesNotMatch(downloadsPage, /\{t\('Download'\)\}/);
});
