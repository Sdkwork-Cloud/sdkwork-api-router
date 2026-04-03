import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventGroupSummary,
  BillingEventRecord,
  BillingEventSummary,
  PortalCommerceMembership,
  PortalCommerceOrder,
  PortalCommerceQuote,
  PortalRouteKey,
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

export interface PortalBillingPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface BillingRunway {
  label: string;
  detail: string;
  projected_days: number | null;
  daily_units: number | null;
}

export interface BillingBundleRecommendation {
  title: string;
  detail: string;
}

export interface BillingRecommendation {
  title: string;
  detail: string;
  plan: SubscriptionPlan | null;
  pack: RechargePack | null;
  runway: BillingRunway;
  bundle: BillingBundleRecommendation;
}

export interface BillingPageData {
  summary: ProjectBillingSummary;
  usage_records: UsageRecord[];
  billing_events: BillingEventRecord[];
  billing_event_summary: BillingEventSummary;
  plans: SubscriptionPlan[];
  packs: RechargePack[];
  orders: PortalCommerceOrder[];
  membership: PortalCommerceMembership | null;
}

export interface BillingEventAnalyticsTotals {
  total_events: number;
  total_request_count: number;
  total_tokens: number;
  total_image_count: number;
  total_audio_seconds: number;
  total_video_seconds: number;
  total_music_seconds: number;
  total_upstream_cost: number;
  total_customer_charge: number;
}

export interface BillingRoutingEvidence {
  events_with_profile: number;
  events_with_compiled_snapshot: number;
  events_with_fallback_reason: number;
}

export interface BillingEventAnalyticsViewModel {
  totals: BillingEventAnalyticsTotals;
  top_capabilities: BillingEventCapabilitySummary[];
  group_chargeback: BillingEventGroupSummary[];
  accounting_mode_mix: BillingEventAccountingModeSummary[];
  recent_events: BillingEventRecord[];
  routing_evidence: BillingRoutingEvidence;
}

export type BillingCheckoutPreview = PortalCommerceQuote;
