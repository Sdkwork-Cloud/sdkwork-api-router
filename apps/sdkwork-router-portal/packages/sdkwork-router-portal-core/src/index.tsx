import { useEffect, useState } from 'react';
import { PortalAccountPage } from 'sdkwork-router-portal-account';
import { PortalApiKeysPage } from 'sdkwork-router-portal-api-keys';
import { PortalLoginPage, PortalRegisterPage } from 'sdkwork-router-portal-auth';
import { PortalBillingPage } from 'sdkwork-router-portal-billing';
import { formatUnits, InlineButton, Pill } from 'sdkwork-router-portal-commons';
import {
  clearPortalSessionToken,
  getPortalDashboard,
  getPortalMe,
  getPortalWorkspace,
  onPortalSessionExpired,
  portalErrorMessage,
  PortalApiError,
  readPortalSessionToken,
} from 'sdkwork-router-portal-portal-api';
import { PortalCreditsPage } from 'sdkwork-router-portal-credits';
import { buildPortalDashboardViewModel, PortalDashboardPage } from 'sdkwork-router-portal-dashboard';
import type {
  PortalAnonymousRouteKey,
  PortalAuthSession,
  PortalDashboardSummary,
  PortalRouteKey,
  PortalWorkspaceSummary,
} from 'sdkwork-router-portal-types';
import { PortalUsagePage } from 'sdkwork-router-portal-usage';

import { portalRoutes } from './routes';

type PortalHashRoute = PortalRouteKey | PortalAnonymousRouteKey;

