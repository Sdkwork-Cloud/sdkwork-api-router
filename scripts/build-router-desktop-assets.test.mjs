import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('desktop asset build plan uses the shared hidden Windows pnpm launcher', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'build-router-desktop-assets.mjs')).href
  );

  const plans = module.createDesktopAssetBuildPlan({
    workspaceRoot,
    platform: 'win32',
  });

  assert.equal(plans.length, 2);
  assert.equal(plans[0].command, 'powershell.exe');
  assert.match(plans[0].args.join(' '), /pnpm\.cjs/);
  assert.match(plans[0].args.join(' '), /build/);
  assert.equal(plans[0].shell, false);
  assert.equal(plans[1].command, 'powershell.exe');
  assert.match(plans[1].args.join(' '), /pnpm\.cjs/);
  assert.match(plans[1].args.join(' '), /build/);
  assert.equal(plans[1].shell, false);
});
