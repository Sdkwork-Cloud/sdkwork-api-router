import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

const runtimeToolingPath = path.join(repoRoot, 'bin', 'lib', 'router-runtime-tooling.mjs');
const startShPath = path.join(repoRoot, 'bin', 'start.sh');
const startPs1Path = path.join(repoRoot, 'bin', 'start.ps1');
const dockerEnvExamplePath = path.join(repoRoot, 'deploy', 'docker', '.env.example');
const dockerComposePath = path.join(repoRoot, 'deploy', 'docker', 'docker-compose.yml');
const helmValuesPath = path.join(repoRoot, 'deploy', 'helm', 'sdkwork-api-router', 'values.yaml');
const helmDeploymentPath = path.join(
  repoRoot,
  'deploy',
  'helm',
  'sdkwork-api-router',
  'templates',
  'deployment.yaml',
);
const helmSecretPath = path.join(
  repoRoot,
  'deploy',
  'helm',
  'sdkwork-api-router',
  'templates',
  'secret.yaml',
);

const runtimeToolingSource = readFileSync(runtimeToolingPath, 'utf8');
const startShSource = readFileSync(startShPath, 'utf8');
const startPs1Source = readFileSync(startPs1Path, 'utf8');
const dockerEnvExampleSource = readFileSync(dockerEnvExamplePath, 'utf8');
const dockerComposeSource = readFileSync(dockerComposePath, 'utf8');
const helmValuesSource = readFileSync(helmValuesPath, 'utf8');
const helmDeploymentSource = readFileSync(helmDeploymentPath, 'utf8');
const helmSecretSource = readFileSync(helmSecretPath, 'utf8');

async function loadRuntimeToolingModule() {
  return import(pathToFileURL(runtimeToolingPath).href);
}

test('sdkwork-api-router runtime env template publishes the canonical user-center vocabulary for native service deployments', async () => {
  const runtimeTooling = await loadRuntimeToolingModule();
  const renderedEnvTemplate = runtimeTooling.renderRuntimeEnvTemplate({
    installRoot: '/opt/sdkwork-api-router',
    mode: 'system',
    platform: 'linux',
  });

  for (const pattern of [
    /SDKWORK_USER_CENTER_MODE=/u,
    /SDKWORK_USER_CENTER_PROVIDER_KEY=/u,
    /SDKWORK_USER_CENTER_APP_ID=/u,
    /SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH=/u,
    /SDKWORK_USER_CENTER_SQLITE_PATH=/u,
    /SDKWORK_USER_CENTER_DATABASE_URL=/u,
    /SDKWORK_USER_CENTER_SCHEMA_NAME=/u,
    /SDKWORK_USER_CENTER_TABLE_PREFIX=/u,
    /SDKWORK_USER_CENTER_APP_API_BASE_URL=/u,
    /SDKWORK_USER_CENTER_EXTERNAL_BASE_URL=/u,
    /SDKWORK_USER_CENTER_SECRET_ID=/u,
    /SDKWORK_USER_CENTER_SHARED_SECRET=/u,
    /SDKWORK_USER_CENTER_AUTHORIZATION_HEADER_NAME="?Authorization"?/u,
    /SDKWORK_USER_CENTER_ACCESS_TOKEN_HEADER_NAME="?Access-Token"?/u,
    /SDKWORK_USER_CENTER_REFRESH_TOKEN_HEADER_NAME="?Refresh-Token"?/u,
    /SDKWORK_USER_CENTER_SESSION_HEADER_NAME="?x-sdkwork-user-center-session-id"?/u,
    /SDKWORK_USER_CENTER_AUTHORIZATION_SCHEME="?Bearer"?/u,
    /SDKWORK_USER_CENTER_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN="?true"?/u,
    /SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS="?30000"?/u,
  ]) {
    assert.match(renderedEnvTemplate, pattern);
  }
});

test('sdkwork-api-router startup entrypoints load router.env so service-managed native installs inherit the canonical user-center env contract', () => {
  for (const [label, source] of [
    ['shell', startShSource],
    ['powershell', startPs1Source],
  ]) {
    assert.match(
      source,
      /router\.env/u,
      `${label} start entrypoint must load router.env before starting the server runtime.`,
    );
    assert.match(
      source,
      /router_load_env_file|Import-RouterEnvFile/u,
      `${label} start entrypoint must source the runtime env loader instead of bypassing router.env.`,
    );
  }
});