function normalizeHashRoute(hash: string): PortalHashRoute {
  const candidate = hash.replace(/^#\/?/, '');
  if (!candidate) {
    return 'login';
  }

  if (candidate === 'login' || candidate === 'register') {
    return candidate;
  }

  if (portalRoutes.some((route) => route.key === candidate)) {
    return candidate as PortalRouteKey;
  }

  return 'login';
}

function writeHashRoute(route: PortalHashRoute): void {
  window.location.hash = `#/${route}`;
}

function buildWorkspacePulse(snapshot: PortalDashboardSummary | null): {
  title: string;
  detail: string;
  route: PortalRouteKey;
  action_label: string;
  points_label: string;
  usage_label: string;
  keys_label: string;
} {
  if (!snapshot) {
    return {
      title: 'Waiting for live workspace pulse',
      detail: 'Once dashboard telemetry loads, the shell will keep your launch posture visible here.',
      route: 'dashboard',
      action_label: 'Open dashboard',
      points_label: 'Loading',
      usage_label: 'Loading',
      keys_label: 'Loading',
    };
  }

  if (snapshot.billing_summary.exhausted) {
    return {
      title: 'Recharge before more traffic lands',
      detail: 'The visible quota is exhausted. Restore headroom before the next production window.',
      route: 'billing',
      action_label: 'Restore runway',
      points_label: 'Exhausted',
      usage_label: formatUnits(snapshot.usage_summary.total_requests),
      keys_label: formatUnits(snapshot.api_key_count),
    };
  }

  if (snapshot.api_key_count === 0) {
    return {
      title: 'Create the first production key',
      detail: 'No active key is visible yet, so external clients still cannot connect safely.',
      route: 'api-keys',
      action_label: 'Issue key',
      points_label: snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined
        ? 'Unlimited'
        : formatUnits(snapshot.billing_summary.remaining_units),
      usage_label: formatUnits(snapshot.usage_summary.total_requests),
      keys_label: 'Needs setup',
    };
  }

  if (snapshot.usage_summary.total_requests === 0) {
    return {
      title: 'Send the first live request',
      detail: 'The portal is ready, but request telemetry has not started flowing through the project yet.',
      route: 'usage',
      action_label: 'Open usage',
      points_label: snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined
        ? 'Unlimited'
        : formatUnits(snapshot.billing_summary.remaining_units),
      usage_label: 'No traffic yet',
      keys_label: formatUnits(snapshot.api_key_count),
    };
  }

  return {
    title: 'Workspace posture is healthy',
    detail: 'Identity, keys, traffic, and visible quota posture are aligned for continued growth.',
    route: 'dashboard',
    action_label: 'Review command center',
    points_label: snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined
      ? 'Unlimited'
      : formatUnits(snapshot.billing_summary.remaining_units),
    usage_label: formatUnits(snapshot.usage_summary.total_requests),
    keys_label: formatUnits(snapshot.api_key_count),
  };
}

function buildLaunchJourney(snapshot: PortalDashboardSummary | null): {
  progress_label: string;
  current_blocker: string;
  next_milestone_title: string;
  next_milestone_detail: string;
  next_route: PortalRouteKey;
  next_action_label: string;
  steps: Array<{ id: string; label: string; complete: boolean }>;
} {
  if (!snapshot) {
    return {
      progress_label: 'Waiting for dashboard snapshot',
      current_blocker: 'The launch journey will appear after live workspace telemetry loads.',
      next_milestone_title: 'Load workspace posture',
      next_milestone_detail: 'Open the dashboard snapshot and the portal will map the remaining path automatically.',
      next_route: 'dashboard',
      next_action_label: 'Open dashboard',
      steps: [
        { id: 'identity', label: 'Identity', complete: false },
        { id: 'keys', label: 'Keys', complete: false },
        { id: 'traffic', label: 'Traffic', complete: false },
        { id: 'runway', label: 'Runway', complete: false },
      ],
    };
  }

  const steps: Array<{
    id: string;
    label: string;
    complete: boolean;
    route: PortalRouteKey;
    detail: string;
    action_label: string;
  }> = [
    {
      id: 'identity',
      label: 'Identity',
      complete: Boolean(snapshot.workspace.project.id && snapshot.workspace.tenant.id),
      route: 'account' as PortalRouteKey,
      detail: 'Workspace identity must be bound before self-service operations remain trustworthy.',
      action_label: 'Review account',
    },
    {
      id: 'keys',
      label: 'Keys',
      complete: snapshot.api_key_count > 0,
      route: 'api-keys' as PortalRouteKey,
      detail: snapshot.api_key_count > 0
        ? 'Environment-scoped credentials are already visible inside the portal boundary.'
        : 'Issue at least one environment key before onboarding traffic.',
      action_label: 'Manage keys',
    },
    {
      id: 'traffic',
      label: 'Traffic',
      complete: snapshot.usage_summary.total_requests > 0,
      route: 'usage' as PortalRouteKey,
      detail: snapshot.usage_summary.total_requests > 0
        ? 'Recent requests are already flowing through the workspace.'
        : 'Send the first request so telemetry and token usage become visible.',
      action_label: 'Open usage',
    },
    {
      id: 'runway',
      label: 'Runway',
      complete: !snapshot.billing_summary.exhausted && (
        snapshot.billing_summary.remaining_units === null
          || snapshot.billing_summary.remaining_units === undefined
          || snapshot.billing_summary.remaining_units >= 5_000
      ),
      route: snapshot.billing_summary.exhausted ? 'billing' : 'credits',
      detail: snapshot.billing_summary.exhausted
        ? 'Quota is exhausted and needs immediate billing recovery.'
        : snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined
          ? 'Visible quota posture is currently unlimited.'
          : `${formatUnits(snapshot.billing_summary.remaining_units)} token units remain in the visible launch buffer.`,
      action_label: snapshot.billing_summary.exhausted ? 'Restore runway' : 'Review credits',
    },
  ];

  const completedCount = steps.filter((step) => step.complete).length;
  const nextStep = steps.find((step) => !step.complete) ?? steps[steps.length - 1];

  return {
    progress_label: `${completedCount}/${steps.length} launch gates complete`,
    current_blocker: completedCount === steps.length
      ? 'No current blockers. The workspace is ready for controlled production growth.'
      : nextStep.label,
    next_milestone_title: completedCount === steps.length
      ? 'Sustain healthy workspace posture'
      : nextStep.label,
    next_milestone_detail: completedCount === steps.length
      ? 'Keep monitoring credentials, traffic, and runway as demand scales.'
      : nextStep.detail,
    next_route: nextStep.route,
    next_action_label: nextStep.action_label,
    steps: steps.map((step) => ({ id: step.id, label: step.label, complete: step.complete })),
  };
}

function buildRecentActivity(snapshot: PortalDashboardSummary | null): {
  title: string;
  latest_evidence: string;
  last_request: string;
  details: string[];
} {
  if (!snapshot) {
    return {
      title: 'Recent activity',
      latest_evidence: 'Waiting for dashboard evidence.',
      last_request: 'Pending',
      details: [
        'Latest evidence will appear once the dashboard snapshot is available.',
        'The portal will summarize credential, traffic, and billing evidence here.',
      ],
    };
  }

  const latestRequest = [...snapshot.recent_requests].sort(
    (left, right) => right.created_at_ms - left.created_at_ms,
  )[0];

  return {
    title: 'Recent activity',
    latest_evidence: latestRequest
      ? `Usage evidence is live from ${latestRequest.model} via ${latestRequest.provider}.`
      : 'No request evidence yet. The first request will establish the initial activity trail.',
    last_request: latestRequest
      ? `${formatUnits(latestRequest.units)} units · ${latestRequest.model}`
      : 'Pending',
    details: [
      snapshot.api_key_count > 0
        ? `${formatUnits(snapshot.api_key_count)} key(s) visible in the current workspace boundary.`
        : 'No API key is visible yet.',
      snapshot.billing_summary.exhausted
        ? 'Quota is exhausted and billing recovery is now required.'
        : snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined
          ? 'Visible quota posture is unlimited.'
          : `${formatUnits(snapshot.billing_summary.remaining_units)} token units remain in the visible buffer.`,
    ],
  };
}

function buildWorkspaceMode(snapshot: PortalDashboardSummary | null): {
  title: string;
  detail: string;
  why_now: string;
  quick_actions: Array<{ id: string; label: string; route: PortalRouteKey; tone: 'primary' | 'secondary' | 'ghost' }>;
} {
  if (!snapshot) {
    return {
      title: 'Preparing workspace mode',
      detail: 'The portal is still loading enough live evidence to decide whether this workspace is in launch, growth, or recovery mode.',
      why_now: 'Open the dashboard once the snapshot loads to see the active operating mode.',
      quick_actions: [
        { id: 'dashboard', label: 'Open dashboard', route: 'dashboard', tone: 'secondary' },
      ],
    };
  }

  if (snapshot.billing_summary.exhausted) {
    return {
      title: 'Recovery mode',
      detail: 'Quota posture has become the dominant constraint, so the product should bias toward restoring runway first.',
      why_now: 'Until billing recovery happens, launch readiness and traffic validation stay blocked.',
      quick_actions: [
        { id: 'billing', label: 'Review billing', route: 'billing', tone: 'primary' },
        { id: 'usage', label: 'Open usage', route: 'usage', tone: 'secondary' },
        { id: 'dashboard', label: 'Open dashboard', route: 'dashboard', tone: 'ghost' },
      ],
    };
  }

  if (snapshot.api_key_count === 0 || snapshot.usage_summary.total_requests === 0) {
    return {
      title: 'Launch mode',
      detail: 'The workspace is still completing its first validated launch loop across credentials, traffic, and runway.',
      why_now: 'Finishing the first key and first request path creates the live evidence the rest of the portal depends on.',
      quick_actions: [
        { id: 'keys', label: 'Manage keys', route: 'api-keys', tone: 'primary' },
        { id: 'usage', label: 'Open usage', route: 'usage', tone: 'secondary' },
        { id: 'credits', label: 'Review credits', route: 'credits', tone: 'ghost' },
      ],
    };
  }

  return {
    title: 'Growth mode',
    detail: 'The workspace has crossed the first-launch boundary and now needs deliberate scaling, burn observation, and commercial shaping.',
    why_now: 'Traffic, credentials, and runway are all present, so the next value comes from optimization rather than initial setup.',
    quick_actions: [
      { id: 'usage', label: 'Open usage', route: 'usage', tone: 'primary' },
      { id: 'billing', label: 'Review billing', route: 'billing', tone: 'secondary' },
      { id: 'account', label: 'Review account', route: 'account', tone: 'ghost' },
    ],
  };
}

function buildOperatingRhythm(snapshot: PortalDashboardSummary | null): Array<{
  id: string;
  title: string;
  detail: string;
}> {
  if (!snapshot) {
    return [
      {
        id: 'before-traffic',
        title: 'Before traffic',
        detail: 'Open dashboard after the live snapshot loads to see the first review rhythm for this workspace.',
      },
      {
        id: 'during-traffic',
        title: 'During live traffic',
        detail: 'The rhythm will update automatically once real request evidence appears.',
      },
      {
        id: 'if-risk-appears',
        title: 'If risk appears',
        detail: 'The portal will redirect attention toward credits or billing when hard blockers show up.',
      },
    ];
  }

  return [
    {
      id: 'before-traffic',
      title: 'Before traffic',
      detail: snapshot.api_key_count > 0
        ? 'Reconfirm keys, route signals, and launch posture before widening rollout.'
        : 'Complete the first key and first request loop before treating the workspace as ready.',
    },
    {
      id: 'during-traffic',
      title: 'During live traffic',
      detail: snapshot.usage_summary.total_requests > 0
        ? 'Use usage diagnostics and evidence timeline as the default review lane while traffic is active.'
        : 'Once the first request lands, usage becomes the main live-review surface.',
    },
    {
      id: 'if-risk-appears',
      title: 'If risk appears',
      detail: snapshot.billing_summary.exhausted
        ? 'Runway recovery is already active, so billing is the dominant risk lane right now.'
        : 'When runway shrinks or launch posture degrades, shift into credits and billing before the next window.',
    },
  ];
}

function buildCommandStrip(
  dashboardViewModel: ReturnType<typeof buildPortalDashboardViewModel> | null,
): Array<{
  id: string;
  label: string;
  title: string;
  detail: string;
  route: PortalRouteKey;
  action_label: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}> {
  if (!dashboardViewModel) {
    return [
      {
        id: 'mission',
        label: 'Primary mission',
        title: 'Load workspace posture',
        detail: 'The command strip will summarize the leading objective once the live dashboard snapshot is available.',
        route: 'dashboard',
        action_label: 'Open dashboard',
        tone: 'accent',
      },
      {
        id: 'next-move',
        label: 'Immediate next move',
        title: 'Restore the live snapshot',
        detail: 'Journey guidance will appear here as soon as the workspace pulse is loaded.',
        route: 'dashboard',
        action_label: 'Open dashboard',
        tone: 'default',
      },
      {
        id: 'risk',
        label: 'Lead risk',
        title: 'Risk signals are still loading',
        detail: 'The portal will elevate the dominant launch or runway risk here after dashboard evidence is ready.',
        route: 'dashboard',
        action_label: 'Open dashboard',
        tone: 'warning',
      },
      {
        id: 'mode',
        label: 'Operating mode',
        title: 'Preparing mode narrative',
        detail: 'Launch, growth, or recovery mode will be pinned here once the product has enough live evidence.',
        route: 'dashboard',
        action_label: 'Open dashboard',
        tone: 'default',
      },
    ];
  }

  const { daily_brief: dailyBrief, journey, mode } = dashboardViewModel;
  const modeRoute: PortalRouteKey = mode.title === 'Recovery mode'
    ? 'billing'
    : mode.title === 'Growth mode'
      ? 'usage'
      : journey.next_route;
  const modeActionLabel = mode.title === 'Recovery mode'
    ? 'Review billing'
    : mode.title === 'Growth mode'
      ? 'Open usage'
      : journey.next_action_label;

  return [
    {
      id: 'mission',
      label: 'Primary mission',
      title: dailyBrief.top_focus.title,
      detail: dailyBrief.detail,
      route: dailyBrief.top_focus.route,
      action_label: dailyBrief.top_focus.action_label,
      tone: dailyBrief.top_focus.tone,
    },
    {
      id: 'next-move',
      label: 'Immediate next move',
      title: journey.next_milestone_title,
      detail: journey.next_milestone_detail,
      route: journey.next_route,
      action_label: journey.next_action_label,
      tone: journey.completed_count === journey.total_count ? 'positive' : 'accent',
    },
    {
      id: 'risk',
      label: 'Lead risk',
      title: dailyBrief.risk_watch.title,
      detail: dailyBrief.risk_watch.detail,
      route: dailyBrief.risk_watch.route,
      action_label: dailyBrief.risk_watch.action_label,
      tone: dailyBrief.risk_watch.tone,
    },
    {
      id: 'mode',
      label: 'Operating mode',
      title: mode.title,
      detail: mode.why_now,
      route: modeRoute,
      action_label: modeActionLabel,
      tone: mode.tone,
    },
  ];
}

function Shell({
  workspace,
  dashboardSnapshot,
  pulseStatus,
  route,
  onNavigate,
  onLogout,
  children,
}: {
  workspace: PortalWorkspaceSummary | null;
  dashboardSnapshot: PortalDashboardSummary | null;
  pulseStatus: string;
  route: PortalRouteKey;
  onNavigate: (route: PortalRouteKey) => void;
  onLogout: () => void;
  children: React.ReactNode;
}) {
  const routeDefinition = portalRoutes.find((item) => item.key === route) ?? portalRoutes[0];
  const pulse = buildWorkspacePulse(dashboardSnapshot);
  const journey = buildLaunchJourney(dashboardSnapshot);
  const activity = buildRecentActivity(dashboardSnapshot);
  const mode = buildWorkspaceMode(dashboardSnapshot);
  const dashboardViewModel = dashboardSnapshot ? buildPortalDashboardViewModel(dashboardSnapshot) : null;
  const dailyBrief = dashboardViewModel?.daily_brief;
  const commandStrip = buildCommandStrip(dashboardViewModel);
  const operatingRhythm = buildOperatingRhythm(dashboardSnapshot);

  return (
    <div className="portalx-app-shell">
      <aside className="portalx-sidebar">
        <div className="portalx-brand">
          <span>SDKWork Router</span>
          <strong>Portal</strong>
        </div>

        <div className="portalx-sidebar-card">
          <p className="portalx-eyebrow">Workspace</p>
          <h2>{workspace?.project.name ?? 'Loading project'}</h2>
          <p>{workspace?.tenant.name ?? 'Loading tenant'}</p>
          <small>{workspace?.user.email ?? 'Loading identity'}</small>
        </div>

        <div className="portalx-sidebar-card portalx-sidebar-card-emphasis">
          <p className="portalx-eyebrow">Workspace pulse</p>
          <h3>{pulse.title}</h3>
          <p>{pulse.detail}</p>
          <div className="portalx-status-row">
            <Pill tone="accent">Points {pulse.points_label}</Pill>
            <Pill tone="positive">Calls {pulse.usage_label}</Pill>
            <Pill tone="warning">Keys {pulse.keys_label}</Pill>
          </div>
          <small>{pulseStatus}</small>
          <div className="portalx-sidebar-card-action">
            <InlineButton onClick={() => onNavigate(pulse.route)} tone="ghost">
              {pulse.action_label}
            </InlineButton>
          </div>
        </div>

        <div className="portalx-sidebar-card">
          <p className="portalx-eyebrow">Workspace mode</p>
          <h3>{mode.title}</h3>
          <p>{mode.detail}</p>
          <div className="portalx-sidebar-subsection">
            <strong>Why now</strong>
            <p>{mode.why_now}</p>
          </div>
          <div className="portalx-sidebar-subsection">
            <strong>Quick actions</strong>
            <div className="portalx-form-actions">
              {mode.quick_actions.map((action) => (
                <InlineButton key={action.id} onClick={() => onNavigate(action.route)} tone={action.tone}>
                  {action.label}
                </InlineButton>
              ))}
            </div>
          </div>
        </div>

        <div className="portalx-sidebar-card">
          <p className="portalx-eyebrow">Daily brief</p>
          <h3>{dailyBrief?.title ?? 'Preparing daily brief'}</h3>
          <p>{dailyBrief?.detail ?? 'The portal will summarize today\'s top focus and risk watch after the live dashboard snapshot loads.'}</p>
          <div className="portalx-sidebar-subsection">
            <div className="portalx-status-row">
              <strong>Top focus</strong>
              {dailyBrief ? <Pill tone={dailyBrief.top_focus.tone}>{dailyBrief.top_focus.priority_label}</Pill> : null}
            </div>
            <p>{dailyBrief?.top_focus.title ?? 'Waiting for the dashboard snapshot.'}</p>
            <small>
              {dailyBrief?.top_focus.detail ?? 'Once the live workspace snapshot loads, the portal will identify the one move that should lead the day.'}
            </small>
            {dailyBrief ? (
              <div className="portalx-sidebar-card-action">
                <InlineButton onClick={() => onNavigate(dailyBrief.top_focus.route)} tone="ghost">
                  {dailyBrief.top_focus.action_label}
                </InlineButton>
              </div>
            ) : null}
          </div>
          <div className="portalx-sidebar-subsection">
            <div className="portalx-status-row">
              <strong>Risk watch</strong>
              {dailyBrief ? <Pill tone={dailyBrief.risk_watch.tone}>{dailyBrief.risk_watch.status_label}</Pill> : null}
            </div>
            <p>{dailyBrief?.risk_watch.title ?? 'Waiting for the dashboard snapshot.'}</p>
            <small>
              {dailyBrief?.risk_watch.detail ?? 'The portal will surface the strongest current risk signal here once live evidence is ready.'}
            </small>
            {dailyBrief ? (
              <div className="portalx-sidebar-card-action">
                <InlineButton onClick={() => onNavigate(dailyBrief.risk_watch.route)} tone="ghost">
                  {dailyBrief.risk_watch.action_label}
                </InlineButton>
              </div>
            ) : null}
          </div>
        </div>

        <div className="portalx-sidebar-card">
          <p className="portalx-eyebrow">Operating rhythm</p>
          <div className="portalx-route-signal-list">
            {operatingRhythm.map((item) => (
              <div className="portalx-route-signal-card" key={item.id}>
                <strong>{item.title}</strong>
                <p>{item.detail}</p>
              </div>
            ))}
          </div>
        </div>

        <div className="portalx-sidebar-card">
          <p className="portalx-eyebrow">Launch journey</p>
          <h3>{journey.progress_label}</h3>
          <div className="portalx-journey-step-row">
            {journey.steps.map((step) => (
              <span
                className={`portalx-journey-chip ${step.complete ? 'portalx-journey-chip-complete' : ''}`}
                key={step.id}
              >
                {step.label}
              </span>
            ))}
          </div>
          <div className="portalx-sidebar-subsection">
            <strong>Current blocker</strong>
            <p>{journey.current_blocker}</p>
          </div>
          <div className="portalx-sidebar-subsection">
            <strong>Next milestone</strong>
            <p>{journey.next_milestone_detail}</p>
          </div>
          <div className="portalx-sidebar-card-action">
            <InlineButton onClick={() => onNavigate(journey.next_route)} tone="secondary">
              {journey.next_action_label}
            </InlineButton>
          </div>
        </div>

        <div className="portalx-sidebar-card">
          <p className="portalx-eyebrow">{activity.title}</p>
          <h3>Latest evidence</h3>
          <p>{activity.latest_evidence}</p>
          <div className="portalx-sidebar-subsection">
            <strong>Last request</strong>
            <p>{activity.last_request}</p>
          </div>
          <ul className="portalx-help-list">
            {activity.details.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </div>

        <div className="portalx-sidebar-card">
          <p className="portalx-eyebrow">Route signals</p>
          <p className="portalx-status">Needs action, Live, Stable, and Healthy states keep the portal navigation informative instead of passive.</p>
          <div className="portalx-route-signal-list">
            {(dashboardViewModel?.route_signals ?? []).map((item) => (
              <button
                className="portalx-route-signal-card"
                key={item.route}
                onClick={() => onNavigate(item.route)}
                type="button"
              >
                <div className="portalx-status-row">
                  <strong>{item.title}</strong>
                  <Pill tone={item.tone}>{item.status_label}</Pill>
                </div>
                <p>{item.detail}</p>
              </button>
            ))}
          </div>
        </div>

        <nav className="portalx-sidebar-nav">
          {portalRoutes.map((item) => (
            <button
              className={`portalx-nav-item ${item.key === route ? 'portalx-nav-item-active' : ''}`}
              key={item.key}
              onClick={() => onNavigate(item.key)}
              type="button"
            >
              <div className="portalx-nav-item-header">
                <span>{item.label}</span>
                {dashboardViewModel ? (
                  <small className="portalx-nav-item-status">
                    {dashboardViewModel.route_signals.find((signal) => signal.route === item.key)?.status_label ?? ''}
                  </small>
                ) : null}
              </div>
              <small>{item.detail}</small>
            </button>
          ))}
        </nav>

        <div className="portalx-sidebar-card">
          <p className="portalx-eyebrow">Need help?</p>
          <h3>Keep launch motion unblocked</h3>
          <ul className="portalx-help-list">
            <li>Use Dashboard when you need a fast production-readiness answer.</li>
            <li>Use Credits before launch windows to check coupon and quota guardrails.</li>
            <li>Use Billing when runway gets tight and you need the next best bundle.</li>
          </ul>
        </div>

        <button className="portalx-logout" onClick={onLogout} type="button">
          Sign out
        </button>
      </aside>

      <main className="portalx-main-shell">
        <section className="portalx-command-bar">
          <div className="portalx-command-bar-top">
            <div>
              <p className="portalx-eyebrow">Command center</p>
              <h1>{routeDefinition.label}</h1>
              <p className="portalx-hero-detail">{routeDefinition.detail}</p>
            </div>
            <div className="portalx-command-meta">
              <Pill tone="accent">{routeDefinition.eyebrow}</Pill>
              <Pill tone="default">
                {dashboardSnapshot?.workspace.project.name ?? workspace?.project.name ?? 'Workspace'}
              </Pill>
              <Pill tone="warning">{pulse.title}</Pill>
            </div>
          </div>
          <div className="portalx-command-strip-block">
            <div className="portalx-command-strip-intro">
              <span className="portalx-command-strip-title">Mission strip</span>
              <p>Keep the current mission, lead risk, and next move visible on every route instead of hiding product guidance inside one page.</p>
            </div>
            <div className="portalx-command-strip">
              {commandStrip.map((item) => (
                <article className="portalx-command-strip-card" key={item.id}>
                  <div className="portalx-status-row">
                    <span className="portalx-command-strip-label">{item.label}</span>
                    <Pill tone={item.tone}>{item.title}</Pill>
                  </div>
                  <p>{item.detail}</p>
                  <InlineButton onClick={() => onNavigate(item.route)} tone="ghost">
                    {item.action_label}
                  </InlineButton>
                </article>
              ))}
            </div>
          </div>
        </section>
        {children}
      </main>
    </div>
  );
}

function PortalBootScreen({ status }: { status: string }) {
  return (
    <section className="portalx-boot">
      <div className="portalx-boot-card">
        <p className="portalx-eyebrow">Portal Bootstrap</p>
        <h1>Restoring workspace access</h1>
        <p>{status}</p>
      </div>
    </section>
  );
}

export function PortalProductApp() {
  const [route, setRoute] = useState<PortalHashRoute>(() => normalizeHashRoute(window.location.hash));
  const [workspace, setWorkspace] = useState<PortalWorkspaceSummary | null>(null);
  const [dashboardSnapshot, setDashboardSnapshot] = useState<PortalDashboardSummary | null>(null);
  const [authenticated, setAuthenticated] = useState(false);
  const [bootstrapped, setBootstrapped] = useState(false);
  const [bootStatus, setBootStatus] = useState('Checking for an existing portal session token.');
  const [pulseStatus, setPulseStatus] = useState('Workspace pulse will appear after sign-in.');

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(normalizeHashRoute(window.location.hash));
    };

    window.addEventListener('hashchange', handleHashChange);
    if (!window.location.hash) {
      writeHashRoute(readPortalSessionToken() ? 'dashboard' : 'login');
    }

    return () => {
      window.removeEventListener('hashchange', handleHashChange);
    };
  }, []);

  useEffect(() => {
    return onPortalSessionExpired(() => {
      clearPortalSessionToken();
      setAuthenticated(false);
      setDashboardSnapshot(null);
      setWorkspace(null);
      setBootStatus('Your portal session expired. Sign in again to continue.');
      setPulseStatus('Workspace pulse is unavailable until the next sign-in.');
      writeHashRoute('login');
    });
  }, []);

  useEffect(() => {
    const token = readPortalSessionToken();
    if (!token) {
      setAuthenticated(false);
      setDashboardSnapshot(null);
      setWorkspace(null);
      setBootstrapped(true);
      setPulseStatus('Sign in to load the live workspace pulse.');
      if (route !== 'login' && route !== 'register') {
        writeHashRoute('login');
      }
      return;
    }

    let cancelled = false;
    setBootStatus('Refreshing workspace identity and navigation context...');

    void Promise.all([getPortalMe(token), getPortalWorkspace(token)])
      .then(([, nextWorkspace]) => {
        if (cancelled) {
          return;
        }

        setAuthenticated(true);
        setWorkspace(nextWorkspace);
        setBootstrapped(true);

        if (route === 'login' || route === 'register') {
          writeHashRoute('dashboard');
        }
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }

        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          setAuthenticated(false);
          setWorkspace(null);
          setBootstrapped(true);
          writeHashRoute('login');
          return;
        }

        setBootStatus(portalErrorMessage(error));
        setBootstrapped(true);
      });

    return () => {
      cancelled = true;
    };
  }, [route]);

  useEffect(() => {
    const token = readPortalSessionToken();
    if (!token || !authenticated) {
      setDashboardSnapshot(null);
      return;
    }

    let cancelled = false;
    setPulseStatus('Refreshing workspace pulse and launch posture...');

    void getPortalDashboard(token)
      .then((snapshot) => {
        if (cancelled) {
          return;
        }

        setDashboardSnapshot(snapshot);
        setPulseStatus('Workspace pulse is synced with the latest dashboard snapshot.');
      })
      .catch((error) => {
        if (cancelled) {
          return;
        }

        if (error instanceof PortalApiError && error.status === 401) {
          clearPortalSessionToken();
          setAuthenticated(false);
          setDashboardSnapshot(null);
          setWorkspace(null);
          setPulseStatus('Your portal session expired. Sign in again to restore live workspace posture.');
          writeHashRoute('login');
          return;
        }

        setPulseStatus(portalErrorMessage(error));
      });

    return () => {
      cancelled = true;
    };
  }, [authenticated, route]);

  function navigate(nextRoute: PortalHashRoute) {
    if (normalizeHashRoute(window.location.hash) !== nextRoute) {
      writeHashRoute(nextRoute);
      return;
    }

    setRoute(nextRoute);
  }

  async function handleAuthenticated(session: PortalAuthSession) {
    setAuthenticated(true);
    setBootStatus('Hydrating workspace after sign-in...');
    setPulseStatus('Restoring workspace pulse after sign-in...');

    try {
      const nextWorkspace = await getPortalWorkspace(session.token);
      setWorkspace(nextWorkspace);
    } catch (error) {
      setBootStatus(portalErrorMessage(error));
    }

    navigate('dashboard');
  }

  function handleLogout() {
    clearPortalSessionToken();
    setAuthenticated(false);
    setWorkspace(null);
    navigate('login');
  }

  if (!bootstrapped) {
    return <PortalBootScreen status={bootStatus} />;
  }

  if (!authenticated || route === 'login' || route === 'register') {
    return (
      <div className="portalx-auth-root">
        {route === 'register' ? (
          <PortalRegisterPage onAuthenticated={handleAuthenticated} onNavigate={navigate} />
        ) : (
          <PortalLoginPage onAuthenticated={handleAuthenticated} onNavigate={navigate} />
        )}
      </div>
    );
  }

  const activeRoute = route as PortalRouteKey;

  return (
    <Shell
      dashboardSnapshot={dashboardSnapshot}
      onLogout={handleLogout}
      onNavigate={navigate}
      pulseStatus={pulseStatus}
      route={activeRoute}
      workspace={workspace}
    >
      {activeRoute === 'dashboard' ? <PortalDashboardPage onNavigate={navigate} /> : null}
      {activeRoute === 'api-keys' ? <PortalApiKeysPage onNavigate={navigate} /> : null}
      {activeRoute === 'usage' ? <PortalUsagePage onNavigate={navigate} /> : null}
      {activeRoute === 'credits' ? <PortalCreditsPage onNavigate={navigate} /> : null}
      {activeRoute === 'billing' ? <PortalBillingPage onNavigate={navigate} /> : null}
      {activeRoute === 'account' ? <PortalAccountPage onNavigate={navigate} workspace={workspace} /> : null}
    </Shell>
  );
}
