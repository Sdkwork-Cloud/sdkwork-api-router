import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  CommercialAccountBalanceSnapshot,
  CommercialAccountBenefitLotRecord,
  CommercialAccountLedgerEntryType,
  CommercialAccountLedgerHistoryEntry,
  CommercialAccountHoldRecord,
  CommercialAccountSummary,
  CommercialPricingPlanRecord,
  CommercialPricingRateRecord,
  CommercialRequestSettlementRecord,
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

export interface PortalAccountCommercialPosture {
  account_id: number | null;
  account_status: string | null;
  account_type: string | null;
  currency_code: string | null;
  credit_unit_code: string | null;
  allow_overdraft: boolean;
  overdraft_limit: number;
  available_balance: number;
  held_balance: number;
  consumed_balance: number;
  grant_balance: number;
  active_lot_count: number;
  benefit_lot_count: number;
  active_benefit_lot_count: number;
  expired_benefit_lot_count: number;
  open_hold_count: number;
  settlement_count: number;
  captured_settlement_amount: number;
  pricing_plan_count: number;
  pricing_rate_count: number;
  primary_plan_display_name: string | null;
  primary_rate_metric_code: string | null;
  primary_rate_charge_unit: CommercialPricingRateRecord['charge_unit'] | null;
  primary_rate_pricing_method: CommercialPricingRateRecord['pricing_method'] | null;
  primary_rate_display_price_unit: string | null;
}

export type PortalAccountHistoryView = 'all' | 'expense' | 'revenue';

export interface PortalAccountHistoryRow extends LedgerEntry {
  id: string;
  kind: 'expense' | 'revenue';
  source: 'usage' | 'ledger';
  scope: 'current' | 'linked';
  occurred_at_ms: number | null;
  share_of_booked_amount: number;
  ledger_entry_type?: CommercialAccountLedgerEntryType | null;
  order_id?: string | null;
  request_id?: number | null;
  hold_id?: number | null;
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
  commercialAccount?: CommercialAccountSummary | null;
  accountBalance?: CommercialAccountBalanceSnapshot | null;
  benefitLots?: CommercialAccountBenefitLotRecord[];
  accountLedgerHistory?: CommercialAccountLedgerHistoryEntry[];
  holds?: CommercialAccountHoldRecord[];
  requestSettlements?: CommercialRequestSettlementRecord[];
  pricingPlans?: CommercialPricingPlanRecord[];
  pricingRates?: CommercialPricingRateRecord[];
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
  commercial_posture: PortalAccountCommercialPosture;
  commercial_account: CommercialAccountSummary | null;
  account_balance: CommercialAccountBalanceSnapshot | null;
  benefit_lots: CommercialAccountBenefitLotRecord[];
  holds: CommercialAccountHoldRecord[];
  request_settlements: CommercialRequestSettlementRecord[];
  pricing_plans: CommercialPricingPlanRecord[];
  pricing_rates: CommercialPricingRateRecord[];
  history_view: PortalAccountHistoryView;
  history_counts: Record<PortalAccountHistoryView, number>;
  visible_history: PortalAccountHistoryRow[];
  pagination: {
    page: number;
    total_pages: number;
    total_items: number;
  };
}
