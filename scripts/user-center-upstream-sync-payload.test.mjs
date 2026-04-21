import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'user-center-upstream-sync-payload.mjs'),
    ).href,
  );
}

test('router upstream sync payload validator accepts canonical sdkwork-appbase dispatch payloads', async () => {
  const module = await loadModule();

  assert.equal(typeof module.assertUserCenterUpstreamSyncPayload, 'function');

  assert.doesNotThrow(() =>
    module.assertUserCenterUpstreamSyncPayload({
      source_ref: 'refs/heads/main',
      source_repository: 'Sdkwork-Cloud/sdkwork-appbase',
      source_sha: 'abc123',
      workflow: 'user-center-upstream-sync',
    }),
  );
});

test('router upstream sync payload validator rejects foreign source repositories', async () => {
  const module = await loadModule();

  assert.throws(
    () =>
      module.assertUserCenterUpstreamSyncPayload({
        source_ref: 'refs/heads/main',
        source_repository: 'Sdkwork-Cloud/rogue-source',
        source_sha: 'abc123',
        workflow: 'user-center-upstream-sync',
      }),
    /sdkwork-appbase|source repository/i,
  );
});
