import {
  confirmPortalCouponRedemption,
  getPortalBillingEventSummary,
  getPortalBillingSummary,
  listPortalCommerceOrders,
  listPortalMarketingMyCoupons,
  listPortalMarketingRedemptions,
  listPortalMarketingRewardHistory,
  reservePortalCouponRedemption,
  validatePortalCoupon,
} from 'sdkwork-router-portal-portal-api';
import type {
  MarketingSubjectScope,
  PortalCouponRedemptionConfirmResponse,
  PortalCouponValidationResponse,
} from 'sdkwork-router-portal-types';

import type { CreditsPageData } from '../types';

export async function loadCreditsPageData(): Promise<CreditsPageData> {
  const [
    summary,
    orders,
    billing_event_summary,
    marketing_codes,
    marketing_reward_history,
    marketing_redemptions,
  ] = await Promise.all([
    getPortalBillingSummary(),
    listPortalCommerceOrders(),
    getPortalBillingEventSummary(),
    listPortalMarketingMyCoupons(),
    listPortalMarketingRewardHistory(),
    listPortalMarketingRedemptions(),
  ]);

  return {
    summary,
    orders,
    billing_event_summary,
    marketing_codes,
    marketing_reward_history,
    marketing_redemptions,
  };
}

export function validateCreditsCouponCode(input: {
  coupon_code: string;
  subject_scope?: MarketingSubjectScope;
}): Promise<PortalCouponValidationResponse> {
  return validatePortalCoupon({
    coupon_code: input.coupon_code.trim().toUpperCase(),
    subject_scope: input.subject_scope ?? 'project',
    target_kind: 'coupon_redemption',
    order_amount_minor: 0,
    reserve_amount_minor: 0,
  });
}

export async function redeemCreditsCouponCode(input: {
  coupon_code: string;
  subject_scope?: MarketingSubjectScope;
  ttl_ms?: number;
}): Promise<PortalCouponRedemptionConfirmResponse> {
  const couponCode = input.coupon_code.trim().toUpperCase();
  const reservation = await reservePortalCouponRedemption({
    coupon_code: couponCode,
    subject_scope: input.subject_scope ?? 'project',
    target_kind: 'coupon_redemption',
    reserve_amount_minor: 0,
    ttl_ms: input.ttl_ms ?? 300_000,
  });

  return confirmPortalCouponRedemption({
    coupon_reservation_id: reservation.reservation.coupon_reservation_id,
    subsidy_amount_minor: 0,
  });
}
