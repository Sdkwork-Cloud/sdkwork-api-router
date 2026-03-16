import { useEffect, useMemo, useState } from 'react';
import { listRechargePacks, listSubscriptionPlans } from 'sdkwork-router-portal-commerce';
import {
  formatUnits,
  InlineButton,
  Pill,
  SectionHero,
  Surface,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { ProjectBillingSummary } from 'sdkwork-router-portal-types';

import { BillingRecommendationCard } from '../components';
import { loadBillingSummary } from '../repository';
import { isRecommendedPack, isRecommendedPlan, recommendBillingChange } from '../services';
import type { PortalBillingPageProps } from '../types';

const emptySummary: ProjectBillingSummary = {
  project_id: '',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  exhausted: false,
};

export function PortalBillingPage({ onNavigate }: PortalBillingPageProps) {
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [status, setStatus] = useState('Loading billing posture...');
  const [selectionStatus, setSelectionStatus] = useState(
    'Choose a plan or recharge path to model the next commerce step for this workspace.',
  );

  useEffect(() => {
    let cancelled = false;

    void loadBillingSummary()
      .then((nextSummary) => {
        if (cancelled) {
          return;
        }

        setSummary(nextSummary);
        setStatus('Live quota posture is paired with seed-backed plan and recharge catalogs.');
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const recommendation = useMemo(() => recommendBillingChange(summary), [summary]);

  return (
    <>
      <SectionHero
        detail="Present plans, recharges, and upgrade motion as a polished product flow while the live backend continues to own quota and ledger truth."
        eyebrow="Billing"
        title="Recharge and subscription"
      />

      <div className="portalx-status-row">
        <Pill tone="accent">Live quota: {summary.exhausted ? 'exhausted' : 'healthy'}</Pill>
        <Pill tone="seed">Catalog: workspace seed</Pill>
        <span className="portalx-status">{status}</span>
      </div>

      <Surface detail={selectionStatus} title="Decision support">
        <BillingRecommendationCard recommendation={recommendation} />
      </Surface>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface detail={recommendation.runway.detail} title="Estimated runway">
          <div className="portalx-readiness-score">
            <span>Projected coverage</span>
            <strong>{recommendation.runway.label}</strong>
          </div>
          <ul className="portalx-bullet-list">
            <li>Observed usage to date: {formatUnits(summary.used_units)} token units.</li>
            <li>
              Visible remaining quota:{' '}
              {summary.remaining_units === null || summary.remaining_units === undefined
                ? 'Unlimited'
                : formatUnits(summary.remaining_units)}
              .
            </li>
            <li>
              {recommendation.runway.daily_units === null
                ? 'The portal needs more live request history before it can infer a stronger daily burn pace.'
                : `Estimated burn pace: ${formatUnits(recommendation.runway.daily_units)} token units per day.`}
            </li>
          </ul>
        </Surface>

        <Surface detail={recommendation.bundle.detail} title="Recommended bundle">
          <div className="portalx-bundle-card">
            <Pill tone="positive">{recommendation.plan?.name ?? 'Subscription'}</Pill>
            <strong>{recommendation.bundle.title}</strong>
            <p>{recommendation.detail}</p>
            <ul className="portalx-fact-list">
              <li>
                <strong>Subscription</strong>
                <span>
                  {recommendation.plan
                    ? `${recommendation.plan.name} · ${recommendation.plan.price_label}`
                    : 'No plan recommendation'}
                </span>
              </li>
              <li>
                <strong>Included units</strong>
                <span>
                  {recommendation.plan
                    ? formatUnits(recommendation.plan.included_units)
                    : 'Unavailable'}
                </span>
              </li>
              <li>
                <strong>Recharge buffer</strong>
                <span>
                  {recommendation.pack
                    ? `${recommendation.pack.label} · ${formatUnits(recommendation.pack.points)}`
                    : 'Optional'}
                </span>
              </li>
            </ul>
          </div>
        </Surface>
      </div>

      <Surface detail={selectionStatus} title="Subscription plans">
        <div className="portalx-plan-grid">
          {listSubscriptionPlans().map((plan) => (
            <article className={`portalx-plan-card ${isRecommendedPlan(plan, recommendation) ? 'portalx-plan-card-featured' : ''}`} key={plan.id}>
              <p className="portalx-eyebrow">{plan.name}</p>
              <h3>
                {plan.price_label}
                <span>{plan.cadence}</span>
              </h3>
              <p>{plan.highlight}</p>
              <Pill tone={isRecommendedPlan(plan, recommendation) ? 'positive' : 'default'}>
                {formatUnits(plan.included_units)} included units
              </Pill>
              <ul className="portalx-bullet-list">
                {plan.features.map((feature) => (
                  <li key={feature}>{feature}</li>
                ))}
              </ul>
              <InlineButton
                onClick={() => setSelectionStatus(`${plan.name} selected for checkout preview.`)}
                tone="primary"
              >
                {plan.cta}
              </InlineButton>
            </article>
          ))}
        </div>
      </Surface>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface detail="Use packs to extend quota without changing the base subscription." title="Recharge packs">
          <div className="portalx-pack-grid">
            {listRechargePacks().map((pack) => (
              <article className={`portalx-pack-card ${isRecommendedPack(pack, recommendation) ? 'portalx-pack-card-featured' : ''}`} key={pack.id}>
                <strong>{pack.label}</strong>
                <span>{formatUnits(pack.points)} points</span>
                <p>{pack.price_label}</p>
                <small>{pack.note}</small>
                <InlineButton
                  onClick={() => setSelectionStatus(`${pack.label} selected for recharge preview.`)}
                  tone="secondary"
                >
                  Add pack
                </InlineButton>
              </article>
            ))}
          </div>
        </Surface>

        <Surface detail="Trust, clarity, and user confidence matter as much as price tables." title="Billing confidence">
          <ul className="portalx-bullet-list">
            <li>Quota posture is pulled from the live workspace billing summary.</li>
            <li>Subscription and recharge catalogs are isolated behind a repository seam for future checkout integration.</li>
            <li>Current recorded usage: {formatUnits(summary.used_units)} token units.</li>
            <li>Remaining quota: {formatUnits(summary.remaining_units ?? 0)} token units.</li>
          </ul>
        </Surface>
      </div>

      <Surface
        detail="After choosing a commercial path, the user journey should continue into validation, not stop at the price table."
        title="Activation path"
      >
        <div className="portalx-checklist-grid">
          <article className="portalx-checklist-card">
            <strong>Return to dashboard and confirm posture</strong>
            <p>Use the command center to verify runway, readiness, and the updated next action after a billing decision.</p>
            <InlineButton onClick={() => onNavigate('dashboard')} tone="primary">
              Open dashboard
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Check request demand against the selected plan</strong>
            <p>Go back to Usage when you want to verify that the observed burn pace really matches the selected bundle.</p>
            <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
              Open usage
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Keep credentials ready for the next launch window</strong>
            <p>After restoring runway, verify environment keys so billing recovery immediately turns into safe traffic activation.</p>
            <InlineButton onClick={() => onNavigate('api-keys')} tone="ghost">
              Manage keys
            </InlineButton>
          </article>
        </div>
      </Surface>
    </>
  );
}