test('sdkwork-api-router container deployment assets expose the canonical user-center configuration and secret contract', () => {
  for (const pattern of [
    /SDKWORK_USER_CENTER_MODE=/u,
    /SDKWORK_USER_CENTER_PROVIDER_KEY=/u,
    /SDKWORK_USER_CENTER_APP_ID=/u,
    /SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH=/u,
    /SDKWORK_USER_CENTER_SQLITE_PATH=/u,
    /SDKWORK_USER_CENTER_DATABASE_URL=/u,
    /SDKWORK_USER_CENTER_SCHEMA_NAME=/u,
    /SDKWORK_USER_CENTER_TABLE_PREFIX=/u,
    /SDKWORK_USER_CENTER_APP_API_BASE_URL=/u,
    /SDKWORK_USER_CENTER_EXTERNAL_BASE_URL=/u,
    /SDKWORK_USER_CENTER_SECRET_ID=/u,
    /SDKWORK_USER_CENTER_SHARED_SECRET=/u,
    /SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS=/u,
  ]) {
    assert.match(
      dockerEnvExampleSource,
      pattern,
      `.env.example must document ${pattern} for Docker and Compose deployments.`,
    );
  }

  for (const pattern of [
    /SDKWORK_USER_CENTER_MODE:/u,
    /SDKWORK_USER_CENTER_PROVIDER_KEY:/u,
    /SDKWORK_USER_CENTER_APP_ID:/u,
    /SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH:/u,
    /SDKWORK_USER_CENTER_SQLITE_PATH:/u,
    /SDKWORK_USER_CENTER_SCHEMA_NAME:/u,
    /SDKWORK_USER_CENTER_TABLE_PREFIX:/u,
    /SDKWORK_USER_CENTER_APP_API_BASE_URL:/u,
    /SDKWORK_USER_CENTER_EXTERNAL_BASE_URL:/u,
    /SDKWORK_USER_CENTER_SECRET_ID:/u,
    /SDKWORK_USER_CENTER_AUTHORIZATION_HEADER_NAME:/u,
    /SDKWORK_USER_CENTER_ACCESS_TOKEN_HEADER_NAME:/u,
    /SDKWORK_USER_CENTER_REFRESH_TOKEN_HEADER_NAME:/u,
    /SDKWORK_USER_CENTER_SESSION_HEADER_NAME:/u,
    /SDKWORK_USER_CENTER_AUTHORIZATION_SCHEME:/u,
    /SDKWORK_USER_CENTER_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN:/u,
    /SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS:/u,
  ]) {
    assert.match(
      dockerComposeSource,
      pattern,
      `docker-compose must export ${pattern} into the product runtime.`,
    );
  }

  for (const pattern of [
    /^userCenter:/mu,
    /mode:\s*builtin-local/u,
    /providerKey:\s*sdkwork-api-router-local/u,
    /localApiBasePath:\s*\/api\/app\/v1\/user-center/u,
    /sqlitePath:\s*\/opt\/sdkwork\/data\/user-center\.db/u,
    /tablePrefix:\s*rp_uc_/u,
    /authorizationHeaderName:\s*Authorization/u,
    /accessTokenHeaderName:\s*Access-Token/u,
    /refreshTokenHeaderName:\s*Refresh-Token/u,
    /sessionHeaderName:\s*x-sdkwork-user-center-session-id/u,
    /authorizationScheme:\s*Bearer/u,
    /allowAuthorizationFallbackToAccessToken:\s*true/u,
    /handshakeFreshnessWindowMs:\s*30000/u,
  ]) {
    assert.match(
      helmValuesSource,
      pattern,
      `Helm values must define ${pattern} for user-center mode switching.`,
    );
  }

  for (const pattern of [
    /SDKWORK_USER_CENTER_MODE/u,
    /SDKWORK_USER_CENTER_PROVIDER_KEY/u,
    /SDKWORK_USER_CENTER_APP_ID/u,
    /SDKWORK_USER_CENTER_LOCAL_API_BASE_PATH/u,
    /SDKWORK_USER_CENTER_SQLITE_PATH/u,
    /SDKWORK_USER_CENTER_SCHEMA_NAME/u,
    /SDKWORK_USER_CENTER_TABLE_PREFIX/u,
    /SDKWORK_USER_CENTER_APP_API_BASE_URL/u,
    /SDKWORK_USER_CENTER_EXTERNAL_BASE_URL/u,
    /SDKWORK_USER_CENTER_SECRET_ID/u,
    /SDKWORK_USER_CENTER_AUTHORIZATION_HEADER_NAME/u,
    /SDKWORK_USER_CENTER_ACCESS_TOKEN_HEADER_NAME/u,
    /SDKWORK_USER_CENTER_REFRESH_TOKEN_HEADER_NAME/u,
    /SDKWORK_USER_CENTER_SESSION_HEADER_NAME/u,
    /SDKWORK_USER_CENTER_AUTHORIZATION_SCHEME/u,
    /SDKWORK_USER_CENTER_ALLOW_AUTHORIZATION_FALLBACK_TO_ACCESS_TOKEN/u,
    /SDKWORK_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS/u,
    /SDKWORK_USER_CENTER_DATABASE_URL/u,
    /SDKWORK_USER_CENTER_SHARED_SECRET/u,
  ]) {
    assert.match(
      helmDeploymentSource,
      pattern,
      `Helm deployment template must project ${pattern} into the runtime container.`,
    );
  }

  for (const pattern of [
    /SDKWORK_USER_CENTER_DATABASE_URL/u,
    /SDKWORK_USER_CENTER_SHARED_SECRET/u,
  ]) {
    assert.match(
      helmSecretSource,
      pattern,
      `Helm secret template must carry optional secret material for ${pattern}.`,
    );
  }
});
