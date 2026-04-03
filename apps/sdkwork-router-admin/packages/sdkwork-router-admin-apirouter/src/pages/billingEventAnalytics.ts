import type {
  BillingAccountingMode,
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventGroupSummary,
  BillingEventProjectSummary,
  BillingEventRecord,
  BillingEventSummary,
} from 'sdkwork-router-admin-types';

export type BillingEventFilterOptions = {
  cutoffMs?: number | null;
  hashedKey?: string | null;
  projectId?: string | null;
  query?: string;
};

export type GatewayBillingRoutingEvidence = {
  events_with_profile: number;
  events_with_compiled_snapshot: number;
  events_with_fallback_reason: number;
};

export type GatewayBillingEventAnalyticsViewModel = {
  totals: Pick<
    BillingEventSummary,
    | 'total_events'
    | 'total_request_count'
    | 'total_tokens'
    | 'total_image_count'
    | 'total_audio_seconds'
    | 'total_video_seconds'
    | 'total_music_seconds'
    | 'total_upstream_cost'
    | 'total_customer_charge'
  >;
  top_capabilities: BillingEventCapabilitySummary[];
  group_chargeback: BillingEventGroupSummary[];
  accounting_mode_mix: BillingEventAccountingModeSummary[];
  recent_events: BillingEventRecord[];
  routing_evidence: GatewayBillingRoutingEvidence;
};

export type BillingEventCsvDocument = {
  headers: string[];
  rows: Array<Array<string | number>>;
};

function includesQuery(event: BillingEventRecord, query: string) {
  if (!query) {
    return true;
  }

  const haystack = [
    event.project_id,
    event.api_key_group_id ?? '',
    event.capability,
    event.route_key,
    event.usage_model,
    event.provider_id,
    event.accounting_mode,
    event.operation_kind,
    event.modality,
    event.reference_id ?? '',
    event.applied_routing_profile_id ?? '',
    event.fallback_reason ?? '',
  ]
    .join(' ')
    .toLowerCase();

  return haystack.includes(query);
}

export function filterBillingEvents(
  events: BillingEventRecord[],
  { cutoffMs, hashedKey, projectId, query }: BillingEventFilterOptions,
) {
  return events.filter((event) => {
    if (cutoffMs && event.created_at_ms < cutoffMs) {
      return false;
    }

    if (hashedKey && event.api_key_hash && event.api_key_hash !== hashedKey) {
      return false;
    }

    if (projectId && event.project_id !== projectId) {
      return false;
    }

    return includesQuery(event, query?.trim().toLowerCase() ?? '');
  });
}

export function emptyBillingEventSummary(): BillingEventSummary {
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

type GroupAccumulator = BillingEventGroupSummary & { projectIds: Set<string> };

function sortByChargeThenEvents<
  T extends {
    event_count: number;
    total_customer_charge: number;
    total_upstream_cost: number;
  },
>(items: T[]) {
  return items.sort(
    (left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.event_count - left.event_count
      || right.total_upstream_cost - left.total_upstream_cost,
  );
}

function sortCapabilityMix(
  items: BillingEventCapabilitySummary[],
): BillingEventCapabilitySummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort(
      (left, right) =>
        right.total_customer_charge - left.total_customer_charge
        || right.request_count - left.request_count
        || right.total_tokens - left.total_tokens
        || left.capability.localeCompare(right.capability),
    );
}

function sortGroupChargeback(
  items: BillingEventGroupSummary[],
): BillingEventGroupSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort(
      (left, right) =>
        right.total_customer_charge - left.total_customer_charge
        || right.request_count - left.request_count
        || (left.api_key_group_id ?? '').localeCompare(right.api_key_group_id ?? ''),
    );
}

function sortAccountingModeMix(
  items: BillingEventAccountingModeSummary[],
): BillingEventAccountingModeSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort(
      (left, right) =>
        right.total_customer_charge - left.total_customer_charge
        || right.request_count - left.request_count
        || left.accounting_mode.localeCompare(right.accounting_mode),
    );
}

