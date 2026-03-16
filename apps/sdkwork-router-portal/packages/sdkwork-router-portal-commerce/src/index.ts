import type { CouponOffer, RechargePack, SubscriptionPlan } from 'sdkwork-router-portal-types';

const subscriptionPlans: SubscriptionPlan[] = [
  {
    id: 'starter',
    name: 'Starter',
    price_label: '$19',
    cadence: '/month',
    included_units: 10000,
    highlight: 'For prototypes and lean internal tools',
    features: ['10k token units included', '2 live API keys', 'Email support'],
    cta: 'Start Starter',
    source: 'workspace_seed',
  },
  {
    id: 'growth',
    name: 'Growth',
    price_label: '$79',
    cadence: '/month',
    included_units: 100000,
    highlight: 'For production workloads and multi-environment delivery',
    features: ['100k token units included', '10 live API keys', 'Priority support'],
    cta: 'Upgrade to Growth',
    source: 'workspace_seed',
  },
  {
    id: 'scale',
    name: 'Scale',
    price_label: '$249',
    cadence: '/month',
    included_units: 500000,
    highlight: 'For platform teams optimizing predictable spend',
    features: ['500k token units included', 'Unlimited keys', 'Architecture advisory'],
    cta: 'Talk to Sales',
    source: 'workspace_seed',
  },
];

const rechargePacks: RechargePack[] = [
  {
    id: 'pack-25k',
    label: 'Boost 25k',
    points: 25000,
    price_label: '$12',
    note: 'Best for launch spikes and testing windows.',
    source: 'workspace_seed',
  },
  {
    id: 'pack-100k',
    label: 'Boost 100k',
    points: 100000,
    price_label: '$40',
    note: 'Designed for monthly usage expansion.',
    source: 'workspace_seed',
  },
  {
    id: 'pack-500k',
    label: 'Boost 500k',
    points: 500000,
    price_label: '$165',
    note: 'For scheduled releases and campaign traffic.',
    source: 'workspace_seed',
  },
];

const couponOffers: CouponOffer[] = [
  {
    code: 'WELCOME100',
    title: 'New workspace activation',
    benefit: '+100 starter points',
    description: 'Apply during onboarding to offset initial exploration traffic.',
    bonus_units: 100,
    source: 'workspace_seed',
  },
  {
    code: 'SPRINGBOOST',
    title: 'Campaign top-up',
    benefit: '10% off Growth',
    description: 'Use on the next subscription change for a temporary expansion window.',
    bonus_units: 10000,
    source: 'workspace_seed',
  },
  {
    code: 'TEAMREADY',
    title: 'Internal rollout bundle',
    benefit: 'Free staging credits',
    description: 'Unlocks extra staging budget for launch validation.',
    bonus_units: 25000,
    source: 'workspace_seed',
  },
];

export function listSubscriptionPlans(): SubscriptionPlan[] {
  return subscriptionPlans;
}

export function listRechargePacks(): RechargePack[] {
  return rechargePacks;
}

export function listCouponOffers(): CouponOffer[] {
  return couponOffers;
}

export function redeemSeedCoupon(code: string): CouponOffer | null {
  const normalized = code.trim().toUpperCase();
  return couponOffers.find((offer) => offer.code === normalized) ?? null;
}
