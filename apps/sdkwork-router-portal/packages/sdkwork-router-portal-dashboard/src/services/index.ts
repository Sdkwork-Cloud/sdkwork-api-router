import { formatCurrency, formatDateTime, formatUnits } from 'sdkwork-router-portal-commons';
import type { PortalDashboardSummary } from 'sdkwork-router-portal-types';

import type {
  DashboardActionItem,
  DashboardConfidenceSignal,
  DashboardDailyBrief,
  DashboardEvidenceItem,
  DashboardFocusBoardItem,
  DashboardInsight,
  DashboardChecklistItem,
  DashboardDecisionPathItem,
  DashboardJourneyGuide,
  DashboardModeNarrative,
  DashboardPlaybookLaneItem,
  DashboardProductionReadiness,
  DashboardReadinessItem,
  DashboardReviewCadenceItem,
  DashboardRiskWatchItem,
  DashboardRouteSignal,
  PortalDashboardPageViewModel,
} from '../types';

function buildInsights(snapshot: PortalDashboardSummary): DashboardInsight[] {
  const insights: DashboardInsight[] = [];

  if (snapshot.api_key_count === 0) {
    insights.push({
      id: 'issue-first-key',
      title: 'Issue your first gateway key',
      detail: 'No API keys are currently visible inside the portal boundary. Create an environment-scoped key before onboarding traffic.',
      tone: 'warning',
      route: 'api-keys',
      action_label: 'Create key',
    });
  }

  if (snapshot.usage_summary.total_requests === 0) {
    insights.push({
      id: 'send-first-request',
      title: 'Send the first API request',
      detail: 'Request telemetry is still empty. A single gateway call will unlock live model, provider, and token-unit visibility.',
      tone: 'accent',
      route: 'usage',
      action_label: 'Open usage',
    });
  }

  if (snapshot.billing_summary.exhausted) {
    insights.push({
      id: 'quota-exhausted',
      title: 'Quota is exhausted',
      detail: 'The current workspace has consumed the visible quota ceiling. Recharge or move to a larger subscription before production traffic resumes.',
      tone: 'warning',
      route: 'billing',
      action_label: 'Review billing',
    });
  } else if ((snapshot.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000) {
    insights.push({
      id: 'quota-running-low',
      title: 'Quota is running low',
      detail: `Only ${formatUnits(snapshot.billing_summary.remaining_units ?? 0)} token units remain in the visible quota posture.`,
      tone: 'warning',
      route: 'credits',
      action_label: 'Review credits',
    });
  }

  if (!insights.length) {
    insights.push({
      id: 'workspace-healthy',
      title: 'Workspace posture looks healthy',
      detail: 'Identity, key inventory, usage visibility, and current billing posture are all aligned for continued traffic growth.',
      tone: 'positive',
      route: 'billing',
      action_label: 'Review growth plan',
    });
  }

  return insights;
}

function buildReadiness(snapshot: PortalDashboardSummary): DashboardReadinessItem[] {
  return [
    {
      id: 'keys',
      label: 'Credential posture',
      value: snapshot.api_key_count > 0 ? `${formatUnits(snapshot.api_key_count)} ready` : 'Needs setup',
      detail: snapshot.api_key_count > 0
        ? 'Environment-scoped keys are visible from the portal.'
        : 'Create at least one key before connecting clients.',
    },
    {
      id: 'traffic',
      label: 'Traffic posture',
      value: snapshot.usage_summary.total_requests > 0
        ? `${formatUnits(snapshot.usage_summary.total_requests)} calls`
        : 'No traffic yet',
      detail: snapshot.usage_summary.total_requests > 0
        ? 'Recent requests are flowing through the project.'
        : 'The first request will populate live telemetry and cost signals.',
    },
    {
      id: 'budget',
      label: 'Budget posture',
      value: snapshot.billing_summary.exhausted
        ? 'Exhausted'
        : snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined
          ? 'Unlimited'
          : formatUnits(snapshot.billing_summary.remaining_units),
      detail: `Booked amount: ${formatCurrency(snapshot.billing_summary.booked_amount)}.`,
    },
  ];
}

function buildActionQueue(snapshot: PortalDashboardSummary): DashboardActionItem[] {
  const actions: DashboardActionItem[] = [];
  const remainingUnits = snapshot.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY;

  if (snapshot.billing_summary.exhausted) {
    actions.push({
      id: 'restore-runway',
      title: 'Restore quota before the next traffic window',
      detail: 'The workspace has exhausted its visible quota ceiling. Recharge or move to a larger plan before production traffic resumes.',
      tone: 'warning',
      priority_label: 'Now',
      route: 'billing',
      action_label: 'Restore runway',
    });
  }

  if (snapshot.api_key_count === 0) {
    actions.push({
      id: 'issue-first-key',
      title: 'Issue the first environment key',
      detail: 'No API key is currently visible. Create at least one scoped key before connecting clients or teammates.',
      tone: 'warning',
      priority_label: 'Now',
      route: 'api-keys',
      action_label: 'Create key',
    });
  }

  if (snapshot.usage_summary.total_requests === 0) {
    actions.push({
      id: 'start-telemetry',
      title: 'Unlock live telemetry with the first request',
      detail: 'A single real request will populate provider, model, token-unit, and booked-amount visibility across the portal.',
      tone: 'accent',
      priority_label: 'Next',
      route: 'usage',
      action_label: 'Open usage',
    });
  }

  if (!snapshot.billing_summary.exhausted && remainingUnits < 5_000) {
    actions.push({
      id: 'protect-launch-buffer',
      title: 'Top up credits before runway gets tight',
      detail: `Only ${formatUnits(snapshot.billing_summary.remaining_units ?? 0)} token units remain in the visible buffer. Redeem a coupon or add a recharge pack before launch.`,
      tone: 'warning',
      priority_label: 'Watch',
      route: 'credits',
      action_label: 'Review credits',
    });
  }

  if (!actions.length) {
    actions.push({
      id: 'scale-deliberately',
      title: 'Promote the workspace toward steady production growth',
      detail: 'Keys, telemetry, and quota posture are all healthy. Review billing and usage together before increasing traffic or inviting more teams.',
      tone: 'positive',
      priority_label: 'Healthy',
      route: 'billing',
      action_label: 'Review growth plan',
    });
  }

  return actions;
}

function buildLaunchChecklist(snapshot: PortalDashboardSummary): DashboardChecklistItem[] {
  return [
    {
      id: 'identity',
      title: 'Workspace identity is bound',
      detail: `${snapshot.workspace.tenant.name} / ${snapshot.workspace.project.name} is already attached to this portal session.`,
      complete: Boolean(snapshot.workspace.project.id && snapshot.workspace.tenant.id),
      route: 'account',
      action_label: 'Review account',
    },
    {
      id: 'keys',
      title: 'Environment key exists',
      detail: snapshot.api_key_count > 0
        ? 'At least one key is visible for local, staging, or production usage.'
        : 'No environment key is visible yet. Issue one before client integration starts.',
      complete: snapshot.api_key_count > 0,
      route: 'api-keys',
      action_label: 'Manage keys',
    },
    {
      id: 'traffic',
      title: 'Telemetry has first request data',
      detail: snapshot.usage_summary.total_requests > 0
        ? 'Recent requests are already flowing and surfacing per-call token usage.'
        : 'Send the first real request so the portal can show live telemetry and cost posture.',
      complete: snapshot.usage_summary.total_requests > 0,
      route: 'usage',
      action_label: 'Open usage',
    },
    {
      id: 'budget',
      title: 'Launch buffer is protected',
      detail: snapshot.billing_summary.exhausted
        ? 'Quota is exhausted and needs a recharge or subscription change.'
        : snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined
          ? 'Unlimited quota posture is visible from the current billing boundary.'
          : `${formatUnits(snapshot.billing_summary.remaining_units)} token units remain before the visible quota limit is reached.`,
      complete: !snapshot.billing_summary.exhausted && (
        snapshot.billing_summary.remaining_units === null
          || snapshot.billing_summary.remaining_units === undefined
          || snapshot.billing_summary.remaining_units >= 5_000
      ),
      route: 'billing',
      action_label: 'Review billing',
    },
  ];
}

function buildProductionReadiness(
  snapshot: PortalDashboardSummary,
  checklist: DashboardChecklistItem[],
): DashboardProductionReadiness {
  const completed = checklist.filter((item) => item.complete);
  const pending = checklist.filter((item) => !item.complete);
  const score = Math.round((completed.length / checklist.length) * 100);
  const remainingUnits = snapshot.billing_summary.remaining_units;
  const detail = snapshot.billing_summary.exhausted
    ? 'Quota is exhausted, so the workspace is not ready for continued production traffic until billing posture is restored.'
    : remainingUnits === null || remainingUnits === undefined
      ? 'The workspace has completed the visible launch gates and currently shows unlimited quota posture.'
      : `The workspace has completed ${completed.length} of ${checklist.length} launch gates with ${formatUnits(remainingUnits)} token units remaining in the visible buffer.`;

  let title = 'Several launch gates remain';
  if (score === 100) {
    title = 'Ready for controlled production traffic';
  } else if (score >= 75) {
    title = 'One gate left before launch';
  } else if (score >= 50) {
    title = 'Foundation is in place';
  }

  return {
    score,
    title,
    detail,
    blockers: pending.map((item) => item.title),
    strengths: completed.map((item) => item.title),
  };
}

function buildJourneyGuide(checklist: DashboardChecklistItem[]): DashboardJourneyGuide {
  const completed = checklist.filter((item) => item.complete);
  const pending = checklist.filter((item) => !item.complete);
  const nextMilestone = pending[0] ?? checklist[checklist.length - 1];

  return {
    completed_count: completed.length,
    total_count: checklist.length,
    progress_label: `${completed.length}/${checklist.length} launch gates complete`,
    current_blocker: pending.length
      ? pending[0].title
      : 'No current blockers. The workspace is ready for controlled production growth.',
    next_milestone_title: pending.length
      ? pending[0].title
      : 'Sustain healthy workspace posture',
    next_milestone_detail: pending.length
      ? pending[0].detail
      : 'Keep monitoring usage, keys, and runway as traffic scales.',
    next_route: nextMilestone.route,
    next_action_label: nextMilestone.action_label,
  };
}

function buildEvidenceTimeline(snapshot: PortalDashboardSummary): DashboardEvidenceItem[] {
  const latestRequest = [...snapshot.recent_requests].sort(
    (left, right) => right.created_at_ms - left.created_at_ms,
  )[0];

  return [
    {
      id: 'workspace-created',
      title: 'Workspace boundary confirmed',
      detail: `${snapshot.workspace.tenant.name} / ${snapshot.workspace.project.name} is attached to the current portal identity.`,
      timestamp_label: formatDateTime(snapshot.workspace.user.created_at_ms),
    },
    {
      id: 'keys-visible',
      title: snapshot.api_key_count > 0 ? 'Credential inventory is visible' : 'Credential inventory still needs setup',
      detail: snapshot.api_key_count > 0
        ? `${formatUnits(snapshot.api_key_count)} API key(s) are visible inside the user portal boundary.`
        : 'No API key is currently visible, so client traffic still cannot authenticate safely.',
      timestamp_label: snapshot.api_key_count > 0 ? 'Live read' : 'Pending',
    },
    {
      id: 'latest-request',
      title: 'Last request',
      detail: latestRequest
        ? `${latestRequest.model} via ${latestRequest.provider} used ${formatUnits(latestRequest.units)} token units.`
        : 'The first request will create the initial usage evidence trail for this workspace.',
      timestamp_label: latestRequest ? formatDateTime(latestRequest.created_at_ms) : 'Pending',
    },
    {
      id: 'quota-posture',
      title: snapshot.billing_summary.exhausted ? 'Quota recovery is required' : 'Quota posture captured',
      detail: snapshot.billing_summary.exhausted
        ? 'Visible quota is exhausted and the billing path now becomes the primary recovery action.'
        : snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined
          ? 'The current billing boundary shows unlimited visible quota posture.'
          : `${formatUnits(snapshot.billing_summary.remaining_units)} token units remain in the current visible buffer.`,
      timestamp_label: 'Live billing read',
    },
  ];
}

function buildConfidenceSignals(snapshot: PortalDashboardSummary): DashboardConfidenceSignal[] {
  const signals: DashboardConfidenceSignal[] = [];

  if (snapshot.api_key_count > 0) {
    signals.push({
      id: 'keys',
      title: 'Credential evidence exists',
      detail: 'Environment-scoped credentials are visible, so authentication setup is no longer inferred.',
      tone: 'positive',
    });
  } else {
    signals.push({
      id: 'keys-missing',
      title: 'Credential evidence is still missing',
      detail: 'Until the first key is issued, launch readiness depends on planned work rather than live evidence.',
      tone: 'warning',
    });
  }

  if (snapshot.usage_summary.total_requests > 0) {
    signals.push({
      id: 'traffic',
      title: 'Traffic evidence is live',
      detail: 'The portal can show real provider, model, and token-unit behavior because requests have already flowed.',
      tone: 'positive',
    });
  } else {
    signals.push({
      id: 'traffic-pending',
      title: 'Traffic evidence is still pending',
      detail: 'The first request is still required before the workspace can validate its launch path with live telemetry.',
      tone: 'accent',
    });
  }

  signals.push({
    id: 'billing',
    title: snapshot.billing_summary.exhausted ? 'Runway evidence points to recovery' : 'Runway evidence is visible',
    detail: snapshot.billing_summary.exhausted
      ? 'Quota exhaustion is confirmed by the live billing summary, so billing recovery becomes a hard operational step.'
      : 'Credits and billing surfaces are backed by the current workspace summary rather than static assumptions.',
    tone: snapshot.billing_summary.exhausted ? 'warning' : 'positive',
  });

  return signals;
}

function buildModeNarrative(snapshot: PortalDashboardSummary): DashboardModeNarrative {
  if (snapshot.billing_summary.exhausted) {
    return {
      title: 'Recovery mode',
      detail: 'The workspace has real traffic or setup intent, but quota exhaustion means recovery is the dominant operating mode right now.',
      why_now: 'Billing posture is now a hard blocker, so the next action should restore runway before more traffic lands.',
      tone: 'warning',
    };
  }

  if (snapshot.api_key_count === 0 || snapshot.usage_summary.total_requests === 0) {
    return {
      title: 'Launch mode',
      detail: 'The portal is still moving from onboarding into the first validated request path.',
      why_now: 'Keys and traffic evidence are still incomplete, so the fastest path is to finish the first-launch loop before optimizing anything else.',
      tone: 'accent',
    };
  }

  return {
    title: 'Growth mode',
    detail: 'The workspace already has live credentials, live traffic, and visible runway, so the job shifts from setup into safe scaling.',
    why_now: 'Use the portal to validate burn, sustain runway, and keep the production path healthy as demand expands.',
    tone: 'positive',
  };
}

function buildDecisionPath(snapshot: PortalDashboardSummary): DashboardDecisionPathItem[] {
  if (snapshot.billing_summary.exhausted) {
    return [
      {
        id: 'restore-runway',
        title: 'Restore runway first',
        detail: 'Recharge or move to a stronger subscription so the workspace can accept more traffic safely.',
        route: 'billing',
        action_label: 'Review billing',
      },
      {
        id: 'validate-burn',
        title: 'Validate current demand',
        detail: 'Use recent requests to confirm whether recovery needs a top-up pack or a steadier subscription posture.',
        route: 'usage',
        action_label: 'Open usage',
      },
      {
        id: 'return-dashboard',
        title: 'Return to command center',
        detail: 'After recovery, verify that launch readiness and journey guidance have moved into a healthy state.',
        route: 'dashboard',
        action_label: 'Open dashboard',
      },
    ];
  }

  if (snapshot.api_key_count === 0 || snapshot.usage_summary.total_requests === 0) {
    return [
      {
        id: 'issue-key',
        title: 'Issue environment credentials',
        detail: 'Create the first key so external clients can authenticate against the gateway boundary.',
        route: 'api-keys',
        action_label: 'Manage keys',
      },
      {
        id: 'send-request',
        title: 'Create live telemetry',
        detail: 'Send the first request so the portal can capture real provider, model, and token-unit evidence.',
        route: 'usage',
        action_label: 'Open usage',
      },
      {
        id: 'protect-buffer',
        title: 'Confirm launch runway',
        detail: 'Before broader rollout, validate that credits and billing posture can absorb the first traffic window.',
        route: 'credits',
        action_label: 'Review credits',
      },
    ];
  }

  return [
    {
      id: 'watch-demand',
      title: 'Watch live demand',
      detail: 'Use usage diagnostics to understand whether growth is concentrated, spiky, or stable.',
      route: 'usage',
      action_label: 'Open usage',
    },
    {
      id: 'shape-commercial-path',
      title: 'Shape the next commercial step',
      detail: 'Move from current burn evidence into the plan or pack path that best supports steady traffic growth.',
      route: 'billing',
      action_label: 'Review billing',
    },
    {
      id: 'keep-trust-aligned',
      title: 'Keep trust and credentials aligned',
      detail: 'As the workspace grows, maintain account trust and environment credential hygiene together.',
      route: 'account',
      action_label: 'Review account',
    },
  ];
}

function buildRouteSignals(snapshot: PortalDashboardSummary): DashboardRouteSignal[] {
  const remainingUnits = snapshot.billing_summary.remaining_units;

  return [
    {
      route: 'dashboard',
      title: 'Dashboard',
      status_label: 'Lead',
      detail: 'The main command surface for launch, growth, and recovery posture.',
      tone: 'accent',
    },
    {
      route: 'api-keys',
      title: 'API Keys',
      status_label: snapshot.api_key_count > 0 ? 'Ready' : 'Needs action',
      detail: snapshot.api_key_count > 0
        ? `${formatUnits(snapshot.api_key_count)} key(s) are visible across environments.`
        : 'Credential setup is still incomplete.',
      tone: snapshot.api_key_count > 0 ? 'positive' : 'warning',
    },
    {
      route: 'usage',
      title: 'Usage',
      status_label: snapshot.usage_summary.total_requests > 0 ? 'Live' : 'Needs action',
      detail: snapshot.usage_summary.total_requests > 0
        ? `${formatUnits(snapshot.usage_summary.total_requests)} request(s) are already recorded.`
        : 'The first request is still needed to create live telemetry.',
      tone: snapshot.usage_summary.total_requests > 0 ? 'positive' : 'accent',
    },
    {
      route: 'credits',
      title: 'Credits',
      status_label: snapshot.billing_summary.exhausted
        ? 'Escalate'
        : remainingUnits === null || remainingUnits === undefined || remainingUnits >= 5_000
          ? 'Stable'
          : 'Watch',
      detail: snapshot.billing_summary.exhausted
        ? 'Quota is exhausted and credits posture needs immediate attention.'
        : remainingUnits === null || remainingUnits === undefined
          ? 'Visible quota posture is unlimited.'
          : `${formatUnits(remainingUnits)} token units remain in the visible buffer.`,
      tone: snapshot.billing_summary.exhausted
        ? 'warning'
        : remainingUnits === null || remainingUnits === undefined || remainingUnits >= 5_000
          ? 'positive'
          : 'warning',
    },
    {
      route: 'billing',
      title: 'Billing',
      status_label: snapshot.billing_summary.exhausted ? 'Urgent' : 'Plan',
      detail: snapshot.billing_summary.exhausted
        ? 'Billing recovery is required before the next traffic window.'
        : 'Billing remains the path for subscription shaping and recovery planning.',
      tone: snapshot.billing_summary.exhausted ? 'warning' : 'default',
    },
    {
      route: 'account',
      title: 'Account',
      status_label: snapshot.workspace.user.active ? 'Healthy' : 'Review',
      detail: snapshot.workspace.user.active
        ? 'Workspace identity is active and trusted inside the portal boundary.'
        : 'Account status needs review before more self-service work continues.',
      tone: snapshot.workspace.user.active ? 'positive' : 'warning',
    },
  ];
}

function buildReviewCadence(snapshot: PortalDashboardSummary): DashboardReviewCadenceItem[] {
  return [
    {
      id: 'before-traffic',
      title: 'Before traffic',
      detail: snapshot.api_key_count > 0
        ? 'Recheck keys, route signals, and launch posture before widening rollout.'
        : 'Complete the first key and first request loop before treating the workspace as launch-ready.',
      route: snapshot.api_key_count > 0 ? 'dashboard' : 'api-keys',
      action_label: snapshot.api_key_count > 0 ? 'Open dashboard' : 'Manage keys',
    },
    {
      id: 'during-traffic',
      title: 'During live traffic',
      detail: snapshot.usage_summary.total_requests > 0
        ? 'Use usage diagnostics and evidence timeline as the standard review lane while traffic is active.'
        : 'Once traffic starts, usage becomes the main daily review surface for demand, provider mix, and token burn.',
      route: 'usage',
      action_label: 'Open usage',
    },
    {
      id: 'if-risk-appears',
      title: 'If risk appears',
      detail: snapshot.billing_summary.exhausted
        ? 'Runway recovery is already active, so billing and credits become the priority control surfaces.'
        : 'When runway shrinks or launch posture degrades, move into credits and billing before the next traffic window.',
      route: snapshot.billing_summary.exhausted ? 'billing' : 'credits',
      action_label: snapshot.billing_summary.exhausted ? 'Review billing' : 'Review credits',
    },
  ];
}

function buildPlaybookLane(snapshot: PortalDashboardSummary): DashboardPlaybookLaneItem[] {
  if (snapshot.billing_summary.exhausted) {
    return [
      {
        id: 'recovery-billing',
        title: 'Recover runway',
        detail: 'Restore quota first so other operational work stops being blocked by billing posture.',
        route: 'billing',
        action_label: 'Review billing',
        tone: 'warning',
      },
      {
        id: 'recovery-usage',
        title: 'Validate real demand',
        detail: 'Check usage evidence so recovery maps to actual burn instead of guesswork.',
        route: 'usage',
        action_label: 'Open usage',
        tone: 'accent',
      },
      {
        id: 'recovery-dashboard',
        title: 'Reconfirm command posture',
        detail: 'After recovery, return to dashboard and verify that route signals and journey status have improved.',
        route: 'dashboard',
        action_label: 'Open dashboard',
        tone: 'positive',
      },
    ];
  }

  if (snapshot.api_key_count === 0 || snapshot.usage_summary.total_requests === 0) {
    return [
      {
        id: 'launch-keys',
        title: 'Issue credentials',
        detail: 'Create environment keys so the workspace can establish a safe traffic boundary.',
        route: 'api-keys',
        action_label: 'Manage keys',
        tone: 'accent',
      },
      {
        id: 'launch-usage',
        title: 'Create live evidence',
        detail: 'Send the first request so the portal stops relying on planned state and starts using real telemetry.',
        route: 'usage',
        action_label: 'Open usage',
        tone: 'warning',
      },
      {
        id: 'launch-runway',
        title: 'Protect the first launch window',
        detail: 'Use credits and billing surfaces to confirm that the initial rollout has enough runway.',
        route: 'credits',
        action_label: 'Review credits',
        tone: 'positive',
      },
    ];
  }

  return [
    {
      id: 'growth-usage',
      title: 'Watch live demand',
      detail: 'Usage becomes the default operating lane once the workspace has crossed into live traffic.',
      route: 'usage',
      action_label: 'Open usage',
      tone: 'positive',
    },
    {
      id: 'growth-billing',
      title: 'Shape the next plan',
      detail: 'Use billing when real burn suggests the workspace is moving beyond ad hoc top-ups.',
      route: 'billing',
      action_label: 'Review billing',
      tone: 'accent',
    },
    {
      id: 'growth-trust',
      title: 'Keep trust aligned',
      detail: 'As the workspace grows, keep account trust and route-level signal health aligned with the traffic path.',
      route: 'account',
      action_label: 'Review account',
      tone: 'default',
    },
  ];
}

function buildFocusBoard(snapshot: PortalDashboardSummary): DashboardFocusBoardItem[] {
  if (snapshot.billing_summary.exhausted) {
    return [
      {
        id: 'focus-recover-runway',
        title: 'Restore billing runway',
        detail: 'Quota is exhausted, so billing recovery has to happen before the next production traffic window can continue safely.',
        priority_label: 'Top focus',
        route: 'billing',
        action_label: 'Review billing',
        tone: 'warning',
      },
      {
        id: 'focus-validate-burn',
        title: 'Validate live burn',
        detail: 'Use recent request evidence to decide whether recovery needs a targeted top-up or a stronger recurring plan.',
        priority_label: 'Next',
        route: 'usage',
        action_label: 'Open usage',
        tone: 'accent',
      },
      {
        id: 'focus-reconfirm-posture',
        title: 'Reconfirm command posture',
        detail: 'Return to dashboard after recovery so the workspace journey and route signals move back into a healthy state.',
        priority_label: 'Then',
        route: 'dashboard',
        action_label: 'Open dashboard',
        tone: 'positive',
      },
    ];
  }

  if (snapshot.api_key_count === 0) {
    return [
      {
        id: 'focus-finish-credentials',
        title: 'Finish credential setup',
        detail: 'Create the first environment key so clients and teammates can authenticate against the portal boundary safely.',
        priority_label: 'Top focus',
        route: 'api-keys',
        action_label: 'Manage keys',
        tone: 'warning',
      },
      {
        id: 'focus-create-telemetry',
        title: 'Create live telemetry',
        detail: 'Send the first request after the key exists so the portal can replace planned posture with live usage evidence.',
        priority_label: 'Next',
        route: 'usage',
        action_label: 'Open usage',
        tone: 'accent',
      },
      {
        id: 'focus-protect-runway',
        title: 'Protect the launch buffer',
        detail: 'Before broader rollout, confirm that credits and billing posture can absorb the first production window.',
        priority_label: 'Then',
        route: 'credits',
        action_label: 'Review credits',
        tone: 'positive',
      },
    ];
  }

  if (snapshot.usage_summary.total_requests === 0) {
    return [
      {
        id: 'focus-send-first-request',
        title: 'Create the first live request',
        detail: 'Traffic evidence is still missing, so the fastest path is to send a real request and unlock model, provider, and token-unit visibility.',
        priority_label: 'Top focus',
        route: 'usage',
        action_label: 'Open usage',
        tone: 'accent',
      },
      {
        id: 'focus-verify-credentials',
        title: 'Verify credential boundary',
        detail: 'Use the API key surface to confirm the active environment key is the one you want leading the initial launch path.',
        priority_label: 'Next',
        route: 'api-keys',
        action_label: 'Manage keys',
        tone: 'positive',
      },
      {
        id: 'focus-confirm-runway',
        title: 'Confirm launch runway',
        detail: 'Validate the initial credit and billing posture before turning a first successful request into broader rollout.',
        priority_label: 'Then',
        route: 'credits',
        action_label: 'Review credits',
        tone: 'positive',
      },
    ];
  }

  return [
    {
      id: 'focus-review-demand',
      title: 'Review live demand',
      detail: 'Traffic is already flowing, so usage becomes the primary surface for spotting spikes, provider mix, and burn concentration.',
      priority_label: 'Top focus',
      route: 'usage',
      action_label: 'Open usage',
      tone: 'positive',
    },
    {
      id: 'focus-shape-plan',
      title: 'Shape the next commercial step',
      detail: 'Move from current burn evidence into the plan or pack posture that best supports the next traffic stage.',
      priority_label: 'Next',
      route: 'billing',
      action_label: 'Review billing',
      tone: 'accent',
    },
    {
      id: 'focus-keep-trust-aligned',
      title: 'Keep trust and credentials aligned',
      detail: 'Use account and key hygiene reviews to keep the production path trustworthy as the workspace scales.',
      priority_label: 'Then',
      route: 'account',
      action_label: 'Review account',
      tone: 'default',
    },
  ];
}

function buildRiskWatchlist(snapshot: PortalDashboardSummary): DashboardRiskWatchItem[] {
  const remainingUnits = snapshot.billing_summary.remaining_units;

  return [
    {
      id: 'risk-runway',
      title: 'Runway posture',
      detail: snapshot.billing_summary.exhausted
        ? 'Quota exhaustion is confirmed by the live billing boundary, so traffic recovery now depends on billing action.'
        : remainingUnits === null || remainingUnits === undefined
          ? 'The current workspace shows unlimited visible quota posture.'
          : `${formatUnits(remainingUnits)} token units remain before the visible launch buffer is exhausted.`,
      status_label: snapshot.billing_summary.exhausted
        ? 'Urgent'
        : remainingUnits === null || remainingUnits === undefined || remainingUnits >= 5_000
          ? 'Stable'
          : 'Watch',
      route: snapshot.billing_summary.exhausted ? 'billing' : 'credits',
      action_label: snapshot.billing_summary.exhausted ? 'Review billing' : 'Review credits',
      tone: snapshot.billing_summary.exhausted
        ? 'warning'
        : remainingUnits === null || remainingUnits === undefined || remainingUnits >= 5_000
          ? 'positive'
          : 'warning',
    },
    {
      id: 'risk-credentials',
      title: 'Credential posture',
      detail: snapshot.api_key_count > 0
        ? `${formatUnits(snapshot.api_key_count)} environment key(s) are visible and can anchor the current traffic path.`
        : 'No API key is currently visible, so launch confidence still depends on pending credential setup.',
      status_label: snapshot.api_key_count > 0 ? 'Ready' : 'Missing',
      route: 'api-keys',
      action_label: 'Manage keys',
      tone: snapshot.api_key_count > 0 ? 'positive' : 'warning',
    },
    {
      id: 'risk-traffic',
      title: 'Traffic evidence',
      detail: snapshot.usage_summary.total_requests > 0
        ? `${formatUnits(snapshot.usage_summary.total_requests)} request(s) are already recorded, so the portal is reading live traffic instead of planned state.`
        : 'The first request is still missing, so usage, cost, and provider guidance remain partially theoretical.',
      status_label: snapshot.usage_summary.total_requests > 0 ? 'Live' : 'Pending',
      route: 'usage',
      action_label: 'Open usage',
      tone: snapshot.usage_summary.total_requests > 0 ? 'positive' : 'accent',
    },
    {
      id: 'risk-account',
      title: 'Account trust',
      detail: snapshot.workspace.user.active
        ? 'The current workspace identity is active and trusted inside the portal boundary.'
        : 'Account status needs review before the workspace should continue self-service operations.',
      status_label: snapshot.workspace.user.active ? 'Healthy' : 'Review',
      route: 'account',
      action_label: 'Review account',
      tone: snapshot.workspace.user.active ? 'positive' : 'warning',
    },
  ];
}

function buildDailyBrief(
  snapshot: PortalDashboardSummary,
  focus_board: DashboardFocusBoardItem[],
  risk_watchlist: DashboardRiskWatchItem[],
): DashboardDailyBrief {
  const top_focus = focus_board[0];
  const risk_watch = risk_watchlist.find((item) => item.tone === 'warning' || item.tone === 'accent') ?? risk_watchlist[0];

  if (snapshot.billing_summary.exhausted) {
    return {
      title: 'Recovery is the operating priority today',
      detail: 'The portal has enough live evidence to show that runway recovery outranks every other action until quota posture is healthy again.',
      top_focus,
      risk_watch,
    };
  }

  if (snapshot.api_key_count === 0) {
    return {
      title: 'Finish credential setup and unlock live evidence',
      detail: 'The workspace is still in launch mode, so today should be biased toward the first key and the first real request path.',
      top_focus,
      risk_watch,
    };
  }

  if (snapshot.usage_summary.total_requests === 0) {
    return {
      title: 'Create the first live request before scaling effort',
      detail: 'The portal can already trust the credential boundary, but it still needs real traffic evidence before the rest of the product can guide with confidence.',
      top_focus,
      risk_watch,
    };
  }

  return {
    title: 'Use live demand to steer the next growth move',
    detail: 'The workspace is healthy enough for a daily operating review, so focus should shift toward burn posture, plan shaping, and trust hygiene.',
    top_focus,
    risk_watch,
  };
}

export function buildPortalDashboardViewModel(
  snapshot: PortalDashboardSummary,
): PortalDashboardPageViewModel {
  const action_queue = buildActionQueue(snapshot);
  const launch_checklist = buildLaunchChecklist(snapshot);
  const production_readiness = buildProductionReadiness(snapshot, launch_checklist);
  const journey = buildJourneyGuide(launch_checklist);
  const evidence_timeline = buildEvidenceTimeline(snapshot);
  const confidence_signals = buildConfidenceSignals(snapshot);
  const mode = buildModeNarrative(snapshot);
  const decision_path = buildDecisionPath(snapshot);
  const route_signals = buildRouteSignals(snapshot);
  const review_cadence = buildReviewCadence(snapshot);
  const playbook_lane = buildPlaybookLane(snapshot);
  const focus_board = buildFocusBoard(snapshot);
  const risk_watchlist = buildRiskWatchlist(snapshot);
  const daily_brief = buildDailyBrief(snapshot, focus_board, risk_watchlist);

  return {
    snapshot,
    insights: buildInsights(snapshot),
    readiness: buildReadiness(snapshot),
    action_queue,
    launch_checklist,
    production_readiness,
    journey,
    evidence_timeline,
    confidence_signals,
    mode,
    decision_path,
    route_signals,
    review_cadence,
    playbook_lane,
    focus_board,
    risk_watchlist,
    daily_brief,
  };
}
