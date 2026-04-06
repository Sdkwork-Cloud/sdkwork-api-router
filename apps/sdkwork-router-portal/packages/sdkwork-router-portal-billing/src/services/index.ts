import { formatUnits } from 'sdkwork-router-portal-commons/format-core';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';
import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventGroupSummary,
  BillingEventRecord,
  BillingEventSummary,
  PortalCommerceOrderCenterEntry,
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import type { BillingRecommendation } from '../types';
import type {
  BillingEventAnalyticsViewModel,
  BillingPaymentHistoryRow,
} from '../types';

export type BillingEventCsvDocument = {
  headers: string[];
  rows: Array<Array<string | number>>;
};

function compareBillingPaymentHistoryRows(
  left: BillingPaymentHistoryRow,
  right: BillingPaymentHistoryRow,
): number {
  const leftKindRank = left.row_kind === 'refunded_order_state' ? 0 : 1;
  const rightKindRank = right.row_kind === 'refunded_order_state' ? 0 : 1;

  return right.received_at_ms - left.received_at_ms
    || leftKindRank - rightKindRank
    || left.order_id.localeCompare(right.order_id)
    || left.id.localeCompare(right.id);
}

function buildPaymentEventHistoryRow(
  entry: PortalCommerceOrderCenterEntry,
  event: PortalCommerceOrderCenterEntry['payment_events'][number],
): BillingPaymentHistoryRow {
  return {
    row_kind: 'payment_event',
    id: event.payment_event_id,
    order_id: entry.order.order_id,
    target_name: entry.order.target_name,
    target_kind: entry.order.target_kind,
    payable_price_label: entry.order.payable_price_label,
    order_status: entry.order.status,
    order_status_after: event.order_status_after ?? null,
    provider: event.provider,
    event_type: event.event_type,
    payment_event_id: event.payment_event_id,
    provider_event_id: event.provider_event_id ?? null,
    processing_status: event.processing_status,
    processing_message: event.processing_message ?? null,
    checkout_reference: entry.checkout_session.reference,
    checkout_session_status: entry.checkout_session.session_status,
    guidance: entry.checkout_session.guidance,
    received_at_ms: event.received_at_ms,
    processed_at_ms: event.processed_at_ms ?? null,
  };
}

function hasRefundPaymentEvent(entry: PortalCommerceOrderCenterEntry): boolean {
  return entry.payment_events.some((event) => event.event_type === 'refunded');
}

function buildRefundedOrderStateRow(
  entry: PortalCommerceOrderCenterEntry,
): BillingPaymentHistoryRow {
  const observedAtMs = Math.max(
    entry.order.updated_at_ms ?? 0,
    entry.latest_payment_event?.processed_at_ms ?? 0,
    entry.latest_payment_event?.received_at_ms ?? 0,
  );

  return {
    row_kind: 'refunded_order_state',
    id: `refund-state:${entry.order.order_id}`,
    order_id: entry.order.order_id,
    target_name: entry.order.target_name,
    target_kind: entry.order.target_kind,
    payable_price_label: entry.order.payable_price_label,
    order_status: entry.order.status,
    order_status_after: 'refunded',
    provider: entry.checkout_session.provider,
    event_type: 'refunded',
    payment_event_id: null,
    provider_event_id: null,
    processing_status: null,
    processing_message: null,
    checkout_reference: entry.checkout_session.reference,
    checkout_session_status: entry.checkout_session.session_status,
    guidance: entry.checkout_session.guidance,
    received_at_ms: observedAtMs,
    processed_at_ms: null,
  };
}

export function buildBillingPaymentHistory(
  entries: PortalCommerceOrderCenterEntry[],
): BillingPaymentHistoryRow[] {
  const rows: BillingPaymentHistoryRow[] = [];

  for (const entry of entries) {
    for (const event of entry.payment_events) {
      rows.push(buildPaymentEventHistoryRow(entry, event));
    }

    if (entry.order.status === 'refunded' && !hasRefundPaymentEvent(entry)) {
      rows.push(buildRefundedOrderStateRow(entry));
    }
  }

  return rows.sort(compareBillingPaymentHistoryRows);
}

