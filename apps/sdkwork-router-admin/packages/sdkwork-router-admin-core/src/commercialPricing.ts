import {
  compareCommercialNumericIdsDesc,
  commercialNumericIdsEqual,
} from 'sdkwork-router-admin-types';
import type {
  CommercialPricingChargeUnit,
  CommercialPricingMethod,
  CommercialPricingPlanRecord,
  CommercialPricingRateRecord,
} from 'sdkwork-router-admin-types';

type AdminTranslateFn = (text: string, values?: Record<string, string | number>) => string;

function normalizeCommercialPricingPlanStatus(value: string | null | undefined): string {
  return value?.trim().toLowerCase() ?? '';
}

function titleCaseToken(value: string): string {
  return value
    .split(/[-_\s]+/g)
    .filter(Boolean)
    .map((segment) =>
      segment.length <= 3
        ? segment.toUpperCase()
        : `${segment.slice(0, 1).toUpperCase()}${segment.slice(1)}`,
    )
    .join(' ');
}

export function commercialPricingChargeUnitLabel(
  chargeUnit: CommercialPricingChargeUnit | null | undefined,
  t: AdminTranslateFn,
): string {
  switch (chargeUnit) {
    case 'input_token':
      return t('Input token');
    case 'output_token':
      return t('Output token');
    case 'cache_read_token':
      return t('Cache read token');
    case 'cache_write_token':
      return t('Cache write token');
    case 'request':
      return t('Request');
    case 'image':
      return t('Image');
    case 'audio_second':
      return t('Audio second');
    case 'audio_minute':
      return t('Audio minute');
    case 'video_second':
      return t('Video second');
    case 'video_minute':
      return t('Video minute');
    case 'music_track':
      return t('Music track');
    case 'character':
      return t('Character');
    case 'storage_mb_day':
      return t('Storage MB day');
    case 'tool_call':
      return t('Tool call');
    case 'unit':
      return t('Unit');
    default:
      return chargeUnit ? titleCaseToken(chargeUnit) : t('n/a');
  }
}

export function commercialPricingMethodLabel(
  pricingMethod: CommercialPricingMethod | null | undefined,
  t: AdminTranslateFn,
): string {
  switch (pricingMethod) {
    case 'per_unit':
      return t('Per unit');
    case 'flat':
      return t('Flat');
    case 'step':
      return t('Step');
    case 'included_then_per_unit':
      return t('Included then per unit');
    default:
      return pricingMethod ? titleCaseToken(pricingMethod) : t('n/a');
  }
}

export function commercialPricingDisplayUnit(
  rate:
    | Pick<CommercialPricingRateRecord, 'display_price_unit' | 'charge_unit' | 'quantity_step'>
    | null
    | undefined,
  t: AdminTranslateFn,
): string {
  if (!rate) {
    return t('n/a');
  }

  if (rate.display_price_unit.trim()) {
    return rate.display_price_unit;
  }

  switch (rate.charge_unit) {
    case 'input_token':
      return rate.quantity_step === 1_000_000
        ? t('USD / 1M input tokens')
        : t('USD / input token');
    case 'request':
      return t('USD / request');
    case 'image':
      return t('USD / image');
    case 'music_track':
      return t('USD / music track');
    default:
      return t('{count} x {unit}', {
        count: String(rate.quantity_step),
        unit: commercialPricingChargeUnitLabel(rate.charge_unit, t).toLowerCase(),
      });
  }
}

export function isCommercialPricingPlanEffectiveAt(
  plan: Pick<CommercialPricingPlanRecord, 'effective_from_ms' | 'effective_to_ms'>,
  nowMs = Date.now(),
): boolean {
  return plan.effective_from_ms <= nowMs
    && (plan.effective_to_ms == null || plan.effective_to_ms >= nowMs);
}

function compareCommercialPricingPlans(
  left: CommercialPricingPlanRecord,
  right: CommercialPricingPlanRecord,
  nowMs: number,
): number {
  const leftStatusRank = normalizeCommercialPricingPlanStatus(left.status) === 'active'
    ? (isCommercialPricingPlanEffectiveAt(left, nowMs) ? 0 : 1)
    : 2;
  const rightStatusRank = normalizeCommercialPricingPlanStatus(right.status) === 'active'
    ? (isCommercialPricingPlanEffectiveAt(right, nowMs) ? 0 : 1)
    : 2;

  return leftStatusRank - rightStatusRank
    || right.plan_version - left.plan_version
    || right.updated_at_ms - left.updated_at_ms
    || right.created_at_ms - left.created_at_ms
    || compareCommercialNumericIdsDesc(left.pricing_plan_id, right.pricing_plan_id);
}

export function compareCommercialPricingRates(
  left: CommercialPricingRateRecord,
  right: CommercialPricingRateRecord,
): number {
  const leftStatusRank = left.status.trim().toLowerCase() === 'active' ? 0 : 1;
  const rightStatusRank = right.status.trim().toLowerCase() === 'active' ? 0 : 1;

  return leftStatusRank - rightStatusRank
    || right.priority - left.priority
    || right.updated_at_ms - left.updated_at_ms
    || right.created_at_ms - left.created_at_ms
    || compareCommercialNumericIdsDesc(left.pricing_rate_id, right.pricing_rate_id);
}

export function selectPrimaryCommercialPricingPlan(
  pricingPlans: CommercialPricingPlanRecord[],
  nowMs = Date.now(),
): CommercialPricingPlanRecord | null {
  return [...pricingPlans].sort((left, right) => compareCommercialPricingPlans(left, right, nowMs))[0] ?? null;
}

export function countCurrentlyEffectiveCommercialPricingPlans(
  pricingPlans: CommercialPricingPlanRecord[],
  nowMs = Date.now(),
): number {
  return pricingPlans.filter(
    (plan) =>
      normalizeCommercialPricingPlanStatus(plan.status) === 'active'
      && isCommercialPricingPlanEffectiveAt(plan, nowMs),
  ).length;
}

export function selectPrimaryCommercialPricingRate(
  pricingRates: CommercialPricingRateRecord[],
  primaryPlan: CommercialPricingPlanRecord | null,
): CommercialPricingRateRecord | null {
  if (primaryPlan) {
    const primaryPlanRate = pricingRates
      .filter((rate) =>
        commercialNumericIdsEqual(rate.pricing_plan_id, primaryPlan.pricing_plan_id))
      .sort(compareCommercialPricingRates)[0];

    if (primaryPlanRate) {
      return primaryPlanRate;
    }
  }

  return [...pricingRates].sort(compareCommercialPricingRates)[0] ?? null;
}
