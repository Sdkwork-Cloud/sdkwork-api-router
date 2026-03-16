import { InlineButton, Pill } from 'sdkwork-router-portal-commons';
import type { PortalRouteKey } from 'sdkwork-router-portal-types';

import type { DashboardInsight, DashboardReadinessItem } from '../types';

export function DashboardInsights({
  insights,
  onNavigate,
}: {
  insights: DashboardInsight[];
  onNavigate: (route: PortalRouteKey) => void;
}) {
  return (
    <div className="portalx-insight-grid">
      {insights.map((insight) => (
        <article className="portalx-insight-card" key={insight.id}>
          <Pill tone={insight.tone}>{insight.title}</Pill>
          <p>{insight.detail}</p>
          {insight.route && insight.action_label ? (
            <InlineButton
              onClick={() => {
                if (insight.route) {
                  onNavigate(insight.route);
                }
              }}
              tone="ghost"
            >
              {insight.action_label}
            </InlineButton>
          ) : null}
        </article>
      ))}
    </div>
  );
}

export function DashboardReadiness({
  items,
}: {
  items: DashboardReadinessItem[];
}) {
  return (
    <div className="portalx-summary-grid">
      {items.map((item) => (
        <article className="portalx-summary-card" key={item.id}>
          <span>{item.label}</span>
          <strong>{item.value}</strong>
          <p>{item.detail}</p>
        </article>
      ))}
    </div>
  );
}
