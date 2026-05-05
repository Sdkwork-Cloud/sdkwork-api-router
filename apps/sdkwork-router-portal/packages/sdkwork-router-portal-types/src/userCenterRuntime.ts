import {
  createDefaultUserCenterConfig,
  createUserCenterRuntimeClient,
  createUserCenterSessionStore,
  createUserCenterTokenStore,
  resolveUserCenterRuntimeConfigInput,
  type UserCenterRuntimeClient,
  type UserCenterRuntimeClientOptions,
  type UserCenterRuntimeConfig,
} from "../../../../../../sdkwork-appbase/packages/pc-react/identity/sdkwork-user-center-core-pc-react/src/index.ts";
import {
  createRouterPortalUserCenterConfig,
  ROUTER_PORTAL_USER_CENTER_GATEWAY_ENV_PREFIX,
  ROUTER_PORTAL_USER_CENTER_RUNTIME_ENV_PREFIX,
  type CreateRouterPortalUserCenterConfigOptions,
} from './userCenter';
import {
  createRouterPortalUserCenterValidationInteropContract,
} from './validation';

export {
  createUserCenterRuntimeClient,
  createUserCenterSessionStore,
  createUserCenterTokenStore,
};

export type CreateRouterPortalCanonicalUserCenterConfigOptions =
  CreateRouterPortalUserCenterConfigOptions;
export type CreateRouterPortalUserCenterRuntimeClientOptions = UserCenterRuntimeClientOptions;
export type RouterPortalCanonicalUserCenterRuntimeConfig = UserCenterRuntimeConfig;
export type RouterPortalUserCenterRuntimeClient = UserCenterRuntimeClient;

export const ROUTER_PORTAL_CANONICAL_USER_CENTER_SQLITE_PATH =
  'app://sdkwork-router-portal/user-center.db';
export const ROUTER_PORTAL_CANONICAL_USER_CENTER_DATABASE_KEY =
  'sdkwork-router-portal-user-center';
export const ROUTER_PORTAL_CANONICAL_USER_CENTER_MIGRATION_NAMESPACE =
  'sdkwork-router-portal.user-center';
export const ROUTER_PORTAL_CANONICAL_USER_CENTER_TABLE_PREFIX = 'rp_uc_';

function resolveRouterPortalRuntimeWindow():
  | Record<string, unknown>
  | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  return window as unknown as Record<string, unknown>;
}

function resolveRouterPortalRuntimeEnv():
  | Record<string, unknown>
  | undefined {
  return (
    (import.meta as ImportMeta & { env?: Record<string, unknown> }).env
    ?? undefined
  );
}

function resolveRouterPortalRuntimeConfigOptions(
  options: CreateRouterPortalCanonicalUserCenterConfigOptions,
): CreateRouterPortalCanonicalUserCenterConfigOptions {
  return resolveUserCenterRuntimeConfigInput(options, {
    env: resolveRouterPortalRuntimeEnv(),
    envPrefix: ROUTER_PORTAL_USER_CENTER_RUNTIME_ENV_PREFIX,
    window: resolveRouterPortalRuntimeWindow(),
    windowPrefix: ROUTER_PORTAL_USER_CENTER_GATEWAY_ENV_PREFIX,
  });
}

function createRouterPortalCanonicalStorageTopology(
  runtimeConfig: UserCenterRuntimeConfig,
) {
  return {
    ...runtimeConfig.storageTopology,
    databaseKey: ROUTER_PORTAL_CANONICAL_USER_CENTER_DATABASE_KEY,
    migrationNamespace: ROUTER_PORTAL_CANONICAL_USER_CENTER_MIGRATION_NAMESPACE,
    tablePrefix: ROUTER_PORTAL_CANONICAL_USER_CENTER_TABLE_PREFIX,
  };
}

function createDefaultRouterPortalValidationInteropContract(
  runtimeConfig: UserCenterRuntimeConfig,
) {
  return createRouterPortalUserCenterValidationInteropContract({
    auth: runtimeConfig.auth,
    localApiBasePath: runtimeConfig.integration.builtinLocal.localApiBasePath,
    mode: runtimeConfig.mode,
    provider: runtimeConfig.provider,
    routes: runtimeConfig.routes,
    storageTopology: runtimeConfig.storageTopology,
  });
}

export function createRouterPortalCanonicalUserCenterConfig(
  options: CreateRouterPortalCanonicalUserCenterConfigOptions = {},
): RouterPortalCanonicalUserCenterRuntimeConfig {
  const resolvedOptions = resolveRouterPortalRuntimeConfigOptions(options);
  const bridgeConfig = createRouterPortalUserCenterConfig(resolvedOptions);

  return createDefaultUserCenterConfig({
    auth: bridgeConfig.auth,
    localApiBasePath: bridgeConfig.integration.builtinLocal.localApiBasePath,
    mode: bridgeConfig.mode,
    namespace: bridgeConfig.namespace,
    provider: bridgeConfig.provider,
    routes: bridgeConfig.routes,
    storage: {
      dialect: 'sqlite',
      sqlitePath: ROUTER_PORTAL_CANONICAL_USER_CENTER_SQLITE_PATH,
    },
    storageTopology: createRouterPortalCanonicalStorageTopology(bridgeConfig),
  });
}

export function createRouterPortalUserCenterRuntimeClient(
  configOptions: CreateRouterPortalCanonicalUserCenterConfigOptions = {},
  options: CreateRouterPortalUserCenterRuntimeClientOptions = {},
): RouterPortalUserCenterRuntimeClient {
  const runtimeConfig = createRouterPortalCanonicalUserCenterConfig(configOptions);

  return createUserCenterRuntimeClient(runtimeConfig, {
    ...options,
    ...(options.validationInteropContract || options.resolveValidationInteropContract
      ? {}
      : {
          validationInteropContract:
            createDefaultRouterPortalValidationInteropContract(runtimeConfig),
        }),
  });
}
