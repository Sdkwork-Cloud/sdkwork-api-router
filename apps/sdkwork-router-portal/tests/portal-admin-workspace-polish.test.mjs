import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('remaining portal workspaces keep compact controls while preserving focused dialog flows', () => {
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(usagePage, /StatCard/);
  assert.match(usagePage, /Total requests/);
  assert.match(usagePage, /Average latency/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.match(usagePage, /data-slot="portal-usage-table"/);
  assert.match(usagePage, /data-slot="portal-usage-pagination"/);
  assert.doesNotMatch(usagePage, /Manage keys/);
  assert.doesNotMatch(usagePage, /Review billing/);
  assert.doesNotMatch(usagePage, /WorkspacePanel/);
  assert.doesNotMatch(usagePage, /ToolbarDisclosure/);
  assert.doesNotMatch(usagePage, /ToolbarSearchField/);
  assert.doesNotMatch(usagePage, /<Tabs/);
  assert.doesNotMatch(usagePage, /AreaChart/);

  assert.match(billingPage, /data-slot="portal-billing-toolbar"/);
  assert.match(billingPage, /<Dialog/);
  assert.match(billingPage, /Open redeem/);
  assert.match(billingPage, /Open usage/);
  assert.match(billingPage, /Open account/);
  assert.match(billingPage, /Checkout preview/);
  assert.match(billingPage, /Plan catalog/);
  assert.match(billingPage, /Order workbench/);
  assert.match(billingPage, /Order lane/);
  assert.match(billingPage, /Pending payment queue/);
  assert.doesNotMatch(billingPage, /<Tabs/);

  assert.match(creditsPage, /data-slot="portal-redeem-entry-card"/);
  assert.match(creditsPage, /data-slot="portal-redeem-invite-card"/);
  assert.match(creditsPage, /data-slot="portal-redeem-history-table"/);
  assert.match(creditsPage, /Redeem/);
  assert.match(creditsPage, /Redeem code/);
  assert.match(creditsPage, /Redeem history/);
  assert.match(creditsPage, /Invite rewards/);
  assert.match(creditsPage, /Copy invite link/);
  assert.match(creditsPage, /Copy invite code/);
  assert.match(
    creditsPage,
    /data-slot="portal-redeem-page"[\s\S]*?data-slot="portal-redeem-entry-card"[\s\S]*?data-slot="portal-redeem-invite-card"[\s\S]*?data-slot="portal-redeem-history-table"/,
  );
  assert.doesNotMatch(creditsPage, /<Tabs/);
  assert.doesNotMatch(creditsPage, /portal-redeem-toolbar/);
  assert.doesNotMatch(creditsPage, /portal-redeem-filter-bar/);
  assert.doesNotMatch(creditsPage, /portal-redeem-invite-filter-bar/);
  assert.doesNotMatch(creditsPage, /portal-redeem-invite-table/);

  assert.match(accountPage, /portal-account-toolbar/);
  assert.match(accountPage, /Search account history/);
  assert.match(accountPage, /<Tabs/);
  assert.match(accountPage, /TabsTrigger value="all"/);
  assert.match(accountPage, /portal-account-history-header/);
  assert.doesNotMatch(accountPage, /Open credits/);
  assert.doesNotMatch(accountPage, /Review billing/);
});

test('usage contracts and financial evidence stay aligned with real server data', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const accountServices = read('packages/sdkwork-router-portal-account/src/services/index.ts');
  const readme = read('README.md');

  assert.match(portalTypes, /input_tokens: number/);
  assert.match(portalTypes, /output_tokens: number/);
  assert.match(portalTypes, /total_tokens: number/);
  assert.match(usagePage, /Input tokens/);
  assert.match(usagePage, /Output tokens/);
  assert.match(usagePage, /Total tokens/);
  assert.match(usagePage, /Total spend/);
  assert.match(usagePage, /Actual spend/);
  assert.match(usagePage, /Reference price/);
  assert.match(usagePage, /data-slot="portal-usage-filter-bar"/);
  assert.doesNotMatch(usagePage, /Search usage/);
  assert.doesNotMatch(accountServices, /Date\.now\(\) - index \* 60_000/);
  assert.match(readme, /sdkwork-router-portal-routing/);
  assert.match(readme, /sdkwork-router-portal-user/);
  assert.match(readme, /User[\s\S]*profile and password rotation/);
  assert.match(readme, /Account[\s\S]*cash balance, billing ledger, and runway posture/);
});

