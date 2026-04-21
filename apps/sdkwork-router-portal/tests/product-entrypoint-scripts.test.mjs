import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const appRoot = path.resolve(import.meta.dirname, '..');
const workspaceRoot = path.resolve(appRoot, '..', '..');

test('portal package exposes product-grade server plan and integrated product checks', async () => {
  const packageJson = await import(pathToFileURL(path.join(appRoot, 'package.json')).href, {
    with: { type: 'json' },
  });

  assert.equal(
    packageJson.default.scripts['test:user-center-standard'],
    'node ../../scripts/run-user-center-standard.mjs',
  );
  assert.equal(
    packageJson.default.scripts['product:start'],
    'node ../../scripts/run-router-product.mjs',
  );
  assert.equal(
    packageJson.default.scripts['server:plan'],
    'node ../../scripts/run-router-product-service.mjs --dry-run --plan-format json',
  );
  assert.equal(
    packageJson.default.scripts['product:check'],
    'node ../../scripts/check-router-product.mjs',
  );
});

test('product check script plans portal and admin regression tests before build and server verification', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-router-product.mjs')).href
  );

  const plan = module.createProductCheckPlan({
    workspaceRoot,
    portalAppDir: appRoot,
    adminAppDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    platform: 'win32',
    env: {},
  });

  const labels = plan.map((step) => step.label);
  const stepByLabel = new Map(plan.map((step) => [step.label, step]));

  assert.deepEqual(labels, [
    'portal typecheck',
    'portal user-center standard',
    'portal regression tests',
    'portal browser runtime smoke',
    'admin typecheck',
    'admin regression tests',
    'admin browser runtime smoke',
    'desktop tauri capability audit',
    'browser storage governance audit',
    'product source tracking audit',
    'server development workspace smoke',
    'portal desktop runtime rust tests',
    'docs bootstrap safety',
    'docs site build',
    'workspace dependency audit',
    'portal desktop runtime payload',
    'server cargo check',
    'server deployment plan',
  ]);
  assert.equal(stepByLabel.get('portal typecheck')?.command, process.execPath);
  assert.match(stepByLabel.get('portal typecheck')?.args.join(' ') ?? '', /run-tsc-cli\.mjs --noEmit/);
  assert.deepEqual(
    stepByLabel.get('portal user-center standard')?.args,
    [path.join(workspaceRoot, 'scripts', 'run-user-center-standard.mjs')],
  );
  assert.deepEqual(stepByLabel.get('portal regression tests')?.args, ['--test', 'tests/*.mjs']);
  assert.match(stepByLabel.get('portal browser runtime smoke')?.args.join(' ') ?? '', /check-portal-browser-runtime\.mjs/);
  assert.equal(stepByLabel.get('admin typecheck')?.command, process.execPath);
  assert.match(stepByLabel.get('admin typecheck')?.args.join(' ') ?? '', /run-tsc-cli\.mjs --noEmit/);
  assert.deepEqual(stepByLabel.get('admin regression tests')?.args, ['--test', 'tests/*.mjs']);
  assert.match(stepByLabel.get('admin browser runtime smoke')?.args.join(' ') ?? '', /check-admin-browser-runtime\.mjs/);
  assert.match(
    stepByLabel.get('desktop tauri capability audit')?.args.join(' ') ?? '',
    /check-tauri-capabilities\.mjs/,
  );
  assert.match(
    stepByLabel.get('browser storage governance audit')?.args.join(' ') ?? '',
    /check-browser-storage-governance\.mjs/,
  );
  assert.match(
    stepByLabel.get('product source tracking audit')?.args.join(' ') ?? '',
    /check-product-source-tracking\.mjs/,
  );
  assert.match(
    stepByLabel.get('server development workspace smoke')?.args.join(' ') ?? '',
    /check-server-dev-workspace\.mjs/,
  );
  assert.match(
    stepByLabel.get('portal desktop runtime rust tests')?.args.join(' ') ?? '',
    /test --quiet --manifest-path .*apps[\\/]+sdkwork-router-portal[\\/]+src-tauri[\\/]+Cargo\.toml/,
  );
  assert.match(stepByLabel.get('docs bootstrap safety')?.args.join(' ') ?? '', /check-router-docs-safety\.mjs/);
  assert.equal(stepByLabel.get('docs site build')?.command, 'powershell.exe');
  assert.deepEqual(stepByLabel.get('docs site build')?.args.slice(0, 4), [
    '-NoProfile',
    '-ExecutionPolicy',
    'Bypass',
    '-Command',
  ]);
  assert.match(stepByLabel.get('docs site build')?.args[4] ?? '', /pnpm\.cjs/);
  assert.match(stepByLabel.get('docs site build')?.args[4] ?? '', /--dir/);
  assert.match(stepByLabel.get('docs site build')?.args[4] ?? '', /docs/);
  assert.match(stepByLabel.get('docs site build')?.args[4] ?? '', /build/);
  assert.match(stepByLabel.get('workspace dependency audit')?.args.join(' ') ?? '', /check-rust-dependency-audit\.mjs/);
  assert.match(stepByLabel.get('portal desktop runtime payload')?.args.join(' ') ?? '', /prepare-router-portal-desktop-runtime\.mjs/);
  assert.match(stepByLabel.get('server deployment plan')?.args.join(' ') ?? '', /--dry-run/);
  assert.match(stepByLabel.get('server deployment plan')?.args.join(' ') ?? '', /--plan-format json/);
  assert.match(stepByLabel.get('server deployment plan')?.args.join(' ') ?? '', /--bind 127\.0\.0\.1:3001/);
});
