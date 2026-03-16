import { Pill } from 'sdkwork-router-portal-commons';

import type { BillingRecommendation } from '../types';

export function BillingRecommendationCard({
  recommendation,
}: {
  recommendation: BillingRecommendation;
}) {
  return (
    <div className="portalx-insight-card">
      <Pill tone="accent">Recommendation</Pill>
      <strong>{recommendation.title}</strong>
      <p>{recommendation.detail}</p>
      <div className="portalx-status-row">
        {recommendation.plan ? <Pill tone="positive">{recommendation.plan.name}</Pill> : null}
        {recommendation.pack ? <Pill tone="warning">{recommendation.pack.label}</Pill> : null}
      </div>
    </div>
  );
}
