import { InlineButton, Pill, SectionHero, StatCard, Surface } from 'sdkwork-router-admin-commons';
import type { AdminPageProps, AdminRouteKey } from 'sdkwork-router-admin-types';

function topPortalUsers(snapshot: AdminPageProps['snapshot']) {
  return [...snapshot.portalUsers]
    .sort((left, right) => (
      right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || right.usage_units - left.usage_units
    ))
    .slice(0, 5);
}

function hottestProjects(snapshot: AdminPageProps['snapshot']) {
  const tokensByProject = new Map<string, number>();
  for (const record of snapshot.usageRecords) {
    tokensByProject.set(
      record.project_id,
      (tokensByProject.get(record.project_id) ?? 0) + record.total_tokens,
    );
  }

  return snapshot.projects
    .map((project) => {
      const traffic = snapshot.usageSummary.projects.find(
        (summary) => summary.project_id === project.id,
      );
      const billing = snapshot.billingSummary.projects.find(
        (summary) => summary.project_id === project.id,
      );

      return {
        ...project,
        request_count: traffic?.request_count ?? 0,
        total_tokens: tokensByProject.get(project.id) ?? 0,
        booked_amount: billing?.booked_amount ?? 0,
      };
    })
    .sort((left, right) => (
      right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || right.booked_amount - left.booked_amount
    ))
    .slice(0, 5);
}

export function OverviewPage({
  snapshot,
  onNavigate,
}: AdminPageProps & { onNavigate: (route: AdminRouteKey) => void }) {
  const rankedUsers = topPortalUsers(snapshot);
  const rankedProjects = hottestProjects(snapshot);

  return (
    <div className="adminx-page-grid">
      <SectionHero
        eyebrow="Overview"
        title="Run daily router operations from one command surface."
        detail="Track platform posture, inspect risk, and jump directly into the management module that needs attention."
        actions={
          <>
            <InlineButton tone="primary" onClick={() => onNavigate('users')}>
              Review users
            </InlineButton>
            <InlineButton onClick={() => onNavigate('operations')}>Check runtime</InlineButton>
          </>
        }
      />

      <section className="adminx-stat-grid">
        {snapshot.overviewMetrics.map((metric) => (
          <StatCard
            key={metric.label}
            label={metric.label}
            value={metric.value}
            detail={metric.detail}
          />
        ))}
      </section>

      <Surface
        title="Operator alerts"
        detail="Alerts are generated from live billing, runtime, catalog, and workspace health signals from the control plane."
      >
        <div className="adminx-card-grid">
          {snapshot.alerts.map((alert) => (
            <article key={alert.id} className="adminx-mini-card">
              <div className="adminx-row">
                <strong>{alert.title}</strong>
                <Pill tone={alert.severity === 'high' ? 'danger' : 'default'}>
                  {alert.severity}
                </Pill>
              </div>
              <p>{alert.detail}</p>
            </article>
          ))}
        </div>
      </Surface>

      <Surface title="Data-source posture" detail="The independent admin shell now runs on live control-plane data across the core operating surfaces.">
        <div className="adminx-card-grid">
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Live admin API</strong>
              <Pill tone="live">live</Pill>
            </div>
            <p>Tenants, projects, keys, channels, providers, credentials, models, usage, billing, and runtime status.</p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Live campaign and identity operations</strong>
              <Pill tone="live">live</Pill>
            </div>
            <p>Coupons, operator identities, portal identities, credential rotation, and catalog retirement actions are all persisted through the admin control plane.</p>
          </article>
        </div>
      </Surface>

      <div className="adminx-users-grid">
        <Surface
          title="Top portal users"
          detail="Portal identities ranked by request count, then token consumption and metered usage."
        >
          <div className="adminx-card-grid">
            {rankedUsers.map((user) => (
              <article key={user.id} className="adminx-mini-card">
                <div className="adminx-row">
                  <strong>{user.display_name}</strong>
                  <Pill tone={user.active ? 'live' : 'danger'}>
                    {user.active ? 'active' : 'disabled'}
                  </Pill>
                </div>
                <p>{user.email}</p>
                <p>
                  Requests: {user.request_count}
                  {' | '}
                  Tokens: {user.total_tokens}
                  {' | '}
                  Units: {user.usage_units}
                </p>
              </article>
            ))}
          </div>
        </Surface>

        <Surface
          title="Hottest projects"
          detail="Projects with the highest traffic and spend signals across usage and billing summaries."
        >
          <div className="adminx-card-grid">
            {rankedProjects.map((project) => (
              <article key={project.id} className="adminx-mini-card">
                <div className="adminx-row">
                  <strong>{project.name}</strong>
                  <Pill tone="default">{project.id}</Pill>
                </div>
                <p>{project.tenant_id}</p>
                <p>
                  Requests: {project.request_count}
                  {' | '}
                  Tokens: {project.total_tokens}
                  {' | '}
                  Amount: {project.booked_amount.toFixed(2)}
                </p>
              </article>
            ))}
          </div>
        </Surface>
      </div>
    </div>
  );
}
