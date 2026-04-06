import type {
  CommerceOrderRecord,
  CommercePaymentEventRecord,
  CommercePaymentEventType,
} from 'sdkwork-router-admin-types';

export type CommercialOrderPaymentAuditRowKind =
  | 'payment_event'
  | 'order_state'
  | 'refunded_order_state';

export interface CommercialOrderPaymentAuditRow {
  id: string;
  row_kind: CommercialOrderPaymentAuditRowKind;
  order_id: string;
  project_id: string;
  user_id: string;
  target_kind: string;
  target_name: string;
  payable_price_cents: number;
  payable_price_label: string;
  order_status: CommerceOrderRecord['status'];
  applied_coupon_code?: string | null;
  subsidy_amount_minor: number;
  payment_event_id: string | null;
  provider: string;
  provider_event_id: string | null;
  event_type: CommercePaymentEventType | null;
  processing_status: CommercePaymentEventRecord['processing_status'] | null;
  processing_message: string | null;
  order_status_after:
    | CommercePaymentEventRecord['order_status_after']
    | CommerceOrderRecord['status']
    | null;
  created_at_ms: number;
  updated_at_ms: number;
  received_at_ms: number | null;
  processed_at_ms: number | null;
  observed_at_ms: number;
}

function latestEventForOrder(
  orderEvents: CommercePaymentEventRecord[],
): CommercePaymentEventRecord | null {
  if (!orderEvents.length) {
    return null;
  }

  return [...orderEvents].sort((left, right) =>
    right.received_at_ms - left.received_at_ms
    || right.payment_event_id.localeCompare(left.payment_event_id),
  )[0] ?? null;
}

function hasRefundPaymentEvent(orderEvents: CommercePaymentEventRecord[]): boolean {
  return orderEvents.some((event) => event.event_type === 'refunded');
}

function buildPaymentEventRow(
  order: CommerceOrderRecord,
  event: CommercePaymentEventRecord,
): CommercialOrderPaymentAuditRow {
  const observedAtMs = event.processed_at_ms ?? event.received_at_ms;

  return {
    id: event.payment_event_id,
    row_kind: 'payment_event',
    order_id: order.order_id,
    project_id: order.project_id,
    user_id: order.user_id,
    target_kind: order.target_kind,
    target_name: order.target_name,
    payable_price_cents: order.payable_price_cents,
    payable_price_label: order.payable_price_label,
    order_status: order.status,
    applied_coupon_code: order.applied_coupon_code ?? null,
    subsidy_amount_minor: order.subsidy_amount_minor ?? 0,
    payment_event_id: event.payment_event_id,
    provider: event.provider,
    provider_event_id: event.provider_event_id ?? null,
    event_type: event.event_type,
    processing_status: event.processing_status,
    processing_message: event.processing_message ?? null,
    order_status_after: event.order_status_after ?? null,
    created_at_ms: order.created_at_ms,
    updated_at_ms: order.updated_at_ms,
    received_at_ms: event.received_at_ms,
    processed_at_ms: event.processed_at_ms ?? null,
    observed_at_ms: observedAtMs,
  };
}

function buildOrderStateRow(
  order: CommerceOrderRecord,
  rowKind: CommercialOrderPaymentAuditRowKind,
  providerHint: string | null,
  processingStatusHint: CommercePaymentEventRecord['processing_status'] | null,
): CommercialOrderPaymentAuditRow {
  const isRefundFallback = rowKind === 'refunded_order_state';

  return {
    id: isRefundFallback ? `refund-state:${order.order_id}` : `order-state:${order.order_id}`,
    row_kind: rowKind,
    order_id: order.order_id,
    project_id: order.project_id,
    user_id: order.user_id,
    target_kind: order.target_kind,
    target_name: order.target_name,
    payable_price_cents: order.payable_price_cents,
    payable_price_label: order.payable_price_label,
    order_status: order.status,
    applied_coupon_code: order.applied_coupon_code ?? null,
    subsidy_amount_minor: order.subsidy_amount_minor ?? 0,
    payment_event_id: null,
    provider: providerHint ?? 'not_recorded',
    provider_event_id: null,
    event_type: isRefundFallback ? 'refunded' : null,
    processing_status: processingStatusHint,
    processing_message: null,
    order_status_after: isRefundFallback ? 'refunded' : order.status,
    created_at_ms: order.created_at_ms,
    updated_at_ms: order.updated_at_ms,
    received_at_ms: null,
    processed_at_ms: null,
    observed_at_ms: order.updated_at_ms,
  };
}

export function buildCommercialOrderPaymentAuditRows(
  orders: CommerceOrderRecord[],
  paymentEvents: CommercePaymentEventRecord[],
): CommercialOrderPaymentAuditRow[] {
  const eventsByOrderId = new Map<string, CommercePaymentEventRecord[]>();
  for (const event of paymentEvents) {
    const existing = eventsByOrderId.get(event.order_id);
    if (existing) {
      existing.push(event);
    } else {
      eventsByOrderId.set(event.order_id, [event]);
    }
  }

  const rows: CommercialOrderPaymentAuditRow[] = [];

  for (const order of orders) {
    const orderEvents = eventsByOrderId.get(order.order_id) ?? [];
    const latestOrderEvent = latestEventForOrder(orderEvents);

    if (!orderEvents.length) {
      rows.push(
        buildOrderStateRow(
          order,
          order.status === 'refunded' ? 'refunded_order_state' : 'order_state',
          null,
          null,
        ),
      );
      continue;
    }

    for (const event of orderEvents) {
      rows.push(buildPaymentEventRow(order, event));
    }

    if (order.status === 'refunded' && !hasRefundPaymentEvent(orderEvents)) {
      rows.push(
        buildOrderStateRow(
          order,
          'refunded_order_state',
          latestOrderEvent?.provider ?? null,
          latestOrderEvent?.processing_status ?? null,
        ),
      );
    }
  }

  rows.sort((left, right) =>
    right.observed_at_ms - left.observed_at_ms
    || right.order_id.localeCompare(left.order_id)
    || (right.payment_event_id ?? '').localeCompare(left.payment_event_id ?? ''),
  );

  return rows;
}

export function buildCommercialRefundAuditRows(
  rows: CommercialOrderPaymentAuditRow[],
): CommercialOrderPaymentAuditRow[] {
  return rows.filter((row) => row.event_type === 'refunded');
}
