import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin pricing module is packaged as a first-class route module with lazy shell wiring', () => {
  const packageJson = JSON.parse(read('package.json'));
  const tsconfig = read('tsconfig.json');
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');
  const routes = read('packages/sdkwork-router-admin-core/src/routes.ts');
  const routePaths = read('packages/sdkwork-router-admin-core/src/routePaths.ts');
  const routeManifest = read('packages/sdkwork-router-admin-core/src/routeManifest.ts');
  const routePrefetch = read(
    'packages/sdkwork-router-admin-shell/src/application/router/routePrefetch.ts',
  );
  const appRoutes = read(
    'packages/sdkwork-router-admin-shell/src/application/router/AppRoutes.tsx',
  );
  const workbenchActions = read('packages/sdkwork-router-admin-core/src/workbenchActions.ts');
  const workbench = read('packages/sdkwork-router-admin-core/src/workbench.tsx');
  const pricingPage = read('packages/sdkwork-router-admin-pricing/src/index.tsx');

  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-admin-pricing', 'package.json')),
    true,
  );
  assert.equal(
    packageJson.dependencies['sdkwork-router-admin-pricing'],
    'workspace:*',
  );
  assert.match(tsconfig, /sdkwork-router-admin-pricing/);

  assert.match(types, /'pricing'/);
  assert.match(types, /'sdkwork-router-admin-pricing'/);
  assert.match(routes, /key:\s*'pricing'/);
  assert.match(routes, /label:\s*'Pricing'/);
  assert.match(routePaths, /PRICING:\s*'\/pricing'/);
  assert.match(routePaths, /pricing:\s*ADMIN_ROUTE_PATHS\.PRICING/);
  assert.match(routeManifest, /moduleId:\s*'sdkwork-router-admin-pricing'/);
  assert.match(routeManifest, /displayName:\s*'Pricing'/);
  assert.match(routeManifest, /pricing-governance/);
  assert.match(routeManifest, /billing-methods/);
  assert.match(routePrefetch, /'sdkwork-router-admin-pricing': \(\) => import\('sdkwork-router-admin-pricing'\)/);
  assert.match(appRoutes, /const PricingPage = lazy/);
  assert.match(appRoutes, /ROUTE_PATHS\.PRICING/);
  assert.match(appRoutes, /<PricingPage snapshot=\{snapshot\} \/>/);
  assert.match(workbenchActions, /handleUpdateCommercialPricingPlan/);
  assert.match(workbenchActions, /handleUpdateCommercialPricingRate/);
  assert.match(workbenchActions, /handleCloneCommercialPricingPlan/);
  assert.match(workbenchActions, /handleScheduleCommercialPricingPlan/);
  assert.match(workbenchActions, /handlePublishCommercialPricingPlan/);
  assert.match(workbenchActions, /handleRetireCommercialPricingPlan/);
  assert.match(workbenchActions, /handleSynchronizeCommercialPricingLifecycle/);
  assert.match(workbench, /handleUpdateCommercialPricingPlan/);
  assert.match(workbench, /handleUpdateCommercialPricingRate/);
  assert.match(workbench, /handleCloneCommercialPricingPlan/);
  assert.match(workbench, /handleScheduleCommercialPricingPlan/);
  assert.match(workbench, /handlePublishCommercialPricingPlan/);
  assert.match(workbench, /handleRetireCommercialPricingPlan/);
  assert.match(workbench, /handleSynchronizeCommercialPricingLifecycle/);
  assert.match(pricingPage, /export function PricingPage/);
  assert.match(pricingPage, /Pricing plans/);
  assert.match(pricingPage, /Charge units/);
  assert.match(pricingPage, /Billing methods/);
  assert.match(pricingPage, /Token pricing/);
  assert.match(pricingPage, /Media pricing/);
  assert.match(pricingPage, /Synchronize lifecycle/);
  assert.match(pricingPage, /Due planned versions/);
  assert.match(pricingPage, /Edit plan/);
  assert.match(pricingPage, /Edit rate/);
  assert.match(pricingPage, /Clone plan/);
  assert.match(pricingPage, /Schedule plan/);
  assert.match(pricingPage, /Publish plan/);
  assert.match(pricingPage, /Retire plan/);
  assert.match(pricingPage, /Effective from/);
  assert.match(pricingPage, /Effective to/);
  assert.match(pricingPage, /Update plan/);
  assert.match(pricingPage, /Update rate/);
});
