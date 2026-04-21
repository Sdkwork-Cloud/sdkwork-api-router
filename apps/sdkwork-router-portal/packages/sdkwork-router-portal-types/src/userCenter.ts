import {
  USER_CENTER_DEFAULT_LOCAL_API_BASE_PATH,
  USER_CENTER_SESSION_HEADER_NAME,
  USER_CENTER_SOURCE_PACKAGE_NAME,
  createUserCenterDeploymentEnvArtifact,
  createUserCenterHandshakeSigningMessage,
  createUserCenterHandshakeVerificationContext,
  createUserCenterPluginDefinition,
  createUserCenterLocalApiRoutes,
  createUserCenterBridgeConfig,
  createUserCenterSignedHandshakeHeaders,
  createUserCenterSessionStore,
  createUserCenterStoragePlan,
  createUserCenterTokenStore,
  selectUserCenterDeploymentVariables,
  USER_CENTER_STANDARD_ENTITY_NAMES,
} from "../../../../../../sdkwork-appbase/packages/pc-react/identity/sdkwork-user-center-pc-react/src/index.ts";
import type {
  UserCenterBridgeConfig,
  UserCenterBridgeConfigInput,
  UserCenterBuiltinLocalIntegrationProfile,
  UserCenterDeploymentArtifact,
  UserCenterDeploymentEnvironmentVariable,
  UserCenterDeploymentProfile,
  UserCenterDeploymentProfileSet,
  UserCenterDeploymentVariable,
  UserCenterExternalAppApiIntegrationProfile,
  UserCenterIntegrationKind,
  UserCenterIntegrationProfileSet,
  UserCenterLocalApiRoutes,
  UserCenterMode,
  UserCenterHandshakeSignature,
  UserCenterHandshakeVerificationContext,
  UserCenterHandshakeVerificationContextInput,
  UserCenterPluginCapabilityName,
  UserCenterPluginDefinition,
  UserCenterPluginDefinitionOptions,
  UserCenterProviderConfig,
  UserCenterProviderKind,
  UserCenterRoutes,
  UserCenterSessionTransport,
  UserCenterSessionStore,
  UserCenterStandardEntityName,
  UserCenterStorageEntityBinding,
  UserCenterStorageEntityBindingInput,
  UserCenterStoragePlan,
  UserCenterStorageTopology,
  UserCenterStorageTopologyInput,
  UserCenterTokenStore,
  UserCenterUserSystemScope,
} from "../../../../../../sdkwork-appbase/packages/pc-react/identity/sdkwork-user-center-pc-react/src/index.ts";

export type RouterPortalUserCenterMode = UserCenterMode;
export type RouterPortalUserCenterProviderKind = UserCenterProviderKind;
export type RouterPortalUserCenterIntegrationKind = UserCenterIntegrationKind;
export type RouterPortalUserCenterSessionTransport = UserCenterSessionTransport;
export type RouterPortalUserCenterUserSystemScope = UserCenterUserSystemScope;
export type RouterPortalUserCenterStandardEntityName = UserCenterStandardEntityName;

export type RouterPortalUserCenterProviderConfig = UserCenterProviderConfig;
export type RouterPortalUserCenterRoutes = UserCenterRoutes;
export type RouterPortalUserCenterStoragePlan = UserCenterStoragePlan;
export type RouterPortalUserCenterLocalApiRoutes = UserCenterLocalApiRoutes;
export type RouterPortalUserCenterStorageEntityBindingInput = UserCenterStorageEntityBindingInput;
export type RouterPortalUserCenterStorageEntityBinding = UserCenterStorageEntityBinding;
export type RouterPortalUserCenterStorageTopologyInput = UserCenterStorageTopologyInput;
export type RouterPortalUserCenterStorageTopology = UserCenterStorageTopology;
export type RouterPortalUserCenterBuiltinLocalIntegrationProfile =
  UserCenterBuiltinLocalIntegrationProfile;
export type RouterPortalUserCenterExternalAppApiIntegrationProfile =
  UserCenterExternalAppApiIntegrationProfile;
export type RouterPortalUserCenterIntegrationProfileSet = UserCenterIntegrationProfileSet;
export type RouterPortalUserCenterHandshakeSignature = UserCenterHandshakeSignature;
export type RouterPortalUserCenterHandshakeVerificationContext =
  UserCenterHandshakeVerificationContext;
export type RouterPortalUserCenterRuntimeConfig = UserCenterBridgeConfig;
export type RouterPortalUserCenterSessionStore = UserCenterSessionStore;
export type RouterPortalUserCenterTokenStore = UserCenterTokenStore;
export type RouterPortalUserCenterPluginCapability = Extract<
  UserCenterPluginCapabilityName,
  "auth" | "user"
>;

