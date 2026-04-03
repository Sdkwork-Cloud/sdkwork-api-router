import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('form-heavy portal pages use focused drawers and admin tables instead of always-expanded forms', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const apiKeyDrawers = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDrawers.tsx',
  );
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');

  assert.match(apiKeysPage, /PortalApiKeyDrawers/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-toolbar"/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-pagination"/);
  assert.doesNotMatch(apiKeysPage, /SectionHeader/);
  assert.doesNotMatch(apiKeysPage, /WorkspacePanel/);
  assert.doesNotMatch(apiKeysPage, /CrudWorkbench/);
  assert.match(apiKeyDrawers, /<Drawer/);
  assert.doesNotMatch(apiKeyDrawers, /<Dialog/);
  assert.match(apiKeyDrawers, /Create API key/);
  assert.match(apiKeyDrawers, /Lifecycle policy/);
  assert.match(apiKeyDrawers, /How to use this key/);

  assert.match(routingPage, /data-slot="portal-routing-toolbar"/);
  assert.match(routingPage, /data-slot="portal-routing-filter-bar"/);
  assert.match(routingPage, /Edit posture/);
  assert.match(routingPage, /Run preview/);
  assert.match(routingPage, /Evidence stream/);
  assert.match(routingPage, /<Dialog/);
  assert.doesNotMatch(routingPage, /<Tabs/);
  assert.doesNotMatch(routingPage, /Policy editor/);

  assert.match(userPage, /<Tabs/);
  assert.match(userPage, /<Dialog/);
  assert.match(userPage, /Security center/);
});

test('portal api key create form reuses shared shadcn-style form primitives instead of local wrappers', () => {
  const createForm = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx');
  const components = read('packages/sdkwork-router-portal-api-keys/src/components/index.tsx');

  assert.match(createForm, /sdkwork-router-portal-commons/);
  assert.match(createForm, /Button/);
  assert.match(createForm, /Input/);
  assert.match(createForm, /Select/);
  assert.match(createForm, /Textarea/);
  assert.match(createForm, /ApiKeyModeChoiceCard/);
  assert.match(components, /ApiKeyModeChoiceCard/);
  assert.doesNotMatch(createForm, /function TextInput/);
  assert.doesNotMatch(createForm, /function SelectInput/);
  assert.doesNotMatch(createForm, /function TextArea/);
  assert.doesNotMatch(createForm, /function SelectionCard/);
  assert.doesNotMatch(createForm, /<button/);
  assert.doesNotMatch(createForm, /<textarea/);
});

test('portal settings center and api key table actions also reuse shared Button primitives', () => {
  const settingsCenter = read(
    'packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx',
  );
  const apiKeyTable = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');

  assert.match(settingsCenter, /Button/);
  assert.match(apiKeyTable, /Button/);
  assert.doesNotMatch(settingsCenter, /<button/);
  assert.doesNotMatch(apiKeyTable, /<button/);
});

test('portal settings center delegates presentation primitives to a dedicated settings module', () => {
  const settingsCenter = read(
    'packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx',
  );
  const settingsPrimitives = read(
    'packages/sdkwork-router-portal-core/src/components/settings/PortalSettingsPrimitives.tsx',
  );

  assert.match(settingsCenter, /PortalSettingsNavButton/);
  assert.match(settingsCenter, /PortalSettingsPanelCard/);
  assert.match(settingsCenter, /PortalThemeModeChoiceCard/);
  assert.match(settingsCenter, /PortalThemeColorSwatch/);
  assert.doesNotMatch(settingsCenter, /function ConfigNavButton/);
  assert.doesNotMatch(settingsCenter, /function ConfigBlock/);
  assert.doesNotMatch(settingsCenter, /function ModeChoice/);
  assert.doesNotMatch(settingsCenter, /function ColorSwatch/);
  assert.match(settingsPrimitives, /export function PortalSettingsNavButton/);
  assert.match(settingsPrimitives, /export function PortalSettingsPanelCard/);
  assert.match(settingsPrimitives, /export function PortalThemeModeChoiceCard/);
  assert.match(settingsPrimitives, /export function PortalThemeColorSwatch/);
});

test('portal api key create form delegates managed-key notice and form shell styling to shared surfaces', () => {
  const createForm = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx');
  const components = read('packages/sdkwork-router-portal-api-keys/src/components/index.tsx');
  const managedNotice = read(
    'packages/sdkwork-router-portal-api-keys/src/components/ApiKeyManagedNoticeCard.tsx',
  );

  assert.match(createForm, /Card/);
  assert.match(createForm, /ApiKeyManagedNoticeCard/);
  assert.match(components, /ApiKeyManagedNoticeCard/);
  assert.doesNotMatch(
    createForm,
    /rounded-\[28px\] border border-zinc-200 bg-zinc-50\/80 p-5 dark:border-zinc-800 dark:bg-zinc-900\/50/,
  );
  assert.doesNotMatch(createForm, /Portal-managed key/);
  assert.match(managedNotice, /export function ApiKeyManagedNoticeCard/);
  assert.match(managedNotice, /Card/);
  assert.match(managedNotice, /Portal-managed key/);
});
