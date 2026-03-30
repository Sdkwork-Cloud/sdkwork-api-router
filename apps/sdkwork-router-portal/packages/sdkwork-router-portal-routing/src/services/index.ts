import type {
  PortalRoutingAssessment,
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingSummary,
  PortalRoutingStrategy,
} from 'sdkwork-router-portal-types';

import type {
  PortalRoutingPageViewModel,
  RoutingEvidenceItem,
  RoutingGuardrailItem,
  RoutingPresetCard,
} from '../types';

function formatRoutingDateTime(timestamp: number): string {
  if (!timestamp) {
    return 'Pending';
  }

  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(timestamp));
}

function normalizeRoutingAssessment(
  assessment: Partial<PortalRoutingAssessment> | null | undefined,
): PortalRoutingAssessment {
  return {
    provider_id: assessment?.provider_id ?? 'Unknown provider',
    available: assessment?.available ?? false,
    health: assessment?.health ?? 'unknown',
    policy_rank: assessment?.policy_rank ?? 0,
    weight: assessment?.weight ?? null,
    cost: assessment?.cost ?? null,
    latency_ms: assessment?.latency_ms ?? null,
    region: assessment?.region ?? null,
    region_match: assessment?.region_match ?? null,
    slo_eligible: assessment?.slo_eligible ?? null,
    slo_violations: Array.isArray(assessment?.slo_violations) ? assessment.slo_violations : [],
    reasons: Array.isArray(assessment?.reasons) ? assessment.reasons : [],
  };
}

function normalizeRoutingDecision(
  decision: Partial<PortalRoutingDecision> | null | undefined,
): PortalRoutingDecision {
  return {
    selected_provider_id: decision?.selected_provider_id ?? 'Unavailable',
    candidate_ids: Array.isArray(decision?.candidate_ids) ? decision.candidate_ids : [],
    matched_policy_id: decision?.matched_policy_id ?? null,
    strategy: decision?.strategy ?? null,
    selection_seed: decision?.selection_seed ?? null,
    selection_reason: decision?.selection_reason ?? null,
    requested_region: decision?.requested_region ?? null,
    slo_applied: decision?.slo_applied ?? false,
    slo_degraded: decision?.slo_degraded ?? false,
    assessments: Array.isArray(decision?.assessments)
      ? decision.assessments.map((assessment) => normalizeRoutingAssessment(assessment))
      : [],
  };
}

function normalizeRoutingDecisionLog(
  log: Partial<PortalRoutingDecisionLog>,
): PortalRoutingDecisionLog {
  return {
    decision_id: log.decision_id ?? 'unknown-decision',
    decision_source: log.decision_source ?? 'unknown',
    tenant_id: log.tenant_id ?? null,
    project_id: log.project_id ?? null,
    capability: log.capability ?? 'unknown',
    route_key: log.route_key ?? 'unknown',
    selected_provider_id: log.selected_provider_id ?? 'Unavailable',
    matched_policy_id: log.matched_policy_id ?? null,
    strategy: log.strategy ?? 'unknown',
    selection_seed: log.selection_seed ?? null,
    selection_reason: log.selection_reason ?? null,
    requested_region: log.requested_region ?? null,
    slo_applied: log.slo_applied ?? false,
    slo_degraded: log.slo_degraded ?? false,
    created_at_ms: log.created_at_ms ?? 0,
    assessments: Array.isArray(log.assessments)
      ? log.assessments.map((assessment) => normalizeRoutingAssessment(assessment))
      : [],
  };
}

export function buildRoutingStrategyLabel(
  strategy?: PortalRoutingStrategy | string | null,
): string {
  switch (strategy) {
    case 'deterministic_priority':
      return 'Predictable order';
    case 'weighted_random':
      return 'Traffic distribution';
    case 'slo_aware':
      return 'Reliability guardrails';
    case 'geo_affinity':
      return 'Regional preference';
    case 'static_fallback':
      return 'Platform fallback';
    default:
      return 'Adaptive routing';
  }
}