test('portal toolbars keep search first and pin actions to one right-aligned row', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const frameworkForm = read('packages/sdkwork-router-portal-commons/src/framework/form.tsx');
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');

  assert.doesNotMatch(commons, /export \* from '\.\/framework'/);
  assert.doesNotMatch(frameworkForm, /export function ToolbarInline/);
  assert.doesNotMatch(frameworkForm, /export function ToolbarSearchField/);
  assert.match(frameworkForm, /FilterBar/);
  assert.match(frameworkForm, /FilterBarSection/);
  assert.match(frameworkForm, /FilterBarActions/);

  assert.match(
    usagePage,
    /<FilterBar[\s\S]*?data-slot="portal-usage-filter-bar"[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('API key'\)\}[\s\S]*?<FilterBarActions/,
  );
  assert.match(
    usagePage,
    /data-slot="portal-usage-filter-bar"[\s\S]*?wrap=\{false\}/,
  );
  assert.match(
    usagePage,
    /data-slot="portal-usage-filter-bar"[\s\S]*?<FilterBarActions className="gap-2\.5 whitespace-nowrap shrink-0" wrap=\{false\}>/,
  );
  assert.match(
    accountPage,
    /portal-account-history-header[\s\S]*?portal-account-history-tabs[\s\S]*?portal-account-toolbar[\s\S]*?<SearchInput/,
  );
  assert.doesNotMatch(accountPage, /portal-account-toolbar[\s\S]*?<FilterBar/);
  assert.doesNotMatch(accountPage, /portal-account-toolbar[\s\S]*?<FilterBarActions/);
  assert.match(
    apiKeysPage,
    /<FilterBar[\s\S]*?data-slot="portal-api-key-toolbar"[\s\S]*?<FilterBarSection[\s\S]*?<SearchInput[\s\S]*?placeholder=\{t\('Search API keys'\)\}[\s\S]*?<FilterBarSection[\s\S]*?<SelectTrigger[\s\S]*?<SelectValue placeholder=\{t\('Environment'\)\}[\s\S]*?<FilterBarActions[\s\S]*?Create API key/,
  );
  assert.match(
    apiKeysPage,
    /data-slot="portal-api-key-toolbar"[\s\S]*?wrap=\{false\}/,
  );
  assert.match(
    apiKeysPage,
    /data-slot="portal-api-key-toolbar"[\s\S]*?<FilterBarSection className="min-w-0 flex-\[1_1_20rem\]" grow=\{false\} wrap=\{false\}>/,
  );
  assert.match(
    apiKeysPage,
    /data-slot="portal-api-key-toolbar"[\s\S]*?<FilterBarSection className="min-w-\[11rem\] shrink-0" grow=\{false\} wrap=\{false\}>/,
  );
  assert.match(
    apiKeysPage,
    /data-slot="portal-api-key-toolbar"[\s\S]*?<FilterBarActions className="gap-2\.5 whitespace-nowrap shrink-0" wrap=\{false\}>/,
  );
  assert.doesNotMatch(apiKeysPage, /data-slot="portal-api-key-status"/);
  assert.doesNotMatch(apiKeysPage, /Open usage/);
  assert.doesNotMatch(apiKeysPage, /Refresh inventory/);
  assert.match(
    billingPage,
    /portal-billing-toolbar[\s\S]*?<FilterBar[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('Search order lifecycle'\)\}[\s\S]*?<SearchInput[\s\S]*?placeholder=\{t\('Search order lifecycle'\)\}[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('Order lane'\)\}[\s\S]*?<FilterBarActions/,
  );
  assert.doesNotMatch(creditsPage, /data-slot="portal-redeem-filter-bar"/);
  assert.doesNotMatch(creditsPage, /data-slot="portal-redeem-invite-filter-bar"/);
  assert.doesNotMatch(creditsPage, /<FilterBar/);
  assert.match(
    gatewayPage,
    /<FilterBar[\s\S]*?data-slot="portal-gateway-filter-bar"[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('Search gateway evidence'\)\}[\s\S]*?<SearchInput[\s\S]*?placeholder=\{t\('Search gateway evidence'\)\}[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('Workbench lane'\)\}[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('Operational focus'\)\}[\s\S]*?<FilterBarActions/,
  );
  assert.match(
    routingPage,
    /<FilterBar[\s\S]*?data-slot="portal-routing-filter-bar"[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('Search routing evidence'\)\}[\s\S]*?<SearchInput[\s\S]*?placeholder=\{t\('Search routing evidence'\)\}[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('Workbench lane'\)\}[\s\S]*?<FilterBarSection[\s\S]*?<FilterField[\s\S]*?label=\{t\('Operational focus'\)\}[\s\S]*?<FilterBarActions/,
  );
  assert.match(
    routingPage,
    /data-slot="portal-routing-filter-bar"[\s\S]*?<FilterBarActions className="gap-2\.5 whitespace-nowrap"/,
  );
  assert.doesNotMatch(
    routingPage,
    /data-slot="portal-routing-filter-bar"[\s\S]*?ml-auto flex shrink-0 items-center gap-2\.5 whitespace-nowrap/,
  );
});

