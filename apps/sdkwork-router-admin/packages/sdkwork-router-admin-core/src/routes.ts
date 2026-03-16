import type { AdminRouteDefinition } from 'sdkwork-router-admin-types';

export const adminRoutes: AdminRouteDefinition[] = [
  {
    key: 'overview',
    label: 'Overview',
    eyebrow: 'Control',
    detail: 'Global health, alerts, and operator shortcuts',
  },
  {
    key: 'users',
    label: 'Users',
    eyebrow: 'Identity',
    detail: 'Operator and portal user management',
  },
  {
    key: 'tenants',
    label: 'Tenants',
    eyebrow: 'Workspace',
    detail: 'Tenants, projects, and gateway keys',
  },
  {
    key: 'coupons',
    label: 'Coupons',
    eyebrow: 'Growth',
    detail: 'Campaign and discount code operations',
  },
  {
    key: 'catalog',
    label: 'Catalog',
    eyebrow: 'Routing Mesh',
    detail: 'Channels, providers, and model exposure',
  },
  {
    key: 'traffic',
    label: 'Traffic',
    eyebrow: 'Audit',
    detail: 'Usage, billing, and request-log visibility',
  },
  {
    key: 'operations',
    label: 'Operations',
    eyebrow: 'Runtime',
    detail: 'Health snapshots, reloads, and runtime posture',
  },
];
