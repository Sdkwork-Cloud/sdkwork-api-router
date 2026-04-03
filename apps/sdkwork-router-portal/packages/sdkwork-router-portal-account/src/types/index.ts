import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  LedgerEntry,
  PortalCommerceMembership,
  PortalRouteKey,
  UsageRecord,
  UsageSummary,
  PortalWorkspaceSummary,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

export interface PortalAccountPageProps {
  workspace: PortalWorkspaceSummary | null;
  onNavigate: (route: PortalRouteKey) => void;
}

export interface AccountMetricSummary {
  revenue: number;
  request_count: number;
  used_units: number;
  average_booked_spend: number;
}

export interface PortalAccountBalanceSummary {
  remaining_units: number | null;
  quota_limit_units: number | null;
  used_units: number;
  utilization_ratio: number | null;
}

export interface PortalAccountMultimodalTotals {
  image_count: number;
  audio_seconds: number;
  video_seconds: number;
  music_seconds: number;
}

export interface PortalAccountFinancialBreakdown {
  total_events: number;
  total_request_count: number;
  total_customer_charge: number;
  top_capabilities: BillingEventCapabilitySummary[];
  accounting_mode_mix: BillingEventAccountingModeSummary[];
  multimodal_totals: PortalAccountMultimodalTotals;
}

export type PortalAccountHistoryView = 'all' | 'expense' | 'revenue';

export interface PortalAccountHistoryRow extends LedgerEntry {
  id: string;
  kind: 'expense' | 'revenue';
  source: 'usage' | 'ledger';
  scope: 'current' | 'linked';
  occurred_at_ms: number | null;
  share_of_booked_amount: number;
  model?: string | null;
  provider?: string | null;
  channel_id?: string | null;
  api_key_hash?: string | null;
}

export interface BuildPortalAccountViewModelInput {
  summary: ProjectBillingSummary;
  membership: PortalCommerceMembership | null;
  usageSummary: UsageSummary | null;
  usageRecords: UsageRecord[];
  ledger: LedgerEntry[];
  billingEventSummary: BillingEventSummary;
  historyView?: PortalAccountHistoryView;
  searchQuery: string;
  page: number;
  pageSize: number;
  now?: number;
}

export interface PortalAccountViewModel {
  billing_summary: ProjectBillingSummary;
  membership: PortalCommerceMembership | null;
  balance: PortalAccountBalanceSummary;
  totals: AccountMetricSummary;
  today: AccountMetricSummary;
  trailing_7d: AccountMetricSummary;
  current_month: AccountMetricSummary;
  financial_breakdown: PortalAccountFinancialBreakdown;
  history_view: PortalAccountHistoryView;
  history_counts: Record<PortalAccountHistoryView, number>;
  visible_history: PortalAccountHistoryRow[];
  pagination: {
    page: number;
    total_pages: number;
    total_items: number;
  };
}