export type CreateRouterPortalUserCenterConfigOptions =
  Omit<UserCenterBridgeConfigInput, "namespace" | "routes"> & {
    routes?: Partial<RouterPortalUserCenterRoutes>;
  };
export type CreateRouterPortalUserCenterHandshakeVerificationContextOptions =
  Omit<UserCenterHandshakeVerificationContextInput, "config"> & {
    config?: RouterPortalUserCenterRuntimeConfig;
  };
export type CreateRouterPortalUserCenterPluginDefinitionOptions =
  Omit<UserCenterPluginDefinitionOptions, "capabilities" | "namespace" | "routes"> & {
    capabilities?: readonly RouterPortalUserCenterPluginCapability[];
    routes?: Partial<RouterPortalUserCenterRoutes>;
  };

export type RouterPortalUserCenterEnvironmentVariable = UserCenterDeploymentEnvironmentVariable;
export type RouterPortalUserCenterDeploymentArtifact = UserCenterDeploymentArtifact;

export interface RouterPortalUserCenterPortalDeploymentProfile {
  artifacts: readonly RouterPortalUserCenterDeploymentArtifact[];
  gatewayEnvArtifact: RouterPortalUserCenterDeploymentArtifact;
  handshakeEnabled: boolean;
  kind: UserCenterDeploymentProfile["kind"];
  providerKey: string;
  runtimeEnvArtifact: RouterPortalUserCenterDeploymentArtifact;
  standard: UserCenterDeploymentProfile;
}

export interface RouterPortalUserCenterPortalDeploymentProfileSet {
  activeKind: UserCenterDeploymentProfileSet["activeKind"];
  builtinLocal: RouterPortalUserCenterPortalDeploymentProfile;
  externalAppApi: RouterPortalUserCenterPortalDeploymentProfile;
}

export interface RouterPortalUserCenterPluginDefinition extends UserCenterPluginDefinition {
  portalDeployment: RouterPortalUserCenterPortalDeploymentProfileSet;
}

export const ROUTER_PORTAL_USER_CENTER_SOURCE_PACKAGE = USER_CENTER_SOURCE_PACKAGE_NAME;
export const ROUTER_PORTAL_USER_CENTER_NAMESPACE = "sdkwork-router-portal";
export const ROUTER_PORTAL_USER_CENTER_SESSION_HEADER_NAME = USER_CENTER_SESSION_HEADER_NAME;
export const ROUTER_PORTAL_USER_CENTER_STANDARD_ENTITIES = USER_CENTER_STANDARD_ENTITY_NAMES;
export const ROUTER_PORTAL_USER_CENTER_PLUGIN_PACKAGES = [
  "sdkwork-router-portal-auth",
  "sdkwork-router-portal-user",
] as const;
export const ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN = createUserCenterStoragePlan(
  ROUTER_PORTAL_USER_CENTER_NAMESPACE,
);
export const ROUTER_PORTAL_USER_CENTER_ROUTES: RouterPortalUserCenterRoutes = {
  authBasePath: "/login",
  userRoutePath: "/console/user",
  vipRoutePath: "/console/account",
};
export const ROUTER_PORTAL_USER_CENTER_LOCAL_API = createUserCenterLocalApiRoutes();
export const ROUTER_PORTAL_USER_CENTER_RUNTIME_ENV_PREFIX = "VITE_ROUTER_PORTAL_USER_CENTER_";
export const ROUTER_PORTAL_USER_CENTER_GATEWAY_ENV_PREFIX = "ROUTER_PORTAL_USER_CENTER_";
export const ROUTER_PORTAL_USER_CENTER_RUNTIME_ENV_ARTIFACT_BASENAME = "runtime.env.example";
export const ROUTER_PORTAL_USER_CENTER_GATEWAY_ENV_ARTIFACT_BASENAME = "gateway.env.example";

function createRouterPortalUserCenterBasePluginArtifacts(
  options: CreateRouterPortalUserCenterPluginDefinitionOptions = {},
): {
  bridgeConfig: RouterPortalUserCenterRuntimeConfig;
  plugin: UserCenterPluginDefinition;
} {
  const bridgeConfig = createRouterPortalUserCenterConfig({
    auth: options.auth,
    localApiBasePath: options.localApiBasePath,
    mode: options.mode,
    provider: options.provider,
    routes: options.routes,
    storageTopology: options.storageTopology,
  });

  const plugin = createUserCenterPluginDefinition({
    auth: options.auth,
    capabilities: options.capabilities ?? ["auth", "user"],
    host: options.host,
    localApiBasePath: options.localApiBasePath ?? USER_CENTER_DEFAULT_LOCAL_API_BASE_PATH,
    mode: options.mode,
    namespace: ROUTER_PORTAL_USER_CENTER_NAMESPACE,
    packageNames: options.packageNames ?? [...ROUTER_PORTAL_USER_CENTER_PLUGIN_PACKAGES],
    provider: options.provider,
    routes: {
      authBasePath: "",
      userRoutePath: options.routes?.userRoutePath ?? ROUTER_PORTAL_USER_CENTER_ROUTES.userRoutePath,
      vipRoutePath: options.routes?.vipRoutePath ?? ROUTER_PORTAL_USER_CENTER_ROUTES.vipRoutePath,
    },
    storageTopology: options.storageTopology,
    theme: options.theme,
    title: options.title ?? "SDKWORK Router Portal User Center",
  });

  return {
    bridgeConfig,
    plugin,
  };
}

