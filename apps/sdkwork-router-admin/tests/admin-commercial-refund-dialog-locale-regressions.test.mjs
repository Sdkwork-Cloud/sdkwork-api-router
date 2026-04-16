import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('refund dialog formats refundable amount placeholders through the active admin locale helpers', () => {
  const source = read(
    'packages/sdkwork-router-admin-commercial/src/paymentRefundDialog.tsx',
  );

  assert.doesNotMatch(source, /placeholder=\{String\(order\.refundable_amount_minor\)\}/);
  assert.match(source, /const \{ formatNumber, t \} = useAdminI18n\(\);/);
  assert.match(source, /placeholder=\{formatNumber\(order\.refundable_amount_minor\)\}/);
});
