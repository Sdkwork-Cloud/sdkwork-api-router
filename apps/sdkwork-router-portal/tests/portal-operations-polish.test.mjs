import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('api keys page exposes lifecycle, environment, and key-handling guidance', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');
  const drawers = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDrawers.tsx');
  const table = read('packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const frameworkDisplay = read('packages/sdkwork-router-portal-commons/src/framework/display.ts');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const repository = read('packages/sdkwork-router-portal-api-keys/src/repository/index.ts');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(apiKeysPage, /PortalApiKeyDrawers/);
  assert.match(apiKeysPage, /Credential inventory/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-toolbar"/);
  assert.match(apiKeysPage, /data-slot="portal-api-key-pagination"/);
  assert.match(apiKeysPage, /PortalApiKeyTable/);
  assert.doesNotMatch(apiKeysPage, /SectionHeader/);
  assert.doesNotMatch(apiKeysPage, /WorkspacePanel/);
  assert.doesNotMatch(apiKeysPage, /CrudWorkbench/);
  assert.doesNotMatch(apiKeysPage, /PortalApiKeyManagerToolbar/);
  assert.match(drawers, /Recommended key setup/);
  assert.match(drawers, /Key label/);
  assert.match(drawers, /Custom environment/);
  assert.match(drawers, /Lifecycle policy/);
  assert.match(drawers, /How to use this key/);
  assert.match(drawers, /DrawerTitle/);
  assert.match(table, /Last authenticated use/);
  assert.match(table, /View details/);
  assert.match(table, /Delete/);
  assert.match(table, /Disable|Enable/);
  assert.match(table, /DataTable/);
  assert.doesNotMatch(table, /if \(!items.length\)/);
  assert.match(table, /No API keys yet/);
  assert.doesNotMatch(commons, /export \* from '\.\/framework'/);
  assert.match(frameworkDisplay, /@sdkwork\/ui-pc-react\/components\/ui\/data-display/);
  assert.match(frameworkDisplay, /DataTable/);
  assert.doesNotMatch(frameworkDisplay, /export function DataTable/);
  assert.doesNotMatch(frameworkDisplay, /type LegacyDataTableColumn/);
  assert.doesNotMatch(commons, /data-slot="table-container"/);
  assert.doesNotMatch(commons, /data-slot="table-header"/);
  assert.doesNotMatch(commons, /data-slot="table-empty"/);
  assert.match(portalApi, /label: string/);
  assert.match(portalApi, /expires_at_ms\?: number \| null/);
  assert.match(portalApi, /updatePortalApiKeyStatus/);
  assert.match(portalApi, /deletePortalApiKey/);
  assert.match(repository, /setPortalApiKeyActive/);
  assert.match(repository, /removePortalApiKey/);
  assert.match(portalTypes, /label: string/);
  assert.match(portalTypes, /created_at_ms: number/);
  assert.match(portalTypes, /last_used_at_ms\?: number \| null/);
  assert.match(portalTypes, /expires_at_ms\?: number \| null/);
});

test('usage page stays focused on a compact request log workbench', () => {
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');

  assert.match(usagePage, /StatCard/);
  assert.match(usagePage, /Total requests/);
  assert.match(usagePage, /Total spend/);
  assert.match(usagePage, /Average latency/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.match(usagePage, /data-slot="portal-usage-table"/);
  assert.match(usagePage, /data-slot="portal-usage-pagination"/);
  assert.doesNotMatch(usagePage, /Manage keys/);
  assert.doesNotMatch(usagePage, /Review billing/);
  assert.match(usagePage, /Recorded/);
  assert.doesNotMatch(usagePage, /WorkspacePanel/);
  assert.doesNotMatch(usagePage, /Search usage/);
  assert.doesNotMatch(usagePage, /Request volume/);
  assert.doesNotMatch(usagePage, /Demand mix/);
  assert.doesNotMatch(usagePage, /Request diagnostics/);
});

test('user page exposes a professional user center with profile, binding, privacy, and security controls', () => {
  const userPage = read('packages/sdkwork-router-portal-user/src/pages/index.tsx');

  assert.match(userPage, /data-slot="portal-user-toolbar"/);
  assert.match(userPage, /User details/);
  assert.match(userPage, /Profile overview/);
  assert.match(userPage, /Phone binding/);
  assert.match(userPage, /WeChat binding/);
  assert.match(userPage, /Privacy preferences/);
  assert.match(userPage, /Password and authentication/);
  assert.match(userPage, /Change password/);
  assert.match(userPage, /data-slot="portal-user-center-tabs"/);
  assert.doesNotMatch(userPage, /Profile facts/);
  assert.doesNotMatch(userPage, /Personal security checklist/);
  assert.doesNotMatch(userPage, /Recovery signals/);
});

test('account page exposes revenue windows and a tabbed finance history workbench', () => {
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(accountPage, /portal-account-toolbar/);
  assert.match(accountPage, /portal-account-pagination/);
  assert.match(accountPage, /Search account history/);
  assert.match(accountPage, /Balance/);
  assert.match(accountPage, /Revenue/);
  assert.match(accountPage, /Total requests/);
  assert.match(accountPage, /Used units/);
  assert.match(accountPage, /viewModel\.today\.used_units/);
  assert.match(accountPage, /viewModel\.trailing_7d\.used_units/);
  assert.match(accountPage, /viewModel\.current_month\.used_units/);
  assert.match(accountPage, /portal-account-used-breakdowns/);
  assert.match(accountPage, /Average booked spend/);
  assert.match(accountPage, /Recharge/);
  assert.match(accountPage, /Redeem/);
  assert.doesNotMatch(accountPage, /Redeem credits/);
  assert.doesNotMatch(accountPage, /Open credits/);
  assert.doesNotMatch(accountPage, /Review billing/);
  assert.match(accountPage, /portal-account-balance-primary/);
  assert.match(accountPage, /portal-account-balance-actions/);
  assert.match(accountPage, /Today/);
  assert.match(accountPage, /7 days/);
  assert.match(accountPage, /This month/);
  assert.doesNotMatch(accountPage, /Account posture/);
  assert.match(accountPage, /Account history/);
  assert.match(accountPage, /TabsTrigger value="all"/);
  assert.match(accountPage, /TabsTrigger value="expense"/);
  assert.match(accountPage, /TabsTrigger value="revenue"/);
  assert.match(accountPage, /data-slot="portal-account-history-tabs"/);
  assert.match(accountPage, /formatDateTime/);
  assert.match(accountPage, /data-slot="portal-account-history-header"/);
  assert.match(accountPage, /data-slot="portal-account-history-search"/);
  assert.doesNotMatch(accountPage, /Quota health/);
  assert.match(accountPage, /Ledger snapshot/);
  assert.match(accountPage, /Expense/);
  assert.match(accountPage, /t\(titleCaseToken\(viewModel\.commercial_posture\.account_status\)\)/);
  assert.match(accountPage, /benefitType: t\(titleCaseToken\(lot\.benefit_type\)\)/);
  assert.match(accountPage, /t\(titleCaseToken\(lot\.source_type\)\)/);
  assert.match(accountPage, /t\(titleCaseToken\(lot\.status\)\)/);
  assert.doesNotMatch(accountPage, /Financial posture/);
  assert.doesNotMatch(accountPage, /Membership posture/);
  assert.doesNotMatch(accountPage, /Remaining units:/);
});
