import { useEffect, useState } from 'react';
import {
  DataTable,
  EmptyState,
  formatCurrency,
  formatDateTime,
  formatUnits,
  InlineButton,
  MetricCard,
  SectionHero,
  Surface,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { UsageRecord, UsageSummary } from 'sdkwork-router-portal-types';

import { UsageFiltersPanel, UsageHighlights } from '../components';
import { loadUsageWorkbenchData } from '../repository';
import { buildUsageWorkbenchViewModel } from '../services';
import type { PortalUsagePageProps, UsageFilters, UsageWorkbenchViewModel } from '../types';

const emptySummary: UsageSummary = {
  total_requests: 0,
  project_count: 0,
  model_count: 0,
  provider_count: 0,
  projects: [],
  providers: [],
  models: [],
};

export function PortalUsagePage({ onNavigate }: PortalUsagePageProps) {
  const [summary, setSummary] = useState<UsageSummary>(emptySummary);
  const [records, setRecords] = useState<UsageRecord[]>([]);
  const [filters, setFilters] = useState<UsageFilters>({ model: '', provider: '', date_range: '30d' });
  const [status, setStatus] = useState('Loading request telemetry...');

  useEffect(() => {
    let cancelled = false;

    void loadUsageWorkbenchData()
      .then((data) => {
        if (cancelled) {
          return;
        }

        setSummary(data.summary);
        setRecords(data.records);
        setStatus('Per-call request telemetry is filtered to your workspace project.');
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

  const viewModel: UsageWorkbenchViewModel = buildUsageWorkbenchViewModel(summary, records, filters);

  return (
    <>
      <SectionHero
        detail="Inspect recent gateway calls, model distribution, provider mix, and token-unit usage for every recorded request."
        eyebrow="Usage"
        title="Request telemetry"
      />

      <div className="portalx-status-row">
        <span className="portalx-status">{status}</span>
      </div>

      <div className="portalx-metric-grid">
        <MetricCard
          detail="Total requests associated with this portal workspace."
          label="Request Count"
          value={formatUnits(summary.total_requests)}
        />
        <MetricCard
          detail="Total token units booked across the visible request history."
          label="Token Units"
          value={formatUnits(viewModel.total_units)}
        />
        <MetricCard
          detail="Distinct models used by this workspace."
          label="Models"
          value={formatUnits(summary.model_count)}
        />
        <MetricCard
          detail="Distinct provider paths selected by routing."
          label="Providers"
          value={formatUnits(summary.provider_count)}
        />
      </div>

      <Surface detail="Narrow the request workbench by model or provider without leaving the portal." title="Usage filters">
        <UsageFiltersPanel
          filters={filters}
          modelOptions={viewModel.model_options}
          onChange={setFilters}
          providerOptions={viewModel.provider_options}
        />
      </Surface>

      <UsageHighlights highlights={viewModel.highlights} />

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail="A quick read on which providers, models, and recent calls currently shape this traffic slice."
          title="Traffic profile"
        >
          <ul className="portalx-fact-list">
            {viewModel.traffic_profile.map((item) => (
              <li key={item.id}>
                <strong>{item.label}</strong>
                <span>{item.value}</span>
                <p>{item.detail}</p>
              </li>
            ))}
          </ul>
        </Surface>

        <Surface
          detail="Translate raw request rows into a quick cost and burn read before you jump into billing."
          title="Spend watch"
        >
          <ul className="portalx-fact-list">
            {viewModel.spend_watch.map((item) => (
              <li key={item.id}>
                <strong>{item.label}</strong>
                <span>{item.value}</span>
                <p>{item.detail}</p>
              </li>
            ))}
          </ul>
        </Surface>
      </div>

      <Surface
        detail="A lightweight interpretation layer for concentration, spikes, and request-slice stability."
        title="Request diagnostics"
      >
        <div className="portalx-guardrail-list">
          {viewModel.diagnostics.map((item) => (
            <article className="portalx-guardrail-card" key={item.id}>
              <strong>{item.title}</strong>
              <p>{item.detail}</p>
            </article>
          ))}
        </div>
      </Surface>

      <Surface
        detail="The usage workbench should lead directly into the next operational or commercial action instead of leaving users to infer it."
        title="Connected actions"
      >
        <div className="portalx-checklist-grid">
          <article className="portalx-checklist-card">
            <strong>Review credits if burn pace is rising</strong>
            <p>Use the points view to decide whether a coupon top-up is enough for the next traffic window.</p>
            <InlineButton onClick={() => onNavigate('credits')} tone="primary">
              Open credits
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Move into billing for sustained growth</strong>
            <p>If usage is becoming steady instead of experimental, compare the current burn slice against subscription and recharge paths.</p>
            <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
              Review billing
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Audit credentials if telemetry looks wrong</strong>
            <p>When traffic does not match expectation, verify environment keys and client targeting before scaling requests.</p>
            <InlineButton onClick={() => onNavigate('api-keys')} tone="ghost">
              Manage keys
            </InlineButton>
          </article>
        </div>
      </Surface>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface detail="Top models by request volume." title="Model distribution">
          {summary.models.length ? (
            <ul className="portalx-fact-list">
              {summary.models.map((model) => (
                <li key={model.model}>
                  <strong>{model.model}</strong>
                  <span>{model.request_count} requests</span>
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState detail="Model distribution will appear after the first gateway requests are recorded." title="No model data yet" />
          )}
        </Surface>

        <Surface detail="Top provider paths selected for this project." title="Provider distribution">
          {summary.providers.length ? (
            <ul className="portalx-fact-list">
              {summary.providers.map((provider) => (
                <li key={provider.provider}>
                  <strong>{provider.provider}</strong>
                  <span>{provider.request_count} requests</span>
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState detail="Provider routing activity will appear after the first requests are recorded." title="No provider data yet" />
          )}
        </Surface>
      </div>

      <Surface detail="Every request row includes token units and booked amount when available." title="Recent API requests">
        {viewModel.filtered_records.length ? (
          <DataTable
            columns={[
              { key: 'model', label: 'Model', render: (row) => row.model },
              { key: 'provider', label: 'Provider', render: (row) => row.provider },
              { key: 'units', label: 'Token Units', render: (row) => formatUnits(row.units) },
              { key: 'amount', label: 'Booked', render: (row) => formatCurrency(row.amount) },
              { key: 'time', label: 'Recorded', render: (row) => formatDateTime(row.created_at_ms) },
            ]}
            empty="No request history recorded yet."
            getKey={(row, index) => `${row.created_at_ms}-${row.model}-${index}`}
            rows={viewModel.filtered_records}
          />
        ) : (
          <EmptyState
            detail="Adjust the filters or run a gateway call from your project and the request list will populate here."
            title="No request history for this slice"
          />
        )}
      </Surface>
    </>
  );
}
