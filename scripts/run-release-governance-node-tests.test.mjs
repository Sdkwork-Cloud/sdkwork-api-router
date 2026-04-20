import assert from 'node:assert/strict';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'run-release-governance-node-tests.mjs'),
    ).href,
  );
}

async function loadCatalogModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release-governance-node-test-catalog.mjs'),
    ).href,
  );
}

test('release governance node test runner exposes the governed test set and canonical node plan', async () => {
  const module = await loadModule();
  const catalogModule = await loadCatalogModule();

  assert.equal(typeof module.listReleaseGovernanceNodeTests, 'function');
  assert.equal(typeof module.createReleaseGovernanceNodeTestPlan, 'function');

  const testFiles = module.listReleaseGovernanceNodeTests();
  assert.deepEqual(
    testFiles,
    catalogModule.listReleaseGovernanceNodeTestFiles(),
  );

  const plan = module.createReleaseGovernanceNodeTestPlan({
    cwd: 'D:/workspace/router',
    env: { SDKWORK_RELEASE_GOVERNANCE: '1' },
    nodeExecutable: 'node-custom',
  });
  assert.equal(plan.command, 'node-custom');
  assert.deepEqual(plan.args, ['--test', '--experimental-test-isolation=none', ...testFiles]);
  assert.equal(plan.cwd, 'D:/workspace/router');
  assert.deepEqual(plan.env, { SDKWORK_RELEASE_GOVERNANCE: '1' });
  assert.equal(plan.shell, false);
  assert.equal(plan.windowsHide, process.platform === 'win32');
});

test('release governance node test runner imports the governed catalog as its single test-list source', async () => {
  const runnerSource = await import('node:fs/promises').then(({ readFile }) => readFile(
    path.join(repoRoot, 'scripts', 'run-release-governance-node-tests.mjs'),
    'utf8',
  ));

  assert.match(runnerSource, /release-governance-node-test-catalog\.mjs/);
  assert.doesNotMatch(runnerSource, /export const RELEASE_GOVERNANCE_NODE_TESTS\s*=\s*\[/);
});

test('release governance node test runner executes the canonical node test command through spawnSync', async () => {
  const module = await loadModule();

  const calls = [];
  const result = module.runReleaseGovernanceNodeTests({
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
      args: ['--test', '--experimental-test-isolation=none', ...module.listReleaseGovernanceNodeTests()],
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
