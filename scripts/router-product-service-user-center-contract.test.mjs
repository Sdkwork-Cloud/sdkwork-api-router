import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const repoRoot = path.resolve(import.meta.dirname, '..');
const serviceEntryPath = path.join(
  repoRoot,
  'services',
  'router-product-service',
  'src',
  'main.rs',
);

function readServiceEntry() {
  return readFileSync(serviceEntryPath, 'utf8');
}

function toRustClapLongFlagPattern(flagName) {
  return new RegExp(`long\\s*=\\s*"${flagName.slice(2)}"`, 'u');
}

test('router product service publishes the canonical user-center env, mode, and auth vocabulary for server deployments', () => {
  assert.equal(existsSync(serviceEntryPath), true, 'missing router product service entry');

  const source = readServiceEntry();

  for (const envName of [
    'SDKWORK_USER_CENTER_MODE',
    'SDKWORK_USER_CENTER_APP_API_BASE_URL',
    'SDKWORK_USER_CENTER_EXTERNAL_BASE_URL',
    'SDKWORK_USER_CENTER_PROVIDER_KEY',
    'SDKWORK_USER_CENTER_APP_ID',
    'SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH',
    'SDKWORK_USER_CENTER_SQLITE_PATH',
    'SDKWORK_USER_CENTER_DATABASE_URL',
    'SDKWORK_USER_CENTER_SCHEMA_NAME',
    'SDKWORK_USER_CENTER_TABLE_PREFIX',
    'SDKWORK_USER_CENTER_SECRET_ID',
    'SDKWORK_USER_CENTER_SHARED_SECRET',
    'SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS',
    'SDKWORK_USER_CENTER_AUTHORIZATION_HEADER_NAME',
    'SDKWORK_USER_CENTER_ACCESS_TOKEN_HEADER_NAME',
    'SDKWORK_USER_CENTER_REFRESH_TOKEN_HEADER_NAME',
    'SDKWORK_USER_CENTER_SESSION_HEADER_NAME',
    'SDKWORK_USER_CENTER_AUTHORIZATION_SCHEME',
    'SDKWORK_USER_CENTER_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN',
  ]) {
    assert.match(source, new RegExp(envName, 'u'));
  }

  for (const flagName of [
    '--user-center-mode',
    '--user-center-app-api-base-url',
    '--user-center-external-base-url',
    '--user-center-provider-key',
    '--user-center-app-id',
    '--user-center-local-api-base-path',
    '--user-center-sqlite-path',
    '--user-center-database-url',
    '--user-center-schema-name',
    '--user-center-table-prefix',
    '--user-center-secret-id',
    '--user-center-shared-secret',
    '--user-center-handshake-freshness-window-ms',
    '--user-center-authorization-header-name',
    '--user-center-access-token-header-name',
    '--user-center-refresh-token-header-name',
    '--user-center-session-header-name',
    '--user-center-authorization-scheme',
    '--user-center-allow-authorization-fallback-to-access-token',
  ]) {
    assert.equal(
      source.includes(flagName) || toRustClapLongFlagPattern(flagName).test(source),
      true,
      `missing CLI flag ${flagName}`,
    );
  }

  assert.match(source, /builtin-local/u);
  assert.match(source, /sdkwork-cloud-app-api/u);
  assert.match(source, /external-user-center/u);
  assert.match(source, /provider-shared-secret/u);
  assert.match(source, /Authorization/u);
  assert.match(source, /Access-Token/u);
  assert.match(source, /Refresh-Token/u);
  assert.match(source, /x-sdkwork-user-center-session-id/u);
});

test('router product service fails closed for under-configured upstream user-center modes and exposes the normalized contract in dry-run plans', () => {
  const source = readServiceEntry();

  for (const envName of [
    'SDKWORK_USER_CENTER_SHARED_SECRET',
    'SDKWORK_USER_CENTER_SECRET_ID',
    'SDKWORK_USER_CENTER_PROVIDER_KEY',
    'SDKWORK_USER_CENTER_APP_API_BASE_URL',
    'SDKWORK_USER_CENTER_EXTERNAL_BASE_URL',
  ]) {
    assert.match(source, new RegExp(envName, 'u'));
  }
  assert.match(source, /is required for the selected user-center mode/u);

  assert.match(source, /user_center/u);
  assert.match(source, /auth_token_header_name/u);
  assert.match(source, /access_token_header_name/u);
  assert.match(source, /refresh_token_header_name/u);
  assert.match(source, /session_header_name/u);
  assert.match(source, /allow_authorization_fallback_to_access_token/u);
  assert.match(source, /handshake_freshness_window_ms/u);
});

