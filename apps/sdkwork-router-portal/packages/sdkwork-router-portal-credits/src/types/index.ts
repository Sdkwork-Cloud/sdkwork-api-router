import type { CouponOffer, LedgerEntry, PortalRouteKey, ProjectBillingSummary } from 'sdkwork-router-portal-types';

export interface PortalCreditsPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface CouponImpactPreview {
  offer: CouponOffer;
  projected_remaining_units: number | null;
  status: string;
}

export interface RecommendedCouponOffer {
  offer: CouponOffer;
  rationale: string;
  preview: CouponImpactPreview;
}

export interface CreditsGuardrail {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface CreditsPageData {
  summary: ProjectBillingSummary;
  ledger: LedgerEntry[];
}
