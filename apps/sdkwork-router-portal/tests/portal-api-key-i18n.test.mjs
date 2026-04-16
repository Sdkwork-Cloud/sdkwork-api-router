import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal api key creation and usage flows localize user-facing copy through shared portal i18n', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const createForm = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx',
  );
  const drawers = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDrawers.tsx',
  );
  const groupsDialog = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyGroupsDialog.tsx',
  );
  const table = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');
  const page = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');

  assert.match(createForm, /usePortalI18n/);
  assert.match(createForm, /label=\{t\('Key label'\)\}/);
  assert.match(createForm, /placeholder=\{t\('Production rollout'\)\}/);
  assert.match(createForm, /label=\{t\('API key group'\)\}/);
  assert.match(createForm, /placeholder=\{t\('API key group'\)\}/);
  assert.match(createForm, /t\('No group binding'\)/);
  assert.match(createForm, /label=\{t\('Gateway key mode'\)\}/);
  assert.doesNotMatch(createForm, /placeholder="skw_live_custom_portal_secret"/);
  assert.match(createForm, /placeholder=\{t\('Example: skw_live_custom_portal_secret'\)\}/);
  assert.match(createForm, /t\('Creating API key\.\.\.'\)/);

  assert.match(drawers, /usePortalI18n/);
  assert.match(drawers, /<DrawerTitle>\{t\('Create API key'\)\}<\/DrawerTitle>/);
  assert.match(drawers, /t\('API key details'\)/);
  assert.match(drawers, /t\('How to use this key'\)/);
  assert.match(drawers, /t\('Apply setup'\)/);

  assert.match(groupsDialog, /usePortalI18n/);
  assert.match(groupsDialog, /t\('API key groups'\)/);
  assert.match(groupsDialog, /t\('Create group'\)/);
  assert.match(groupsDialog, /t\('Delete group'\)/);
  assert.match(groupsDialog, /t\('Routing profile'\)/);
  assert.match(groupsDialog, /t\('No routing profile override'\)/);
  assert.match(groupsDialog, /t\('Accounting mode'\)/);

  assert.match(table, /usePortalI18n/);
  assert.match(table, /label: t\('Name'\)/);
  assert.match(table, /label: t\('Key group'\)/);
  assert.match(table, /sdkwork-router-portal-commons\/framework/);
  assert.match(table, /t\('No API keys yet'\)/);
  assert.match(table, /t\('View details'\)/);

  assert.match(page, /usePortalI18n/);
  assert.match(page, /placeholder=\{t\('Search API keys'\)\}/);
  assert.match(page, /placeholder=\{t\('Environment'\)\}/);
  assert.match(page, /placeholder=\{t\('Key group'\)\}/);
  assert.match(page, /t\('Manage groups'\)/);
  assert.match(page, /t\('Key label is required so credentials remain auditable after creation\.'\)/);
  assert.match(page, /t\('Plaintext key copied to clipboard\.'\)/);
  assert.match(page, /t\('Applying \{label\} setup\.\.\.', \{ label: selectedPlan\.label \}\)/);

  assert.match(commons, /'Key label'/);
  assert.match(commons, /'API key group'/);
  assert.match(commons, /'No group binding'/);
  assert.match(commons, /'Key group'/);
  assert.match(commons, /'Gateway key mode'/);
  assert.match(commons, /'Example: skw_live_custom_portal_secret'/);
  assert.match(commons, /'Creating API key\.\.\.'/);
  assert.match(commons, /'API key details'/);
  assert.match(commons, /'Manage groups'/);
  assert.match(commons, /'API key groups'/);
  assert.match(commons, /'Create group'/);
  assert.match(commons, /'Delete group'/);
  assert.match(commons, /'Accounting mode'/);
  assert.match(commons, /'View details'/);
  assert.match(commons, /'No API keys yet'/);
  assert.match(commons, /'Plaintext key copied to clipboard\.'/);
});
