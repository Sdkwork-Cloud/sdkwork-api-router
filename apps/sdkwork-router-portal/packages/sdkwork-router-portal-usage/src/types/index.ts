import type { PortalRouteKey, UsageRecord, UsageSummary } from 'sdkwork-router-portal-types';

export interface PortalUsagePageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export type UsageDateRange = '24h' | '7d' | '30d' | 'all';

export interface UsageFilters {
  model: string;
  provider: string;
  date_range: UsageDateRange;
}

export interface UsageHighlight {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface UsageProfileItem {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface UsageDiagnostic {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface UsageWorkbenchViewModel {
  summary: UsageSummary;
  filtered_records: UsageRecord[];
  total_units: number;
  total_amount: number;
  model_options: string[];
  provider_options: string[];
  highlights: UsageHighlight[];
  traffic_profile: UsageProfileItem[];
  spend_watch: UsageProfileItem[];
  diagnostics: UsageDiagnostic[];
}
