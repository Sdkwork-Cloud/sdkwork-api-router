import {
  cancelPortalCommerceOrder,
  createPortalCommerceOrder,
  getPortalCommercialAccountHistory,
  getPortalBillingEventSummary,
  getPortalBillingEvents,
  getPortalCommerceCheckoutSession,
  getPortalCommerceCatalog,
  getPortalCommerceOrderCenter,
  getPortalBillingSummary,
  listPortalCommercialPricingPlans,
  listPortalCommercialPricingRates,
  listPortalUsageRecords,
  previewPortalCommerceQuote,
  sendPortalCommercePaymentEvent,
  settlePortalCommerceOrder,
} from 'sdkwork-router-portal-portal-api';
import type { PortalCommerceOrder, PortalCommerceQuote } from 'sdkwork-router-portal-types';
import type {
  PortalCommerceCheckoutSession,
  PortalCommercePaymentEventRequest,
} from 'sdkwork-router-portal-types';

import type { BillingPageData } from '../types';
import {
  buildBillingPaymentHistory,
  buildBillingRefundHistory,
} from '../services';

export async function loadBillingPageData(): Promise<BillingPageData> {
  const [
    summary,
    usage_records,
    billing_event_summary,
    billing_events,
    catalog,
    order_center,
    commercial_history,
    commercial_pricing_plans,
    commercial_pricing_rates,
  ] = await Promise.all([
    getPortalBillingSummary(),
    listPortalUsageRecords(),
    getPortalBillingEventSummary(),
    getPortalBillingEvents(),
    getPortalCommerceCatalog(),
    getPortalCommerceOrderCenter(),
    getPortalCommercialAccountHistory(),
    listPortalCommercialPricingPlans(),
    listPortalCommercialPricingRates(),
  ]);

  const commercial_account = {
    account: commercial_history.account,
    available_balance: commercial_history.balance.available_balance,
    held_balance: commercial_history.balance.held_balance,
    consumed_balance: commercial_history.balance.consumed_balance,
    grant_balance: commercial_history.balance.grant_balance,
    active_lot_count: commercial_history.balance.active_lot_count,
  };
  const orderCenterEntries = order_center.orders;

  return {
    summary,
    usage_records,
    billing_events,
    billing_event_summary,
    plans: catalog.plans,
    packs: catalog.packs,
    orders: orderCenterEntries.map((entry) => entry.order),
    payment_history: buildBillingPaymentHistory(orderCenterEntries),
    refund_history: buildBillingRefundHistory(orderCenterEntries),
    membership: order_center.membership,
    commercial_reconciliation: order_center.reconciliation,
    commercial_account,
    commercial_balance: commercial_history.balance,
    commercial_benefit_lots: commercial_history.benefit_lots,
    commercial_holds: commercial_history.holds,
    commercial_request_settlements: commercial_history.request_settlements,
    commercial_pricing_plans,
    commercial_pricing_rates,
  };
}

export function previewBillingCheckout(input: {
  target_kind: 'subscription_plan' | 'recharge_pack';
  target_id: string;
  coupon_code?: string | null;
  current_remaining_units?: number | null;
}): Promise<PortalCommerceQuote> {
  return previewPortalCommerceQuote(input);
}

export function createBillingOrder(input: {
  target_kind: 'subscription_plan' | 'recharge_pack';
  target_id: string;
  coupon_code?: string | null;
}): Promise<PortalCommerceOrder> {
  return createPortalCommerceOrder(input);
}

export function settleBillingOrder(order_id: string): Promise<PortalCommerceOrder> {
  return settlePortalCommerceOrder(order_id);
}

export function cancelBillingOrder(order_id: string): Promise<PortalCommerceOrder> {
  return cancelPortalCommerceOrder(order_id);
}

export function getBillingCheckoutSession(
  order_id: string,
): Promise<PortalCommerceCheckoutSession> {
  return getPortalCommerceCheckoutSession(order_id);
}

export function getBillingCommercialAccountHistory() {
  return getPortalCommercialAccountHistory();
}

export function getBillingOrderCenter() {
  return getPortalCommerceOrderCenter();
}

export function sendBillingPaymentEvent(
  order_id: string,
  input: PortalCommercePaymentEventRequest,
): Promise<PortalCommerceOrder> {
  return sendPortalCommercePaymentEvent(order_id, input);
}
