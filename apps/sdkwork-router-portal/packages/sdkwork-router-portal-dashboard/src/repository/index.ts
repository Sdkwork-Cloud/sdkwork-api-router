import { getPortalDashboard } from 'sdkwork-router-portal-portal-api';
import type { PortalDashboardSummary } from 'sdkwork-router-portal-types';

export function loadPortalDashboardSnapshot(): Promise<PortalDashboardSummary> {
  return getPortalDashboard();
}
