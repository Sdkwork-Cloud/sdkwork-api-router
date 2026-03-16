import {
  getPortalUsageSummary,
  listPortalUsageRecords,
} from 'sdkwork-router-portal-portal-api';
import type { UsageRecord, UsageSummary } from 'sdkwork-router-portal-types';

export async function loadUsageWorkbenchData(): Promise<{
  summary: UsageSummary;
  records: UsageRecord[];
}> {
  const [summary, records] = await Promise.all([
    getPortalUsageSummary(),
    listPortalUsageRecords(),
  ]);

  return { summary, records };
}