export function buildBillingRefundHistory(
  entries: PortalCommerceOrderCenterEntry[],
): BillingPaymentHistoryRow[] {
  return buildBillingPaymentHistory(entries).filter((row) => row.event_type === 'refunded');
}

function buildDailyUsageSeries(usageRecords: UsageRecord[]): number[] {
  const daily = new Map<string, number>();

  for (const record of usageRecords) {
    if (!record.created_at_ms) {
      continue;
    }

    const key = new Date(record.created_at_ms).toISOString().slice(0, 10);
    daily.set(key, (daily.get(key) ?? 0) + record.units);
  }

  return [...daily.entries()]
    .sort((left, right) => left[0].localeCompare(right[0]))
    .map(([, units]) => units);
}

function exponentialMovingAverage(values: number[], alpha = 0.45): number | null {
  if (!values.length) {
    return null;
  }

  let smoothed = values[0];
  for (let index = 1; index < values.length; index += 1) {
    smoothed = alpha * values[index] + (1 - alpha) * smoothed;
  }

  return smoothed;
}

function estimateDailyUnits(
  summary: ProjectBillingSummary,
  usageRecords: UsageRecord[],
): number | null {
  const smoothedDailyUnits = exponentialMovingAverage(buildDailyUsageSeries(usageRecords));
  if (smoothedDailyUnits && Number.isFinite(smoothedDailyUnits)) {
    return Math.max(1, Math.round(smoothedDailyUnits));
  }

  if (summary.used_units <= 0) {
    return null;
  }

  return Math.max(1, Math.ceil(summary.used_units / 30));
}

function buildRunway(
  summary: ProjectBillingSummary,
  usageRecords: UsageRecord[],
): BillingRecommendation['runway'] {
  const daily_units = estimateDailyUnits(summary, usageRecords);

  if (summary.exhausted) {
    return {
      label: translatePortalText('0 days'),
      detail: translatePortalText(
        'Visible quota is already exhausted, so the workspace needs an immediate recharge or plan change before additional traffic is expected.',
      ),
      projected_days: 0,
      daily_units,
    };
  }

  if (summary.remaining_units === null || summary.remaining_units === undefined) {
    return {
      label: translatePortalText('Unlimited'),
      detail: translatePortalText(
        'The current billing summary exposes no visible quota ceiling, so the portal treats runway as unlimited for this workspace.',
      ),
      projected_days: null,
      daily_units,
    };
  }

  if (!daily_units) {
    return {
      label: translatePortalText('Needs first traffic signal'),
      detail: translatePortalText(
        'There is not enough recorded usage yet to project a meaningful burn pace. Send live traffic, then revisit billing decisions.',
      ),
      projected_days: null,
      daily_units: null,
    };
  }

  const projected_days = Math.floor(summary.remaining_units / daily_units);
  const label = projected_days < 1
    ? translatePortalText('< 1 day')
    : translatePortalText('{days} days', { days: projected_days });

  return {
    label,
    detail: translatePortalText(
      'Estimated from an exponentially smoothed burn pace of {units} token units per day.',
      { units: formatUnits(daily_units) },
    ),
    projected_days,
    daily_units,
  };
}

