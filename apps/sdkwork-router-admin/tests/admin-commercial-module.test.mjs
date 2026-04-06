import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin commercial module is packaged as a first-class route module with lazy shell wiring', () => {
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

  assert.equal(
    existsSync(path.join(appRoot, 'packages', 'sdkwork-router-admin-commercial', 'package.json')),
    true,
  );
  assert.equal(
    packageJson.dependencies['sdkwork-router-admin-commercial'],
    'workspace:*',
  );
  assert.equal(
    packageJson.dependencies['sdkwork-router-admin-admin-api'],
    'workspace:*',
  );
  assert.match(tsconfig, /sdkwork-router-admin-commercial/);

  assert.match(types, /'commercial'/);
  assert.match(types, /'sdkwork-router-admin-commercial'/);
  assert.match(routes, /key:\s*'commercial'/);
  assert.match(routes, /label:\s*'Commercial'/);
  assert.match(routePaths, /COMMERCIAL:\s*'\/commercial'/);
  assert.match(routePaths, /commercial:\s*ADMIN_ROUTE_PATHS\.COMMERCIAL/);
  assert.match(routeManifest, /moduleId:\s*'sdkwork-router-admin-commercial'/);
  assert.match(routeManifest, /displayName:\s*'Commercial'/);
  assert.match(routeManifest, /settlement-explorer/);
  assert.match(routeManifest, /commercial-accounts/);
  assert.match(routePrefetch, /'sdkwork-router-admin-commercial': \(\) => import\('sdkwork-router-admin-commercial'\)/);
  assert.match(appRoutes, /const CommercialPage = lazy/);
  assert.match(appRoutes, /ROUTE_PATHS\.COMMERCIAL/);
  assert.match(appRoutes, /<CommercialPage snapshot=\{snapshot\} \/>/);
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /export function CommercialPage/,
  );
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /Commercial accounts/,
  );
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /Settlement explorer/,
  );
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /Pricing governance/,
  );
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /getCommerceOrderAudit/,
  );
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /normalizeCommercialOrderAuditLookupValue/,
  );
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /Order audit detail/,
  );
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /Find order audit/,
  );
  assert.match(
    read('packages/sdkwork-router-admin-commercial/src/index.tsx'),
    /Enter an order id to open order audit detail/,
  );
});
