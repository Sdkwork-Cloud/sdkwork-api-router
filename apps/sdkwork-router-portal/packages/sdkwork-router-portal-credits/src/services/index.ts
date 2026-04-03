import { formatUnits } from 'sdkwork-router-portal-commons/format-core';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';
import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  PortalCommerceCoupon,
  PortalCommerceOrder,
  PortalCommerceQuote,
  PortalWorkspaceSummary,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import type {
  CouponImpactPreview,
  CreditsGuardrail,
  PortalCreditsFinanceProjection,
  PortalCreditsRedemptionCoverage,
  RecommendedCouponOffer,
  RedeemInviteProgram,
  RedeemInviteRow,
} from '../types';

function sortCouponsByImpact(coupons: PortalCommerceCoupon[]): PortalCommerceCoupon[] {
  return coupons
    .slice()
    .sort(
      (left, right) =>
        right.bonus_units - left.bonus_units || left.code.localeCompare(right.code),
    );
}

function emptyBillingEventSummary(): BillingEventSummary {
  return {
    total_events: 0,
    project_count: 0,
    group_count: 0,
    capability_count: 0,
    total_request_count: 0,
    total_units: 0,
    total_input_tokens: 0,
    total_output_tokens: 0,
    total_tokens: 0,
    total_image_count: 0,
    total_audio_seconds: 0,
    total_video_seconds: 0,
    total_music_seconds: 0,
    total_upstream_cost: 0,
    total_customer_charge: 0,
    projects: [],
    groups: [],
    capabilities: [],
    accounting_modes: [],
  };
}

