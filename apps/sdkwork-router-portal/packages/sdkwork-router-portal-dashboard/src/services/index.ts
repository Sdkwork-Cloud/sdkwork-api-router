import {
  formatCurrency,
  formatDateTime,
  formatUnits,
} from 'sdkwork-router-portal-commons/format-core';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';
import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  PortalCommerceMembership,
  PortalDashboardSummary,
  PortalRouteKey,
  PortalRoutingDecisionLog,
  PortalRoutingProviderOption,
  PortalRoutingSummary,
  PortalRoutingStrategy,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import type {
  DashboardActivityItem,
  DashboardBalanceSummary,
  DashboardBreakdownItem,
  DashboardCommercialHighlights,
  DashboardDemandPoint,
  DashboardDistributionPoint,
  DashboardInsight,
  DashboardMetric,
  DashboardMetricSummary,
  DashboardModuleItem,
  DashboardRoutingPosture,
  DashboardSeriesPoint,
  DashboardStatusVariant,
  DashboardSpendTrendPoint,
  DashboardTrafficTrendPoint,
  PortalDashboardPageViewModel,
} from '../types';

function safeArray<T>(value: T[] | null | undefined): T[] {
  return Array.isArray(value) ? value : [];
}

function normalizeDashboardSummary(snapshot: PortalDashboardSummary): PortalDashboardSummary {
  return {
    ...snapshot,
    usage_summary: {
      ...snapshot.usage_summary,
      projects: safeArray(snapshot.usage_summary.projects),
      providers: safeArray(snapshot.usage_summary.providers),
      models: safeArray(snapshot.usage_summary.models),
    },
    recent_requests: safeArray(snapshot.recent_requests),
  };
}

function normalizeRoutingSummary(
  routingSummary?: PortalRoutingSummary | null,
): PortalRoutingSummary | null {
  if (!routingSummary) {
    return null;
  }

  return {
    ...routingSummary,
    preferences: {
      ...routingSummary.preferences,
      ordered_provider_ids: safeArray(routingSummary.preferences.ordered_provider_ids),
    },
    preview: {
      ...routingSummary.preview,
      candidate_ids: safeArray(routingSummary.preview.candidate_ids),
      assessments: safeArray(routingSummary.preview.assessments),
    },
    provider_options: safeArray(routingSummary.provider_options),
  };
}

function normalizeRoutingLogs(logs?: PortalRoutingDecisionLog[] | null): PortalRoutingDecisionLog[] {
  return safeArray(logs).map((log) => ({
    ...log,
    assessments: safeArray(log.assessments),
  }));
}

function startOfDayMs(value: number): number {
  const date = new Date(value);
  date.setHours(0, 0, 0, 0);
  return date.getTime();
}

function startOfTrailing7dMs(value: number): number {
  const date = new Date(value);
  date.setHours(0, 0, 0, 0);
  date.setDate(date.getDate() - 6);
  return date.getTime();
}

function startOfMonthMs(value: number): number {
  const date = new Date(value);
  return new Date(date.getFullYear(), date.getMonth(), 1).getTime();
}

function summarizeUsageRecords(records: UsageRecord[]): DashboardMetricSummary {
  const revenue = records.reduce((sum, record) => sum + record.amount, 0);
  const request_count = records.length;
  const used_units = records.reduce((sum, record) => sum + record.units, 0);

  return {
    revenue,
    request_count,
    used_units,
    average_booked_spend: request_count > 0 ? revenue / request_count : 0,
  };
}

function emptyBillingEventSummary(): BillingEventSummary {
  return {
    total_events: 0,
    project_count: 0,
    group_count: 0,
    capability_count: 0,
    total_request_count: 0,
    total_units: 0,
    total_input_tokens: 0,
    total_output_tokens: 0,
    total_tokens: 0,
    total_image_count: 0,
    total_audio_seconds: 0,
    total_video_seconds: 0,
    total_music_seconds: 0,
    total_upstream_cost: 0,
    total_customer_charge: 0,
    projects: [],
    groups: [],
    capabilities: [],
    accounting_modes: [],
  };
}

