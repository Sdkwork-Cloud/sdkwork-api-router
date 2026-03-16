import type { PortalRouteDefinition } from 'sdkwork-router-portal-types';

export const portalRoutes: PortalRouteDefinition[] = [
  {
    key: 'dashboard',
    label: 'Dashboard',
    eyebrow: 'Overview',
    detail: 'Workspace posture, request activity, and next steps',
  },
  {
    key: 'api-keys',
    label: 'API Keys',
    eyebrow: 'Credentials',
    detail: 'Issue, inspect, and govern environment keys',
  },
  {
    key: 'usage',
    label: 'Usage',
    eyebrow: 'Telemetry',
    detail: 'Calls, models, providers, and token-unit history',
  },
  {
    key: 'credits',
    label: 'Credits',
    eyebrow: 'Points',
    detail: 'Quota posture, ledger entries, and coupon redemption',
  },
  {
    key: 'billing',
    label: 'Billing',
    eyebrow: 'Commerce',
    detail: 'Subscriptions, recharge packs, and trust signals',
  },
  {
    key: 'account',
    label: 'Account',
    eyebrow: 'Identity',
    detail: 'Profile, workspace ownership, and password rotation',
  },
];
