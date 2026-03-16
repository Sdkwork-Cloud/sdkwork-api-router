import { listRechargePacks, listSubscriptionPlans } from 'sdkwork-router-portal-commerce';
import { formatUnits } from 'sdkwork-router-portal-commons';
import type { ProjectBillingSummary, RechargePack, SubscriptionPlan } from 'sdkwork-router-portal-types';

import type { BillingRecommendation } from '../types';

function estimateDailyUnits(summary: ProjectBillingSummary): number | null {
  if (summary.used_units <= 0) {
    return null;
  }

  return Math.max(Math.ceil(summary.used_units / 30), 250);
}

function buildRunway(summary: ProjectBillingSummary): BillingRecommendation['runway'] {
  const daily_units = estimateDailyUnits(summary);

  if (summary.exhausted) {
    return {
      label: '0 days',
      detail: 'Visible quota is already exhausted, so the workspace needs an immediate recharge or plan change before additional traffic is expected.',
      projected_days: 0,
      daily_units,
    };
  }

  if (summary.remaining_units === null || summary.remaining_units === undefined) {
    return {
      label: 'Unlimited',
      detail: 'The current billing summary exposes no visible quota ceiling, so the portal treats runway as unlimited for this workspace.',
      projected_days: null,
      daily_units,
    };
  }

  if (!daily_units) {
    return {
      label: 'Needs first traffic signal',
      detail: 'There is not enough recorded usage yet to project a meaningful burn pace. Send live traffic, then revisit billing decisions.',
      projected_days: null,
      daily_units: null,
    };
  }

  const projected_days = Math.floor(summary.remaining_units / daily_units);
  const label = projected_days < 1 ? '< 1 day' : `${projected_days} days`;

  return {
    label,
    detail: `Estimated from the current observed burn pace of ${formatUnits(daily_units)} token units per day.`,
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
      title: 'Billing catalog unavailable',
      detail: 'The portal could not build a plan-plus-pack recommendation from the current seed catalog.',
    };
  }

  if (summary.exhausted) {
    return {
      title: `${plan?.name ?? 'Subscription'} + ${pack?.label ?? 'Recharge pack'}`,
      detail: 'The workspace needs both immediate runway recovery and a steadier monthly posture, so the portal recommends a plan and a recharge together.',
    };
  }

  if ((summary.remaining_units ?? 0) < 10_000) {
    return {
      title: `${plan?.name ?? 'Subscription'} with ${pack?.label ?? 'Recharge pack'} as buffer`,
      detail: 'Current quota is still active, but remaining headroom is tight enough that a plan-plus-buffer path is the lowest-friction next move.',
    };
  }

  return {
    title: `${plan?.name ?? 'Subscription'} as the next growth step`,
    detail: 'The workspace is stable today, so the recommended bundle focuses on the cleanest subscription path while keeping the top-up pack available only if demand spikes.',
  };
}

export function recommendBillingChange(summary: ProjectBillingSummary): BillingRecommendation {
  const plans = listSubscriptionPlans();
  const packs = listRechargePacks();
  const currentDemand = Math.max(summary.used_units, 1);
  const recommendedPlan = plans.find((plan) => plan.included_units >= currentDemand) ?? plans[plans.length - 1];
  const recommendedPack = packs.find((pack) => pack.points >= Math.max(10_000, summary.used_units / 2)) ?? packs[packs.length - 1];
  const runway = buildRunway(summary);
  const bundle = buildRecommendedBundle(summary, recommendedPlan, recommendedPack);

  if (summary.exhausted) {
    return {
      title: 'Quota is exhausted',
      detail: `Move to ${recommendedPlan.name} or add ${recommendedPack.label} to restore headroom immediately.`,
      plan: recommendedPlan,
      pack: recommendedPack,
      runway,
      bundle,
    };
  }

  if ((summary.remaining_units ?? 0) < 10_000) {
    return {
      title: 'Headroom is getting tight',
      detail: `Add ${recommendedPack.label} for near-term coverage, or move to ${recommendedPlan.name} for a steadier monthly posture.`,
      plan: recommendedPlan,
      pack: recommendedPack,
      runway,
      bundle,
    };
  }

  return {
    title: 'Current workspace is stable',
    detail: `Based on ${formatUnits(summary.used_units)} used units, ${recommendedPlan.name} is the cleanest next subscription step when traffic grows.`,
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
