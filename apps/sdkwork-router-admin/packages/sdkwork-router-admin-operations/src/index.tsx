import { useState } from 'react';
import type { FormEvent } from 'react';

import {
  DataTable,
  InlineButton,
  Pill,
  SectionHero,
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, RuntimeReloadReport } from 'sdkwork-router-admin-types';

function formatTimestamp(timestamp: number): string {
  if (!timestamp) {
    return '-';
  }

  return new Date(timestamp).toLocaleString();
}

export function OperationsPage({
  snapshot,
  onReloadRuntimes,
}: AdminPageProps & {
  onReloadRuntimes: (input?: {
    extension_id?: string;
    instance_id?: string;
  }) => Promise<RuntimeReloadReport>;
}) {
  const healthyProviders = snapshot.providerHealth.filter((snapshotItem) => snapshotItem.healthy).length;
  const healthyRuntimes = snapshot.runtimeStatuses.filter((runtime) => runtime.healthy).length;
  const [reloadDraft, setReloadDraft] = useState({
    extension_id: '',
    instance_id: '',
  });
  const [lastReloadReport, setLastReloadReport] = useState<RuntimeReloadReport | null>(null);

  async function handleReload(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const report = await onReloadRuntimes({
      extension_id: reloadDraft.extension_id.trim() || undefined,
      instance_id: reloadDraft.instance_id.trim() || undefined,
    });
    setLastReloadReport(report);
  }

  return (
    <div className="adminx-page-grid">
      <SectionHero
        eyebrow="Runtime"
        title="Monitor provider health and managed runtime posture."
        detail="Operations focuses on the runtime side of the router: provider health, extension status, and operational readiness."
        actions={(
          <InlineButton tone="primary" onClick={() => void onReloadRuntimes().then(setLastReloadReport)}>
            Reload runtimes
          </InlineButton>
        )}
      />

      <section className="adminx-stat-grid">
        <StatCard
          label="Provider health snapshots"
          value={String(snapshot.providerHealth.length)}
          detail="Recent live provider-health records."
        />
        <StatCard
          label="Healthy providers"
          value={String(healthyProviders)}
          detail="Providers currently marked healthy."
        />
        <StatCard
          label="Runtime statuses"
          value={String(snapshot.runtimeStatuses.length)}
          detail="Managed runtime and connector status records."
        />
        <StatCard
          label="Healthy runtimes"
          value={String(healthyRuntimes)}
          detail="Runtimes reporting healthy state."
        />
      </section>

      <div className="adminx-users-grid">
        <Surface
          title="Reload runtimes"
          detail="Run a global reload or target a specific extension or instance without leaving the admin workspace."
        >
          <form className="adminx-form-grid" onSubmit={(event) => void handleReload(event)}>
            <label className="adminx-field">
              <span>Extension id</span>
              <input
                value={reloadDraft.extension_id}
                onChange={(event) => setReloadDraft((current) => ({ ...current, extension_id: event.target.value }))}
                placeholder="optional extension id"
              />
            </label>
            <label className="adminx-field">
              <span>Instance id</span>
              <input
                value={reloadDraft.instance_id}
                onChange={(event) => setReloadDraft((current) => ({ ...current, instance_id: event.target.value }))}
                placeholder="optional instance id"
              />
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                Reload runtimes
              </InlineButton>
              <InlineButton onClick={() => setReloadDraft({ extension_id: '', instance_id: '' })}>
                Clear scope
              </InlineButton>
            </div>
          </form>

          {lastReloadReport ? (
            <div className="adminx-note">
              <strong>Last reload report</strong>
              <p>
                Scope: {lastReloadReport.scope}
                {' | '}
                Active runtimes: {lastReloadReport.active_runtime_count}
                {' | '}
                Loadable packages: {lastReloadReport.loadable_package_count}
                {' | '}
                Reloaded: {formatTimestamp(lastReloadReport.reloaded_at_ms)}
              </p>
            </div>
          ) : (
            <div className="adminx-note">
              <strong>Reload behavior</strong>
              <p>Leave both fields blank to reload all managed runtimes, or target a single extension or instance for a narrower blast radius.</p>
            </div>
          )}
        </Surface>

        <Surface
          title="Runtime posture"
          detail="Runtime states are refreshed after every reload so operators can immediately confirm outcome."
        >
          <div className="adminx-card-grid">
            {snapshot.runtimeStatuses.map((runtime) => (
              <article
                key={`${runtime.runtime}:${runtime.extension_id}:${runtime.instance_id ?? 'global'}`}
                className="adminx-mini-card"
              >
                <div className="adminx-row">
                  <strong>{runtime.display_name}</strong>
                  <Pill tone={runtime.healthy ? 'live' : 'danger'}>
                    {runtime.healthy ? 'healthy' : 'attention'}
                  </Pill>
                </div>
                <p>{runtime.extension_id}</p>
                <p>
                  Running: {String(runtime.running)}
                  {' | '}
                  Instance: {runtime.instance_id ?? 'global'}
                </p>
              </article>
            ))}
          </div>
        </Surface>
      </div>

      <Surface title="Provider health" detail="Latest provider-health snapshots from the admin API.">
        <DataTable
          columns={[
            { key: 'provider', label: 'Provider', render: (item) => <strong>{item.provider_id}</strong> },
            { key: 'status', label: 'Status', render: (item) => item.status },
            {
              key: 'healthy',
              label: 'Healthy',
              render: (item) => (
                <Pill tone={item.healthy ? 'live' : 'danger'}>
                  {item.healthy ? 'healthy' : 'attention'}
                </Pill>
              ),
            },
            { key: 'message', label: 'Message', render: (item) => item.message ?? '-' },
          ]}
          rows={snapshot.providerHealth}
          empty="No provider health data available."
          getKey={(item) => `${item.provider_id}:${item.observed_at_ms}`}
        />
      </Surface>

      <Surface title="Managed runtimes" detail="Runtime status and extension-health view.">
        <DataTable
          columns={[
            { key: 'display', label: 'Runtime', render: (runtime) => <strong>{runtime.display_name}</strong> },
            { key: 'family', label: 'Family', render: (runtime) => runtime.runtime },
            { key: 'instance', label: 'Instance', render: (runtime) => runtime.instance_id ?? runtime.extension_id },
            { key: 'running', label: 'Running', render: (runtime) => String(runtime.running) },
            {
              key: 'healthy',
              label: 'Healthy',
              render: (runtime) => (
                <Pill tone={runtime.healthy ? 'live' : 'danger'}>
                  {runtime.healthy ? 'healthy' : 'attention'}
                </Pill>
              ),
            },
            { key: 'message', label: 'Message', render: (runtime) => runtime.message ?? '-' },
          ]}
          rows={snapshot.runtimeStatuses}
          empty="No runtime statuses available."
          getKey={(runtime) => `${runtime.runtime}:${runtime.extension_id}:${runtime.instance_id ?? 'global'}`}
        />
      </Surface>
    </div>
  );
}