function sortAccountingModes(
  items: BillingEventAccountingModeSummary[],
): BillingEventAccountingModeSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || left.accounting_mode.localeCompare(right.accounting_mode),
    );
}

function sortCapabilities(
  items: BillingEventCapabilitySummary[],
): BillingEventCapabilitySummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || left.capability.localeCompare(right.capability),
    );
}

export function buildDashboardCommercialHighlights(
  summary?: BillingEventSummary | null,
): DashboardCommercialHighlights {
  const safeSummary = summary ?? emptyBillingEventSummary();

  return {
    total_customer_charge: safeSummary.total_customer_charge,
    leading_accounting_mode: sortAccountingModes(safeSummary.accounting_modes)[0] ?? null,
    leading_capability: sortCapabilities(safeSummary.capabilities)[0] ?? null,
    multimodal_totals: {
      image_count: safeSummary.total_image_count,
      audio_seconds: safeSummary.total_audio_seconds,
      video_seconds: safeSummary.total_video_seconds,
      music_seconds: safeSummary.total_music_seconds,
    },
  };
}

function routingStrategyLabel(strategy?: PortalRoutingStrategy | string | null): string {
  switch (strategy) {
    case 'deterministic_priority':
      return translatePortalText('Predictable order');
    case 'weighted_random':
      return translatePortalText('Traffic distribution');
    case 'slo_aware':
      return translatePortalText('Reliability guardrails');
    case 'geo_affinity':
      return translatePortalText('Regional preference');
    default:
      return translatePortalText('Adaptive routing');
  }
}

function titleCaseToken(value: string): string {
  return value
    .split(/[\s._:-]+/)
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ');
}

function resolveKnownProviderLabel(providerId: string): string | null {
  const normalized = providerId.trim().toLowerCase();

  if (!normalized) {
    return null;
  }

  if (normalized.includes('openai')) {
    return translatePortalText('OpenAI');
  }

  if (normalized.includes('anthropic')) {
    return translatePortalText('Anthropic');
  }

  if (normalized.includes('gemini')) {
    return translatePortalText('Gemini');
  }

  return null;
}

function resolveProviderDisplayLabel(
  providerId: string | null | undefined,
  providerOptions: PortalRoutingProviderOption[] = [],
): string {
  const trimmed = providerId?.trim();

  if (!trimmed) {
    return translatePortalText('Unknown provider');
  }

  const exactOption = providerOptions.find(
    (option) => option.provider_id.trim().toLowerCase() === trimmed.toLowerCase(),
  );
  const optionLabel = exactOption?.display_name?.trim();

  if (optionLabel) {
    return optionLabel;
  }

  return resolveKnownProviderLabel(trimmed) ?? titleCaseToken(trimmed);
}

function buildBalanceSummary(snapshot: PortalDashboardSummary): DashboardBalanceSummary {
  const quotaLimitUnits = snapshot.billing_summary.quota_limit_units ?? null;
  const usedUnits = snapshot.billing_summary.used_units;

  return {
    remaining_units: snapshot.billing_summary.remaining_units ?? null,
    quota_limit_units: quotaLimitUnits,
    used_units: usedUnits,
    utilization_ratio:
      quotaLimitUnits && quotaLimitUnits > 0
        ? Math.min(1, Math.max(0, usedUnits / quotaLimitUnits))
        : null,
  };
}

function remainingUnitsLabel(snapshot: PortalDashboardSummary): string {
  if (snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined) {
    return translatePortalText('Unlimited');
  }

  return formatUnits(snapshot.billing_summary.remaining_units);
}

function insight(
  id: string,
  title: string,
  detail: string,
  status_label: string,
  status_variant: DashboardStatusVariant,
  route?: PortalRouteKey,
  action_label?: string,
): DashboardInsight {
  return {
    id,
    title,
    detail,
    status_label,
    status_variant,
    route,
    action_label,
  };
}