function sortRecentEvents(events: BillingEventRecord[]): BillingEventRecord[] {
  return [...events].sort(
    (left, right) =>
      right.created_at_ms - left.created_at_ms
      || right.customer_charge - left.customer_charge
      || right.units - left.units
      || left.event_id.localeCompare(right.event_id),
  );
}

export function summarizeBillingEvents(events: BillingEventRecord[]): BillingEventSummary {
  if (!events.length) {
    return emptyBillingEventSummary();
  }

  const projects = new Map<string, BillingEventProjectSummary>();
  const groups = new Map<string, GroupAccumulator>();
  const capabilities = new Map<string, BillingEventCapabilitySummary>();
  const accountingModes = new Map<BillingAccountingMode, BillingEventAccountingModeSummary>();

  const summary = emptyBillingEventSummary();
  summary.total_events = events.length;

  for (const event of events) {
    summary.total_request_count += event.request_count;
    summary.total_units += event.units;
    summary.total_input_tokens += event.input_tokens;
    summary.total_output_tokens += event.output_tokens;
    summary.total_tokens += event.total_tokens;
    summary.total_image_count += event.image_count;
    summary.total_audio_seconds += event.audio_seconds;
    summary.total_video_seconds += event.video_seconds;
    summary.total_music_seconds += event.music_seconds;
    summary.total_upstream_cost += event.upstream_cost;
    summary.total_customer_charge += event.customer_charge;

    const project = projects.get(event.project_id) ?? {
      project_id: event.project_id,
      event_count: 0,
      request_count: 0,
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
    };
    project.event_count += 1;
    project.request_count += event.request_count;
    project.total_units += event.units;
    project.total_input_tokens += event.input_tokens;
    project.total_output_tokens += event.output_tokens;
    project.total_tokens += event.total_tokens;
    project.total_image_count += event.image_count;
    project.total_audio_seconds += event.audio_seconds;
    project.total_video_seconds += event.video_seconds;
    project.total_music_seconds += event.music_seconds;
    project.total_upstream_cost += event.upstream_cost;
    project.total_customer_charge += event.customer_charge;
    projects.set(event.project_id, project);

    const groupKey = event.api_key_group_id ?? '__ungrouped__';
    const group = groups.get(groupKey) ?? {
      api_key_group_id: event.api_key_group_id ?? null,
      project_count: 0,
      event_count: 0,
      request_count: 0,
      total_upstream_cost: 0,
      total_customer_charge: 0,
      projectIds: new Set<string>(),
    };
    group.event_count += 1;
    group.request_count += event.request_count;
    group.total_upstream_cost += event.upstream_cost;
    group.total_customer_charge += event.customer_charge;
    group.projectIds.add(event.project_id);
    groups.set(groupKey, group);

    const capability = capabilities.get(event.capability) ?? {
      capability: event.capability,
      event_count: 0,
      request_count: 0,
      total_tokens: 0,
      image_count: 0,
      audio_seconds: 0,
      video_seconds: 0,
      music_seconds: 0,
      total_upstream_cost: 0,
      total_customer_charge: 0,
    };
    capability.event_count += 1;
    capability.request_count += event.request_count;
    capability.total_tokens += event.total_tokens;
    capability.image_count += event.image_count;
    capability.audio_seconds += event.audio_seconds;
    capability.video_seconds += event.video_seconds;
    capability.music_seconds += event.music_seconds;
    capability.total_upstream_cost += event.upstream_cost;
    capability.total_customer_charge += event.customer_charge;
    capabilities.set(event.capability, capability);

    const accounting = accountingModes.get(event.accounting_mode) ?? {
      accounting_mode: event.accounting_mode,
      event_count: 0,
      request_count: 0,
      total_upstream_cost: 0,
      total_customer_charge: 0,
    };
    accounting.event_count += 1;
    accounting.request_count += event.request_count;
    accounting.total_upstream_cost += event.upstream_cost;
    accounting.total_customer_charge += event.customer_charge;
    accountingModes.set(event.accounting_mode, accounting);
  }

  summary.projects = Array.from(projects.values()).sort(
    (left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.event_count - left.event_count,
  );
  summary.groups = sortByChargeThenEvents(
    Array.from(groups.values()).map(({ projectIds, ...group }) => ({
      ...group,
      project_count: projectIds.size,
    })),
  );
  summary.capabilities = sortByChargeThenEvents(Array.from(capabilities.values()));
  summary.accounting_modes = sortByChargeThenEvents(Array.from(accountingModes.values()));
  summary.project_count = summary.projects.length;
  summary.group_count = summary.groups.length;
  summary.capability_count = summary.capabilities.length;

  return summary;
}

export function buildGatewayBillingEventAnalytics(
  summary: BillingEventSummary,
  events: BillingEventRecord[],
  limits: {
    capabilities?: number;
    groups?: number;
    accounting_modes?: number;
    recent_events?: number;
  } = {},
): GatewayBillingEventAnalyticsViewModel {
  const capabilityLimit = limits.capabilities ?? 6;
  const groupLimit = limits.groups ?? 6;
  const accountingModeLimit = limits.accounting_modes ?? 3;
  const recentEventLimit = limits.recent_events ?? 6;

  return {
    totals: {
      total_events: summary.total_events,
      total_request_count: summary.total_request_count,
      total_tokens: summary.total_tokens,
      total_image_count: summary.total_image_count,
      total_audio_seconds: summary.total_audio_seconds,
      total_video_seconds: summary.total_video_seconds,
      total_music_seconds: summary.total_music_seconds,
      total_upstream_cost: summary.total_upstream_cost,
      total_customer_charge: summary.total_customer_charge,
    },
    top_capabilities: sortCapabilityMix(summary.capabilities).slice(0, capabilityLimit),
    group_chargeback: sortGroupChargeback(summary.groups).slice(0, groupLimit),
    accounting_mode_mix: sortAccountingModeMix(summary.accounting_modes).slice(
      0,
      accountingModeLimit,
    ),
    recent_events: sortRecentEvents(events).slice(0, recentEventLimit),
    routing_evidence: {
      events_with_profile: events.filter((event) => event.applied_routing_profile_id).length,
      events_with_compiled_snapshot: events.filter(
        (event) => event.compiled_routing_snapshot_id,
      ).length,
      events_with_fallback_reason: events.filter((event) => event.fallback_reason).length,
    },
  };
}

export function buildBillingEventCsvDocument(
  events: BillingEventRecord[],
): BillingEventCsvDocument {
  return {
    headers: [
      'event_id',
      'tenant_id',
      'project_id',
      'api_key_group_id',
      'capability',
      'route_key',
      'usage_model',
      'provider_id',
      'accounting_mode',
      'operation_kind',
      'modality',
      'api_key_hash',
      'channel_id',
      'reference_id',
      'latency_ms',
      'units',
      'request_count',
      'input_tokens',
      'output_tokens',
      'total_tokens',
      'cache_read_tokens',
      'cache_write_tokens',
      'image_count',
      'audio_seconds',
      'video_seconds',
      'music_seconds',
      'upstream_cost',
      'customer_charge',
      'applied_routing_profile_id',
      'compiled_routing_snapshot_id',
      'fallback_reason',
      'created_at',
    ],
    rows: events.map((event) => [
      event.event_id,
      event.tenant_id,
      event.project_id,
      event.api_key_group_id ?? '',
      event.capability,
      event.route_key,
      event.usage_model,
      event.provider_id,
      event.accounting_mode,
      event.operation_kind,
      event.modality,
      event.api_key_hash ?? '',
      event.channel_id ?? '',
      event.reference_id ?? '',
      event.latency_ms ?? '',
      event.units,
      event.request_count,
      event.input_tokens,
      event.output_tokens,
      event.total_tokens,
      event.cache_read_tokens,
      event.cache_write_tokens,
      event.image_count,
      event.audio_seconds,
      event.video_seconds,
      event.music_seconds,
      event.upstream_cost.toFixed(4),
      event.customer_charge.toFixed(4),
      event.applied_routing_profile_id ?? '',
      event.compiled_routing_snapshot_id ?? '',
      event.fallback_reason ?? '',
      new Date(event.created_at_ms).toISOString(),
    ]),
  };
}