function buildRecommendedBundle(
  summary: ProjectBillingSummary,
  plan: SubscriptionPlan | null,
  pack: RechargePack | null,
): BillingRecommendation['bundle'] {
  if (!plan && !pack) {
    return {
      title: translatePortalText('Billing catalog unavailable'),
      detail: translatePortalText(
        'The portal could not build a plan-plus-pack recommendation from the current seed catalog.',
      ),
    };
  }

  if (summary.exhausted) {
    return {
      title: translatePortalText('{plan} + {pack}', {
        plan: plan?.name ?? translatePortalText('Subscription'),
        pack: pack?.label ?? translatePortalText('Recharge pack'),
      }),
      detail: translatePortalText(
        'The workspace needs both immediate runway recovery and a steadier monthly posture, so the portal recommends a plan and a recharge together.',
      ),
    };
  }

  if ((summary.remaining_units ?? 0) < 10_000) {
    return {
      title: translatePortalText('{plan} with {pack} as buffer', {
        plan: plan?.name ?? translatePortalText('Subscription'),
        pack: pack?.label ?? translatePortalText('Recharge pack'),
      }),
      detail: translatePortalText(
        'Current quota is still active, but remaining headroom is tight enough that a plan-plus-buffer path is the lowest-friction next move.',
      ),
    };
  }

  return {
    title: translatePortalText('{plan} as the next growth step', {
      plan: plan?.name ?? translatePortalText('Subscription'),
    }),
    detail: translatePortalText(
      'The workspace is stable today, so the recommended bundle focuses on the cleanest subscription path while keeping the top-up pack available only if demand spikes.',
    ),
  };
}

export function recommendBillingChange(
  summary: ProjectBillingSummary,
  plans: SubscriptionPlan[],
  packs: RechargePack[],
  usageRecords: UsageRecord[] = [],
): BillingRecommendation {
  const runway = buildRunway(summary, usageRecords);
  const projectedMonthlyUnits = runway.daily_units
    ? runway.daily_units * 30
    : Math.max(summary.used_units, 1);
  const recommendedPlan = plans.length
    ? (plans.find((plan) => plan.included_units >= projectedMonthlyUnits) ?? plans[plans.length - 1])
    : null;
  const recommendedPack = packs.length
    ? (packs.find((pack) => pack.points >= Math.max(10_000, Math.round(projectedMonthlyUnits / 4))) ??
      packs[packs.length - 1])
    : null;
  const bundle = buildRecommendedBundle(summary, recommendedPlan, recommendedPack);

  if (summary.exhausted && recommendedPlan && recommendedPack) {
    return {
      title: translatePortalText('Quota is exhausted'),
      detail: translatePortalText(
        'Move to {plan} or add {pack} to restore headroom immediately.',
        {
          plan: recommendedPlan.name,
          pack: recommendedPack.label,
        },
      ),
      plan: recommendedPlan,
      pack: recommendedPack,
      runway,
      bundle,
    };
  }

  if ((summary.remaining_units ?? 0) < 10_000 && recommendedPlan && recommendedPack) {
    return {
      title: translatePortalText('Headroom is getting tight'),
      detail: translatePortalText(
        'Add {pack} for near-term coverage, or move to {plan} for a steadier monthly posture.',
        {
          pack: recommendedPack.label,
          plan: recommendedPlan.name,
        },
      ),
      plan: recommendedPlan,
      pack: recommendedPack,
      runway,
      bundle,
    };
  }

  return {
    title: recommendedPlan
      ? translatePortalText('Current workspace is stable')
      : translatePortalText('Billing catalog unavailable'),
    detail: recommendedPlan
      ? translatePortalText(
        'Based on a projected monthly demand of {units} units, {plan} is the cleanest next subscription step when traffic grows.',
        {
          units: formatUnits(projectedMonthlyUnits),
          plan: recommendedPlan.name,
        },
      )
      : translatePortalText('The portal could not load a live commerce catalog for this workspace.'),
    plan: recommendedPlan,
    pack: recommendedPack,
    runway,
    bundle,
  };
}

export function isRecommendedPlan(
  plan: SubscriptionPlan,
  recommendation: BillingRecommendation,
): boolean {
  return recommendation.plan?.id === plan.id;
}

export function isRecommendedPack(
  pack: RechargePack,
  recommendation: BillingRecommendation,
): boolean {
  return recommendation.pack?.id === pack.id;
}

function sortCapabilityMix(
  items: BillingEventCapabilitySummary[],
): BillingEventCapabilitySummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || left.capability.localeCompare(right.capability),
    );
}

function sortGroupChargeback(
  items: BillingEventGroupSummary[],
): BillingEventGroupSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || (left.api_key_group_id ?? '').localeCompare(right.api_key_group_id ?? ''),
    );
}