function buildInsights(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
): DashboardInsight[] {
  const items: DashboardInsight[] = [];

  if (snapshot.billing_summary.exhausted) {
    items.push(
      insight(
        'quota-exhausted',
        translatePortalText('Quota exhausted'),
        translatePortalText('Recharge or move to a higher plan before production traffic resumes.'),
        translatePortalText('Action'),
        'danger',
        'billing',
        translatePortalText('Review billing'),
      ),
    );
  } else if ((snapshot.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000) {
    items.push(
      insight(
        'quota-low',
        translatePortalText('Runway is getting tight'),
        translatePortalText('Only {units} token units remain in the visible quota buffer.', {
          units: remainingUnitsLabel(snapshot),
        }),
        translatePortalText('Watch'),
        'warning',
        'credits',
        translatePortalText('Review credits'),
      ),
    );
  }

  if (snapshot.api_key_count === 0) {
    items.push(
      insight(
        'missing-key',
        translatePortalText('API key setup incomplete'),
        translatePortalText('Create at least one project key before inviting clients or teammates.'),
        translatePortalText('Setup'),
        'warning',
        'api-keys',
        translatePortalText('Create key'),
      ),
    );
  }

  if (snapshot.usage_summary.total_requests === 0) {
    items.push(
      insight(
        'missing-traffic',
        translatePortalText('Traffic overview is still empty'),
        translatePortalText('Send the first request to unlock live model, provider, and spend telemetry.'),
        translatePortalText('Advisory'),
        'secondary',
        'usage',
        translatePortalText('Open usage'),
      ),
    );
  }

  if (routingSummary?.preview.slo_degraded) {
    items.push(
      insight(
        'routing-degraded',
        translatePortalText('Routing is protecting availability'),
        translatePortalText(
          'The current preview degraded from the preferred path. Review provider health and fallback behavior.',
        ),
        translatePortalText('Review'),
        'warning',
        'routing',
        translatePortalText('Open routing'),
      ),
    );
  }

  if (!items.length) {
    items.push(
      insight(
        'healthy-workspace',
        translatePortalText('Workspace is ready'),
        translatePortalText('Traffic, access, routing, and quota posture are aligned for steady API usage.'),
        translatePortalText('Ready'),
        'success',
        'usage',
        translatePortalText('Open usage'),
      ),
    );
  }

  return items.slice(0, 3);
}

function buildMetrics(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
): DashboardMetric[] {
  return [
    {
      id: 'requests',
      label: translatePortalText('Requests'),
      value: formatUnits(snapshot.usage_summary.total_requests),
      detail: translatePortalText('Completed gateway calls recorded in the current workspace.'),
    },
    {
      id: 'booked-amount',
      label: translatePortalText('Booked spend'),
      value: formatCurrency(snapshot.billing_summary.booked_amount),
      detail: translatePortalText('Total booked amount attached to the visible billing summary.'),
    },
    {
      id: 'remaining-units',
      label: translatePortalText('Remaining units'),
      value: remainingUnitsLabel(snapshot),
      detail: translatePortalText('Token-unit runway remaining before the visible quota ceiling is reached.'),
    },
    {
      id: 'keys',
      label: translatePortalText('API keys'),
      value: formatUnits(snapshot.api_key_count),
      detail: translatePortalText('Active key inventory visible inside this portal session.'),
    },
    {
      id: 'providers',
      label: translatePortalText('Providers'),
      value: formatUnits(snapshot.usage_summary.provider_count),
      detail: translatePortalText('Providers that served recent visible traffic.'),
    },
    {
      id: 'route',
      label: translatePortalText('Default route'),
      value: routingSummary
        ? resolveProviderDisplayLabel(
          routingSummary.preview.selected_provider_id,
          routingSummary.provider_options,
        )
        : translatePortalText('Pending'),
      detail: translatePortalText('Provider currently selected by the routing preview.'),
    },
  ];
}

function buildQuickActions(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
): DashboardInsight[] {
  const actions: DashboardInsight[] = [];

  if (snapshot.api_key_count === 0) {
    actions.push(
      insight(
        'action-create-key',
        translatePortalText('Create the first API key'),
        translatePortalText('Set up a scoped key so external clients can start sending traffic safely.'),
        translatePortalText('Setup'),
        'warning',
        'api-keys',
        translatePortalText('Create key'),
      ),
    );
  }

  if (snapshot.usage_summary.total_requests === 0) {
    actions.push(
      insight(
        'action-start-traffic',
        translatePortalText('Send the first API request'),
        translatePortalText(
          'The first real call will populate demand, cost, and provider telemetry across the portal.',
        ),
        translatePortalText('Advisory'),
        'secondary',
        'usage',
        translatePortalText('Open usage'),
      ),
    );
  }

  if (snapshot.billing_summary.exhausted) {
    actions.push(
      insight(
        'action-recover-billing',
        translatePortalText('Restore quota before the next traffic window'),
        translatePortalText(
          'Billing recovery is the blocking action before more gateway requests can land.',
        ),
        translatePortalText('Action'),
        'danger',
        'billing',
        translatePortalText('Review billing'),
      ),
    );
  } else if ((snapshot.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000) {
    actions.push(
      insight(
        'action-protect-runway',
        translatePortalText('Top up credits before runway gets tight'),
        translatePortalText('Only {units} token units remain in the visible launch buffer.', {
          units: remainingUnitsLabel(snapshot),
        }),
        translatePortalText('Watch'),
        'warning',
        'credits',
        translatePortalText('Review credits'),
      ),
    );
  }

  if (routingSummary) {
    actions.push(
      insight(
        'action-review-routing',
        translatePortalText('Review the active route'),
        translatePortalText(
          'Confirm provider order, fallback posture, and region preference before scaling traffic.',
        ),
        routingSummary.preview.slo_degraded
          ? translatePortalText('Review')
          : translatePortalText('Ready'),
        routingSummary.preview.slo_degraded ? 'warning' : 'default',
        'routing',
        translatePortalText('Open routing'),
      ),
    );
  }

  if (!actions.length) {
    actions.push(
      insight(
        'action-scale',
        translatePortalText('Inspect live usage'),
        translatePortalText(
          'With the workspace in a healthy state, usage is the best place to validate growth before widening rollout.',
        ),
        translatePortalText('Ready'),
        'success',
        'usage',
        translatePortalText('Open usage'),
      ),
    );
  }

  return actions.slice(0, 4);
}

function buildRoutingPosture(
  routingSummary?: PortalRoutingSummary | null,
  routingLogs: PortalRoutingDecisionLog[] = [],
): DashboardRoutingPosture | null {
  if (!routingSummary) {
    return null;
  }

  const latestLog = [...routingLogs].sort((left, right) => right.created_at_ms - left.created_at_ms)[0];
  const strategyLabel = routingStrategyLabel(routingSummary.preferences.strategy);
  const preferredRegion = routingSummary.preferences.preferred_region
    ?? routingSummary.preview.requested_region
    ?? 'Global';

  if (routingSummary.preview.slo_degraded) {
    return {
      title: translatePortalText('Fallback protection is active'),
      detail: translatePortalText(
        'The current preview degraded from the preferred path. Review provider health and hard constraints.',
      ),
      strategy_label: strategyLabel,
      selected_provider: resolveProviderDisplayLabel(
        routingSummary.preview.selected_provider_id,
        routingSummary.provider_options,
      ),
      preferred_region: preferredRegion,
      evidence_count: formatUnits(routingLogs.length),
      status_label: translatePortalText('Review'),
      latest_reason:
        latestLog?.selection_reason
        ?? routingSummary.preview.selection_reason
        ?? translatePortalText('A fallback provider was selected to protect availability.'),
      status_variant: 'warning',
      route: 'routing',
      action_label: translatePortalText('Open routing'),
    };
  }

  if (!routingLogs.length) {
    return {
      title: translatePortalText('Routing is configured and waiting for traffic'),
      detail: translatePortalText(
        'The project has a default routing posture, but no recent evidence has been recorded yet.',
      ),
      strategy_label: strategyLabel,
      selected_provider: resolveProviderDisplayLabel(
        routingSummary.preview.selected_provider_id,
        routingSummary.provider_options,
      ),
      preferred_region: preferredRegion,
      evidence_count: '0',
      status_label: translatePortalText('Advisory'),
      latest_reason:
        routingSummary.preview.selection_reason
        ?? translatePortalText('Run a preview or send live traffic to capture the first route decision.'),
      status_variant: 'secondary',
      route: 'routing',
      action_label: translatePortalText('Run preview'),
    };
  }

  return {
    title: translatePortalText('Routing is healthy'),
    detail: translatePortalText('The latest routing evidence selected {provider}.', {
      provider: resolveProviderDisplayLabel(
        latestLog?.selected_provider_id ?? routingSummary.preview.selected_provider_id,
        routingSummary.provider_options,
      ),
    }),
    strategy_label: strategyLabel,
    selected_provider: resolveProviderDisplayLabel(
      routingSummary.preview.selected_provider_id,
      routingSummary.provider_options,
    ),
    preferred_region: preferredRegion,
    evidence_count: formatUnits(routingLogs.length),
    status_label: translatePortalText('Ready'),
    latest_reason:
      latestLog?.selection_reason
      ?? routingSummary.preview.selection_reason
      ?? translatePortalText('Routing evidence is available for review in the routing workbench.'),
    status_variant: 'success',
    route: 'routing',
    action_label: translatePortalText('Open routing'),
  };
}

function buildProviderMix(
  snapshot: PortalDashboardSummary,
  providerOptions: PortalRoutingProviderOption[] = [],
): DashboardBreakdownItem[] {
  const totalRequests = snapshot.usage_summary.total_requests || 1;

  return [...snapshot.usage_summary.providers]
    .sort((left, right) => right.request_count - left.request_count)
    .slice(0, 5)
    .map((provider) => ({
      id: provider.provider,
      label: resolveProviderDisplayLabel(provider.provider, providerOptions),
      secondary_label: translatePortalText('{count} project(s)', {
        count: formatUnits(provider.project_count),
      }),
      value_label: translatePortalText('{count} requests', {
        count: formatUnits(provider.request_count),
      }),
      share: Math.max(6, Math.round((provider.request_count / totalRequests) * 100)),
    }));
}

function buildModelMix(snapshot: PortalDashboardSummary): DashboardBreakdownItem[] {
  const totalRequests = snapshot.usage_summary.total_requests || 1;

  return [...snapshot.usage_summary.models]
    .sort((left, right) => right.request_count - left.request_count)
    .slice(0, 5)
    .map((model) => ({
      id: model.model,
      label: model.model,
      secondary_label: translatePortalText('{count} provider(s)', {
        count: formatUnits(model.provider_count),
      }),
      value_label: translatePortalText('{count} requests', {
        count: formatUnits(model.request_count),
      }),
      share: Math.max(6, Math.round((model.request_count / totalRequests) * 100)),
    }));
}

function seriesBucketLabel(timestamp: number): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
  }).format(new Date(timestamp));
}

