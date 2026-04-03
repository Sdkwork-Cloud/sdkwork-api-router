import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  PortalCommerceMembership,
  PortalCommerceOrder,
  PortalCommerceQuote,
  PortalCustomRechargePolicy,
  PortalRechargeOption,
  PortalRouteKey,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

export interface PortalRechargePageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface PortalRechargePageData {
  summary: ProjectBillingSummary;
  rechargeOptions: PortalRechargeOption[];
  customRechargePolicy: PortalCustomRechargePolicy | null;
  orders: PortalCommerceOrder[];
  membership: PortalCommerceMembership | null;
  billing_event_summary: BillingEventSummary;
}

export interface PortalRechargeSummaryCard {
  label: string;
  value: string;
  detail: string;
}

export interface PortalRechargeQuoteSnapshot {
  amountLabel: string;
  projectedBalanceLabel: string;
  grantedUnitsLabel: string;
  effectiveRatioLabel: string;
  pricingRuleLabel: string;
}

export type PortalRechargeSelectionMode = 'preset' | 'custom';

export interface PortalRechargeSelection {
  amountCents: number;
  mode: PortalRechargeSelectionMode;
}

export interface PortalRechargePageState {
  quote: PortalCommerceQuote | null;
  selection: PortalRechargeSelection | null;
}

export interface PortalRechargeMultimodalTotals {
  image_count: number;
  audio_seconds: number;
  video_seconds: number;
  music_seconds: number;
}

export interface PortalRechargeFinanceProjection {
  membership: PortalCommerceMembership | null;
  leading_accounting_mode: BillingEventAccountingModeSummary | null;
  leading_capability: BillingEventCapabilitySummary | null;
  multimodal_totals: PortalRechargeMultimodalTotals;
}
