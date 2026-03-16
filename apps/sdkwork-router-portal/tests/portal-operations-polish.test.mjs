import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('api keys page exposes environment strategy and key-handling guidance', () => {
  const apiKeysPage = read('packages/sdkwork-router-portal-api-keys/src/pages/index.tsx');

  assert.match(apiKeysPage, /Environment strategy/);
  assert.match(apiKeysPage, /Rotation checklist/);
  assert.match(apiKeysPage, /Key handling guardrails/);
});

test('usage page exposes traffic and spend diagnosis surfaces', () => {
  const usagePage = read('packages/sdkwork-router-portal-usage/src/pages/index.tsx');

  assert.match(usagePage, /Traffic profile/);
  assert.match(usagePage, /Spend watch/);
  assert.match(usagePage, /Request diagnostics/);
});

test('account page exposes trust, security, and recovery guidance', () => {
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');

  assert.match(accountPage, /Workspace trust center/);
  assert.match(accountPage, /Security checklist/);
  assert.match(accountPage, /Recovery signals/);
});
