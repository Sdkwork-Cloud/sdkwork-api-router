import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import test from 'node:test';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('start-servers.ps1 exposes dry-run launch plans for all backend services', { skip: process.platform !== 'win32' }, () => {
  const result = spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-ExecutionPolicy',
      'Bypass',
      '-File',
      path.join(repoRoot, 'scripts', 'dev', 'start-servers.ps1'),
      '-DryRun',
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /\[start-servers\] SDKWORK_DATABASE_URL=\(local default via config loader\)/);
  assert.match(result.stdout, /\[start-servers\] SDKWORK_ADMIN_BIND=127\.0\.0\.1:9981/);
  assert.match(result.stdout, /\[start-servers\] SDKWORK_GATEWAY_BIND=127\.0\.0\.1:9980/);
  assert.match(result.stdout, /\[start-servers\] SDKWORK_PORTAL_BIND=127\.0\.0\.1:9982/);
  assert.match(result.stdout, /\[start-servers\] SDKWORK_CONFIG_FILE=.*router\.yaml/);
  assert.match(result.stdout, /<window 'sdkwork admin-api-service' running cargo run -p admin-api-service>/);
  assert.match(result.stdout, /<window 'sdkwork gateway-service' running cargo run -p gateway-service>/);
  assert.match(result.stdout, /<window 'sdkwork portal-api-service' running cargo run -p portal-api-service>/);
});
