import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  PortalCommerceCoupon,
  PortalCommerceOrder,
  PortalCommerceQuote,
  PortalRouteKey,
  PortalWorkspaceSummary,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

export interface PortalCreditsPageProps {
  onNavigate: (route: PortalRouteKey) => void;
  workspace?: PortalWorkspaceSummary | null;
}

export interface CouponImpactPreview {
  coupon: PortalCommerceCoupon;
  quote: PortalCommerceQuote;
  status: string;
}

export interface RecommendedCouponOffer {
  offer: PortalCommerceCoupon;
  rationale: string;
  preview: CouponImpactPreview;
}

export interface CreditsGuardrail {
  id: string;
  title: string;
  detail: string;
  tone: 'default' | 'secondary' | 'success' | 'warning';
}

export interface RedeemInviteProgram {
  code: string;
  link: string;
  owner_label: string;
  audience_label: string;
}

export interface RedeemInviteRow {
  id: string;
  workspace_name: string;
  contact: string;
  invited_at_ms: number;
  activated_at_ms: number | null;
  reward_units: number;
  reward_state: 'pending' | 'rewarded';
}

export interface CreditsPageData {
  summary: ProjectBillingSummary;
  coupons: PortalCommerceCoupon[];
  orders: PortalCommerceOrder[];
  billing_event_summary: BillingEventSummary;
}

export interface PortalCreditsMultimodalTotals {
  image_count: number;
  audio_seconds: number;
  video_seconds: number;
  music_seconds: number;
}

export interface PortalCreditsRedemptionCoverage {
  fulfilled_redemptions: number;
  granted_units: number;
  bonus_units: number;
  next_funding_path: 'redeem' | 'recharge';
}

export interface PortalCreditsFinanceProjection {
  redemption_coverage: PortalCreditsRedemptionCoverage;
  leading_accounting_mode: BillingEventAccountingModeSummary | null;
  leading_capability: BillingEventCapabilitySummary | null;
  multimodal_totals: PortalCreditsMultimodalTotals;
}
