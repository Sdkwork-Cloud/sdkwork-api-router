import {
  compareCommercialNumericIdsDesc,
  commercialNumericIdsEqual,
} from 'sdkwork-router-portal-types';
import type {
  CommercialAccountBenefitLotRecord,
  CommercialAccountHoldRecord,
  CommercialPricingPlanRecord,
  CommercialPricingRateRecord,
  CommercialRequestSettlementRecord,
} from 'sdkwork-router-portal-types';

import type {
  BuildPortalSettlementsViewModelInput,
  PortalSettlementsViewModel,
} from '../types';

function isActiveBenefitLot(lot: CommercialAccountBenefitLotRecord): boolean {
  return lot.status === 'active';
}

function isExpiredBenefitLot(lot: CommercialAccountBenefitLotRecord): boolean {
  return lot.status === 'expired';
}

function isOpenCommercialHold(hold: CommercialAccountHoldRecord): boolean {
  return hold.status === 'held'
    || hold.status === 'captured'
    || hold.status === 'partially_released';
}

function compareSettlements(
  left: CommercialRequestSettlementRecord,
  right: CommercialRequestSettlementRecord,
): number {
  return right.settled_at_ms - left.settled_at_ms
    || right.updated_at_ms - left.updated_at_ms
    || compareCommercialNumericIdsDesc(left.request_settlement_id, right.request_settlement_id);
}

function compareHolds(
  left: CommercialAccountHoldRecord,
  right: CommercialAccountHoldRecord,
): number {
  return right.created_at_ms - left.created_at_ms
    || right.updated_at_ms - left.updated_at_ms
    || compareCommercialNumericIdsDesc(left.hold_id, right.hold_id);
}

function compareBenefitLots(
  left: CommercialAccountBenefitLotRecord,
  right: CommercialAccountBenefitLotRecord,
): number {
  return right.remaining_quantity - left.remaining_quantity
    || right.priority - left.priority
    || compareCommercialNumericIdsDesc(left.lot_id, right.lot_id);
}

function isPricingPlanEffectiveAt(
  plan: Pick<CommercialPricingPlanRecord, 'effective_from_ms' | 'effective_to_ms'>,
  nowMs: number,
): boolean {
  return plan.effective_from_ms <= nowMs
    && (plan.effective_to_ms == null || plan.effective_to_ms >= nowMs);
}

function selectPrimaryPricingPlan(
  pricingPlans: CommercialPricingPlanRecord[],
  nowMs: number,
): CommercialPricingPlanRecord | null {
  const comparePlans = (
    left: CommercialPricingPlanRecord,
    right: CommercialPricingPlanRecord,
  ): number => {
    const leftRank = left.status.trim().toLowerCase() === 'active'
      ? (isPricingPlanEffectiveAt(left, nowMs) ? 0 : 1)
      : 2;
    const rightRank = right.status.trim().toLowerCase() === 'active'
      ? (isPricingPlanEffectiveAt(right, nowMs) ? 0 : 1)
      : 2;

    return leftRank - rightRank
      || right.plan_version - left.plan_version
      || right.updated_at_ms - left.updated_at_ms
      || right.created_at_ms - left.created_at_ms
      || compareCommercialNumericIdsDesc(left.pricing_plan_id, right.pricing_plan_id);
  };

  return [...pricingPlans].sort(comparePlans)[0] ?? null;
}

function selectPrimaryPricingRate(
  pricingRates: CommercialPricingRateRecord[],
  primaryPlan: CommercialPricingPlanRecord | null,
): CommercialPricingRateRecord | null {
  const compareRates = (
    left: CommercialPricingRateRecord,
    right: CommercialPricingRateRecord,
  ): number => {
    const leftStatusRank = left.status.trim().toLowerCase() === 'active' ? 0 : 1;
    const rightStatusRank = right.status.trim().toLowerCase() === 'active' ? 0 : 1;

    return leftStatusRank - rightStatusRank
      || right.priority - left.priority
      || right.updated_at_ms - left.updated_at_ms
      || right.created_at_ms - left.created_at_ms
      || compareCommercialNumericIdsDesc(left.pricing_rate_id, right.pricing_rate_id);
  };

  if (primaryPlan) {
    const planRate = pricingRates
      .filter((rate) =>
        commercialNumericIdsEqual(rate.pricing_plan_id, primaryPlan.pricing_plan_id))
      .sort(compareRates)[0];
    if (planRate) {
      return planRate;
    }
  }

  return [...pricingRates].sort(compareRates)[0] ?? null;
}

export function buildPortalSettlementsViewModel(
  input: BuildPortalSettlementsViewModelInput,
): PortalSettlementsViewModel {
  const commercialAccount = input.commercialAccount ?? null;
  const accountBalance = input.accountBalance ?? null;
  const benefitLots = [...(input.benefitLots ?? [])];
  const holds = [...(input.holds ?? [])];
  const requestSettlements = [...(input.requestSettlements ?? [])].sort(compareSettlements);
  const pricingPlans = [...(input.pricingPlans ?? [])];
  const pricingRates = [...(input.pricingRates ?? [])];
  const openHolds = holds.filter(isOpenCommercialHold).sort(compareHolds);
  const activeBenefitLots = benefitLots.filter(isActiveBenefitLot).sort(compareBenefitLots);
  const expiredBenefitLotCount = benefitLots.filter(isExpiredBenefitLot).length;
  const primaryPlan = selectPrimaryPricingPlan(pricingPlans, Date.now());
  const primaryRate = selectPrimaryPricingRate(pricingRates, primaryPlan);
  const capturedSettlementCount = requestSettlements.filter(
    (settlement) => settlement.status === 'captured',
  ).length;
  const refundedSettlementCount = requestSettlements.filter(
    (settlement) => settlement.status === 'refunded',
  ).length;

  return {
    account_id: commercialAccount?.account.account_id ?? accountBalance?.account_id ?? null,
    account_status: commercialAccount?.account.status ?? null,
    available_balance:
      accountBalance?.available_balance ?? commercialAccount?.available_balance ?? 0,
    held_balance: accountBalance?.held_balance ?? commercialAccount?.held_balance ?? 0,
    grant_balance: accountBalance?.grant_balance ?? commercialAccount?.grant_balance ?? 0,
    active_benefit_lot_count: activeBenefitLots.length,
    expired_benefit_lot_count: expiredBenefitLotCount,
    open_hold_count: openHolds.length,
    settlement_count: requestSettlements.length,
    captured_settlement_count: capturedSettlementCount,
    refunded_settlement_count: refundedSettlementCount,
    captured_credit_amount: requestSettlements.reduce(
      (sum, settlement) => sum + settlement.captured_credit_amount,
      0,
    ),
    refunded_credit_amount: requestSettlements.reduce(
      (sum, settlement) => sum + settlement.refunded_amount,
      0,
    ),
    primary_plan_display_name: primaryPlan?.display_name ?? null,
    primary_rate_metric_code: primaryRate?.metric_code ?? null,
    primary_rate_charge_unit: primaryRate?.charge_unit ?? null,
    primary_rate_pricing_method: primaryRate?.pricing_method ?? null,
    primary_rate_display_price_unit: primaryRate?.display_price_unit ?? null,
    priced_metric_count: new Set(pricingRates.map((rate) => rate.metric_code)).size,
    latest_settlements: requestSettlements.slice(0, 6),
    request_settlements: requestSettlements,
    open_holds: openHolds,
    active_benefit_lots: activeBenefitLots.slice(0, 6),
    pricing_plans: pricingPlans,
    pricing_rates: pricingRates,
  };
}