function createRouterPortalUserCenterEnvName(prefix: string, canonicalName: string): string {
  const suffix = canonicalName.replace(/^SDKWORK_USER_CENTER_/, "");
  return `${prefix}${suffix}`;
}

function mapRouterPortalUserCenterEnvironmentVariables(
  variables: readonly UserCenterDeploymentVariable[],
  prefix: string,
): RouterPortalUserCenterEnvironmentVariable[] {
  return variables.map((variable) => ({
    ...(variable.canonicalName ? { canonicalName: variable.canonicalName } : {}),
    ...(variable.defaultValue ? { defaultValue: variable.defaultValue } : {}),
    description: variable.description,
    envName: createRouterPortalUserCenterEnvName(prefix, variable.canonicalName),
    ...(variable.exampleValue ? { exampleValue: variable.exampleValue } : {}),
    required: variable.required,
    ...(variable.secret ? { secret: true } : {}),
  }));
}

function mergeRouterPortalDeploymentVariables(
  ...groups: readonly UserCenterDeploymentVariable[][]
): UserCenterDeploymentVariable[] {
  const seen = new Set<string>();
  const merged: UserCenterDeploymentVariable[] = [];

  for (const group of groups) {
    for (const variable of group) {
      if (seen.has(variable.canonicalName)) {
        continue;
      }

      seen.add(variable.canonicalName);
      merged.push(variable);
    }
  }

  return merged;
}

function createRouterPortalDeploymentArtifactFileName(
  kind: UserCenterDeploymentProfile["kind"],
  basename: string,
): string {
  return `router-portal.${kind}.${basename}`;
}

function createRouterPortalUserCenterPortalDeploymentProfile(
  profile: UserCenterDeploymentProfile,
): RouterPortalUserCenterPortalDeploymentProfile {
  const runtimeEnv = Object.freeze(mapRouterPortalUserCenterEnvironmentVariables(
    selectUserCenterDeploymentVariables(profile, "application-runtime"),
    ROUTER_PORTAL_USER_CENTER_RUNTIME_ENV_PREFIX,
  ));
  const gatewayEnv = Object.freeze(mapRouterPortalUserCenterEnvironmentVariables(
    mergeRouterPortalDeploymentVariables(
      selectUserCenterDeploymentVariables(profile, "upstream-bridge"),
      selectUserCenterDeploymentVariables(profile, "local-authority"),
    ),
    ROUTER_PORTAL_USER_CENTER_GATEWAY_ENV_PREFIX,
  ));
  const runtimeEnvArtifact = Object.freeze(createUserCenterDeploymentEnvArtifact({
    audience: "application-runtime",
    fileName: createRouterPortalDeploymentArtifactFileName(
      profile.kind,
      ROUTER_PORTAL_USER_CENTER_RUNTIME_ENV_ARTIFACT_BASENAME,
    ),
    headerComment: `Router Portal ${profile.kind} runtime env`,
    purpose: `Public runtime env artifact for the Router Portal ${profile.kind} user-center deployment.`,
    variables: runtimeEnv,
  }));
  const gatewayEnvArtifact = Object.freeze(createUserCenterDeploymentEnvArtifact({
    audience: "gateway-runtime",
    fileName: createRouterPortalDeploymentArtifactFileName(
      profile.kind,
      ROUTER_PORTAL_USER_CENTER_GATEWAY_ENV_ARTIFACT_BASENAME,
    ),
    headerComment: `Router Portal ${profile.kind} gateway env`,
    purpose: `Private gateway env artifact for the Router Portal ${profile.kind} user-center deployment.`,
    variables: gatewayEnv,
  }));

  return Object.freeze({
    artifacts: Object.freeze([runtimeEnvArtifact, gatewayEnvArtifact]),
    gatewayEnvArtifact,
    handshakeEnabled: profile.handshake.enabled,
    kind: profile.kind,
    providerKey: profile.providerKey,
    runtimeEnvArtifact,
    standard: profile,
  });
}

