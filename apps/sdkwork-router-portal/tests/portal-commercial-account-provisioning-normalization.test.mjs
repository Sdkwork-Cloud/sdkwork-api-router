import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('commercial account provisioning normalization matches semantic markers instead of exact English sentences', () => {
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');

  assert.match(
    portalApi,
    /normalizedMessage[\s\S]*commercial account[\s\S]*not provisioned/,
  );
  assert.match(
    portalApi,
    /normalizedMessage[\s\S]*commercial account[\s\S]*being prepared/,
  );
  assert.doesNotMatch(
    portalApi,
    /normalizedMessage === 'workspace commercial account is not provisioned'/,
  );
  assert.doesNotMatch(
    portalApi,
    /normalizedMessage === 'commercial account is not provisioned'/,
  );
});
