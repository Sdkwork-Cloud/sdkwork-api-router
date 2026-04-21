import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('root workspace package exposes packaged desktop and server dev entrypoints', () => {
  const rootPackage = JSON.parse(
    readFileSync(path.join(workspaceRoot, 'package.json'), 'utf8'),
  );

  assert.equal(rootPackage.private, true);
  assert.equal(rootPackage.packageManager, 'pnpm@10.30.2');
  assert.equal(
    rootPackage.scripts['product:check'],
    'node scripts/run-router-product.mjs check',
  );
  assert.equal(
    rootPackage.scripts['test:user-center-standard'],
    'node scripts/run-user-center-standard.mjs',
  );
  assert.equal(
    rootPackage.scripts['tauri:dev'],
    'node scripts/run-router-product.mjs desktop',
  );
  assert.equal(
    rootPackage.scripts['server:dev'],
    'node scripts/run-router-product.mjs server',
  );
});

test('router product launcher preserves forwarded mode arguments after --', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const parsed = module.parseRouterProductArgs(['server', '--', '--help']);

  assert.equal(parsed.mode, 'server');
  assert.equal(parsed.help, false);
  assert.deepEqual(parsed.extraArgs, ['--help']);
});

test('router product launcher forwards mode-specific help to the selected runtime instead of printing root help', () => {
  const result = spawnSync(
    process.execPath,
    [path.join(workspaceRoot, 'scripts', 'run-router-product.mjs'), 'server', '--help'],
    {
      cwd: workspaceRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(result.status, 0);
  assert.match(result.stdout, /Usage: node scripts\/dev\/start-workspace\.mjs \[options\]/);
  assert.doesNotMatch(result.stdout, /Usage: node scripts\/run-router-product\.mjs/);
  assert.doesNotMatch(result.stderr, /\[run-router-product\]/);
});

test('router product server dry-run expands the nested workspace launch plan', () => {
  const result = spawnSync(
    process.execPath,
    [path.join(workspaceRoot, 'scripts', 'run-router-product.mjs'), 'server', '--dry-run'],
    {
      cwd: workspaceRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(result.status, 0);
  assert.match(result.stderr, /\[run-router-product\].*scripts[\\/]+dev[\\/]+start-workspace\.mjs --proxy-dev/);
  assert.match(result.stdout, /\[start-workspace\] unified launch settings/);
  assert.match(result.stdout, /\[start-workspace\] backend:/);
  assert.match(result.stdout, /\[start-workspace\] admin-browser:/);
  assert.match(result.stdout, /\[start-workspace\] portal-browser:/);
  assert.match(result.stdout, /\[start-workspace\] web-proxy-dev:/);
});

test('router product launcher still prints root help when no mode is selected', () => {
  const result = spawnSync(
    process.execPath,
    [path.join(workspaceRoot, 'scripts', 'run-router-product.mjs'), '--help'],
    {
      cwd: workspaceRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(result.status, 0);
  assert.match(result.stdout, /Usage: node scripts\/run-router-product\.mjs \[mode\] \[options\] \[mode-args\.\.\.\]/);
});

test('router product launcher defaults to desktop mode and installs dependencies when requested', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'desktop',
    install: true,
    platform: 'win32',
    env: {},
    extraArgs: [],
  });

  assert.equal(plan.length, 2);
  assert.equal(plan[0].label, 'portal install');
  assert.deepEqual(plan[0].args, ['--dir', 'apps/sdkwork-router-portal', 'install']);
  assert.equal(plan[0].command, 'pnpm.cmd');
  assert.equal(plan[0].shell, true);
  assert.equal(plan[0].windowsHide, true);
  assert.equal(plan[1].label, 'portal desktop runtime');
  assert.equal(plan[1].command, process.execPath);
  assert.equal(plan[1].cwd, path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'));
  assert.deepEqual(plan[1].args, [
    path.join(workspaceRoot, 'scripts', 'run-tauri-cli.mjs'),
    'dev',
  ]);
  assert.equal(plan[1].shell, false);
  assert.equal(plan[1].windowsHide, true);
});

test('router product launcher forwards workspace arguments into server mode', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'server',
    install: false,
    platform: 'linux',
    env: {},
    extraArgs: [
      '--database-url',
      'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router',
      '--gateway-bind',
      '0.0.0.0:9980',
      '--web-bind',
      '127.0.0.1:9983',
    ],
  });

  assert.equal(plan.length, 1);
  assert.equal(plan[0].label, 'server development workspace');
  assert.equal(plan[0].command, process.execPath);
  assert.deepEqual(plan[0].args, [
    path.join(workspaceRoot, 'scripts', 'dev', 'start-workspace.mjs'),
    '--proxy-dev',
    '--database-url',
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router',
    '--gateway-bind',
    '0.0.0.0:9980',
    '--web-bind',
    '127.0.0.1:9983',
  ]);
  assert.equal(plan[0].shell, false);
  assert.equal(plan[0].windowsHide, false);
});

test('router product launcher installs admin and portal dependencies before server workspace mode when requested', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'server',
    install: true,
    platform: 'linux',
    env: {},
    extraArgs: [],
  });

  assert.equal(plan.length, 3);
  assert.equal(plan[0].label, 'admin install');
  assert.equal(plan[0].command, 'pnpm');
  assert.deepEqual(plan[0].args, ['--dir', 'apps/sdkwork-router-admin', 'install']);
  assert.equal(plan[1].label, 'portal install');
  assert.equal(plan[1].command, 'pnpm');
  assert.deepEqual(plan[1].args, ['--dir', 'apps/sdkwork-router-portal', 'install']);
  assert.equal(plan[2].label, 'server development workspace');
  assert.equal(plan[2].command, process.execPath);
  assert.deepEqual(plan[2].args, [
    path.join(workspaceRoot, 'scripts', 'dev', 'start-workspace.mjs'),
    '--proxy-dev',
  ]);
});

