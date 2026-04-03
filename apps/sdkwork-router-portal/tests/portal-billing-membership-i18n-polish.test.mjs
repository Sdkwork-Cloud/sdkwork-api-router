import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('billing membership facts productize status labels instead of exposing raw enum values', () => {
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(billingPage, /function resolveMembershipStatusLabel\(/);
  assert.doesNotMatch(billingPage, /value:\s*membership\?\.status \?\? t\('Inactive'\)/);
  assert.match(commons, /'Past due'/);
  assert.match(commons, /'Grace period'/);
  assert.match(commons, /'Paused'/);
});
