import type {
  AdminRouteKey,
  AdminRouteManifestEntry,
  AdminRouteModuleId,
  AdminProductModuleManifest,
} from 'sdkwork-router-admin-types';

import { adminRoutePathByKey } from './routePaths';
import { adminRoutes } from './routes';

export const adminProductModules: AdminProductModuleManifest[] = [
  {
    moduleId: 'sdkwork-router-admin-overview',
    pluginId: 'sdkwork-router-admin-overview',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-overview',
    displayName: 'Overview',
    routeKeys: ['overview'],
    capabilityTags: ['operator-overview', 'alerts', 'health'],
    requiredPermissions: ['admin.overview.read'],
    navigation: {
      group: 'Control Plane',
      order: 10,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'overview',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-users',
    pluginId: 'sdkwork-router-admin-users',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-users',
    displayName: 'Users',
    routeKeys: ['users'],
    capabilityTags: ['operator-identities', 'portal-identities'],
    requiredPermissions: ['admin.users.read', 'admin.users.write'],
    navigation: {
      group: 'Workspace Ops',
      order: 20,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'workspace-users',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-tenants',
    pluginId: 'sdkwork-router-admin-tenants',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-tenants',
    displayName: 'Tenants',
    routeKeys: ['tenants'],
    capabilityTags: ['tenant-governance', 'project-governance', 'gateway-key-issuance'],
    requiredPermissions: ['admin.tenants.read', 'admin.tenants.write'],
    navigation: {
      group: 'Workspace Ops',
      order: 30,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'workspace-tenants',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-coupons',
    pluginId: 'sdkwork-router-admin-coupons',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-coupons',
    displayName: 'Coupons',
    routeKeys: ['coupons'],
    capabilityTags: ['growth-campaigns', 'coupon-governance'],
    requiredPermissions: ['admin.coupons.read', 'admin.coupons.write'],
    navigation: {
      group: 'Workspace Ops',
      order: 40,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'workspace-growth',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-commercial',
    pluginId: 'sdkwork-router-admin-commercial',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-commercial',
    displayName: 'Commercial',
    routeKeys: ['commercial'],
    capabilityTags: ['commercial-accounts', 'settlement-explorer', 'pricing-governance'],
    requiredPermissions: ['admin.commercial.read'],
    navigation: {
      group: 'Commercial',
      order: 45,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'commercial',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-pricing',
    pluginId: 'sdkwork-router-admin-pricing',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-pricing',
    displayName: 'Pricing',
    routeKeys: ['pricing'],
    capabilityTags: ['pricing-governance', 'billing-methods', 'charge-unit-governance'],
    requiredPermissions: ['admin.commercial.read', 'admin.commercial.write'],
    navigation: {
      group: 'Commercial',
      order: 46,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'pricing',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-apirouter',
    pluginId: 'sdkwork-router-admin-apirouter',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-apirouter',
    displayName: 'API Router',
    routeKeys: ['api-keys', 'rate-limits', 'route-config', 'model-mapping', 'usage-records'],
    capabilityTags: ['api-key-governance', 'routing-policy', 'rate-limit-policy', 'usage-audit'],
    requiredPermissions: ['admin.gateway.read', 'admin.gateway.write'],
    navigation: {
      group: 'API Router',
      order: 50,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'api-router',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-catalog',
    pluginId: 'sdkwork-router-admin-catalog',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-catalog',
    displayName: 'Catalog',
    routeKeys: ['catalog'],
    capabilityTags: ['provider-catalog', 'model-catalog', 'credential-lifecycle'],
    requiredPermissions: ['admin.catalog.read', 'admin.catalog.write'],
    navigation: {
      group: 'Routing Mesh',
      order: 60,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'catalog',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-traffic',
    pluginId: 'sdkwork-router-admin-traffic',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-traffic',
    displayName: 'Traffic',
    routeKeys: ['traffic'],
    capabilityTags: ['traffic-analytics', 'billing-audit', 'routing-evidence'],
    requiredPermissions: ['admin.traffic.read'],
    navigation: {
      group: 'Routing Mesh',
      order: 70,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'traffic',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-operations',
    pluginId: 'sdkwork-router-admin-operations',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-operations',
    displayName: 'Operations',
    routeKeys: ['operations'],
    capabilityTags: ['runtime-health', 'runtime-reload', 'rollout-evidence'],
    requiredPermissions: ['admin.operations.read', 'admin.operations.execute'],
    navigation: {
      group: 'System',
      order: 80,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'operations',
    },
  },
  {
    moduleId: 'sdkwork-router-admin-settings',
    pluginId: 'sdkwork-router-admin-settings',
    pluginKind: 'admin-module',
    packageName: 'sdkwork-router-admin-settings',
    displayName: 'Settings',
    routeKeys: ['settings'],
    capabilityTags: ['workspace-preferences', 'shell-settings'],
    requiredPermissions: ['admin.settings.read', 'admin.settings.write'],
    navigation: {
      group: 'System',
      order: 90,
      sidebar: true,
    },
    loading: {
      strategy: 'lazy',
      prefetch: 'intent',
      chunkGroup: 'settings',
    },
  },
];

const adminProductModuleById = Object.fromEntries(
  adminProductModules.map((productModule) => [productModule.moduleId, productModule]),
) as Record<AdminRouteModuleId, AdminProductModuleManifest>;

const adminProductModuleByRouteKey = adminProductModules.reduce(
  (accumulator, productModule) => {
    for (const routeKey of productModule.routeKeys) {
      accumulator[routeKey] = productModule;
    }

    return accumulator;
  },
  {} as Record<AdminRouteKey, AdminProductModuleManifest>,
);

const adminRouteModuleByKey = Object.fromEntries(
  Object.entries(adminProductModuleByRouteKey).map(([routeKey, productModule]) => [
    routeKey,
    productModule.moduleId,
  ]),
) as Record<AdminRouteKey, AdminRouteModuleId>;

export const adminRouteManifest: AdminRouteManifestEntry[] = adminRoutes.map((route) => ({
  ...route,
  path: adminRoutePathByKey[route.key],
  moduleId: adminRouteModuleByKey[route.key],
  prefetchGroup: adminProductModuleByRouteKey[route.key].loading.chunkGroup,
  productModule: adminProductModuleByRouteKey[route.key],
}));

export function resolveAdminPath(routeKey: AdminRouteKey): string {
  return adminRoutePathByKey[routeKey];
}

export function resolveAdminProductModule(
  moduleId: AdminRouteModuleId,
): AdminProductModuleManifest {
  return adminProductModuleById[moduleId];
}
