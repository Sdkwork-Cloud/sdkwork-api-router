import { useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import { listCouponOffers, redeemSeedCoupon } from 'sdkwork-router-portal-commerce';
import {
  DataTable,
  EmptyState,
  formatCurrency,
  formatUnits,
  InlineButton,
  MetricCard,
  Pill,
  SectionHero,
  Surface,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { CouponOffer, LedgerEntry, ProjectBillingSummary } from 'sdkwork-router-portal-types';

import { CouponImpactCard } from '../components';
import { loadCreditsPageData } from '../repository';
import {
  buildCouponImpactPreview,
  buildRecommendedCouponOffer,
  buildRedemptionGuardrails,
  recommendCouponOffer,
} from '../services';
import type { PortalCreditsPageProps } from '../types';

const emptySummary: ProjectBillingSummary = {
  project_id: '',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  exhausted: false,
};

export function PortalCreditsPage({ onNavigate }: PortalCreditsPageProps) {
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [couponCode, setCouponCode] = useState('');
  const [selectedOffer, setSelectedOffer] = useState<CouponOffer | null>(null);
  const [couponStatus, setCouponStatus] = useState('Redeem workspace offers and keep points posture visible before traffic spikes.');
  const [status, setStatus] = useState('Loading points posture...');

  useEffect(() => {
    let cancelled = false;

    void loadCreditsPageData()
      .then((data) => {
        if (cancelled) {
          return;
        }

        setSummary(data.summary);
        setLedger(data.ledger);
        setSelectedOffer(recommendCouponOffer(data.summary));
        setStatus('Live billing data is mapped into a points-oriented portal view.');
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

  function handleCouponSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const offer = redeemSeedCoupon(couponCode);
    if (!offer) {
      setCouponStatus('Coupon code not recognized in the current seeded commerce catalog.');
      setSelectedOffer(null);
      return;
    }

    setSelectedOffer(offer);
    setCouponStatus(`${offer.code} accepted for preview: ${offer.benefit}. Backend redemption can replace this seam without changing the UI contract.`);
  }

  const remainingUnits = summary.remaining_units ?? 0;
  const couponPreview = useMemo(
    () => (selectedOffer ? buildCouponImpactPreview(summary, selectedOffer) : null),
    [selectedOffer, summary],
  );
  const recommendedOffer = useMemo(() => buildRecommendedCouponOffer(summary), [summary]);
  const guardrails = useMemo(() => buildRedemptionGuardrails(summary), [summary]);

  return (
    <>
      <SectionHero
        detail="Translate quota and ledger data into a clear points view, then expose coupon redemption as the front door for workspace growth mechanics."
        eyebrow="Credits"
        title="Points and quota posture"
      />

      <div className="portalx-status-row">
        <Pill tone="accent">Quota {summary.exhausted ? 'exhausted' : 'available'}</Pill>
        <Pill tone="seed">Coupon catalog: seed-backed</Pill>
        <span className="portalx-status">{status}</span>
      </div>

      <div className="portalx-metric-grid">
        <MetricCard
          detail="Remaining token units within the current quota boundary."
          label="Available Points"
          value={formatUnits(remainingUnits)}
        />
        <MetricCard
          detail="Consumed token units recorded for this project."
          label="Used Points"
          value={formatUnits(summary.used_units)}
        />
        <MetricCard
          detail="Ledger entries currently visible in the billing read model."
          label="Ledger Entries"
          value={formatUnits(summary.entry_count)}
        />
        <MetricCard
          detail="Booked amount associated with usage to date."
          label="Booked Amount"
          value={formatCurrency(summary.booked_amount)}
        />
      </div>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface detail={recommendedOffer.rationale} title="Recommended offer">
          <CouponImpactCard preview={recommendedOffer.preview} />
          <ul className="portalx-fact-list">
            <li>
              <strong>Coupon code</strong>
              <span>{recommendedOffer.offer.code}</span>
            </li>
            <li>
              <strong>Benefit</strong>
              <span>{recommendedOffer.offer.benefit}</span>
            </li>
            <li>
              <strong>Best for</strong>
              <span>{recommendedOffer.offer.description}</span>
            </li>
          </ul>
        </Surface>

        <Surface
          detail="Use coupons as a productized quota extension path, but keep the rules visible so launch motion stays safe."
          title="Redemption guardrails"
        >
          <div className="portalx-guardrail-list">
            {guardrails.map((guardrail) => (
              <article className="portalx-guardrail-card" key={guardrail.id}>
                <Pill tone={guardrail.tone}>{guardrail.title}</Pill>
                <p>{guardrail.detail}</p>
              </article>
            ))}
          </div>
        </Surface>
      </div>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface detail={couponStatus} title="Redeem a coupon">
          <form className="portalx-form portalx-form-card" onSubmit={handleCouponSubmit}>
            <label className="portalx-field">
              <span>Coupon code</span>
              <input
                onChange={(event) => setCouponCode(event.target.value)}
                placeholder="WELCOME100"
                value={couponCode}
              />
            </label>
            <div className="portalx-form-actions">
              <InlineButton tone="primary" type="submit">
                Redeem preview
              </InlineButton>
            </div>
          </form>
          {couponPreview ? <CouponImpactCard preview={couponPreview} /> : null}
        </Surface>

        <Surface detail="Current workspace offers prepared behind a commerce repository seam." title="Available offers">
          <ul className="portalx-offer-list">
            {listCouponOffers().map((offer) => (
              <li key={offer.code}>
                <strong>{offer.title}</strong>
                <span>{offer.benefit}</span>
                <p>{offer.description}</p>
              </li>
            ))}
          </ul>
        </Surface>
      </div>

      <Surface detail="Ledger entries are sourced from the live portal billing boundary." title="Points ledger">
        {ledger.length ? (
          <DataTable
            columns={[
              { key: 'units', label: 'Units', render: (row) => formatUnits(row.units) },
              { key: 'amount', label: 'Amount', render: (row) => formatCurrency(row.amount) },
              { key: 'project', label: 'Project', render: (row) => row.project_id },
            ]}
            empty="No points ledger entries recorded yet."
            getKey={(row, index) => `${row.project_id}-${row.units}-${index}`}
            rows={ledger}
          />
        ) : (
          <EmptyState
            detail="Once requests are billed, the points ledger will provide a transaction-style view here."
            title="No ledger entries yet"
          />
        )}
      </Surface>

      <Surface
        detail="Credits should flow naturally into the next commercial decision instead of ending at coupon redemption."
        title="Recharge decision"
      >
        <div className="portalx-checklist-grid">
          <article className="portalx-checklist-card">
            <strong>Escalate to billing when coupon support is not enough</strong>
            <p>Use subscriptions and recharge packs when demand has moved beyond one-off promotional top-ups.</p>
            <InlineButton onClick={() => onNavigate('billing')} tone="primary">
              Review billing
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Return to dashboard for launch posture</strong>
            <p>After redeeming or evaluating an offer, verify that readiness, quota, and action queue all move into a safer state.</p>
            <InlineButton onClick={() => onNavigate('dashboard')} tone="secondary">
              Open dashboard
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Inspect usage before choosing the next commercial step</strong>
            <p>Use request history and token burn to decide whether a coupon, a pack, or a subscription is the right move.</p>
            <InlineButton onClick={() => onNavigate('usage')} tone="ghost">
              Open usage
            </InlineButton>
          </article>
        </div>
      </Surface>
    </>
  );
}
