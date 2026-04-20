import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');
const productGovernanceCatalog = await import(
  pathToFileURL(
    path.join(repoRoot, 'scripts', 'product-governance-node-test-catalog.mjs'),
  ).href,
);

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-product-governance-node-tests.mjs'),
    ).href
  );
}

test('product governance node test runner exposes the governed test set and canonical node plan', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listProductGovernanceNodeTests, 'function');
  assert.equal(typeof module.createProductGovernanceNodeTestPlan, 'function');

  const testFiles = module.listProductGovernanceNodeTests();
  assert.deepEqual(
    testFiles,
    productGovernanceCatalog.listProductGovernanceNodeTestFiles(),
  );

  const plan = module.createProductGovernanceNodeTestPlan({
    cwd: 'D:/workspace/router',
    env: { SDKWORK_STRICT_FRONTEND_INSTALLS: '1' },
    nodeExecutable: 'node-custom',
  });
  assert.equal(plan.command, 'node-custom');
  assert.deepEqual(
    plan.args,
    ['--test', '--experimental-test-isolation=none', ...testFiles],
  );
  assert.equal(plan.cwd, 'D:/workspace/router');
  assert.deepEqual(plan.env, { SDKWORK_STRICT_FRONTEND_INSTALLS: '1' });
  assert.equal(plan.shell, false);
  assert.equal(plan.windowsHide, process.platform === 'win32');
});

test('product governance node test runner imports the governed catalog as its single test-list source', async () => {
  const runnerSource = await readFile(
    path.join(repoRoot, 'scripts', 'run-product-governance-node-tests.mjs'),
    'utf8',
  );

  assert.match(runnerSource, /product-governance-node-test-catalog\.mjs/);
  assert.doesNotMatch(runnerSource, /export const PRODUCT_GOVERNANCE_NODE_TESTS\b/);
});

test('product governance node test runner executes the canonical node test command through spawnSync', async () => {
  const module = await loadModule();

  const calls = [];
  const result = module.runProductGovernanceNodeTests({
    cwd: 'D:/workspace/router',
    env: { SDKWORK_ENV: '1' },
    nodeExecutable: 'node-custom',
    spawnSyncImpl(command, args, options) {
      calls.push({ command, args, options });
      return {
        status: 0,
        stdout: '',
        stderr: '',
      };
    },
  });

  assert.equal(result.status, 0);
  assert.deepEqual(calls, [
    {
      command: 'node-custom',
      args: [
        '--test',
        '--experimental-test-isolation=none',
        ...module.listProductGovernanceNodeTests(),
      ],
      options: {
        cwd: 'D:/workspace/router',
        env: { SDKWORK_ENV: '1' },
        shell: false,
        stdio: 'inherit',
        windowsHide: process.platform === 'win32',
      },
    },
  ]);
});