function createRouterPortalUserCenterPortalDeploymentProfileSet(
  plugin: UserCenterPluginDefinition,
): RouterPortalUserCenterPortalDeploymentProfileSet {
  return Object.freeze({
    activeKind: plugin.deployment.activeKind,
    builtinLocal: createRouterPortalUserCenterPortalDeploymentProfile(
      plugin.deployment.builtinLocal,
    ),
    externalAppApi: createRouterPortalUserCenterPortalDeploymentProfile(
      plugin.deployment.externalAppApi,
    ),
  });
}

export function createRouterPortalUserCenterSessionStore(
  storagePlan: RouterPortalUserCenterStoragePlan = ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN,
): RouterPortalUserCenterSessionStore {
  return createUserCenterSessionStore(storagePlan);
}

export function createRouterPortalUserCenterTokenStore(
  storagePlan: RouterPortalUserCenterStoragePlan = ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN,
): RouterPortalUserCenterTokenStore {
  return createUserCenterTokenStore(storagePlan);
}

export function createRouterPortalUserCenterHandshakeSigningMessage(options: {
  config?: RouterPortalUserCenterRuntimeConfig;
  method: "GET" | "PATCH" | "POST";
  path: string;
  signedAt: string;
}): string {
  return createUserCenterHandshakeSigningMessage({
    config: options.config ?? ROUTER_PORTAL_USER_CENTER_RUNTIME_CONFIG,
    method: options.method,
    path: options.path,
    signedAt: options.signedAt,
  });
}

export function createRouterPortalUserCenterSignedHandshakeHeaders(
  signature: RouterPortalUserCenterHandshakeSignature,
  config: RouterPortalUserCenterRuntimeConfig = ROUTER_PORTAL_USER_CENTER_RUNTIME_CONFIG,
): Record<string, string> {
  return createUserCenterSignedHandshakeHeaders(config, signature);
}

export function createRouterPortalUserCenterHandshakeVerificationContext(
  options: CreateRouterPortalUserCenterHandshakeVerificationContextOptions,
): RouterPortalUserCenterHandshakeVerificationContext {
  return createUserCenterHandshakeVerificationContext({
    ...options,
    config: options.config ?? ROUTER_PORTAL_USER_CENTER_RUNTIME_CONFIG,
  });
}

export function createRouterPortalUserCenterConfig(
  options: CreateRouterPortalUserCenterConfigOptions = {},
): RouterPortalUserCenterRuntimeConfig {
  return createUserCenterBridgeConfig({
    auth: options.auth,
    localApiBasePath: options.localApiBasePath ?? USER_CENTER_DEFAULT_LOCAL_API_BASE_PATH,
    mode: options.mode,
    namespace: ROUTER_PORTAL_USER_CENTER_NAMESPACE,
    provider: options.provider,
    routes: {
      authBasePath: options.routes?.authBasePath ?? ROUTER_PORTAL_USER_CENTER_ROUTES.authBasePath,
      userRoutePath: options.routes?.userRoutePath ?? ROUTER_PORTAL_USER_CENTER_ROUTES.userRoutePath,
      vipRoutePath: options.routes?.vipRoutePath ?? ROUTER_PORTAL_USER_CENTER_ROUTES.vipRoutePath,
    },
    storageTopology: options.storageTopology,
  });
}

export function createRouterPortalUserCenterPluginDefinition(
  options: CreateRouterPortalUserCenterPluginDefinitionOptions = {},
): RouterPortalUserCenterPluginDefinition {
  const { bridgeConfig, plugin } = createRouterPortalUserCenterBasePluginArtifacts(options);

  return {
    auth: bridgeConfig.auth,
    ...plugin,
    bridgeConfig,
    integration: bridgeConfig.integration,
    manifests: {
      ...plugin.manifests,
      ...(plugin.manifests.auth
        ? {
            auth: {
              ...plugin.manifests.auth,
              forgotPasswordRoutePath: "/forgot-password",
              loginRoutePath: bridgeConfig.routes.authBasePath,
              oauthCallbackRoutePattern: "/oauth/callback/:provider",
              qrRoutePath: undefined,
              registerRoutePath: "/register",
            },
          }
        : {}),
    },
    portalDeployment: createRouterPortalUserCenterPortalDeploymentProfileSet(plugin),
    storageTopology: bridgeConfig.storageTopology,
    storagePlan: bridgeConfig.storagePlan,
  };
}

export function createRouterPortalUserCenterPortalDeploymentProfiles(
  options: CreateRouterPortalUserCenterPluginDefinitionOptions = {},
): RouterPortalUserCenterPortalDeploymentProfileSet {
  const { plugin } = createRouterPortalUserCenterBasePluginArtifacts(options);
  return createRouterPortalUserCenterPortalDeploymentProfileSet(plugin);
}

export const ROUTER_PORTAL_USER_CENTER_RUNTIME_CONFIG = createRouterPortalUserCenterConfig();
