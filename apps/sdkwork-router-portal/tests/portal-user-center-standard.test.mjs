import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

import { registerPortalNodeTsWorkspaceHooks } from './helpers/node-ts-workspace-hooks.mjs';
import {
  resolvePortalAppRoot,
  resolvePortalAppbaseRoot,
} from './helpers/portal-paths.mjs';

const appRoot = resolvePortalAppRoot(import.meta.url);
const appbaseRoot = resolvePortalAppbaseRoot(import.meta.url);

registerPortalNodeTsWorkspaceHooks({
  appRoot,
  appbaseRoot,
});

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function readAppbase(relativePath) {
  return readFileSync(path.join(appbaseRoot, relativePath), 'utf8');
}

function collectSourceFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const resolvedPath = path.join(directory, entry.name);

    if (entry.isDirectory()) {
      return collectSourceFiles(resolvedPath);
    }

    return entry.isFile() && /\.(?:[cm]?js|tsx?)$/u.test(entry.name) ? [resolvedPath] : [];
  });
}

function normalizeTestPath(filePath) {
  return filePath.split(path.sep).join('/');
}

async function loadPortalApi() {
  return import(
    pathToFileURL(
      path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-portal-api',
        'src',
        'index.ts',
      ),
    ).href,
  );
}

async function loadPortalUserCenterBridge() {
  return import(
    pathToFileURL(
      path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-types',
        'src',
        'userCenter.ts',
      ),
    ).href,
  );
}

async function loadPortalValidationBridge() {
  return import(
    pathToFileURL(
      path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-types',
        'src',
        'validation.ts',
      ),
    ).href,
  );
}

function storageDouble() {
  const store = new Map();

  return {
    getItem(key) {
      return store.has(key) ? store.get(key) : null;
    },
    setItem(key, value) {
      store.set(key, String(value));
    },
    removeItem(key) {
      store.delete(key);
    },
  };
}

function throwingStorageDouble() {
  return {
    getItem() {
      throw new Error('storage unavailable');
    },
    setItem() {
      throw new Error('storage unavailable');
    },
    removeItem() {
      throw new Error('storage unavailable');
    },
  };
}

const DEFAULT_AUTH_TOKEN_HEADERS = {
  accessTokenHeaderName: 'Access-Token',
  authorizationHeaderName: 'Authorization',
  authorizationScheme: 'Bearer',
  refreshTokenHeaderName: 'Refresh-Token',
  sessionHeaderName: 'x-sdkwork-user-center-session-id',
};

const DEFAULT_AUTH_CACHE_POLICY = {
  bundleMemoryCache: true,
  secretResolutionTtlMs: 300000,
  unverifiedClaimsTtlMs: 30000,
  verifiedTokenTtlMs: 30000,
};

const DEFAULT_HANDSHAKE_HEADER_NAMES = {
  appIdHeaderName: 'x-sdkwork-app-id',
  providerKeyHeaderName: 'x-sdkwork-user-center-provider-key',
  secretIdHeaderName: 'x-sdkwork-user-center-secret-id',
  signatureHeaderName: 'x-sdkwork-user-center-signature',
  signedAtHeaderName: 'x-sdkwork-user-center-signed-at',
};
const DEFAULT_HANDSHAKE_FRESHNESS_WINDOW_MS = 30000;

function createLocalAuthContract() {
  return {
    allowAuthorizationFallbackToAccessToken: true,
    cachePolicy: DEFAULT_AUTH_CACHE_POLICY,
    handshake: {
      enabled: false,
      freshnessWindowMs: DEFAULT_HANDSHAKE_FRESHNESS_WINDOW_MS,
      headerNames: DEFAULT_HANDSHAKE_HEADER_NAMES,
      mode: 'disabled',
      staticHeaders: {},
    },
    mode: 'auth-access-token',
    secretResolution: {
      organizationClaimKey: 'organizationId',
      resolverKind: 'local-static',
      scope: 'organization-preferred',
      tenantClaimKey: 'tenantId',
    },
    tokenHeaders: DEFAULT_AUTH_TOKEN_HEADERS,
    validationStrategy: 'auth-access-token',
  };
}

function createUpstreamAuthContract(appId, providerKey) {
  return {
    allowAuthorizationFallbackToAccessToken: true,
    cachePolicy: DEFAULT_AUTH_CACHE_POLICY,
    handshake: {
      enabled: true,
      freshnessWindowMs: DEFAULT_HANDSHAKE_FRESHNESS_WINDOW_MS,
      headerNames: DEFAULT_HANDSHAKE_HEADER_NAMES,
      mode: 'provider-shared-secret',
      staticHeaders: {
        'x-sdkwork-app-id': appId,
        'x-sdkwork-user-center-handshake-mode': 'provider-shared-secret',
        'x-sdkwork-user-center-provider-key': providerKey,
      },
    },
    mode: 'upstream-app-api-token-bridge',
    secretResolution: {
      organizationClaimKey: 'organizationId',
      resolverKind: 'upstream-secret-bridge',
      scope: 'organization-preferred',
      tenantClaimKey: 'tenantId',
    },
    tokenHeaders: DEFAULT_AUTH_TOKEN_HEADERS,
    validationStrategy: 'auth-access-token',
  };
}

