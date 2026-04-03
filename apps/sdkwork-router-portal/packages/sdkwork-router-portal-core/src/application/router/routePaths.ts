import type {
  PortalAnonymousRouteKey,
  PortalRouteKey,
  PortalTopLevelRouteKey,
} from 'sdkwork-router-portal-types';

export const PORTAL_ROUTE_PATHS: Record<
  PortalAnonymousRouteKey | PortalRouteKey | PortalTopLevelRouteKey,
  string
> = {
  home: '/',
  console: '/console',
  models: '/models',
  docs: '/docs',
  downloads: '/downloads',
  login: '/login',
  register: '/register',
  'forgot-password': '/forgot-password',
  gateway: '/console/gateway',
  dashboard: '/console/dashboard',
  routing: '/console/routing',
  'api-keys': '/console/api-keys',
  usage: '/console/usage',
  user: '/console/user',
  credits: '/console/redeem',
  recharge: '/console/recharge',
  billing: '/console/billing',
  account: '/console/account',
};

export function toRouteElementPath(pathname: string): string {
  return pathname.replace(/^\//, '');
}
