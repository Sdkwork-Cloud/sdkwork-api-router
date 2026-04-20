import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-node-test-catalog.mjs'),
    ).href,
  );
}

test('release governance node test catalog publishes the exact governed test file set', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listReleaseGovernanceNodeTestFiles, 'function');
  assert.deepEqual(
    module.listReleaseGovernanceNodeTestFiles(),
    [
      'scripts/release-governance-node-test-catalog.test.mjs',
      'scripts/release-governance-workflow-step-contract-catalog.test.mjs',
      'scripts/release-governance-workflow-watch-catalog.test.mjs',
      'scripts/release-governance-workflow.test.mjs',
      'scripts/run-release-governance-node-tests.test.mjs',
      'scripts/strict-contract-catalog.test.mjs',
      'scripts/release/tests/installed-runtime-smoke-lib.test.mjs',
      'scripts/release/tests/release-cli-format-catalog.test.mjs',
      'scripts/release/tests/release-governance-plan-catalog.test.mjs',
      'scripts/release/tests/release-governance-runner.test.mjs',
      'scripts/release/tests/materialize-third-party-governance.test.mjs',
    ],
  );
});

test('release governance node test catalog exposes strict file lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findReleaseGovernanceNodeTestFile, 'function');
  assert.equal(typeof module.listReleaseGovernanceNodeTestFilesByPaths, 'function');

  assert.equal(
    module.findReleaseGovernanceNodeTestFile('scripts/strict-contract-catalog.test.mjs'),
    'scripts/strict-contract-catalog.test.mjs',
  );
  assert.deepEqual(
    module.listReleaseGovernanceNodeTestFilesByPaths([
      'scripts/strict-contract-catalog.test.mjs',
      'scripts/release/tests/release-governance-plan-catalog.test.mjs',
    ]),
    [
      'scripts/strict-contract-catalog.test.mjs',
      'scripts/release/tests/release-governance-plan-catalog.test.mjs',
    ],
  );
  assert.throws(
    () => module.findReleaseGovernanceNodeTestFile('scripts/missing-release-governance-node-test.test.mjs'),
    /missing release governance node test file.*missing-release-governance-node-test/i,
  );
});
