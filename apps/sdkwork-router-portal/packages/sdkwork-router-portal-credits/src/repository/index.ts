import {
  createPortalCommerceOrder,
  getPortalBillingEventSummary,
  getPortalCommerceCatalog,
  getPortalBillingSummary,
  listPortalCommerceOrders,
  previewPortalCommerceQuote,
} from 'sdkwork-router-portal-portal-api';
import type {
  PortalCommerceOrder,
  PortalCommerceQuote,
} from 'sdkwork-router-portal-types';

import type { CreditsPageData } from '../types';

export async function loadCreditsPageData(): Promise<CreditsPageData> {
  const [summary, catalog, orders, billing_event_summary] = await Promise.all([
    getPortalBillingSummary(),
    getPortalCommerceCatalog(),
    listPortalCommerceOrders(),
    getPortalBillingEventSummary(),
  ]);

  return {
    summary,
    coupons: catalog.coupons.filter((coupon) => coupon.bonus_units > 0),
    orders,
    billing_event_summary,
  };
}

export function previewCreditsCouponRedemption(input: {
  target_id: string;
  current_remaining_units?: number | null;
}): Promise<PortalCommerceQuote> {
  return previewPortalCommerceQuote({
    target_kind: 'coupon_redemption',
    target_id: input.target_id,
    current_remaining_units: input.current_remaining_units,
  });
}

export function createCreditsCouponRedemption(input: {
  target_id: string;
}): Promise<PortalCommerceOrder> {
  return createPortalCommerceOrder({
    target_kind: 'coupon_redemption',
    target_id: input.target_id,
  });
}
