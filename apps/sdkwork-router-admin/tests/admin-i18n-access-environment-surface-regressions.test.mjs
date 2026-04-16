import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('gateway access registry, detail panel, and edit dialog localize standard environment and accounting mode labels', () => {
  const shared = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/shared.ts',
  );
  const registry = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessRegistrySection.tsx',
  );
  const detail = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayAccessDetailPanel.tsx',
  );
  const editDialog = read(
    'packages/sdkwork-router-admin-apirouter/src/pages/access/GatewayApiKeyEditDialog.tsx',
  );

  assert.match(shared, /export function formatEnvironmentLabel\(/);
  assert.match(shared, /case 'live':\s*return translate\('Live'\);/);
  assert.match(shared, /case 'staging':\s*return translate\('Staging'\);/);
  assert.match(shared, /case 'test':\s*return translate\('Test'\);/);
  assert.match(shared, /case 'production':\s*return translate\('Production'\);/);
  assert.match(shared, /case 'development':\s*return translate\('Development'\);/);
  assert.match(shared, /default:\s*return value;/);

  assert.match(shared, /export function formatAccountingModeLabel\(/);
  assert.match(shared, /case 'platform_credit':\s*return translate\('Platform credit'\);/);
  assert.match(shared, /case 'byok':\s*return translate\('BYOK'\);/);
  assert.match(shared, /case 'passthrough':\s*return translate\('Passthrough'\);/);

  assert.match(registry, /formatEnvironmentLabel\(key\.environment, t\)/);
  assert.match(detail, /formatEnvironmentLabel\(selectedKey\.environment, t\)/);
  assert.match(detail, /formatAccountingModeLabel\(group\?\.default_accounting_mode, t\)/);
  assert.match(editDialog, /formatEnvironmentLabel\(editingKey\.environment, t\)/);
});