test('router product launcher enables hidden tray service mode for desktop startup', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'service',
    install: false,
    platform: 'linux',
    env: {},
    extraArgs: [],
  });

  assert.equal(plan.length, 1);
  assert.equal(plan[0].label, 'portal service runtime');
  assert.equal(plan[0].command, process.execPath);
  assert.equal(plan[0].cwd, path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'));
  assert.equal(plan[0].env.SDKWORK_ROUTER_BACKGROUND, '1');
  assert.equal(plan[0].env.SDKWORK_ROUTER_PORTAL_START_HIDDEN, '1');
  assert.equal(plan[0].env.SDKWORK_ROUTER_SERVICE_MODE, '1');
  assert.deepEqual(plan[0].args, [
    path.join(workspaceRoot, 'scripts', 'run-tauri-cli.mjs'),
    'dev',
  ]);
  assert.equal(plan[0].shell, false);
  assert.equal(plan[0].windowsHide, false);
});

test('router product launcher forwards desktop mode arguments directly into the shared tauri wrapper', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'desktop',
    install: false,
    platform: 'linux',
    env: {},
    extraArgs: ['--help'],
  });

  assert.equal(plan.length, 1);
  assert.equal(plan[0].command, process.execPath);
  assert.equal(plan[0].cwd, path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'));
  assert.deepEqual(plan[0].args, [
    path.join(workspaceRoot, 'scripts', 'run-tauri-cli.mjs'),
    'dev',
    '--help',
  ]);
  assert.equal(plan[0].shell, false);
});

test('router product launcher exposes machine-readable plan mode through the unified entrypoint', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'plan',
    install: false,
    platform: 'linux',
    env: {},
    extraArgs: ['--roles', 'web'],
  });

  assert.equal(plan.length, 1);
  assert.equal(plan[0].label, 'portal deployment plan');
  assert.equal(plan[0].command, process.execPath);
  assert.deepEqual(plan[0].args, [
    path.join(workspaceRoot, 'scripts', 'run-router-product-service.mjs'),
    '--dry-run',
    '--plan-format',
    'json',
    '--roles',
    'web',
  ]);
});

test('router product launcher exposes product check mode through the unified root entrypoint', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'run-router-product.mjs')).href
  );

  const plan = module.createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'check',
    install: false,
    platform: 'linux',
    env: {},
    extraArgs: [],
  });

  assert.equal(plan.length, 1);
  assert.equal(plan[0].label, 'portal product check');
  assert.equal(plan[0].command, 'pnpm');
  assert.deepEqual(plan[0].args, [
    '--dir',
    'apps/sdkwork-router-portal',
    'product:check',
  ]);
  assert.equal(plan[0].cwd, workspaceRoot);
  assert.equal(plan[0].shell, false);
  assert.equal(plan[0].windowsHide, false);
});
