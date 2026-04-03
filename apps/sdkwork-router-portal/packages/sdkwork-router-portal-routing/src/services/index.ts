import type {
  PortalRoutingAssessment,
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingSummary,
  PortalRoutingStrategy,
} from 'sdkwork-router-portal-types';
import { formatDateTime } from 'sdkwork-router-portal-commons/format-core';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';

import type {
  PortalRoutingPageViewModel,
  RoutingEvidenceItem,
  RoutingGuardrailItem,
  RoutingPresetCard,
} from '../types';

function normalizeRoutingAssessment(
  assessment: Partial<PortalRoutingAssessment> | null | undefined,
): PortalRoutingAssessment {
  return {
    provider_id: assessment?.provider_id ?? translatePortalText('Unknown provider'),
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
    selected_provider_id: decision?.selected_provider_id ?? translatePortalText('Unavailable'),
    candidate_ids: Array.isArray(decision?.candidate_ids) ? decision.candidate_ids : [],
    matched_policy_id: decision?.matched_policy_id ?? null,
    applied_routing_profile_id: decision?.applied_routing_profile_id ?? null,
    compiled_routing_snapshot_id: decision?.compiled_routing_snapshot_id ?? null,
    strategy: decision?.strategy ?? null,
    selection_seed: decision?.selection_seed ?? null,
    selection_reason: decision?.selection_reason ?? null,
    fallback_reason: decision?.fallback_reason ?? null,
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
    api_key_group_id: log.api_key_group_id ?? null,
    capability: log.capability ?? 'unknown',
    route_key: log.route_key ?? 'unknown',
    selected_provider_id: log.selected_provider_id ?? translatePortalText('Unavailable'),
    matched_policy_id: log.matched_policy_id ?? null,
    applied_routing_profile_id: log.applied_routing_profile_id ?? null,
    compiled_routing_snapshot_id: log.compiled_routing_snapshot_id ?? null,
    strategy: log.strategy ?? 'unknown',
    selection_seed: log.selection_seed ?? null,
    selection_reason: log.selection_reason ?? null,
    fallback_reason: log.fallback_reason ?? null,
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
      return translatePortalText('Predictable order');
    case 'weighted_random':
      return translatePortalText('Traffic distribution');
    case 'slo_aware':
      return translatePortalText('Reliability guardrails');
    case 'geo_affinity':
      return translatePortalText('Regional preference');
    case 'static_fallback':
      return translatePortalText('Platform fallback');
    default:
      return translatePortalText('Adaptive routing');
  }
}

function humanizeRoutingToken(value?: string | null): string {
  const normalized = value?.trim();

  if (!normalized) {
    return '';
  }

  return normalized
    .split(/[_./-]+/)
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ');
}

export function buildRoutingCapabilityLabel(
  capability?: string | null,
): string {
  switch (capability?.trim().toLowerCase()) {
    case 'chat':
    case 'chat_completion':
    case 'chat_completions':
      return translatePortalText('Chat completions');
    case 'responses':
      return translatePortalText('Responses');
    case 'embeddings':
      return translatePortalText('Embeddings');
    case 'image_generation':
    case 'images':
      return translatePortalText('Image generation');
    case 'music':
    case 'music_generation':
      return translatePortalText('Music generation');
    case 'audio':
    case 'audio_transcription':
      return translatePortalText('Audio transcription');
    case 'multimodal':
      return translatePortalText('Multimodal reasoning');
    default:
      return humanizeRoutingToken(capability) || translatePortalText('Unknown capability');
  }
}

export function buildRoutingDecisionSourceLabel(
  source?: string | null,
): string {
  switch (source?.trim().toLowerCase()) {
    case 'gateway':
      return translatePortalText('Live traffic');
    case 'admin_simulation':
      return translatePortalText('Control plane simulation');
    case 'portal_simulation':
      return translatePortalText('Preview request');
    default:
      return humanizeRoutingToken(source) || translatePortalText('Unknown source');
  }
}

