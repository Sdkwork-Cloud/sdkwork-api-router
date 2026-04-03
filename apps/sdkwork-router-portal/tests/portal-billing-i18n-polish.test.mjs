import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('billing workspace localizes order lifecycle labels and dynamic checkout copy through shared portal i18n', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');

  assert.match(billingPage, /header:\s*t\('Offer'\)/);
  assert.match(billingPage, /header:\s*t\('Kind'\)/);
  assert.match(billingPage, /header:\s*t\('Coupon'\)/);
  assert.match(billingPage, /header:\s*t\('Payable'\)/);
  assert.match(billingPage, /header:\s*t\('Granted units'\)/);
  assert.match(billingPage, /header:\s*t\('Status'\)/);
  assert.match(billingPage, /header:\s*t\('Created'\)/);
  assert.match(billingPage, /header:\s*t\('Actions'\)/);
  assert.match(
    billingPage,
    /setCheckoutSessionStatus\(t\('Open session from Pending payment queue to inspect the payment rail\.'\)\)/,
  );
  assert.match(
    billingPage,
    /setCheckoutSessionStatus\(\s*t\('Loading checkout session for \{orderId\}\.\.\.',[\s\S]*?\{\s*orderId\s*\}[\s\S]*?\)\s*\)/,
  );
  assert.match(
    billingPage,
    /setCheckoutStatus\(\s*t\('Loading live checkout pricing for \{targetId\}\.\.\.',[\s\S]*?\{\s*targetId:\s*selection\.target\.id[\s\S]*?\}\s*[\s\S]*?\)\s*\)/,
  );
  assert.match(
    billingPage,
    /setCheckoutStatus\(\s*t\('Creating a checkout order for \{targetId\}\.\.\.',[\s\S]*?\{\s*targetId:\s*checkoutSelection\.target\.id[\s\S]*?\}\s*[\s\S]*?\)\s*\)/,
  );
  assert.match(billingPage, /function targetKindLabel\(/);
  assert.match(billingPage, /function orderStatusLabel\(/);
  assert.match(billingPage, /function checkoutSessionStatusLabel\(/);
  assert.match(billingPage, /function checkoutMethodActionLabel\(/);
  assert.match(billingPage, /function checkoutMethodAvailabilityLabel\(/);
  assert.match(billingPage, /targetKindLabel\(row\.target_kind,\s*t\)/);
  assert.match(billingPage, /orderStatusLabel\(row\.status,\s*t\)/);
  assert.match(billingPage, /checkoutSessionStatusLabel\(checkoutSession\.session_status,\s*t\)/);
  assert.match(billingPage, /checkoutMethodActionLabel\(method\.action,\s*t\)/);
  assert.match(billingPage, /checkoutMethodAvailabilityLabel\(method\.availability,\s*t\)/);
  assert.match(billingPage, /t\('\{kind\} \/ \{price\}',\s*\{\s*kind:/);
  assert.doesNotMatch(billingPage, /è·¯/);

  for (const key of [
    'Offer',
    'Kind',
    'Coupon',
    'Payable',
    'Payment pending',
    'Fulfilled',
    'Canceled',
    'Failed',
    'Open session',
    'Loading checkout session for {orderId}...',
    'Loading live checkout pricing for {targetId}...',
    'Creating a checkout order for {targetId}...',
    'Subscription plan',
    'Recharge pack',
    'Settle order',
    'Cancel order',
    'Provider handoff',
    'Planned',
    'Closed',
    '{kind} / {price}',
    '{planName} is the active workspace membership and defines the current subscription entitlement baseline.',
    'No active membership is recorded yet. Settle a subscription order to activate monthly entitlement posture.',
    'Awaiting pending order',
  ]) {
    assert.match(commons, new RegExp(`'${key.replace(/[.*+?^${}()|[\\]\\\\]/g, '\\\\$&')}'`));
  }
});
