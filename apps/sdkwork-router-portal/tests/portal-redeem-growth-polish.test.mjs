import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('redeem becomes the visible finance-growth entry while account stays focused on balance history', () => {
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(routes, /key:\s*'credits'/);
  assert.match(routes, /labelKey:\s*'Redeem'/);
  assert.match(routes, /eyebrowKey:\s*'Growth'/);
  assert.match(routes, /detailKey:\s*'Coupons, invites, and activation rewards'/);
  assert.match(routePaths, /credits: '\/console\/redeem'/);
  assert.match(accountPage, /Redeem/);
  assert.doesNotMatch(accountPage, /Redeem credits/);
  assert.doesNotMatch(accountPage, /Open credits/);
  assert.doesNotMatch(accountPage, /Review billing/);
});

test('redeem page combines coupon redemption and invite growth tooling on one focused workspace surface', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');

  assert.match(creditsPage, /portal-redeem-entry-card/);
  assert.match(creditsPage, /portal-redeem-entry-hero/);
  assert.match(creditsPage, /portal-redeem-invite-card/);
  assert.match(creditsPage, /portal-redeem-history-table/);
  assert.match(creditsPage, /Redeem code/);
  assert.match(creditsPage, /Redeem now/);
  assert.match(creditsPage, /Redeem history/);
  assert.match(creditsPage, /Invite rewards/);
  assert.match(creditsPage, /Copy invite link/);
  assert.match(creditsPage, /Copy invite code/);
  assert.match(creditsPage, /copyText/);
  assert.match(
    creditsPage,
    /className="border-primary-500\/15 bg-primary-500\/8 shadow-none dark:border-primary-500\/20 dark:bg-primary-500\/10"/,
  );
  assert.match(
    creditsPage,
    /data-slot="portal-redeem-entry-hero"[\s\S]*?Balance[\s\S]*?Eligible offers/,
  );
  assert.match(
    creditsPage,
    /data-slot="portal-redeem-invite-card"[\s\S]*?border-zinc-200 bg-white/,
  );
  assert.doesNotMatch(creditsPage, /portal-redeem-toolbar/);
  assert.doesNotMatch(creditsPage, /portal-redeem-invite-filter-bar/);
  assert.doesNotMatch(creditsPage, /portal-redeem-invite-table/);
  assert.doesNotMatch(creditsPage, /Search redeem offers/);
  assert.doesNotMatch(creditsPage, /Pending invites/);
  assert.doesNotMatch(creditsPage, /Rewarded activations/);
  assert.match(commons, /'Redeem'/);
  assert.match(commons, /'Redeem code'/);
  assert.match(commons, /'Redeem now'/);
  assert.match(commons, /'Redeem history'/);
  assert.match(commons, /'Invite rewards'/);
  assert.match(commons, /'Copy invite link'/);
  assert.match(commons, /'Copy invite code'/);
});
