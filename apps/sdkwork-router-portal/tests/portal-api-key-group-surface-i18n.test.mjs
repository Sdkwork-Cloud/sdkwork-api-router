import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal api key group dialog localizes standard environment and accounting mode labels while preserving custom values', () => {
  const groupsDialog = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyGroupsDialog.tsx',
  );
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(groupsDialog, /function formatEnvironmentLabel\(/);
  assert.match(groupsDialog, /case 'live':\s*return t\('Live'\);/);
  assert.match(groupsDialog, /case 'staging':\s*return t\('Staging'\);/);
  assert.match(groupsDialog, /case 'test':\s*return t\('Test'\);/);
  assert.match(groupsDialog, /default:\s*return value;/);
  assert.match(groupsDialog, /<Badge variant="outline">\{formatEnvironmentLabel\(group\.environment, t\)\}<\/Badge>/);
  assert.match(
    groupsDialog,
    /<SelectItem key=\{option\} value=\{option\}>\s*\{formatEnvironmentLabel\(option, t\)\}\s*<\/SelectItem>/,
  );

  assert.match(groupsDialog, /function formatAccountingModeLabel\(/);
  assert.match(
    groupsDialog,
    /case 'platform_credit':\s*return t\('Platform credit'\);/,
  );
  assert.match(groupsDialog, /case 'byok':\s*return t\('BYOK'\);/);
  assert.match(groupsDialog, /case 'passthrough':\s*return t\('Passthrough'\);/);
  assert.match(
    groupsDialog,
    /<Badge variant="outline">\{formatAccountingModeLabel\(group\.default_accounting_mode, t\)\}<\/Badge>/,
  );

  assert.match(commons, /'Live'/);
  assert.match(commons, /'Staging'/);
  assert.match(commons, /'Test'/);
  assert.match(commons, /'Platform credit'/);
  assert.match(commons, /'BYOK'/);
  assert.match(commons, /'Passthrough'/);
});