test('routing and gateway workbench filters localize search and filter labels through shared portal i18n', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');

  assert.match(gatewayPage, /usePortalI18n/);
  assert.match(gatewayPage, /SearchInput/);
  assert.match(gatewayPage, /placeholder=\{t\('Search gateway evidence'\)\}/);
  assert.match(gatewayPage, /label=\{t\('Workbench lane'\)\}/);
  assert.match(gatewayPage, /label=\{t\('Operational focus'\)\}/);

  assert.match(routingPage, /usePortalI18n/);
  assert.match(routingPage, /SearchInput/);
  assert.match(routingPage, /placeholder=\{t\('Search routing evidence'\)\}/);
  assert.match(routingPage, /label=\{t\('Workbench lane'\)\}/);
  assert.match(routingPage, /label=\{t\('Operational focus'\)\}/);

  assert.match(commons, /'Search gateway evidence'/);
  assert.match(commons, /'Search routing evidence'/);
  assert.match(commons, /'Workbench lane'/);
  assert.match(commons, /'Operational focus'/);
});

test('gateway command center localizes status feedback and primary workbench actions through shared portal i18n', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');

  assert.match(gatewayPage, /usePortalI18n/);
  assert.match(gatewayPage, /t\('Loading the router product command center and current launch posture\.\.\.'\)/);
  assert.match(gatewayPage, /t\('Refresh command center'\)/);
  assert.match(gatewayPage, /t\('Refreshing command center\.\.\.'\)/);
  assert.match(gatewayPage, /t\('Refresh service health'\)/);
  assert.match(gatewayPage, /t\('Refreshing service health\.\.\.'\)/);
  assert.match(gatewayPage, /t\('Clear filters'\)/);
  assert.match(gatewayPage, /title=\{t\('Gateway posture'\)\}/);
  assert.match(gatewayPage, /title=\{t\('Preparing gateway command center'\)\}/);

  assert.match(commons, /'Loading the router product command center and current launch posture\.\.\.'/);
  assert.match(commons, /'Refresh command center'/);
  assert.match(commons, /'Refresh service health'/);
  assert.match(commons, /'Preparing gateway command center'/);
});

