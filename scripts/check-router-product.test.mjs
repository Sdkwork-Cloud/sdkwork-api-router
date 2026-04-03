import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('check-router-product exposes Windows-safe pnpm and rust runner plans without ambient globals', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-product.mjs')).href,
  );

  const rustRunner = module.resolveRustRunner('win32', {
    USERPROFILE: process.env.USERPROFILE ?? '',
  });
  assert.equal(typeof rustRunner.command, 'string');
  assert.ok(Array.isArray(rustRunner.args));

  const plan = module.createProductCheckPlan({
    workspaceRoot,
    platform: 'win32',
    env: {},
  });

  assert.equal(plan[0].label, 'portal typecheck');
  assert.equal(plan[0].command, 'powershell.exe');
  assert.match(plan[0].args.join(' '), /typecheck/);
  assert.equal(plan[2].label, 'admin typecheck');
  assert.equal(plan[2].command, 'powershell.exe');
  assert.match(plan[2].args.join(' '), /typecheck/);
});