export function buildRoutingAssessmentHealthLabel(
  health?: string | null,
): string {
  switch (health?.trim().toLowerCase()) {
    case 'healthy':
      return translatePortalText('Healthy');
    case 'unhealthy':
      return translatePortalText('Unhealthy');
    default:
      return translatePortalText('Unknown');
  }
}

function buildPresetCards(
  preferences: PortalRoutingPreferences,
): RoutingPresetCard[] {
  return [
    {
      id: 'predictable',
      title: translatePortalText('Predictable order'),
      detail: translatePortalText(
        'The first healthy available provider in your ordered list wins, and the next provider becomes the deterministic fallback.',
      ),
      strategy: 'deterministic_priority',
      active: preferences.strategy === 'deterministic_priority',
    },
    {
      id: 'distribution',
      title: translatePortalText('Traffic distribution'),
      detail: translatePortalText(
        'Spread traffic across eligible providers when you want to balance exposure instead of pinning every request to one path.',
      ),
      strategy: 'weighted_random',
      active: preferences.strategy === 'weighted_random',
    },
    {
      id: 'reliability',
      title: translatePortalText('Reliability guardrails'),
      detail: translatePortalText(
        'Bias toward healthy, low-latency, and policy-compliant providers when production confidence matters more than raw spread.',
      ),
      strategy: 'slo_aware',
      active: preferences.strategy === 'slo_aware',
    },
    {
      id: 'regional',
      title: translatePortalText('Regional preference'),
      detail: translatePortalText(
        'Prefer providers that match the target region so routing stays closer to user locality and compliance boundaries.',
      ),
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
      label: translatePortalText('Default provider'),
      value: preferences.default_provider_id ?? translatePortalText('Auto'),
      detail: translatePortalText(
        'A default provider acts as the stable fallback when multiple candidates remain eligible.',
      ),
    },
    {
      id: 'cost',
      label: translatePortalText('Max cost'),
      value: preferences.max_cost === null || preferences.max_cost === undefined
        ? translatePortalText('Open')
        : `$${preferences.max_cost.toFixed(2)}`,
      detail: translatePortalText(
        'Keep a cost ceiling visible so route posture reflects commercial intent, not only technical possibility.',
      ),
    },
    {
      id: 'latency',
      label: translatePortalText('Max latency'),
      value: preferences.max_latency_ms === null || preferences.max_latency_ms === undefined
        ? translatePortalText('Open')
        : `${preferences.max_latency_ms}ms`,
      detail: translatePortalText(
        'Latency guardrails let the workspace make reliability posture explicit before traffic starts flowing.',
      ),
    },
    {
      id: 'region',
      label: translatePortalText('Preferred region'),
      value: preview.requested_region ?? preferences.preferred_region ?? translatePortalText('Auto'),
      detail: translatePortalText(
        'The active route preview should always show the region signal that influenced provider selection.',
      ),
    },
  ];
}

function buildEvidence(
  logs: PortalRoutingDecisionLog[],
): RoutingEvidenceItem[] {
  return logs.slice(0, 4).map((log) => ({
    id: log.decision_id,
    title: translatePortalText('{routeKey} -> {providerId}', {
      routeKey: log.route_key,
      providerId: log.selected_provider_id,
    }),
    detail: translatePortalText('{source} used {strategy}{regionSuffix}.', {
      source: buildRoutingDecisionSourceLabel(log.decision_source),
      strategy: buildRoutingStrategyLabel(log.strategy),
      regionSuffix: log.requested_region
        ? translatePortalText(' in {region}', { region: log.requested_region })
        : '',
    }),
    timestamp_label: formatDateTime(log.created_at_ms),
    snapshot_id: log.compiled_routing_snapshot_id ?? null,
    fallback_reason: log.fallback_reason ?? null,
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