function buildPresetCards(
  preferences: PortalRoutingPreferences,
): RoutingPresetCard[] {
  return [
    {
      id: 'predictable',
      title: 'Predictable order',
      detail: 'The first healthy available provider in your ordered list wins, and the next provider becomes the deterministic fallback.',
      strategy: 'deterministic_priority',
      active: preferences.strategy === 'deterministic_priority',
    },
    {
      id: 'distribution',
      title: 'Traffic distribution',
      detail: 'Spread traffic across eligible providers when you want to balance exposure instead of pinning every request to one path.',
      strategy: 'weighted_random',
      active: preferences.strategy === 'weighted_random',
    },
    {
      id: 'reliability',
      title: 'Reliability guardrails',
      detail: 'Bias toward healthy, low-latency, and policy-compliant providers when production confidence matters more than raw spread.',
      strategy: 'slo_aware',
      active: preferences.strategy === 'slo_aware',
    },
    {
      id: 'regional',
      title: 'Regional preference',
      detail: 'Prefer providers that match the target region so routing stays closer to user locality and compliance boundaries.',
      strategy: 'geo_affinity',
      active: preferences.strategy === 'geo_affinity',
    },
  ];
}

function buildGuardrails(
  preferences: PortalRoutingPreferences,
  preview: PortalRoutingDecision,
): RoutingGuardrailItem[] {
  return [
    {
      id: 'provider-default',
      label: 'Default provider',
      value: preferences.default_provider_id ?? 'Auto',
      detail: 'A default provider acts as the stable fallback when multiple candidates remain eligible.',
    },
    {
      id: 'cost',
      label: 'Max cost',
      value: preferences.max_cost === null || preferences.max_cost === undefined
        ? 'Open'
        : `$${preferences.max_cost.toFixed(2)}`,
      detail: 'Keep a cost ceiling visible so route posture reflects commercial intent, not only technical possibility.',
    },
    {
      id: 'latency',
      label: 'Max latency',
      value: preferences.max_latency_ms === null || preferences.max_latency_ms === undefined
        ? 'Open'
        : `${preferences.max_latency_ms}ms`,
      detail: 'Latency guardrails let the workspace make reliability posture explicit before traffic starts flowing.',
    },
    {
      id: 'region',
      label: 'Preferred region',
      value: preview.requested_region ?? preferences.preferred_region ?? 'Auto',
      detail: 'The active route preview should always show the region signal that influenced provider selection.',
    },
  ];
}

function buildEvidence(
  logs: PortalRoutingDecisionLog[],
): RoutingEvidenceItem[] {
  return logs.slice(0, 4).map((log) => ({
    id: log.decision_id,
    title: `${log.route_key} -> ${log.selected_provider_id}`,
    detail: `${log.decision_source} used ${log.strategy}${log.requested_region ? ` in ${log.requested_region}` : ''}.`,
    timestamp_label: formatRoutingDateTime(log.created_at_ms),
  }));
}

export function buildPortalRoutingViewModel(
  summary: PortalRoutingSummary,
  logs: PortalRoutingDecisionLog[],
  preview?: PortalRoutingDecision | null,
): PortalRoutingPageViewModel {
  const normalizedLogs = Array.isArray(logs)
    ? logs.map((log) => normalizeRoutingDecisionLog(log))
    : [];
  const normalizedSummaryPreview = normalizeRoutingDecision(summary.preview);
  const activePreview = normalizeRoutingDecision(preview ?? normalizedSummaryPreview);

  return {
    summary,
    preview: activePreview,
    preset_cards: buildPresetCards(summary.preferences),
    guardrails: buildGuardrails(summary.preferences, activePreview),
    evidence: buildEvidence(normalizedLogs),
    provider_options: summary.provider_options,
    logs: normalizedLogs,
  };
}