function sortAccountingModes(
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

function sortCapabilities(
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

function buildRedemptionCoverage(
  summary: ProjectBillingSummary,
  orders: PortalCommerceOrder[],
): PortalCreditsRedemptionCoverage {
  const fulfilledRedemptions = orders.filter(
    (order) => order.target_kind === 'coupon_redemption' && order.status === 'fulfilled',
  );
  const granted_units = fulfilledRedemptions.reduce(
    (sum, order) => sum + order.granted_units,
    0,
  );
  const bonus_units = fulfilledRedemptions.reduce((sum, order) => sum + order.bonus_units, 0);
  const redeemed_units = granted_units + bonus_units;
  const remainingUnits = summary.remaining_units;
  const lowRunway = remainingUnits !== null && remainingUnits !== undefined && remainingUnits < 5_000;
  const next_funding_path =
    summary.exhausted || (lowRunway && redeemed_units > 0 && summary.used_units >= redeemed_units)
      ? 'recharge'
      : 'redeem';

  return {
    fulfilled_redemptions: fulfilledRedemptions.length,
    granted_units,
    bonus_units,
    next_funding_path,
  };
}

export function buildPortalCreditsFinanceProjection(input: {
  summary: ProjectBillingSummary;
  orders: PortalCommerceOrder[];
  billingEventSummary: BillingEventSummary | null | undefined;
}): PortalCreditsFinanceProjection {
  const billingSummary = input.billingEventSummary ?? emptyBillingEventSummary();

  return {
    redemption_coverage: buildRedemptionCoverage(input.summary, input.orders),
    leading_accounting_mode: sortAccountingModes(billingSummary.accounting_modes)[0] ?? null,
    leading_capability: sortCapabilities(billingSummary.capabilities)[0] ?? null,
    multimodal_totals: {
      image_count: billingSummary.total_image_count,
      audio_seconds: billingSummary.total_audio_seconds,
      video_seconds: billingSummary.total_video_seconds,
      music_seconds: billingSummary.total_music_seconds,
    },
  };
}

export function recommendCouponOffer(
  summary: ProjectBillingSummary,
  coupons: PortalCommerceCoupon[],
): PortalCommerceCoupon | null {
  if (!coupons.length) {
    return null;
  }

  const rankedCoupons = sortCouponsByImpact(coupons);

  if (summary.exhausted || (summary.remaining_units ?? 0) < 5_000) {
    return rankedCoupons[0];
  }

  if (summary.used_units === 0) {
    return coupons.find((coupon) => coupon.code === 'WELCOME100') ?? rankedCoupons[0];
  }

  return coupons.find((coupon) => coupon.code === 'TEAMREADY') ?? rankedCoupons[0];
}

export function buildCouponImpactPreview(
  coupon: PortalCommerceCoupon,
  quote: PortalCommerceQuote,
): CouponImpactPreview {
  const projected_remaining_units =
    quote.projected_remaining_units === null || quote.projected_remaining_units === undefined
      ? null
      : quote.projected_remaining_units;

  return {
    coupon,
    quote,
    status:
      projected_remaining_units === null
        ? translatePortalText(
          '{code} would add {units} bonus units on top of an unlimited quota posture.',
          {
            code: coupon.code,
            units: formatUnits(quote.bonus_units),
          },
        )
        : translatePortalText(
          '{code} would increase visible remaining units to {units}.',
          {
            code: coupon.code,
            units: formatUnits(projected_remaining_units),
          },
        ),
  };
}

export function buildRecommendedCouponOffer(
  summary: ProjectBillingSummary,
  coupons: PortalCommerceCoupon[],
  quote: PortalCommerceQuote,
): RecommendedCouponOffer | null {
  const offer = recommendCouponOffer(summary, coupons);
  if (!offer) {
    return null;
  }

  let rationale = translatePortalText(
    'This offer is the cleanest fit for the current workspace posture.',
  );

  if (summary.exhausted) {
    rationale =
      translatePortalText(
        'Quota is exhausted, so the portal recommends the highest-impact coupon path before the next launch window.',
      );
  } else if ((summary.remaining_units ?? 0) < 5_000) {
    rationale =
      translatePortalText(
        'Remaining points are low, so the recommended offer prioritizes restoring a safer launch buffer.',
      );
  } else if (summary.used_units === 0) {
    rationale =
      translatePortalText(
        'No usage has been recorded yet, so the portal recommends a first-run offer that lowers the cost of initial experimentation.',
      );
  }

  return {
    offer,
    rationale,
    preview: buildCouponImpactPreview(offer, quote),
  };
}

export function buildRedemptionGuardrails(summary: ProjectBillingSummary): CreditsGuardrail[] {
  const guardrails: CreditsGuardrail[] = [];
  const remainingUnits = summary.remaining_units ?? 0;

  if (summary.exhausted) {
    guardrails.push({
      id: 'restore-before-launch',
      title: translatePortalText('Restore quota before new traffic is scheduled'),
      detail: translatePortalText(
        'When visible quota is exhausted, redeeming a coupon should happen before the next test or production launch window.',
      ),
      tone: 'warning',
    });
  } else if (remainingUnits < 5_000) {
    guardrails.push({
      id: 'protect-buffer',
      title: translatePortalText('Treat coupons as a launch buffer, not a surprise fix'),
      detail: translatePortalText(
        'Only {units} units remain. Redeem before runway becomes operationally tight.',
        { units: formatUnits(remainingUnits) },
      ),
      tone: 'warning',
    });
  } else {
    guardrails.push({
      id: 'redeem-with-intent',
      title: translatePortalText('Redeem with a clear demand event in mind'),
      detail: translatePortalText(
        'Current quota posture is healthy, so coupon usage should align with an onboarding push, load test, or growth moment.',
      ),
      tone: 'success',
    });
  }

  if (summary.used_units === 0) {
    guardrails.push({
      id: 'pair-with-first-request',
      title: translatePortalText('Pair the first coupon with the first real request'),
      detail: translatePortalText(
        'The cleanest user experience is to unlock telemetry and bonus points together in the first launch path.',
      ),
      tone: 'default',
    });
  } else {
    guardrails.push({
      id: 'watch-burn-rate',
      title: translatePortalText('Match redemption to observed consumption'),
      detail: translatePortalText(
        'The workspace has already consumed {units} units, so coupon choice should follow actual burn instead of guesswork.',
        { units: formatUnits(summary.used_units) },
      ),
      tone: 'default',
    });
  }

  guardrails.push({
    id: 'checkout-boundary',
    title: translatePortalText('Escalate to billing when coupons stop being enough'),
    detail: translatePortalText(
      'Coupons are a controlled top-up path. Persistent growth should move into recharge packs or subscriptions instead of repeated redemption.',
    ),
    tone: 'secondary',
  });

  return guardrails;
}

function sanitizeToken(value: string | null | undefined): string {
  return (value ?? '')
    .trim()
    .replace(/[^a-zA-Z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .toUpperCase();
}

export function buildRedeemInviteProgram(
  workspace: PortalWorkspaceSummary | null | undefined,
): RedeemInviteProgram {
  const projectName = workspace?.project.name?.trim() || translatePortalText('Portal workspace');
  const tenantToken = sanitizeToken(workspace?.tenant.id).slice(0, 4) || 'TEAM';
  const projectToken = sanitizeToken(workspace?.project.id).slice(0, 4) || 'FLOW';
  const inviteCode = `${tenantToken}-${projectToken}`;
  const projectSlug = sanitizeToken(workspace?.project.name).toLowerCase() || 'portal-workspace';

  return {
    code: inviteCode,
    link: `https://sdkwork.ai/invite/${projectSlug}?code=${inviteCode}`,
    owner_label: workspace?.user.display_name || workspace?.user.email || projectName,
    audience_label: translatePortalText('Operators, finance owners, and new workspace activations'),
  };
}

export function buildRedeemInviteRows(
  workspace: PortalWorkspaceSummary | null | undefined,
): RedeemInviteRow[] {
  const projectName = workspace?.project.name?.trim() || translatePortalText('Portal workspace');

  return [
    {
      id: 'invite-aurora',
      workspace_name: `${projectName} Growth Pod`,
      contact: 'growth@aurora.ai',
      invited_at_ms: Date.UTC(2026, 2, 28, 9, 20),
      activated_at_ms: null,
      reward_units: 18_000,
      reward_state: 'pending',
    },
    {
      id: 'invite-ops',
      workspace_name: `${projectName} Ops Desk`,
      contact: 'ops@northwind.dev',
      invited_at_ms: Date.UTC(2026, 2, 24, 11, 10),
      activated_at_ms: Date.UTC(2026, 2, 26, 8, 45),
      reward_units: 30_000,
      reward_state: 'rewarded',
    },
    {
      id: 'invite-finance',
      workspace_name: `${projectName} Finance Hub`,
      contact: 'finance@contoso.cloud',
      invited_at_ms: Date.UTC(2026, 2, 30, 15, 5),
      activated_at_ms: null,
      reward_units: 12_000,
      reward_state: 'pending',
    },
    {
      id: 'invite-studio',
      workspace_name: `${projectName} Studio Team`,
      contact: 'studio@fabrikam.design',
      invited_at_ms: Date.UTC(2026, 2, 21, 13, 30),
      activated_at_ms: Date.UTC(2026, 2, 22, 17, 20),
      reward_units: 24_000,
      reward_state: 'rewarded',
    },
  ];
}