function sortAccountingModeMix(
  items: BillingEventAccountingModeSummary[],
): BillingEventAccountingModeSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || left.accounting_mode.localeCompare(right.accounting_mode),
    );
}

function sortRecentEvents(events: BillingEventRecord[]): BillingEventRecord[] {
  return [...events].sort((left, right) =>
    right.created_at_ms - left.created_at_ms
    || right.customer_charge - left.customer_charge
    || right.units - left.units
    || left.event_id.localeCompare(right.event_id),
  );
}

export function buildBillingEventAnalytics(
  summary: BillingEventSummary,
  events: BillingEventRecord[],
  limits: {
    capabilities?: number;
    groups?: number;
    accounting_modes?: number;
    recent_events?: number;
  } = {},
): BillingEventAnalyticsViewModel {
  const capabilityLimit = limits.capabilities ?? 6;
  const groupLimit = limits.groups ?? 6;
  const accountingModeLimit = limits.accounting_modes ?? 3;
  const recentEventLimit = limits.recent_events ?? 6;

  return {
    totals: {
      total_events: summary.total_events,
      total_request_count: summary.total_request_count,
      total_tokens: summary.total_tokens,
      total_image_count: summary.total_image_count,
      total_audio_seconds: summary.total_audio_seconds,
      total_video_seconds: summary.total_video_seconds,
      total_music_seconds: summary.total_music_seconds,
      total_upstream_cost: summary.total_upstream_cost,
      total_customer_charge: summary.total_customer_charge,
    },
    top_capabilities: sortCapabilityMix(summary.capabilities).slice(0, capabilityLimit),
    group_chargeback: sortGroupChargeback(summary.groups).slice(0, groupLimit),
    accounting_mode_mix: sortAccountingModeMix(summary.accounting_modes).slice(
      0,
      accountingModeLimit,
    ),
    recent_events: sortRecentEvents(events).slice(0, recentEventLimit),
    routing_evidence: {
      events_with_profile: events.filter((event) => event.applied_routing_profile_id).length,
      events_with_compiled_snapshot: events.filter(
        (event) => event.compiled_routing_snapshot_id,
      ).length,
      events_with_fallback_reason: events.filter((event) => event.fallback_reason).length,
    },
  };
}

export function buildBillingEventCsvDocument(
  events: BillingEventRecord[],
): BillingEventCsvDocument {
  return {
    headers: [
      'event_id',
      'tenant_id',
      'project_id',
      'api_key_group_id',
      'capability',
      'route_key',
      'usage_model',
      'provider_id',
      'accounting_mode',
      'operation_kind',
      'modality',
      'api_key_hash',
      'channel_id',
      'reference_id',
      'latency_ms',
      'units',
      'request_count',
      'input_tokens',
      'output_tokens',
      'total_tokens',
      'cache_read_tokens',
      'cache_write_tokens',
      'image_count',
      'audio_seconds',
      'video_seconds',
      'music_seconds',
      'upstream_cost',
      'customer_charge',
      'applied_routing_profile_id',
      'compiled_routing_snapshot_id',
      'fallback_reason',
      'created_at',
    ],
    rows: events.map((event) => [
      event.event_id,
      event.tenant_id,
      event.project_id,
      event.api_key_group_id ?? '',
      event.capability,
      event.route_key,
      event.usage_model,
      event.provider_id,
      event.accounting_mode,
      event.operation_kind,
      event.modality,
      event.api_key_hash ?? '',
      event.channel_id ?? '',
      event.reference_id ?? '',
      event.latency_ms ?? '',
      event.units,
      event.request_count,
      event.input_tokens,
      event.output_tokens,
      event.total_tokens,
      event.cache_read_tokens,
      event.cache_write_tokens,
      event.image_count,
      event.audio_seconds,
      event.video_seconds,
      event.music_seconds,
      event.upstream_cost.toFixed(4),
      event.customer_charge.toFixed(4),
      event.applied_routing_profile_id ?? '',
      event.compiled_routing_snapshot_id ?? '',
      event.fallback_reason ?? '',
      new Date(event.created_at_ms).toISOString(),
    ]),
  };
}
