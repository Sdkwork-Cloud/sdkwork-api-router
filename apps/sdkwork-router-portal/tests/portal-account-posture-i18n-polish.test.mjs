import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('account posture productizes membership status and quota policy labels', () => {
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(accountPage, /function resolveMembershipStatusLabel\(/);
  assert.match(accountPage, /function resolveQuotaPolicyLabel\(/);
  assert.doesNotMatch(accountPage, /value=\{viewModel\.membership\?\.status \?\? t\('Inactive'\)\}/);
  assert.doesNotMatch(accountPage, /viewModel\.billing_summary\.quota_policy_id \?\? t\('Workspace default'\)/);

  assert.match(commons, /'Past due'/);
  assert.match(commons, /'Grace period'/);
  assert.match(commons, /'Enterprise quota'/);
});
