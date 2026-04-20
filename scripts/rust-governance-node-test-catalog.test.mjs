import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'rust-governance-node-test-catalog.mjs'),
    ).href,
  );
}

test('rust governance node test catalog publishes the exact governed test file set', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listRustGovernanceNodeTestFiles, 'function');
  assert.deepEqual(
    module.listRustGovernanceNodeTestFiles(),
    [
      'scripts/check-rust-dependency-audit.test.mjs',
      'scripts/check-rust-verification-matrix.test.mjs',
      'scripts/rust-governance-node-test-catalog.test.mjs',
      'scripts/rust-verification-workflow.test.mjs',
      'scripts/run-rust-governance-node-tests.test.mjs',
    ],
  );
});

test('rust governance node test catalog exposes strict file lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.findRustGovernanceNodeTestFile, 'function');
  assert.equal(typeof module.listRustGovernanceNodeTestFilesByPaths, 'function');

  assert.equal(
    module.findRustGovernanceNodeTestFile('scripts/check-rust-dependency-audit.test.mjs'),
    'scripts/check-rust-dependency-audit.test.mjs',
  );
  assert.deepEqual(
    module.listRustGovernanceNodeTestFilesByPaths([
      'scripts/check-rust-dependency-audit.test.mjs',
      'scripts/run-rust-governance-node-tests.test.mjs',
    ]),
    [
      'scripts/check-rust-dependency-audit.test.mjs',
      'scripts/run-rust-governance-node-tests.test.mjs',
    ],
  );
  assert.throws(
    () => module.findRustGovernanceNodeTestFile('scripts/missing-rust-governance-node-test.test.mjs'),
    /missing rust governance node test file.*missing-rust-governance-node-test/i,
  );
});
