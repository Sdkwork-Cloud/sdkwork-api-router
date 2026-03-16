import { useEffect, useState } from 'react';
import {
  DataTable,
  EmptyState,
  formatCurrency,
  formatDateTime,
  formatUnits,
  InlineButton,
  MetricCard,
  Pill,
  SectionHero,
  Surface,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import { DashboardInsights, DashboardReadiness } from '../components';
import { loadPortalDashboardSnapshot } from '../repository';
import { buildPortalDashboardViewModel } from '../services';
import type { PortalDashboardPageProps, PortalDashboardPageViewModel } from '../types';

export function PortalDashboardPage({ onNavigate }: PortalDashboardPageProps) {
  const [viewModel, setViewModel] = useState<PortalDashboardPageViewModel | null>(null);
  const [status, setStatus] = useState('Loading your workspace posture...');

  useEffect(() => {
    let cancelled = false;

    void loadPortalDashboardSnapshot()
      .then((snapshot) => {
        if (cancelled) {
          return;
        }

        setViewModel(buildPortalDashboardViewModel(snapshot));
        setStatus('Live workspace, usage, and points posture are up to date.');
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

  const snapshot = viewModel?.snapshot;
  const remainingUnits = snapshot?.billing_summary.remaining_units;

  return (
    <>
      <SectionHero
        actions={
          <>
            <InlineButton onClick={() => onNavigate('api-keys')} tone="secondary">
              Manage keys
            </InlineButton>
            <InlineButton onClick={() => onNavigate('billing')} tone="primary">
              Upgrade plan
            </InlineButton>
          </>
        }
        detail="A live view of your workspace identity, recent API activity, quota posture, and monetization next steps."
        eyebrow="Portal Dashboard"
        title={snapshot?.workspace.user.display_name ?? 'Developer workspace'}
      />

      <div className="portalx-status-row">
        <Pill tone="accent">Project: {snapshot?.workspace.project.name ?? 'Loading'}</Pill>
        <Pill tone="positive">Requests: {snapshot?.usage_summary.total_requests ?? 0}</Pill>
        <Pill tone="warning">Status: {status}</Pill>
      </div>

      {viewModel ? <DashboardReadiness items={viewModel.readiness} /> : null}

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail={viewModel?.daily_brief.detail ?? 'The portal will summarize the top operating focus after the live dashboard snapshot loads.'}
          title="Focus board"
        >
          {viewModel ? (
            <div className="portalx-journey-step-grid">
              {viewModel.focus_board.map((item) => (
                <article className="portalx-journey-step-card" key={item.id}>
                  <div className="portalx-status-row">
                    <Pill tone={item.tone}>{item.priority_label}</Pill>
                    <strong>{item.title}</strong>
                  </div>
                  <p>{item.detail}</p>
                  <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                    {item.action_label}
                  </InlineButton>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState detail="Focus board will appear after the live dashboard snapshot is ready." title="Preparing focus" />
          )}
        </Surface>

        <Surface
          detail="Risk watchlist keeps the current workspace risks explicit so daily reviews start with exposure, not guesswork."
          title="Risk watchlist"
        >
          {viewModel ? (
            <div className="portalx-guardrail-list">
              {viewModel.risk_watchlist.map((item) => (
                <article className="portalx-guardrail-card" key={item.id}>
                  <div className="portalx-status-row">
                    <strong>{item.title}</strong>
                    <Pill tone={item.tone}>{item.status_label}</Pill>
                  </div>
                  <p>{item.detail}</p>
                  <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                    {item.action_label}
                  </InlineButton>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState detail="Risk watchlist will appear after the same live dashboard snapshot loads." title="Preparing risks" />
          )}
        </Surface>
      </div>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail="The portal should teach users when to revisit each module, not only where those modules are."
          title="Review cadence"
        >
          {viewModel ? (
            <div className="portalx-journey-step-grid">
              {viewModel.review_cadence.map((item) => (
                <article className="portalx-journey-step-card" key={item.id}>
                  <strong>{item.title}</strong>
                  <p>{item.detail}</p>
                  <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                    {item.action_label}
                  </InlineButton>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState detail="Review cadence will appear after the live dashboard snapshot loads." title="Preparing cadence" />
          )}
        </Surface>

        <Surface
          detail="The current mode should collapse into one dominant lane of execution instead of leaving users with a flat list of options."
          title="Playbook lane"
        >
          {viewModel ? (
            <div className="portalx-journey-step-grid">
              {viewModel.playbook_lane.map((item, index) => (
                <article className="portalx-journey-step-card" key={item.id}>
                  <div className="portalx-status-row">
                    <Pill tone={item.tone}>Step {index + 1}</Pill>
                    <strong>{item.title}</strong>
                  </div>
                  <p>{item.detail}</p>
                  <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                    {item.action_label}
                  </InlineButton>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState detail="Playbook lane will appear after the same live snapshot is ready." title="Preparing playbook" />
          )}
        </Surface>
      </div>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail={viewModel?.mode.detail ?? 'Workspace mode will appear after the live dashboard snapshot loads.'}
          title="Mode narrative"
        >
          {viewModel ? (
            <div className="portalx-readiness-panel">
              <div className="portalx-checklist-card">
                <Pill tone={viewModel.mode.tone}>{viewModel.mode.title}</Pill>
                <strong>{viewModel.mode.detail}</strong>
                <p>{viewModel.mode.why_now}</p>
              </div>
            </div>
          ) : (
            <EmptyState detail="The mode narrative depends on the same live snapshot as the rest of the dashboard." title="Preparing mode" />
          )}
        </Surface>

        <Surface
          detail="Make the dominant operating path explicit so users do not need to derive sequencing from multiple cards."
          title="Decision path"
        >
          {viewModel ? (
            <div className="portalx-journey-step-grid">
              {viewModel.decision_path.map((item, index) => (
                <article className="portalx-journey-step-card" key={item.id}>
                  <div className="portalx-status-row">
                    <Pill tone="accent">Step {index + 1}</Pill>
                    <strong>{item.title}</strong>
                  </div>
                  <p>{item.detail}</p>
                  <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                    {item.action_label}
                  </InlineButton>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState detail="The decision path will appear once the live dashboard snapshot is ready." title="Preparing path" />
          )}
        </Surface>
      </div>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail={viewModel?.journey.next_milestone_detail ?? 'Launch-journey guidance will appear after the dashboard snapshot loads.'}
          title="Journey progress"
        >
          {viewModel ? (
            <div className="portalx-readiness-panel">
              <div className="portalx-readiness-score">
                <span>Launch progress</span>
                <strong>{viewModel.journey.progress_label}</strong>
              </div>
              <div className="portalx-checklist-grid">
                <article className="portalx-checklist-card">
                  <Pill tone="warning">Current blocker</Pill>
                  <strong>{viewModel.journey.current_blocker}</strong>
                  <p>The first incomplete gate is treated as the main blocker so teams do not need to infer sequencing from raw metrics.</p>
                </article>
                <article className="portalx-checklist-card">
                  <Pill tone="accent">Next milestone</Pill>
                  <strong>{viewModel.journey.next_milestone_title}</strong>
                  <p>{viewModel.journey.next_milestone_detail}</p>
                  <InlineButton onClick={() => onNavigate(viewModel.journey.next_route)} tone="ghost">
                    {viewModel.journey.next_action_label}
                  </InlineButton>
                </article>
              </div>
            </div>
          ) : (
            <EmptyState detail="The launch journey is derived from the same live checklist that powers the rest of the dashboard." title="Preparing journey" />
          )}
        </Surface>

        <Surface
          detail="A persistent milestone map makes the onboarding-to-production path explicit for every workspace."
          title="Milestone map"
        >
          {viewModel ? (
            <div className="portalx-journey-step-grid">
              {viewModel.launch_checklist.map((item, index) => (
                <article className="portalx-journey-step-card" key={item.id}>
                  <div className="portalx-status-row">
                    <Pill tone={item.complete ? 'positive' : 'warning'}>
                      {item.complete ? 'Done' : `Step ${index + 1}`}
                    </Pill>
                    <strong>{item.title}</strong>
                  </div>
                  <p>{item.detail}</p>
                  <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                    {item.action_label}
                  </InlineButton>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState detail="The milestone map will appear once the dashboard snapshot is ready." title="Preparing milestones" />
          )}
        </Surface>
      </div>

      <Surface
        detail="Module-level route signals make it easy to scan which part of the portal currently needs attention."
        title="Route signal map"
      >
        {viewModel ? (
          <div className="portalx-route-signal-grid">
            {viewModel.route_signals.map((item) => (
              <article className="portalx-checklist-card" key={item.route}>
                <div className="portalx-status-row">
                  <strong>{item.title}</strong>
                  <Pill tone={item.tone}>{item.status_label}</Pill>
                </div>
                <p>{item.detail}</p>
                <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                  Open {item.title}
                </InlineButton>
              </article>
            ))}
          </div>
        ) : (
          <EmptyState detail="Route signal map will appear after the dashboard snapshot loads." title="Preparing route signals" />
        )}
      </Surface>

      <div className="portalx-metric-grid">
        <MetricCard
          detail="Remaining token-unit budget before the current quota is exhausted."
          label="Available Points"
          value={remainingUnits === undefined || remainingUnits === null ? 'Unlimited' : formatUnits(remainingUnits)}
        />
        <MetricCard
          detail="Token units booked against the current workspace project."
          label="Used Token Units"
          value={formatUnits(snapshot?.billing_summary.used_units ?? 0)}
        />
        <MetricCard
          detail="Completed gateway requests recorded for this workspace."
          label="Total Calls"
          value={formatUnits(snapshot?.usage_summary.total_requests ?? 0)}
        />
        <MetricCard
          detail="Issued API keys currently visible to the portal boundary."
          label="API Keys"
          value={formatUnits(snapshot?.api_key_count ?? 0)}
        />
      </div>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail="The dashboard should show concrete evidence, not only guidance, so users can validate why a recommendation exists."
          title="Evidence timeline"
        >
          {viewModel ? (
            <ol className="portalx-queue-list">
              {viewModel.evidence_timeline.map((item) => (
                <li className="portalx-queue-card" key={item.id}>
                  <div className="portalx-status-row">
                    <strong>{item.title}</strong>
                    <span>{item.timestamp_label}</span>
                  </div>
                  <p>{item.detail}</p>
                </li>
              ))}
            </ol>
          ) : (
            <EmptyState detail="Evidence will appear after the live dashboard snapshot loads." title="Preparing evidence" />
          )}
        </Surface>

        <Surface
          detail="Confidence signals explain how much of the current launch posture is backed by live evidence versus pending actions."
          title="Confidence signals"
        >
          {viewModel ? (
            <div className="portalx-guardrail-list">
              {viewModel.confidence_signals.map((item) => (
                <article className="portalx-guardrail-card" key={item.id}>
                  <Pill tone={item.tone}>{item.title}</Pill>
                  <p>{item.detail}</p>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState detail="Confidence signals will appear after the same live snapshot loads." title="Preparing confidence" />
          )}
        </Surface>
      </div>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail="Prioritized actions derived from live key inventory, request telemetry, and quota posture."
          title="Action queue"
        >
          {viewModel ? (
            <ol className="portalx-queue-list">
              {viewModel.action_queue.map((item) => (
                <li className="portalx-queue-card" key={item.id}>
                  <div className="portalx-status-row">
                    <Pill tone={item.tone}>{item.priority_label}</Pill>
                    <strong>{item.title}</strong>
                  </div>
                  <p>{item.detail}</p>
                  <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                    {item.action_label}
                  </InlineButton>
                </li>
              ))}
            </ol>
          ) : (
            <EmptyState detail="The action queue will appear once the dashboard snapshot finishes loading." title="Preparing actions" />
          )}
        </Surface>

        <Surface
          detail={viewModel?.production_readiness.detail ?? 'Workspace launch posture will appear after the dashboard snapshot loads.'}
          title="Production readiness"
        >
          {viewModel ? (
            <div className="portalx-readiness-panel">
              <div className="portalx-readiness-score">
                <span>Readiness score</span>
                <strong>{viewModel.production_readiness.score}%</strong>
              </div>
              <div className="portalx-checklist-grid">
                <article className="portalx-checklist-card">
                  <Pill tone="positive">Strengths</Pill>
                  <strong>{viewModel.production_readiness.title}</strong>
                  <ul className="portalx-bullet-list">
                    {viewModel.production_readiness.strengths.length ? (
                      viewModel.production_readiness.strengths.map((item) => <li key={item}>{item}</li>)
                    ) : (
                      <li>Launch signals will appear here as soon as the first gates are completed.</li>
                    )}
                  </ul>
                </article>
                <article className="portalx-checklist-card">
                  <Pill tone={viewModel.production_readiness.blockers.length ? 'warning' : 'positive'}>
                    {viewModel.production_readiness.blockers.length ? 'Blockers' : 'Clear'}
                  </Pill>
                  <strong>
                    {viewModel.production_readiness.blockers.length
                      ? 'What still needs attention'
                      : 'No current launch blockers'}
                  </strong>
                  <ul className="portalx-bullet-list">
                    {viewModel.production_readiness.blockers.length ? (
                      viewModel.production_readiness.blockers.map((item) => <li key={item}>{item}</li>)
                    ) : (
                      <li>Keys, telemetry, identity, and visible quota posture are aligned.</li>
                    )}
                  </ul>
                </article>
              </div>
            </div>
          ) : (
            <EmptyState detail="The readiness score depends on the same live dashboard snapshot as the rest of the page." title="Preparing readiness" />
          )}
        </Surface>
      </div>

      <Surface
        detail="Concrete gates that move a new workspace from onboarding into production-ready posture."
        title="Launch checklist"
      >
        {viewModel ? (
          <div className="portalx-checklist-grid">
            {viewModel.launch_checklist.map((item) => (
              <article className="portalx-checklist-card" key={item.id}>
                <Pill tone={item.complete ? 'positive' : 'warning'}>
                  {item.complete ? 'Ready' : 'Needs action'}
                </Pill>
                <strong>{item.title}</strong>
                <p>{item.detail}</p>
                <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                  {item.action_label}
                </InlineButton>
              </article>
            ))}
          </div>
        ) : (
          <EmptyState detail="Checklist gates will appear once the dashboard snapshot is ready." title="Preparing checklist" />
        )}
      </Surface>

      {viewModel ? (
        <Surface detail="The portal highlights the next operational move instead of forcing users to interpret raw metrics alone." title="Workspace insights">
          <DashboardInsights insights={viewModel.insights} onNavigate={onNavigate} />
        </Surface>
      ) : null}

      <Surface
        detail="Critical workspace identifiers and quota state at a glance."
        title="Workspace identity"
      >
        {snapshot ? (
          <ul className="portalx-fact-list">
            <li>
              <strong>User</strong>
              <span>{snapshot.workspace.user.email}</span>
            </li>
            <li>
              <strong>Tenant</strong>
              <span>{snapshot.workspace.tenant.name}</span>
            </li>
            <li>
              <strong>Project</strong>
              <span>{snapshot.workspace.project.id}</span>
            </li>
            <li>
              <strong>Booked amount</strong>
              <span>{formatCurrency(snapshot.billing_summary.booked_amount)}</span>
            </li>
          </ul>
        ) : (
          <EmptyState detail="The dashboard snapshot will appear after portal data loads." title="Preparing workspace" />
        )}
      </Surface>

      <Surface
        actions={
          <InlineButton onClick={() => onNavigate('usage')} tone="ghost">
            Open usage workbench
          </InlineButton>
        }
        detail="Recent API calls with provider, model, token units, and booked amount."
        title="Recent requests"
      >
        {snapshot?.recent_requests.length ? (
          <DataTable
            columns={[
              {
                key: 'model',
                label: 'Model',
                render: (row) => row.model,
              },
              {
                key: 'provider',
                label: 'Provider',
                render: (row) => row.provider,
              },
              {
                key: 'units',
                label: 'Token Units',
                render: (row) => formatUnits(row.units),
              },
              {
                key: 'amount',
                label: 'Booked',
                render: (row) => formatCurrency(row.amount),
              },
              {
                key: 'created',
                label: 'Recorded',
                render: (row) => formatDateTime(row.created_at_ms),
              },
            ]}
            empty="No request telemetry recorded for this project yet."
            getKey={(row, index) => `${row.project_id}-${row.model}-${row.created_at_ms}-${index}`}
            rows={snapshot.recent_requests}
          />
        ) : (
          <EmptyState
            detail="Once gateway requests start flowing through your project, per-call token-unit usage will appear here."
            title="No recent requests yet"
          />
        )}
      </Surface>
    </>
  );
}