function seriesBucketKey(timestamp: number): string {
  return new Intl.DateTimeFormat('en-CA', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(new Date(timestamp));
}

function buildTrafficTrendPoints(
  snapshot: PortalDashboardSummary,
  usageRecords: UsageRecord[],
): DashboardTrafficTrendPoint[] {
  const records = usageRecords.length ? usageRecords : snapshot.recent_requests;
  const grouped = new Map<string, DashboardTrafficTrendPoint>();

  for (const record of records) {
    const label = seriesBucketLabel(record.created_at_ms);
    const bucketKey = seriesBucketKey(record.created_at_ms);
    const current = grouped.get(bucketKey) ?? {
      label,
      bucket_key: bucketKey,
      request_count: 0,
      amount: 0,
      total_tokens: 0,
      input_tokens: 0,
      output_tokens: 0,
    };
    current.request_count += 1;
    current.amount += record.amount;
    current.total_tokens += record.total_tokens;
    current.input_tokens += record.input_tokens;
    current.output_tokens += record.output_tokens;
    grouped.set(bucketKey, current);
  }

  return [...grouped.values()]
    .sort((left, right) => left.bucket_key.localeCompare(right.bucket_key))
    .slice(-7);
}

function buildRequestVolumeSeries(
  snapshot: PortalDashboardSummary,
  usageRecords: UsageRecord[],
): DashboardSeriesPoint[] {
  return buildTrafficTrendPoints(snapshot, usageRecords).map((point) => ({
    bucket: point.label,
    requests: point.request_count,
    amount: Number(point.amount.toFixed(2)),
  }));
}

function buildSpendTrendPoints(
  snapshot: PortalDashboardSummary,
  usageRecords: UsageRecord[],
): DashboardSpendTrendPoint[] {
  return buildTrafficTrendPoints(snapshot, usageRecords).map((point) => ({
    label: point.label,
    bucket_key: point.bucket_key,
    amount: Number(point.amount.toFixed(2)),
    requests: point.request_count,
  }));
}

function buildSpendSeries(
  snapshot: PortalDashboardSummary,
  usageRecords: UsageRecord[],
): DashboardSeriesPoint[] {
  return buildSpendTrendPoints(snapshot, usageRecords).map((point) => ({
    bucket: point.label,
    requests: point.requests,
    amount: point.amount,
  }));
}

function buildProviderShareSeries(
  snapshot: PortalDashboardSummary,
  providerOptions: PortalRoutingProviderOption[] = [],
): DashboardDistributionPoint[] {
  return [...snapshot.usage_summary.providers]
    .sort((left, right) => right.request_count - left.request_count)
    .slice(0, 5)
    .map((provider) => ({
      name: resolveProviderDisplayLabel(provider.provider, providerOptions),
      value: provider.request_count,
    }));
}

function buildModelDemandSeries(snapshot: PortalDashboardSummary): DashboardDemandPoint[] {
  return [...snapshot.usage_summary.models]
    .sort((left, right) => right.request_count - left.request_count)
    .slice(0, 5)
    .map((model) => ({
      name: model.model,
      requests: model.request_count,
    }));
}

function buildActivityFeed(
  snapshot: PortalDashboardSummary,
  routingLogs: PortalRoutingDecisionLog[] = [],
  providerOptions: PortalRoutingProviderOption[] = [],
): DashboardActivityItem[] {
  const requestItems = snapshot.recent_requests.map((request) => ({
    id: `request-${request.project_id}-${request.created_at_ms}-${request.model}`,
    title: translatePortalText('{model} via {provider}', {
      model: request.model,
      provider: resolveProviderDisplayLabel(request.provider, providerOptions),
    }),
    detail: translatePortalText('{units} token units booked for {amount}.', {
      units: formatUnits(request.units),
      amount: formatCurrency(request.amount),
    }),
    timestamp_label: formatDateTime(request.created_at_ms),
    timestamp_ms: request.created_at_ms,
    status_label: translatePortalText('Tracked'),
    status_variant: 'default' as DashboardStatusVariant,
    route: 'usage' as PortalRouteKey,
    action_label: translatePortalText('Open usage'),
  }));

  const routingItems = routingLogs.map((log) => ({
    id: `routing-${log.decision_id}`,
    title: translatePortalText('Route selected {provider}', {
      provider: resolveProviderDisplayLabel(log.selected_provider_id, providerOptions),
    }),
    detail: log.selection_reason
      ? translatePortalText('{strategy}: {reason}', {
        strategy: routingStrategyLabel(log.strategy),
        reason: log.selection_reason,
      })
      : routingStrategyLabel(log.strategy),
    timestamp_label: formatDateTime(log.created_at_ms),
    timestamp_ms: log.created_at_ms,
    status_label: log.slo_degraded ? translatePortalText('Review') : translatePortalText('Healthy'),
    status_variant: log.slo_degraded
      ? ('warning' as DashboardStatusVariant)
      : ('success' as DashboardStatusVariant),
    route: 'routing' as PortalRouteKey,
    action_label: translatePortalText('Open routing'),
  }));

  return [...requestItems, ...routingItems]
    .sort((left, right) => right.timestamp_ms - left.timestamp_ms)
    .slice(0, 6)
    .map(({ timestamp_ms: _timestampMs, ...item }) => item);
}

function buildModules(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
): DashboardModuleItem[] {
  const creditsStatusVariant: DashboardStatusVariant = snapshot.billing_summary.exhausted
    ? 'danger'
    : (snapshot.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000
      ? 'warning'
      : 'success';

  return [
    {
      route: 'routing',
      title: translatePortalText('Routing'),
      status_label: routingSummary?.preview.slo_degraded ? translatePortalText('Review') : translatePortalText('Ready'),
      detail: routingSummary
        ? translatePortalText('{strategy} across {count} providers.', {
          strategy: routingStrategyLabel(routingSummary.preferences.strategy),
          count: formatUnits(routingSummary.provider_options.length),
        })
        : translatePortalText('Routing preview data is still loading.'),
      status_variant: routingSummary?.preview.slo_degraded ? 'warning' : 'success',
      action_label: translatePortalText('Open routing'),
    },
    {
      route: 'api-keys',
      title: translatePortalText('API Keys'),
      status_label: snapshot.api_key_count > 0 ? translatePortalText('Ready') : translatePortalText('Setup'),
      detail: snapshot.api_key_count > 0
        ? translatePortalText('{count} visible project keys.', { count: formatUnits(snapshot.api_key_count) })
        : translatePortalText('No project key is visible yet.'),
      status_variant: snapshot.api_key_count > 0 ? 'success' : 'warning',
      action_label: translatePortalText('Manage keys'),
    },
    {
      route: 'usage',
      title: translatePortalText('Usage'),
      status_label: snapshot.usage_summary.total_requests > 0 ? translatePortalText('Live') : translatePortalText('Quiet'),
      detail: snapshot.usage_summary.total_requests > 0
        ? translatePortalText('{requests} requests across {models} models.', {
          requests: formatUnits(snapshot.usage_summary.total_requests),
          models: formatUnits(snapshot.usage_summary.model_count),
        })
        : translatePortalText('The first request will unlock live telemetry.'),
      status_variant: snapshot.usage_summary.total_requests > 0 ? 'success' : 'secondary',
      action_label: translatePortalText('Open usage'),
    },
    {
      route: 'user',
      title: translatePortalText('User'),
      status_label: snapshot.workspace.user.active ? translatePortalText('Healthy') : translatePortalText('Review'),
      detail: translatePortalText('Personal profile, session identity, and security controls.'),
      status_variant: snapshot.workspace.user.active ? 'success' : 'warning',
      action_label: translatePortalText('Open user'),
    },
    {
      route: 'credits',
      title: translatePortalText('Redeem'),
      status_label: snapshot.billing_summary.exhausted
        ? translatePortalText('Exhausted')
        : creditsStatusVariant === 'warning'
          ? translatePortalText('Watch')
          : translatePortalText('Healthy'),
      detail: snapshot.billing_summary.exhausted
        ? translatePortalText('Quota is exhausted and requires immediate recovery.')
        : translatePortalText('{units} token units remain in the visible balance.', { units: remainingUnitsLabel(snapshot) }),
      status_variant: creditsStatusVariant,
      action_label: translatePortalText('Open redeem'),
    },
    {
      route: 'billing',
      title: translatePortalText('Billing'),
      status_label: snapshot.billing_summary.exhausted ? translatePortalText('Action') : translatePortalText('Ready'),
      detail: translatePortalText('Current booked amount is {amount}.', {
        amount: formatCurrency(snapshot.billing_summary.booked_amount),
      }),
      status_variant: snapshot.billing_summary.exhausted ? 'danger' : 'default',
      action_label: translatePortalText('Open billing'),
    },
    {
      route: 'account',
      title: translatePortalText('Account'),
      status_label: snapshot.billing_summary.booked_amount > 0 ? translatePortalText('Active') : translatePortalText('Ready'),
      detail: translatePortalText('Cash balance, ledger visibility, and payment-side posture.'),
      status_variant: snapshot.billing_summary.booked_amount > 0 ? 'default' : 'success',
      action_label: translatePortalText('Open account'),
    },
  ];
}

export function buildPortalDashboardViewModel(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
  routingLogs: PortalRoutingDecisionLog[] = [],
  usageRecords: UsageRecord[] = [],
  membership: PortalCommerceMembership | null = null,
  now: number = Date.now(),
  billingEventSummary: BillingEventSummary | null = null,
): PortalDashboardPageViewModel {
  const normalizedSnapshot = normalizeDashboardSummary(snapshot);
  const normalizedRoutingSummary = normalizeRoutingSummary(routingSummary);
  const normalizedRoutingLogs = normalizeRoutingLogs(routingLogs);
  const normalizedUsageRecords = safeArray(usageRecords);
  const records = normalizedUsageRecords.length
    ? normalizedUsageRecords
    : normalizedSnapshot.recent_requests;
  const providerOptions = normalizedRoutingSummary?.provider_options ?? [];
  const usageSummary = summarizeUsageRecords(records);
  const totalRevenue =
    normalizedSnapshot.billing_summary.booked_amount > 0
      ? normalizedSnapshot.billing_summary.booked_amount
      : usageSummary.revenue;
  const totalRequests =
    normalizedSnapshot.usage_summary.total_requests > 0
      ? normalizedSnapshot.usage_summary.total_requests
      : usageSummary.request_count;
  const totalUsedUnits =
    normalizedSnapshot.billing_summary.used_units > 0
      ? normalizedSnapshot.billing_summary.used_units
      : usageSummary.used_units;
  const todayStart = startOfDayMs(now);
  const trailing7dStart = startOfTrailing7dMs(now);
  const monthStart = startOfMonthMs(now);
  const todayRecords = records.filter((record) => record.created_at_ms >= todayStart);
  const trailing7dRecords = records.filter((record) => record.created_at_ms >= trailing7dStart);
  const currentMonthRecords = records.filter((record) => record.created_at_ms >= monthStart);
  const traffic_trend_points = buildTrafficTrendPoints(normalizedSnapshot, normalizedUsageRecords);
  const spend_trend_points = buildSpendTrendPoints(normalizedSnapshot, normalizedUsageRecords);

  return {
    snapshot: normalizedSnapshot,
    membership,
    commercial_highlights: buildDashboardCommercialHighlights(billingEventSummary),
    balance: buildBalanceSummary(normalizedSnapshot),
    totals: {
      revenue: totalRevenue,
      request_count: totalRequests,
      used_units: totalUsedUnits,
      average_booked_spend: totalRequests > 0 ? totalRevenue / totalRequests : 0,
    },
    today: summarizeUsageRecords(todayRecords),
    trailing_7d: summarizeUsageRecords(trailing7dRecords),
    current_month: summarizeUsageRecords(currentMonthRecords),
    insights: buildInsights(normalizedSnapshot, normalizedRoutingSummary),
    metrics: buildMetrics(normalizedSnapshot, normalizedRoutingSummary),
    routing_posture: buildRoutingPosture(normalizedRoutingSummary, normalizedRoutingLogs),
    quick_actions: buildQuickActions(normalizedSnapshot, normalizedRoutingSummary),
    provider_mix: buildProviderMix(normalizedSnapshot, providerOptions),
    model_mix: buildModelMix(normalizedSnapshot),
    request_volume_series: buildRequestVolumeSeries(normalizedSnapshot, normalizedUsageRecords),
    spend_series: buildSpendSeries(normalizedSnapshot, normalizedUsageRecords),
    traffic_trend_points,
    spend_trend_points,
    provider_share_series: buildProviderShareSeries(normalizedSnapshot, providerOptions),
    model_demand_series: buildModelDemandSeries(normalizedSnapshot),
    activity_feed: buildActivityFeed(normalizedSnapshot, normalizedRoutingLogs, providerOptions),
    modules: buildModules(normalizedSnapshot, normalizedRoutingSummary),
  };
}
