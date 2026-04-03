import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('account history and redeem invite rewards productize workspace labels instead of exposing raw project ids', () => {
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const creditsPage = read('packages/sdkwork-router-portal-credits/src/pages/index.tsx');

  assert.match(accountPage, /function resolveHistoryProjectLabel\(/);
  assert.doesNotMatch(accountPage, /return row\.project_id;/);
  assert.doesNotMatch(accountPage, /row\.project_id,\s*\]/);

  assert.match(creditsPage, /Invite rewards/);
  assert.match(creditsPage, /Copy invite link/);
  assert.doesNotMatch(creditsPage, /<strong>\{summary\.project_id\}<\/strong>/);
});
