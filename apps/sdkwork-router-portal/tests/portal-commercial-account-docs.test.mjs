import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const workspaceRoot = path.resolve(appRoot, '..', '..');

function readWorkspace(relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8');
}

test('portal API reference documents auto-provisioned commercial account routes and includes account-history', () => {
  const portalApiDoc = readWorkspace('docs/api-reference/portal-api.md');

  assert.match(portalApiDoc, /GET \/portal\/billing\/account-history/);
  assert.match(
    portalApiDoc,
    /portal commercial account routes auto-provision the workspace primary commercial account when it is missing/i,
  );
  assert.doesNotMatch(
    portalApiDoc,
    /portal commercial account routes return `404` when the workspace commercial account has not been provisioned yet/i,
  );
});
