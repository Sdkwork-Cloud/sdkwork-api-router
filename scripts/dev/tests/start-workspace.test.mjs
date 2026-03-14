import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildWorkspaceCommandPlan,
  parseWorkspaceArgs,
} from '../workspace-launch-lib.mjs';

test('parseWorkspaceArgs returns browser-mode defaults', () => {
  const settings = parseWorkspaceArgs([]);

  assert.deepEqual(settings, {
    databaseUrl: 'sqlite://sdkwork-api-server.db',
    gatewayBind: '127.0.0.1:8080',
    adminBind: '127.0.0.1:8081',
    portalBind: '127.0.0.1:8082',
    install: false,
    preview: false,
    tauri: false,
    dryRun: false,
    help: false,
  });
});

test('parseWorkspaceArgs forwards install, preview, tauri, and bind overrides', () => {
  const settings = parseWorkspaceArgs([
    '--database-url',
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
    '--gateway-bind',
    '0.0.0.0:18080',
    '--admin-bind',
    '0.0.0.0:18081',
    '--portal-bind',
    '0.0.0.0:18082',
    '--install',
    '--preview',
    '--tauri',
    '--dry-run',
  ]);

  assert.equal(
    settings.databaseUrl,
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
  );
  assert.equal(settings.gatewayBind, '0.0.0.0:18080');
  assert.equal(settings.adminBind, '0.0.0.0:18081');
  assert.equal(settings.portalBind, '0.0.0.0:18082');
  assert.equal(settings.install, true);
  assert.equal(settings.preview, true);
  assert.equal(settings.tauri, true);
  assert.equal(settings.dryRun, true);
});

test('buildWorkspaceCommandPlan forwards backend and console flags to child scripts', () => {
  const plan = buildWorkspaceCommandPlan({
    databaseUrl: 'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
    gatewayBind: '0.0.0.0:18080',
    adminBind: '0.0.0.0:18081',
    portalBind: '0.0.0.0:18082',
    install: true,
    preview: false,
    tauri: true,
    dryRun: true,
    help: false,
  });

  assert.equal(plan.backend.scriptPath, 'scripts/dev/start-stack.mjs');
  assert.deepEqual(plan.backend.args, [
    'scripts/dev/start-stack.mjs',
    '--database-url',
    'postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_server',
    '--gateway-bind',
    '0.0.0.0:18080',
    '--admin-bind',
    '0.0.0.0:18081',
    '--portal-bind',
    '0.0.0.0:18082',
    '--dry-run',
  ]);

  assert.equal(plan.console.scriptPath, 'scripts/dev/start-console.mjs');
  assert.deepEqual(plan.console.args, [
    'scripts/dev/start-console.mjs',
    '--install',
    '--tauri',
    '--dry-run',
  ]);
});

test('parseWorkspaceArgs rejects missing values and unknown flags', () => {
  assert.throws(() => parseWorkspaceArgs(['--database-url']), {
    message: /requires a value/,
  });
  assert.throws(() => parseWorkspaceArgs(['--unknown-flag']), {
    message: /unknown option/,
  });
});
