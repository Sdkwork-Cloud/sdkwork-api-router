import { getPortalBillingSummary } from 'sdkwork-router-portal-portal-api';
import type { ProjectBillingSummary } from 'sdkwork-router-portal-types';

export function loadBillingSummary(): Promise<ProjectBillingSummary> {
  return getPortalBillingSummary();
}