function createBuiltinLocalIntegration(localApiBasePath) {
  return {
    authMode: 'auth-access-token',
    enabled: true,
    handshakeEnabled: false,
    kind: 'builtin-local',
    localApiBasePath,
    secretResolverKind: 'local-static',
    sessionTransport: 'header',
    userSystemScope: 'application',
    validationStrategy: 'auth-access-token',
  };
}

function createExternalAppApiIntegration({
  enabled,
  handshakeEnabled,
  providerKey,
  upstreamBaseUrl,
}) {
  return {
    authMode: 'upstream-app-api-token-bridge',
    enabled,
    handshakeEnabled,
    kind: 'spring-ai-plus-app-api',
    providerKey,
    secretResolverKind: 'upstream-secret-bridge',
    sessionTransport: 'header',
    ...(upstreamBaseUrl ? { upstreamBaseUrl } : {}),
    validationStrategy: 'auth-access-token',
  };
}

test('router portal user-center bridge aligns to the sdkwork-appbase canonical package through the public barrel', () => {
  const upstreamIndexPath = path.join(
    appbaseRoot,
    'packages/pc-react/identity/sdkwork-user-center-pc-react/src/index.ts',
  );
  const bridgePath = path.join(
    appRoot,
    'packages/sdkwork-router-portal-types/src/userCenter.ts',
  );

  assert.equal(existsSync(upstreamIndexPath), true);
  assert.equal(existsSync(bridgePath), true);

  const upstreamIndexSource = readAppbase(
    'packages/pc-react/identity/sdkwork-user-center-pc-react/src/index.ts',
  );
  const upstreamConfigSource = readAppbase(
    'packages/pc-react/identity/sdkwork-user-center-pc-react/src/domain/userCenterConfig.ts',
  );
  const upstreamTypesSource = readAppbase(
    'packages/pc-react/identity/sdkwork-user-center-pc-react/src/types/userCenterTypes.ts',
  );
  const bridgeSource = read('packages/sdkwork-router-portal-types/src/userCenter.ts');

  assert.match(upstreamIndexSource, /@sdkwork\/user-center-pc-react/);
  assert.match(upstreamConfigSource, /USER_CENTER_STANDARD_ENTITY_NAMES/);
  assert.match(upstreamTypesSource, /export type UserCenterIntegrationKind = "builtin-local" \| "spring-ai-plus-app-api";/);
  assert.match(upstreamTypesSource, /export interface UserCenterStorageTopology/);

  for (const exportName of [
    'ROUTER_PORTAL_USER_CENTER_SOURCE_PACKAGE',
    'ROUTER_PORTAL_USER_CENTER_NAMESPACE',
    'ROUTER_PORTAL_USER_CENTER_STANDARD_ENTITIES',
    'ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN',
    'ROUTER_PORTAL_USER_CENTER_LOCAL_API',
    'ROUTER_PORTAL_USER_CENTER_ROUTES',
    'ROUTER_PORTAL_USER_CENTER_RUNTIME_ENV_PREFIX',
    'ROUTER_PORTAL_USER_CENTER_GATEWAY_ENV_PREFIX',
    'ROUTER_PORTAL_USER_CENTER_RUNTIME_ENV_ARTIFACT_BASENAME',
    'ROUTER_PORTAL_USER_CENTER_GATEWAY_ENV_ARTIFACT_BASENAME',
    'createRouterPortalUserCenterHandshakeSigningMessage',
    'createRouterPortalUserCenterHandshakeVerificationContext',
    'createRouterPortalUserCenterSignedHandshakeHeaders',
    'createRouterPortalUserCenterConfig',
    'createRouterPortalUserCenterPluginDefinition',
    'createRouterPortalUserCenterPortalDeploymentProfiles',
    'createRouterPortalUserCenterSessionStore',
    'createRouterPortalUserCenterTokenStore',
  ]) {
    assert.match(bridgeSource, new RegExp(`export\\s+(?:const|function)\\s+${exportName}`));
  }

  assert.match(bridgeSource, /USER_CENTER_SOURCE_PACKAGE_NAME/);
  assert.match(bridgeSource, /sdkwork-user-center-pc-react\/src\/index\.ts/);
  assert.doesNotMatch(bridgeSource, /sdkwork-user-center-pc-react\/src\/domain\//);
  assert.doesNotMatch(bridgeSource, /sdkwork-user-center-pc-react\/src\/types\//);
  assert.match(bridgeSource, /USER_CENTER_STANDARD_ENTITY_NAMES/);
  assert.match(bridgeSource, /USER_CENTER_DEFAULT_LOCAL_API_BASE_PATH/);
  assert.match(bridgeSource, /createUserCenterBridgeConfig/);
  assert.match(bridgeSource, /createUserCenterStoragePlan/);
  assert.doesNotMatch(bridgeSource, /function normalizeIdentifier/);
  assert.doesNotMatch(bridgeSource, /function normalizeDatabaseKey/);
  assert.doesNotMatch(bridgeSource, /function createStorageTopology/);
  assert.match(bridgeSource, /auth:\s*options\.auth/);
  assert.match(bridgeSource, /auth:\s*bridgeConfig\.auth/);
  assert.match(bridgeSource, /integration:\s*bridgeConfig\.integration/);
  assert.match(bridgeSource, /storagePlan:\s*bridgeConfig\.storagePlan/);
});

test('router portal packages restrict canonical appbase user-center imports to the local bridge module', () => {
  const directImportPattern =
    /sdkwork-appbase[\\/]packages[\\/]pc-react[\\/]identity[\\/]sdkwork-user-center-pc-react[\\/]src[\\/]index\.ts/u;
  const allowedImporters = new Set([
    normalizeTestPath(
      path.join(appRoot, 'packages', 'sdkwork-router-portal-types', 'src', 'userCenter.ts'),
    ),
  ]);

  const offenders = collectSourceFiles(path.join(appRoot, 'packages'))
    .filter((filePath) => directImportPattern.test(readFileSync(filePath, 'utf8')))
    .map((filePath) => normalizeTestPath(filePath))
    .filter((filePath) => !allowedImporters.has(filePath));

  assert.deepEqual(offenders, []);
});

test('router portal validation bridge aligns to the canonical validation package and depends on the local user-center bridge', () => {
  const upstreamIndexPath = path.join(
    appbaseRoot,
    'packages/pc-react/identity/sdkwork-user-center-validation-pc-react/src/index.ts',
  );
  const bridgePath = path.join(
    appRoot,
    'packages/sdkwork-router-portal-types/src/validation.ts',
  );

  assert.equal(existsSync(upstreamIndexPath), true);
  assert.equal(existsSync(bridgePath), true);

  const upstreamIndexSource = readAppbase(
    'packages/pc-react/identity/sdkwork-user-center-validation-pc-react/src/index.ts',
  );
  const bridgeSource = read('packages/sdkwork-router-portal-types/src/validation.ts');

  assert.match(upstreamIndexSource, /@sdkwork\/user-center-validation-pc-react/);

  for (const exportName of [
    'ROUTER_PORTAL_USER_CENTER_VALIDATION_SOURCE_PACKAGE',
    'ROUTER_PORTAL_USER_CENTER_VALIDATION_PLUGIN_PACKAGES',
    'createRouterPortalUserCenterValidationInteropContract',
    'createRouterPortalUserCenterValidationPluginDefinition',
    'createRouterPortalUserCenterValidationPreflightReport',
    'createRouterPortalUserCenterValidationSnapshot',
    'assertRouterPortalUserCenterValidationPreflight',
    'resolveRouterPortalProtectedToken',
    'requireRouterPortalProtectedToken',
  ]) {
    assert.match(bridgeSource, new RegExp(`export\\s+(?:const|function)\\s+${exportName}`));
  }

  assert.match(bridgeSource, /sdkwork-user-center-validation-pc-react\/src\/index\.ts/);
  assert.match(bridgeSource, /from '\.\/userCenter'/);
  assert.doesNotMatch(bridgeSource, /sdkwork-user-center-validation-pc-react\/src\/domain\//);
  assert.doesNotMatch(bridgeSource, /sdkwork-user-center-validation-pc-react\/src\/types\//);
});

test('router portal packages restrict canonical appbase validation imports to the local validation bridge module', () => {
  const directImportPattern =
    /sdkwork-appbase[\\/]packages[\\/]pc-react[\\/]identity[\\/]sdkwork-user-center-validation-pc-react[\\/]src[\\/]index\.ts/u;
  const allowedImporters = new Set([
    normalizeTestPath(
      path.join(appRoot, 'packages', 'sdkwork-router-portal-types', 'src', 'validation.ts'),
    ),
  ]);

  const offenders = collectSourceFiles(path.join(appRoot, 'packages'))
    .filter((filePath) => directImportPattern.test(readFileSync(filePath, 'utf8')))
    .map((filePath) => normalizeTestPath(filePath))
    .filter((filePath) => !allowedImporters.has(filePath));

  assert.deepEqual(offenders, []);
});

test('router portal user-center bridge materializes canonical integration and storage topology contracts', async () => {
  const bridge = await loadPortalUserCenterBridge();

  const localConfig = bridge.createRouterPortalUserCenterConfig();

  assert.equal(localConfig.mode, 'local-native');
  assert.equal(localConfig.provider.kind, 'local');
  assert.equal(localConfig.provider.providerKey, 'sdkwork-router-portal-local');
  assert.deepEqual(localConfig.auth, createLocalAuthContract());
  assert.equal(localConfig.integration.activeKind, 'builtin-local');
  assert.deepEqual(
    localConfig.integration.builtinLocal,
    createBuiltinLocalIntegration('/api/app/v1/user-center'),
  );
  assert.deepEqual(
    localConfig.integration.externalAppApi,
    createExternalAppApiIntegration({
      enabled: false,
      handshakeEnabled: false,
      providerKey: 'sdkwork-router-portal-app-api',
    }),
  );
  assert.equal(localConfig.localApi.sessionLogin, '/api/app/v1/user-center/session/login');
  assert.equal(localConfig.storagePlan.sessionHeaderName, 'x-sdkwork-user-center-session-id');
  assert.equal(localConfig.storagePlan.storageScope, 'sdkwork-router-portal.user-center');
  assert.equal(
    localConfig.storagePlan.sessionTokenKey,
    'sdkwork-router-portal.user-center.session-token',
  );
  assert.equal(localConfig.storageTopology.databaseKey, 'sdkwork-router-portal-user-center');
  assert.equal(localConfig.storageTopology.migrationNamespace, 'sdkwork-router-portal.user-center');
  assert.equal(localConfig.storageTopology.tablePrefix, 'uc_');
  assert.deepEqual(
    localConfig.storageTopology.entityBindings.map((binding) => binding.standardEntityName),
    Array.from(bridge.ROUTER_PORTAL_USER_CENTER_STANDARD_ENTITIES),
  );
  assert.deepEqual(
    localConfig.storageTopology.entityBindings.map((binding) => binding.tableName),
    ['uc_user', 'uc_tenant', 'uc_account', 'uc_vip_user', 'uc_organization_member', 'uc_member_relations'],
  );

  const remoteConfig = bridge.createRouterPortalUserCenterConfig({
    mode: 'app-api-hub',
    provider: {
      baseUrl: ' https://app-api.example.com/tenant-edge/ ',
      kind: 'spring-ai-plus-app-api',
      providerKey: ' Tenant Edge App API ',
    },
  });

  assert.equal(remoteConfig.mode, 'app-api-hub');
  assert.equal(remoteConfig.provider.kind, 'spring-ai-plus-app-api');
  assert.equal(remoteConfig.provider.providerKey, 'tenant-edge-app-api');
  assert.equal(remoteConfig.provider.baseUrl, 'https://app-api.example.com/tenant-edge');
  assert.deepEqual(
    remoteConfig.auth,
    createUpstreamAuthContract('sdkwork-router-portal', 'tenant-edge-app-api'),
  );
  assert.equal(remoteConfig.integration.activeKind, 'spring-ai-plus-app-api');
  assert.deepEqual(
    remoteConfig.integration.builtinLocal,
    createBuiltinLocalIntegration('/api/app/v1/user-center'),
  );
  assert.deepEqual(
    remoteConfig.integration.externalAppApi,
    createExternalAppApiIntegration({
      enabled: true,
      handshakeEnabled: true,
      providerKey: 'tenant-edge-app-api',
      upstreamBaseUrl: 'https://app-api.example.com/tenant-edge',
    }),
  );
  assert.equal(remoteConfig.storageTopology.databaseKey, 'sdkwork-router-portal-user-center');
  assert.deepEqual(
    bridge.createRouterPortalUserCenterHandshakeVerificationContext({
      config: remoteConfig,
      headers: {
        'x-sdkwork-app-id': 'sdkwork-router-portal',
        'x-sdkwork-user-center-handshake-mode': 'provider-shared-secret',
        'x-sdkwork-user-center-provider-key': 'tenant-edge-app-api',
        'x-sdkwork-user-center-secret-id': 'secret-301',
        'x-sdkwork-user-center-signature': 'signature-301',
        'x-sdkwork-user-center-signed-at': '2026-04-21T10:00:00Z',
      },
      method: 'GET',
      now: '2026-04-21T10:00:20Z',
      path: remoteConfig.localApi.profile,
    }).handshake,
    {
      appId: 'sdkwork-router-portal',
      handshakeMode: 'provider-shared-secret',
      providerKey: 'tenant-edge-app-api',
      secretId: 'secret-301',
      signature: 'signature-301',
      signedAt: '2026-04-21T10:00:00Z',
    },
  );

  const plugin = bridge.createRouterPortalUserCenterPluginDefinition();

  assert.deepEqual(plugin.capabilities, ['auth', 'user']);
  assert.deepEqual(plugin.auth, createLocalAuthContract());
  assert.equal(plugin.bridgeConfig.namespace, 'sdkwork-router-portal');
  assert.deepEqual(plugin.bridgeConfig.auth, createLocalAuthContract());
  assert.equal(plugin.manifests.auth.loginRoutePath, '/login');
  assert.equal(plugin.manifests.auth.registerRoutePath, '/register');
  assert.equal(plugin.manifests.auth.forgotPasswordRoutePath, '/forgot-password');
  assert.equal(plugin.manifests.auth.oauthCallbackRoutePattern, '/oauth/callback/:provider');
  assert.equal(plugin.manifests.auth.qrRoutePath, undefined);
  assert.equal(plugin.manifests.user.routePath, '/console/user');
  assert.equal(plugin.manifests.vip, undefined);

  assert.equal(plugin.portalDeployment.activeKind, 'builtin-local');
  assert.equal(plugin.portalDeployment.builtinLocal.standard.handshake.freshnessWindowMs, 30000);
  assert.deepEqual(
    plugin.portalDeployment.builtinLocal.artifacts.map((artifact) => artifact.fileName),
    [
      'router-portal.builtin-local.runtime.env.example',
      'router-portal.builtin-local.gateway.env.example',
    ],
  );
  assert.deepEqual(
    plugin.portalDeployment.builtinLocal.runtimeEnvArtifact.variables.map((entry) => entry.envName),
    [
      'VITE_ROUTER_PORTAL_USER_CENTER_MODE',
      'VITE_ROUTER_PORTAL_USER_CENTER_PROVIDER_KEY',
      'VITE_ROUTER_PORTAL_USER_CENTER_LOCAL_API_BASE_PATH',
    ],
  );
  assert.deepEqual(
    plugin.portalDeployment.builtinLocal.gatewayEnvArtifact.variables.map((entry) => entry.envName),
    [
      'ROUTER_PORTAL_USER_CENTER_MODE',
      'ROUTER_PORTAL_USER_CENTER_PROVIDER_KEY',
      'ROUTER_PORTAL_USER_CENTER_LOCAL_API_BASE_PATH',
      'ROUTER_PORTAL_USER_CENTER_SQLITE_PATH',
      'ROUTER_PORTAL_USER_CENTER_DATABASE_URL',
      'ROUTER_PORTAL_USER_CENTER_SCHEMA_NAME',
      'ROUTER_PORTAL_USER_CENTER_TABLE_PREFIX',
    ],
  );
  assert.equal(plugin.portalDeployment.builtinLocal.runtimeEnvArtifact.audience, 'application-runtime');
  assert.equal(plugin.portalDeployment.builtinLocal.runtimeEnvArtifact.format, 'dotenv');
  assert.equal(
    plugin.portalDeployment.builtinLocal.gatewayEnvArtifact.audience,
    'gateway-runtime',
  );
  assert.equal(plugin.portalDeployment.builtinLocal.gatewayEnvArtifact.format, 'dotenv');
  assert.match(
    plugin.portalDeployment.builtinLocal.runtimeEnvArtifact.content,
    /VITE_ROUTER_PORTAL_USER_CENTER_MODE=builtin-local/,
  );
  assert.doesNotMatch(
    plugin.portalDeployment.builtinLocal.runtimeEnvArtifact.content,
    /SHARED_SECRET/,
  );
  assert.match(
    plugin.portalDeployment.builtinLocal.gatewayEnvArtifact.content,
    /ROUTER_PORTAL_USER_CENTER_SQLITE_PATH=\.\/data\/user-center\.db/,
  );

  const remotePlugin = bridge.createRouterPortalUserCenterPluginDefinition({
    mode: 'app-api-hub',
    provider: {
      baseUrl: 'https://app-api.example.com/tenant-edge',
      kind: 'spring-ai-plus-app-api',
      providerKey: 'tenant-edge-app-api',
    },
  });

  assert.equal(remotePlugin.portalDeployment.activeKind, 'spring-ai-plus-app-api');
  assert.equal(remotePlugin.portalDeployment.externalAppApi.providerKey, 'tenant-edge-app-api');
  assert.equal(remotePlugin.portalDeployment.externalAppApi.handshakeEnabled, true);
  assert.equal(
    remotePlugin.portalDeployment.externalAppApi.standard.handshake.freshnessWindowMs,
    30000,
  );
  assert.deepEqual(
    remotePlugin.portalDeployment.externalAppApi.artifacts.map((artifact) => artifact.fileName),
    [
      'router-portal.spring-ai-plus-app-api.runtime.env.example',
      'router-portal.spring-ai-plus-app-api.gateway.env.example',
    ],
  );
  assert.deepEqual(
    remotePlugin.portalDeployment.externalAppApi.runtimeEnvArtifact.variables.map((entry) => entry.envName),
    [
      'VITE_ROUTER_PORTAL_USER_CENTER_MODE',
      'VITE_ROUTER_PORTAL_USER_CENTER_PROVIDER_KEY',
      'VITE_ROUTER_PORTAL_USER_CENTER_LOCAL_API_BASE_PATH',
    ],
  );
  assert.deepEqual(
    remotePlugin.portalDeployment.externalAppApi.gatewayEnvArtifact.variables.map((entry) => entry.envName),
    [
      'ROUTER_PORTAL_USER_CENTER_MODE',
      'ROUTER_PORTAL_USER_CENTER_PROVIDER_KEY',
      'ROUTER_PORTAL_USER_CENTER_LOCAL_API_BASE_PATH',
      'ROUTER_PORTAL_USER_CENTER_APP_API_BASE_URL',
      'ROUTER_PORTAL_USER_CENTER_APP_ID',
      'ROUTER_PORTAL_USER_CENTER_SECRET_ID',
      'ROUTER_PORTAL_USER_CENTER_SHARED_SECRET',
      'ROUTER_PORTAL_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS',
      'ROUTER_PORTAL_USER_CENTER_SQLITE_PATH',
      'ROUTER_PORTAL_USER_CENTER_DATABASE_URL',
      'ROUTER_PORTAL_USER_CENTER_SCHEMA_NAME',
      'ROUTER_PORTAL_USER_CENTER_TABLE_PREFIX',
    ],
  );
  assert.deepEqual(
    remotePlugin.portalDeployment.externalAppApi.gatewayEnvArtifact.variables
      .filter((entry) => entry.required)
      .map((entry) => entry.envName),
    [
      'ROUTER_PORTAL_USER_CENTER_APP_API_BASE_URL',
      'ROUTER_PORTAL_USER_CENTER_SECRET_ID',
      'ROUTER_PORTAL_USER_CENTER_SHARED_SECRET',
    ],
  );
  assert.equal(
    remotePlugin.portalDeployment.externalAppApi.runtimeEnvArtifact.variables.some(
      (entry) => entry.envName === 'VITE_ROUTER_PORTAL_USER_CENTER_SHARED_SECRET',
    ),
    false,
  );
  assert.match(
    remotePlugin.portalDeployment.externalAppApi.runtimeEnvArtifact.content,
    /VITE_ROUTER_PORTAL_USER_CENTER_MODE=spring-ai-plus-app-api/,
  );
  assert.doesNotMatch(
    remotePlugin.portalDeployment.externalAppApi.runtimeEnvArtifact.content,
    /SHARED_SECRET/,
  );
  assert.match(
    remotePlugin.portalDeployment.externalAppApi.gatewayEnvArtifact.content,
    /ROUTER_PORTAL_USER_CENTER_SHARED_SECRET=<required-secret>/,
  );
  assert.match(
    remotePlugin.portalDeployment.externalAppApi.gatewayEnvArtifact.content,
    /ROUTER_PORTAL_USER_CENTER_HANDSHAKE_FRESHNESS_WINDOW_MS=30000/,
  );
  assert.match(
    remotePlugin.portalDeployment.externalAppApi.gatewayEnvArtifact.content,
    /ROUTER_PORTAL_USER_CENTER_APP_API_BASE_URL=https:\/\/app-api\.example\.com\/tenant-edge/,
  );
});

test('router portal auth token helpers and user preferences consume the shared user-center bridge without persisting app-owned auth state', () => {
  const typesEntry = read('packages/sdkwork-router-portal-types/src/index.ts');
  const authStore = read('packages/sdkwork-router-portal-core/src/store/usePortalAuthStore.ts');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const userServices = read('packages/sdkwork-router-portal-user/src/services/index.ts');

  const userCenterExports = Array.from(
    typesEntry.matchAll(/export \* from '\.\/userCenter';/g),
  );
  assert.equal(userCenterExports.length, 1);

  assert.doesNotMatch(authStore, /ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN/);
  assert.doesNotMatch(authStore, /runtimeStateKey/);
  assert.doesNotMatch(authStore, /persist\(/);
  assert.doesNotMatch(authStore, /sdkwork-router-portal\.auth\.v1/);

  assert.match(portalApi, /ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN/);
  assert.match(portalApi, /createRouterPortalUserCenterSessionStore/);
  assert.match(portalApi, /createRouterPortalUserCenterTokenStore/);
  assert.doesNotMatch(portalApi, /sdkwork-user-center-pc-react\/src\/index\.ts/);
  assert.doesNotMatch(portalApi, /sdkwork-user-center-pc-react\/src\/domain\//);
  assert.match(portalApi, /readPortalTokenBundle/);
  assert.match(portalApi, /persistPortalTokenBundle/);
  assert.match(portalApi, /clearPortalTokenBundle/);
  assert.doesNotMatch(portalApi, /sdkwork\.router\.portal\.session-token/);

  assert.match(userServices, /ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN/);
  assert.match(userServices, /preferencesKey/);
  assert.doesNotMatch(userServices, /sdkwork-router-portal\.user-center\.v1/);
});

test('router portal validation bridge materializes canonical validation policy and portal-api delegates token protection to it', async () => {
  const bridge = await loadPortalValidationBridge();
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const typesEntry = read('packages/sdkwork-router-portal-types/src/index.ts');

  const validation = bridge.createRouterPortalUserCenterValidationPluginDefinition({
    mode: 'app-api-hub',
    provider: {
      baseUrl: 'https://app-api.example.com/tenant-edge',
      kind: 'spring-ai-plus-app-api',
      providerKey: 'tenant-edge-app-api',
    },
  });

  assert.equal(validation.capability, 'user-center-validation');
  assert.equal(validation.dependency.capability, 'user-center');
  assert.equal(validation.dependency.namespace, 'sdkwork-router-portal');
  assert.equal(validation.dependency.providerKey, 'tenant-edge-app-api');
  assert.equal(validation.validation.authMode, 'upstream-app-api-token-bridge');
  assert.equal(validation.validation.handshake.enabled, true);
  assert.equal(validation.validation.handshake.freshnessWindowMs, 30000);
  assert.deepEqual(validation.validation.secretResolution, {
    organizationClaimKey: 'organizationId',
    resolverKind: 'upstream-secret-bridge',
    scope: 'organization-preferred',
    tenantClaimKey: 'tenantId',
  });
  assert.deepEqual(validation.validation.governedHeaderNames, [
    'Authorization',
    'Access-Token',
    'Refresh-Token',
    'x-sdkwork-user-center-session-id',
    'x-sdkwork-app-id',
    'x-sdkwork-user-center-handshake-mode',
    'x-sdkwork-user-center-provider-key',
    'x-sdkwork-user-center-secret-id',
    'x-sdkwork-user-center-signature',
    'x-sdkwork-user-center-signed-at',
  ]);
  assert.equal(validation.manifests.validation.capability, 'validation');
  assert.equal(validation.manifests.validation.dependencyCapability, 'user-center');

  assert.equal(
    bridge.resolveRouterPortalProtectedToken({
      providedToken: 'session-token',
      tokenBundle: {
        authToken: 'auth-token',
        sessionToken: 'session-token',
      },
    }),
    'auth-token',
  );
  assert.equal(
    bridge.requireRouterPortalProtectedToken({
      tokenBundle: {
        accessToken: 'access-token',
      },
    }),
    'access-token',
  );

  const interopContract = bridge.createRouterPortalUserCenterValidationInteropContract({
    mode: 'app-api-hub',
    provider: {
      baseUrl: 'https://app-api.example.com/tenant-edge',
      kind: 'spring-ai-plus-app-api',
      providerKey: 'tenant-edge-app-api',
    },
  });

  const preflight = bridge.createRouterPortalUserCenterValidationPreflightReport({
    mode: 'app-api-hub',
    peerContract: interopContract,
    provider: {
      baseUrl: 'https://app-api.example.com/tenant-edge',
      kind: 'spring-ai-plus-app-api',
      providerKey: 'tenant-edge-app-api',
    },
  });

  assert.deepEqual(preflight, {
    compatible: true,
    diff: {
      compatible: true,
      mismatches: [],
    },
    localContract: interopContract,
    peerContract: interopContract,
  });
  assert.deepEqual(
    bridge.assertRouterPortalUserCenterValidationPreflight({
      mode: 'app-api-hub',
      peerContract: interopContract,
      provider: {
        baseUrl: 'https://app-api.example.com/tenant-edge',
        kind: 'spring-ai-plus-app-api',
        providerKey: 'tenant-edge-app-api',
      },
    }),
    preflight,
  );

  const mismatchedPeerContract = {
    ...interopContract,
    tokenHeaders: {
      ...interopContract.tokenHeaders,
      authorizationHeaderName: 'Auth-Token',
    },
  };

  assert.deepEqual(
    bridge.createRouterPortalUserCenterValidationPreflightReport({
      mode: 'app-api-hub',
      peerContract: mismatchedPeerContract,
      provider: {
        baseUrl: 'https://app-api.example.com/tenant-edge',
        kind: 'spring-ai-plus-app-api',
        providerKey: 'tenant-edge-app-api',
      },
    }).diff,
    {
      compatible: false,
      mismatches: [
        {
          actual: 'Authorization',
          expected: 'Auth-Token',
          fieldPath: 'tokenHeaders.authorizationHeaderName',
        },
      ],
    },
  );
  assert.throws(
    () => bridge.assertRouterPortalUserCenterValidationPreflight({
      mode: 'app-api-hub',
      peerContract: mismatchedPeerContract,
      provider: {
        baseUrl: 'https://app-api.example.com/tenant-edge',
        kind: 'spring-ai-plus-app-api',
        providerKey: 'tenant-edge-app-api',
      },
    }),
    /tokenHeaders\.authorizationHeaderName/u,
  );

  const validationExports = Array.from(
    typesEntry.matchAll(/export \* from '\.\/validation';/g),
  );
  assert.equal(validationExports.length, 1);

  assert.match(portalApi, /requireRouterPortalProtectedToken/);
  assert.doesNotMatch(portalApi, /function requiredPortalToken/);
  assert.doesNotMatch(portalApi, /function resolveStoredPortalToken/);
});

test('router portal-api session helpers migrate legacy local storage tokens into session storage', async () => {
  const portalApi = await loadPortalApi();
  const bridge = await loadPortalUserCenterBridge();
  const previousSessionStorage = globalThis.sessionStorage;
  const previousLocalStorage = globalThis.localStorage;
  const sessionStorage = storageDouble();
  const localStorage = storageDouble();

  localStorage.setItem(
    bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey,
    'legacy-session-token',
  );

  globalThis.sessionStorage = sessionStorage;
  globalThis.localStorage = localStorage;

  try {
    assert.equal(portalApi.readPortalSessionToken(), 'legacy-session-token');
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      'legacy-session-token',
    );
    assert.equal(
      localStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      null,
    );

    portalApi.persistPortalSessionToken('fresh-session-token');

    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      'fresh-session-token',
    );

    portalApi.clearPortalSessionToken();

    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      null,
    );
    assert.equal(
      localStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      null,
    );
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }
});

test('router portal-api token bundle helpers persist the canonical auth tokens without hardcoded storage keys', async () => {
  const portalApi = await loadPortalApi();
  const bridge = await loadPortalUserCenterBridge();
  const previousSessionStorage = globalThis.sessionStorage;
  const previousLocalStorage = globalThis.localStorage;
  const sessionStorage = storageDouble();
  const localStorage = storageDouble();

  globalThis.sessionStorage = sessionStorage;
  globalThis.localStorage = localStorage;

  try {
    portalApi.persistPortalTokenBundle({
      accessToken: 'tenant-demo-access',
      authToken: 'tenant-demo-auth',
      refreshToken: 'tenant-demo-refresh',
      sessionToken: 'tenant-demo-session',
      tokenType: 'Bearer',
    });

    assert.deepEqual(portalApi.readPortalTokenBundle(), {
      accessToken: 'tenant-demo-access',
      authToken: 'tenant-demo-auth',
      refreshToken: 'tenant-demo-refresh',
      sessionToken: 'tenant-demo-session',
      tokenType: 'Bearer',
    });
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.accessTokenKey),
      'tenant-demo-access',
    );
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.authTokenKey),
      'tenant-demo-auth',
    );
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.refreshTokenKey),
      'tenant-demo-refresh',
    );
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      'tenant-demo-session',
    );

    portalApi.clearPortalTokenBundle();

    assert.deepEqual(portalApi.readPortalTokenBundle(), {});
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.accessTokenKey),
      null,
    );
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.authTokenKey),
      null,
    );
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.refreshTokenKey),
      null,
    );
    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      null,
    );
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }
});

test('router portal-api session helpers reject malformed tokens and fail closed when browser storage is unavailable', async () => {
  const portalApi = await loadPortalApi();
  const bridge = await loadPortalUserCenterBridge();
  const previousSessionStorage = globalThis.sessionStorage;
  const previousLocalStorage = globalThis.localStorage;

  globalThis.sessionStorage = throwingStorageDouble();
  globalThis.localStorage = throwingStorageDouble();

  try {
    assert.equal(portalApi.readPortalSessionToken(), null);
    assert.doesNotThrow(() => portalApi.persistPortalSessionToken('tenant-demo-session'));
    assert.doesNotThrow(() => portalApi.clearPortalSessionToken());
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }

  const sessionStorage = storageDouble();
  const localStorage = storageDouble();
  globalThis.sessionStorage = sessionStorage;
  globalThis.localStorage = localStorage;

  try {
    assert.throws(
      () => portalApi.persistPortalSessionToken(''),
      {
        name: 'TypeError',
      },
    );

    assert.equal(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      null,
    );
    assert.equal(
      localStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.sessionTokenKey),
      null,
    );
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }
});
