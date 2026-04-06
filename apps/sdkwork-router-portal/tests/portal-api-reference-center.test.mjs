import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('public site exposes a dedicated API reference center after models and before docs', () => {
  const topNavigation = read('packages/sdkwork-router-portal-core/src/components/PortalTopNavigation.tsx');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const apiReferencePage = read('packages/sdkwork-router-portal-api-reference/src/index.tsx');

  assert.match(topNavigation, /key:\s*'api-reference'/);
  assert.match(topNavigation, /labelKey:\s*'API Reference'/);
  assert.match(topNavigation, /href:\s*'\/api-reference'/);
  assert.match(topNavigation, /key:\s*'models'[\s\S]*key:\s*'api-reference'[\s\S]*key:\s*'docs'/);

  assert.match(portalTypes, /'api-reference'/);
  assert.match(routePaths, /api-reference:\s*'\/api-reference'/);

  assert.match(appRoutes, /PortalApiReferencePage/);
  assert.match(appRoutes, /import\('sdkwork-router-portal-api-reference'\)/);
  assert.match(appRoutes, /PORTAL_ROUTE_PATHS\[['"]api-reference['"]\]|PORTAL_ROUTE_PATHS/);

  assert.match(apiReferencePage, /OpenAPI 3\.1/);
  assert.match(apiReferencePage, /Gateway API/);
  assert.match(apiReferencePage, /Portal API/);
  assert.match(apiReferencePage, /\/openapi\.json/);
  assert.match(apiReferencePage, /\/api\/portal\/openapi\.json|\/portal\/openapi\.json/);
});

test('API reference center derives route coverage and schema facts from live OpenAPI endpoints', () => {
  const apiReferencePage = read('packages/sdkwork-router-portal-api-reference/src/index.tsx');

  assert.match(apiReferencePage, /fetch\(/);
  assert.match(apiReferencePage, /Promise\.all|Promise\.allSettled/);
  assert.match(apiReferencePage, /operationCount/);
  assert.match(apiReferencePage, /schemaVersion/);
  assert.match(apiReferencePage, /tagCount|routeFamilyCount/);
  assert.match(apiReferencePage, /specEndpoint:\s*'\/openapi\.json'/);
  assert.match(apiReferencePage, /specEndpoint:\s*'\/api\/portal\/openapi\.json'/);
  assert.match(apiReferencePage, /case 'conversations':/);
  assert.match(apiReferencePage, /case 'files':/);
  assert.match(apiReferencePage, /case 'uploads':/);
  assert.match(apiReferencePage, /case 'batches':/);
  assert.match(apiReferencePage, /case 'vector-stores':/);
  assert.match(apiReferencePage, /case 'threads':/);
  assert.match(apiReferencePage, /case 'runs':/);
  assert.doesNotMatch(apiReferencePage, /routeFamilies:\s*\[/);
});
