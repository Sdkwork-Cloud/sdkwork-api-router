import {
  getPortalBillingSummary,
  listPortalBillingLedger,
} from 'sdkwork-router-portal-portal-api';
import type { LedgerEntry, ProjectBillingSummary } from 'sdkwork-router-portal-types';

export async function loadCreditsPageData(): Promise<{
  summary: ProjectBillingSummary;
  ledger: LedgerEntry[];
}> {
  const [summary, ledger] = await Promise.all([
    getPortalBillingSummary(),
    listPortalBillingLedger(),
  ]);

  return { summary, ledger };
}
