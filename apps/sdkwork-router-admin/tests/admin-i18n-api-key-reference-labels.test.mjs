import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('apirouter api key selectors and detail references localize embedded environment labels', () => {
  const shared = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/shared.ts',
  );
  const usagePage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');
  const routeDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyRouteDialog.tsx',
  );
  const rateLimitsPage = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/GatewayRateLimitsPage.tsx',
  );
  const rateLimitPolicyDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/rate-limits/GatewayRateLimitPolicyDialog.tsx',
  );
  const usageDetailPanel = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/usage/GatewayUsageDetailPanel.tsx',
  );

  assert.match(shared, /export function formatApiKeyReferenceLabel\(/);
  assert.match(
    shared,
    /\$\{key\.label \|\| key\.project_id\} \(\$\{formatEnvironmentLabel\(key\.environment, translate\)\}\)/,
  );

  assert.match(usagePage, /label: formatApiKeyReferenceLabel\(key, t\),/);
  assert.match(routeDialog, /value=\{formatApiKeyReferenceLabel\(routeKey, t\)\}/);
  assert.match(rateLimitsPage, /formatApiKeyReferenceLabel\(matchedApiKey, t\)/);
  assert.match(rateLimitPolicyDialog, /label: formatApiKeyReferenceLabel\(apiKey, t\),/);
  assert.match(usageDetailPanel, /formatApiKeyReferenceLabel\(selectedKeyRecord, t\)/);
});
