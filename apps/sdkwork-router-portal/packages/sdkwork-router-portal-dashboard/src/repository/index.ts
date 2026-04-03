import {
  getPortalBillingEventSummary,
  getPortalCommerceMembership,
  getPortalDashboard,
  getPortalRoutingSummary,
  listPortalUsageRecords,
  listPortalRoutingDecisionLogs,
} from 'sdkwork-router-portal-portal-api';
import type { PortalDashboardSummary } from 'sdkwork-router-portal-types';

import type { PortalDashboardSnapshotBundle } from '../types';

export async function loadPortalDashboardSnapshot(
  initialDashboard?: PortalDashboardSummary | null,
): Promise<PortalDashboardSnapshotBundle> {
  const [dashboard, membership, billing_event_summary, routing_summary, routing_logs, usage_records] = await Promise.all([
    initialDashboard ? Promise.resolve(initialDashboard) : getPortalDashboard(),
    getPortalCommerceMembership(),
    getPortalBillingEventSummary(),
    getPortalRoutingSummary(),
    listPortalRoutingDecisionLogs(),
    listPortalUsageRecords(),
  ]);

  return {
    dashboard,
    membership,
    billing_event_summary,
    routing_summary,
    routing_logs,
    usage_records,
  };
}