test('gateway command center localizes lower commercial and deployment surfaces through shared portal i18n', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');

  assert.match(gatewayPage, /title=\{t\('Launch readiness'\)\}/);
  assert.match(
    gatewayPage,
    /t\(\s*'Critical blockers and watchpoints stay visible before launch traffic expands\.'/,
  );
  assert.match(gatewayPage, /title=\{t\('Desktop runtime'\)\}/);
  assert.match(
    gatewayPage,
    /t\(\s*'Desktop runtime cards keep the local bind story visible while Restart desktop runtime remains intentionally narrow\.'/,
  );
  assert.match(gatewayPage, /title=\{t\('Deployment playbooks'\)\}/);
  assert.match(gatewayPage, /t\(\s*'Mode switchboard'/);
  assert.match(gatewayPage, /t\(\s*'Topology playbooks'/);
  assert.match(gatewayPage, /title=\{t\('Commercial runway'\)\}/);
  assert.match(gatewayPage, /t\(\s*'Commerce catalog'/);
  assert.match(gatewayPage, /t\(\s*'Launch actions'/);
  assert.match(
    gatewayPage,
    /t\(\s*'Open API Keys, Open Routing, and Open Billing are the three fastest actions for turning this command center into a real launch workflow\.'/,
  );

  assert.match(commons, /'Launch readiness'/);
  assert.match(commons, /'Desktop runtime'/);
  assert.match(commons, /'Deployment playbooks'/);
  assert.match(commons, /'Mode switchboard'/);
  assert.match(commons, /'Topology playbooks'/);
  assert.match(commons, /'Commercial runway'/);
  assert.match(commons, /'Commerce catalog'/);
  assert.match(commons, /'Launch actions'/);
});

test('gateway workbench configuration and row statuses localize through shared portal i18n instead of raw helper english', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');

  assert.match(gatewayPage, /translatePortalText/);
  assert.match(
    gatewayPage,
    /<Badge variant="outline">\{t\(config\.laneLabel\)\}<\/Badge>/,
  );
  assert.match(gatewayPage, /description=\{t\(config\.detail\)\}/);
  assert.match(gatewayPage, /\{WORKBENCH_LANE_OPTIONS\.map\(\(option\) => \([\s\S]*?\{t\(option\.label\)\}/);
  assert.match(gatewayPage, /\{config\.focusOptions\.map\(\(option\) => \([\s\S]*?\{t\(option\.label\)\}/);
  assert.match(gatewayPage, /header:\s*t\(config\.subjectLabel\)/);
  assert.match(gatewayPage, /header:\s*t\(config\.scopeLabel\)/);
  assert.match(gatewayPage, /header:\s*t\(config\.meterLabel\)/);
  assert.match(gatewayPage, /header:\s*t\(config\.detailLabel\)/);
  assert.match(gatewayPage, /cell:\s*\(row\)\s*=>\s*row\.subject/);
  assert.match(gatewayPage, /cell:\s*\(row\)\s*=>\s*row\.scope/);
  assert.match(gatewayPage, /\{t\(config\.emptyTitle\)\}/);
  assert.match(gatewayPage, /\{t\(config\.emptyDetail\)\}/);
  assert.match(gatewayPage, /return translatePortalText\('Healthy'\)/);
  assert.match(gatewayPage, /return translatePortalText\('Degraded'\)/);
  assert.match(gatewayPage, /return translatePortalText\('Unreachable'\)/);
  assert.match(gatewayPage, /translatePortalText\('No latency sample'\)/);
  assert.match(gatewayPage, /translatePortalText\('Ready to run'\)/);

  assert.match(commons, /'Service health'/);
  assert.match(commons, /'Compatibility routes'/);
  assert.match(commons, /'Verification commands'/);
  assert.match(commons, /'No latency sample'/);
  assert.match(commons, /'Ready to run'/);
});

test('portal form primitives keep a shadcn-style contract and portal settings flows stay on shared form shells', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const frameworkEntry = read('packages/sdkwork-router-portal-commons/src/framework/entry.ts');
  const frameworkForm = read('packages/sdkwork-router-portal-commons/src/framework/form.tsx');
  const settingsCenter = read('packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx');
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const createForm = read(
    'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx',
  );

  assert.doesNotMatch(commons, /export \* from '\.\/framework'/);
  assert.match(frameworkEntry, /Input/);
  assert.match(frameworkEntry, /Label/);
  assert.match(frameworkForm, /SettingsField/);
  assert.match(frameworkForm, /export function SearchInput/);
  assert.doesNotMatch(frameworkForm, /export function FormField/);
  assert.match(frameworkForm, /paddingLeft:\s*['"]2\.75rem['"]/);
  assert.match(settingsCenter, /SettingsField/);
  assert.match(settingsCenter, /SearchInput/);
  assert.match(createForm, /SettingsField/);
  assert.match(routingPage, /SettingsField/);
  assert.doesNotMatch(settingsCenter, /<Search className="absolute left-3 top-1\/2/);
  assert.doesNotMatch(settingsCenter, /<label className="text-sm font-medium text-zinc-700 dark:text-zinc-300">/);
  assert.match(routingPage, /Label/);
  assert.doesNotMatch(routingPage, /<label className="flex items-center gap-3 text-sm font-medium text-zinc-700 dark:text-zinc-300">/);
});
