import { useDeferredValue, useEffect, useState } from 'react';
import type { ChangeEvent, ReactNode } from 'react';

import {
  Button,
  Card,
  CardContent,
  DataTable,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  InlineAlert,
  Input,
  Label,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  SegmentedControl,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Search } from 'lucide-react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  formatAdminCurrency,
  formatAdminDateTime,
  formatAdminNumber,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { AdminPageProps, BillingEventRecord, ManagedUser, RoutingDecisionLogRecord, UsageRecord } from 'sdkwork-router-admin-types';

type ViewMode = 'usage' | 'routing' | 'billing' | 'users' | 'projects';
type RecentWindow = 'all' | '24h' | '7d' | '30d';
type DetailItem = { label: string; mono?: boolean; value: ReactNode };
type BillingRow = AdminPageProps['snapshot']['billingEventSummary']['projects'][number] & {
  kind: 'billing';
  exhausted: boolean;
  quota_policy_id?: string | null;
  remaining_units?: number | null;
};
type UsageRow = UsageRecord & { kind: 'usage' };
type RoutingRow = RoutingDecisionLogRecord & { kind: 'routing' };
type UserTrafficRow = ManagedUser & {
  filtered_amount: number;
  filtered_request_count: number;
  filtered_total_tokens: number;
  filtered_usage_units: number;
  kind: 'users';
};
type ProjectHotspotRow = { kind: 'projects'; project_id: string; request_count: number; total_amount: number; total_tokens: number; total_units: number };
type TrafficRow = BillingRow | ProjectHotspotRow | RoutingRow | UsageRow | UserTrafficRow;
type TranslateFn = (text: string, values?: Record<string, number | string>) => string;

const pageSize = 10;
const viewModeOptions: Array<{ label: string; value: ViewMode }> = [
  { label: 'Usage', value: 'usage' },
  { label: 'Routing', value: 'routing' },
  { label: 'Billing', value: 'billing' },
  { label: 'Users', value: 'users' },
  { label: 'Projects', value: 'projects' },
];
const recentWindowOptions: Array<{ label: string; value: RecentWindow }> = [
  { label: 'All time', value: 'all' },
  { label: 'Last 24 hours', value: '24h' },
  { label: 'Last 7 days', value: '7d' },
  { label: 'Last 30 days', value: '30d' },
];

function recentWindowCutoff(window: RecentWindow) {
  const now = Date.now();
  if (window === '24h') return now - 24 * 60 * 60 * 1000;
  if (window === '7d') return now - 7 * 24 * 60 * 60 * 1000;
  if (window === '30d') return now - 30 * 24 * 60 * 60 * 1000;
  return null;
}

function formatCount(value: number) {
  return formatAdminNumber(value);
}

function formatCurrency(value: number, maximumFractionDigits = 4) {
  return formatAdminCurrency(value, maximumFractionDigits);
}

function formatDateTime(value: number) {
  return formatAdminDateTime(value);
}

function formatAccountingMode(value: string) {
  return value.replaceAll('_', ' ');
}

function filterVisibleBillingEvents(
  events: BillingEventRecord[],
  {
    cutoffMs,
    query,
  }: {
    cutoffMs?: number | null;
    query?: string;
  },
) {
  return events.filter((event) => {
    if (cutoffMs && event.created_at_ms < cutoffMs) {
      return false;
    }

    if (!query) {
      return true;
    }

    return [
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
      .toLowerCase()
      .includes(query);
  });
}

function summarizeVisibleBillingEvents(events: BillingEventRecord[]) {
  const empty = {
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
    projects: [] as AdminPageProps['snapshot']['billingEventSummary']['projects'],
    groups: [] as AdminPageProps['snapshot']['billingEventSummary']['groups'],
    capabilities: [] as AdminPageProps['snapshot']['billingEventSummary']['capabilities'],
    accounting_modes: [] as AdminPageProps['snapshot']['billingEventSummary']['accounting_modes'],
  };
  if (!events.length) {
    return empty;
  }

  const projects = new Map<string, AdminPageProps['snapshot']['billingEventSummary']['projects'][number]>();
  const groups = new Map<string, AdminPageProps['snapshot']['billingEventSummary']['groups'][number] & { project_ids: Set<string> }>();
  const capabilities = new Map<string, AdminPageProps['snapshot']['billingEventSummary']['capabilities'][number]>();
  const accountingModes = new Map<string, AdminPageProps['snapshot']['billingEventSummary']['accounting_modes'][number]>();

  for (const event of events) {
    empty.total_events += 1;
    empty.total_request_count += event.request_count;
    empty.total_units += event.units;
    empty.total_input_tokens += event.input_tokens;
    empty.total_output_tokens += event.output_tokens;
    empty.total_tokens += event.total_tokens;
    empty.total_image_count += event.image_count;
    empty.total_audio_seconds += event.audio_seconds;
    empty.total_video_seconds += event.video_seconds;
    empty.total_music_seconds += event.music_seconds;
    empty.total_upstream_cost += event.upstream_cost;
    empty.total_customer_charge += event.customer_charge;

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
      project_ids: new Set<string>(),
    };
    group.event_count += 1;
    group.request_count += event.request_count;
    group.total_upstream_cost += event.upstream_cost;
    group.total_customer_charge += event.customer_charge;
    group.project_ids.add(event.project_id);
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

  empty.projects = Array.from(projects.values()).sort((left, right) => right.total_customer_charge - left.total_customer_charge || right.event_count - left.event_count);
  empty.groups = Array.from(groups.values())
    .map(({ project_ids, ...group }) => ({ ...group, project_count: project_ids.size }))
    .sort((left, right) => right.total_customer_charge - left.total_customer_charge || right.event_count - left.event_count);
  empty.capabilities = Array.from(capabilities.values()).sort((left, right) => right.total_customer_charge - left.total_customer_charge || right.event_count - left.event_count);
  empty.accounting_modes = Array.from(accountingModes.values()).sort((left, right) => right.total_customer_charge - left.total_customer_charge || right.event_count - left.event_count);
  empty.project_count = empty.projects.length;
  empty.group_count = empty.groups.length;
  empty.capability_count = empty.capabilities.length;

  return empty;
}

function csvValue(value: string | number | boolean | null | undefined) {
  const normalized = value == null ? '' : String(value);
  return `"${normalized.replaceAll('"', '""')}"`;
}

function downloadCsv(filename: string, headers: string[], rows: Array<Array<string | number | boolean | null | undefined>>) {
  const contents = [headers.map(csvValue).join(','), ...rows.map((row) => row.map(csvValue).join(','))].join('\n');
  const blob = new Blob([contents], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

function rowId(row: TrafficRow) {
  if (row.kind === 'usage') return ['usage', row.project_id, row.model, row.provider, row.created_at_ms].join(':');
  if (row.kind === 'routing') return `routing:${row.decision_id}`;
  if (row.kind === 'billing') return `billing:${row.project_id}`;
  if (row.kind === 'projects') return `project:${row.project_id}`;
  return `user:${row.id}`;
}

function userMatchesQuery(user: ManagedUser, query: string) {
  return [user.display_name, user.email, user.workspace_tenant_id ?? '', user.workspace_project_id ?? ''].join(' ').toLowerCase().includes(query);
}

function SelectField<T extends string>({
  label,
  labelVisibility = 'visible',
  onValueChange,
  options,
  value,
}: {
  label: string;
  labelVisibility?: 'visible' | 'sr-only';
  onValueChange: (value: T) => void;
  options: Array<{ label: string; value: T }>;
  value: T;
}) {
  const hiddenLabel = labelVisibility === 'sr-only';

  return (
    <div className={`min-w-[12rem] ${hiddenLabel ? 'space-y-0' : 'space-y-2'}`}>
      <Label className={hiddenLabel ? 'sr-only' : undefined}>{label}</Label>
      <Select onValueChange={(nextValue: string) => onValueChange(nextValue as T)} value={value}>
        <SelectTrigger><SelectValue placeholder={label} /></SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>{option.label}</SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

function DetailGrid({ items }: { items: DetailItem[] }) {
  return (
    <DescriptionList columns={1}>
      {items.map((item) => (
        <DescriptionItem key={item.label}>
          <DescriptionTerm>{item.label}</DescriptionTerm>
          <DescriptionDetails mono={item.mono}>{item.value}</DescriptionDetails>
        </DescriptionItem>
      ))}
    </DescriptionList>
  );
}

function SpotlightList({
  emptyLabel,
  items,
}: {
  emptyLabel: string;
  items: Array<{ key: string; label: string; value: string }>;
}) {
  if (!items.length) {
    return <div className="text-sm text-[var(--sdk-color-text-secondary)]">{emptyLabel}</div>;
  }

  return (
    <div className="space-y-2 text-sm">
      {items.map((item) => (
        <div className="flex items-center justify-between gap-3" key={item.key}>
          <div className="truncate text-[var(--sdk-color-text-primary)]">{item.label}</div>
          <div className="text-[var(--sdk-color-text-secondary)]">{item.value}</div>
        </div>
      ))}
    </div>
  );
}

function buildDetail(row: TrafficRow, t: TranslateFn) {
  if (row.kind === 'usage') {
    return {
      title: t('{model} request', { model: row.model }),
      description: `${row.project_id} / ${row.provider}`,
      alertTitle: t('Usage request'),
      alertDescription: t('Request-level metering for the selected project interaction.'),
      alertTone: 'info' as const,
      items: [
        { label: t('Project'), mono: true, value: row.project_id },
        { label: t('Model'), value: row.model },
        { label: t('Provider'), value: row.provider },
        { label: t('Input tokens'), value: formatCount(row.input_tokens) },
        { label: t('Output tokens'), value: formatCount(row.output_tokens) },
        { label: t('Total tokens'), value: formatCount(row.total_tokens) },
        { label: t('Usage units'), value: formatCount(row.units) },
        { label: t('Amount'), value: formatCurrency(row.amount, 4) },
        { label: t('Created'), value: formatDateTime(row.created_at_ms) },
      ],
    };
  }

  if (row.kind === 'routing') {
    return {
      title: row.route_key,
      description: `${row.selected_provider_id} / ${row.capability}`,
      alertTitle: row.slo_degraded ? t('SLO degraded') : t('SLO stable'),
      alertDescription: t('Provider selection, route key, and strategy stay visible for routing audits.'),
      alertTone: row.slo_degraded ? 'warning' as const : 'success' as const,
      items: [
        { label: t('Decision id'), mono: true, value: row.decision_id },
        { label: t('Selected provider'), value: row.selected_provider_id },
        { label: t('Capability'), value: row.capability },
        { label: t('Route key'), mono: true, value: row.route_key },
        { label: t('Strategy'), value: row.strategy ?? t('Default strategy') },
        { label: t('Reason'), value: row.selection_reason ?? t('Not recorded') },
        { label: t('Fallback reason'), value: row.fallback_reason ?? t('No fallback used') },
        { label: t('Compiled snapshot'), mono: true, value: row.compiled_routing_snapshot_id ?? t('Not captured') },
        { label: t('Region'), value: row.requested_region ?? t('Global') },
        { label: t('Created'), value: formatDateTime(row.created_at_ms) },
      ],
    };
  }

  if (row.kind === 'billing') {
    return {
      title: row.project_id,
      description: row.project_id,
      alertTitle: row.exhausted ? t('Quota exhausted') : t('Quota healthy'),
      alertDescription: t('Billing events stay aligned with quota posture and remaining project headroom.'),
      alertTone: row.exhausted ? 'warning' as const : 'success' as const,
      items: [
        { label: t('Project'), mono: true, value: row.project_id },
        { label: t('Billing events'), value: formatCount(row.event_count) },
        { label: t('Requests'), value: formatCount(row.request_count) },
        { label: t('Used units'), value: formatCount(row.total_units) },
        { label: t('Customer charge'), value: formatCurrency(row.total_customer_charge, 4) },
        { label: t('Upstream cost'), value: formatCurrency(row.total_upstream_cost, 4) },
        { label: t('Quota policy'), mono: true, value: row.quota_policy_id ?? t('Not assigned') },
        { label: t('Remaining units'), value: row.remaining_units ?? t('Unlimited') },
      ],
    };
  }

  if (row.kind === 'users') {
    return {
      title: row.display_name,
      description: row.email,
      alertTitle: t('Portal user traffic'),
      alertDescription: t('Usage attribution stays attached to the mapped workspace and identity.'),
      alertTone: 'info' as const,
      items: [
        { label: t('User'), value: row.display_name },
        { label: t('Email'), value: row.email },
        { label: t('Workspace project'), mono: true, value: row.workspace_project_id ?? t('Unassigned') },
        { label: t('Requests'), value: formatCount(row.filtered_request_count) },
        { label: t('Tokens'), value: formatCount(row.filtered_total_tokens) },
        { label: t('Usage units'), value: formatCount(row.filtered_usage_units) },
        { label: t('Amount'), value: formatCurrency(row.filtered_amount, 4) },
      ],
    };
  }

  return {
    title: row.project_id,
    description: row.project_id,
    alertTitle: t('Project hotspot'),
    alertDescription: t('Hotspot projects combine request volume, token load, and amount in one ranked view.'),
    alertTone: 'info' as const,
    items: [
      { label: t('Project'), mono: true, value: row.project_id },
      { label: t('Requests'), value: formatCount(row.request_count) },
      { label: t('Tokens'), value: formatCount(row.total_tokens) },
      { label: t('Usage units'), value: formatCount(row.total_units) },
      { label: t('Amount'), value: formatCurrency(row.total_amount, 4) },
    ],
  };
}

// page

export function TrafficPage({ snapshot }: AdminPageProps) {
  const { t } = useAdminI18n();
  const localizedViewModeOptions = viewModeOptions.map((option) => ({ ...option, label: t(option.label) }));
  const localizedRecentWindowOptions = recentWindowOptions.map((option) => ({ ...option, label: t(option.label) }));
  const [search, setSearch] = useState('');
  const [viewMode, setViewMode] = useState<ViewMode>('usage');
  const [recentWindow, setRecentWindow] = useState<RecentWindow>('all');
  const [page, setPage] = useState(1);
  const [selectedRowId, setSelectedRowId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());
  const recentCutoff = recentWindowCutoff(recentWindow);

  const usageRows: UsageRow[] = snapshot.usageRecords
    .filter((record) => (!recentCutoff || record.created_at_ms >= recentCutoff) && [record.project_id, record.model, record.provider].join(' ').toLowerCase().includes(deferredQuery))
    .map((record) => ({ ...record, kind: 'usage' as const }));
  const routingRows: RoutingRow[] = snapshot.routingLogs
    .filter((log) => (!recentCutoff || log.created_at_ms >= recentCutoff) && [log.selected_provider_id, log.capability, log.route_key, log.strategy ?? '', log.selection_reason ?? '', log.fallback_reason ?? '', log.compiled_routing_snapshot_id ?? '', log.requested_region ?? ''].join(' ').toLowerCase().includes(deferredQuery))
    .map((log) => ({ ...log, kind: 'routing' as const }));
  const usageByProject = usageRows.reduce((projects, row) => {
    projects.set(row.project_id, {
      request_count: (projects.get(row.project_id)?.request_count ?? 0) + 1,
      total_amount: (projects.get(row.project_id)?.total_amount ?? 0) + row.amount,
      total_tokens: (projects.get(row.project_id)?.total_tokens ?? 0) + row.total_tokens,
      total_units: (projects.get(row.project_id)?.total_units ?? 0) + row.units,
    });
    return projects;
  }, new Map<string, { request_count: number; total_amount: number; total_tokens: number; total_units: number }>());
  const filteredBillingEvents = filterVisibleBillingEvents(snapshot.billingEvents, { cutoffMs: recentCutoff, query: deferredQuery });
  const filteredBillingEventSummary = summarizeVisibleBillingEvents(filteredBillingEvents);
  const billingEventSummary = snapshot.billingEventSummary;
  const legacyBillingByProject = new Map(snapshot.billingSummary.projects.map((project) => [project.project_id, project]));
  const billingRows: BillingRow[] = filteredBillingEventSummary.projects.map((project) => {
    const legacy = legacyBillingByProject.get(project.project_id);
    return {
      ...project,
      kind: 'billing' as const,
      exhausted: legacy?.exhausted ?? false,
      quota_policy_id: legacy?.quota_policy_id ?? null,
      remaining_units: legacy?.remaining_units ?? null,
    };
  });
  const userRows: UserTrafficRow[] = snapshot.portalUsers
    .map((user) => ({
      ...user,
      filtered_amount: usageByProject.get(user.workspace_project_id ?? '')?.total_amount ?? 0,
      filtered_request_count: usageByProject.get(user.workspace_project_id ?? '')?.request_count ?? 0,
      filtered_total_tokens: usageByProject.get(user.workspace_project_id ?? '')?.total_tokens ?? 0,
      filtered_usage_units: usageByProject.get(user.workspace_project_id ?? '')?.total_units ?? 0,
      kind: 'users' as const,
    }))
    .filter((user) => !deferredQuery || userMatchesQuery(user, deferredQuery) || user.filtered_request_count > 0)
    .sort((left, right) => right.filtered_request_count - left.filtered_request_count || right.filtered_total_tokens - left.filtered_total_tokens || right.filtered_amount - left.filtered_amount);
  const projectRows: ProjectHotspotRow[] = Array.from(usageByProject.entries())
    .map(([project_id, totals]) => ({ kind: 'projects' as const, project_id, ...totals }))
    .sort((left, right) => right.request_count - left.request_count || right.total_tokens - left.total_tokens || right.total_amount - left.total_amount);
  const groupChargebackSpotlight = (filteredBillingEventSummary.groups.length ? filteredBillingEventSummary.groups : billingEventSummary.groups).slice(0, 3);
  const capabilityMixSpotlight = (filteredBillingEventSummary.capabilities.length ? filteredBillingEventSummary.capabilities : billingEventSummary.capabilities).slice(0, 3);
  const accountingModeSpotlight = (filteredBillingEventSummary.accounting_modes.length ? filteredBillingEventSummary.accounting_modes : billingEventSummary.accounting_modes).slice(0, 3);

  let rows: TrafficRow[] = usageRows;
  let columns: Array<DataTableColumn<TrafficRow>> = [
    {
      id: 'project',
      header: t('Project'),
      cell: (row) => row.kind === 'usage' ? (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">{row.project_id}</div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">{row.provider}</div>
        </div>
      ) : null,
    },
    {
      id: 'model',
      header: t('Model'),
      cell: (row) => row.kind === 'usage' ? (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">{row.model}</div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">{formatDateTime(row.created_at_ms)}</div>
        </div>
      ) : null,
    },
    { id: 'tokens', align: 'right', header: t('Tokens'), cell: (row) => row.kind === 'usage' ? formatCount(row.total_tokens) : null, width: 120 },
    { id: 'units', align: 'right', header: t('Units'), cell: (row) => row.kind === 'usage' ? formatCount(row.units) : null, width: 120 },
    { id: 'amount', align: 'right', header: t('Amount'), cell: (row) => row.kind === 'usage' ? formatCurrency(row.amount, 4) : null, width: 140 },
  ];
  let tableCopy = { description: t('Request-level cost and token accounting for every visible interaction.'), emptyDescription: t('Broaden the query or widen the time window to inspect more usage records.'), emptyTitle: t('No usage records match the current filters'), title: t('Usage ledger') };
  let metrics = [
    { label: t('Requests'), value: formatCount(usageRows.length) },
    { label: t('Tokens'), value: formatCount(usageRows.reduce((sum, row) => sum + row.total_tokens, 0)) },
    { label: t('Units'), value: formatCount(usageRows.reduce((sum, row) => sum + row.units, 0)) },
    { label: t('Amount'), value: formatCurrency(usageRows.reduce((sum, row) => sum + row.amount, 0), 4) },
  ];

  if (viewMode === 'routing') {
    rows = routingRows;
    columns = [
      {
        id: 'provider',
        header: t('Selected provider'),
        cell: (row) => row.kind === 'routing' ? (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">{row.selected_provider_id}</div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">{row.capability}</div>
          </div>
        ) : null,
      },
      {
        id: 'route',
        header: t('Route key'),
        cell: (row) => row.kind === 'routing' ? (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">{row.route_key}</div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">{row.strategy ?? t('Default strategy')}</div>
            {row.compiled_routing_snapshot_id ? (
              <div className="font-mono text-xs text-[var(--sdk-color-text-muted)]">
                {row.compiled_routing_snapshot_id}
              </div>
            ) : null}
          </div>
        ) : null,
      },
      {
        id: 'reason',
        header: t('Reason'),
        cell: (row) => row.kind === 'routing' ? (
          <div className="max-w-[20rem] space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{row.selection_reason ?? t('Not recorded')}</div>
            {row.fallback_reason ? (
              <div className="text-[var(--sdk-color-text-primary)]">{row.fallback_reason}</div>
            ) : null}
          </div>
        ) : null,
      },
      {
        id: 'slo',
        header: t('SLO posture'),
        cell: (row) => row.kind === 'routing' ? (
          <StatusBadge
            label={row.slo_degraded ? t('SLO degraded') : row.slo_applied ? t('SLO applied') : t('Neutral')}
            showIcon
            status={row.slo_degraded ? 'degraded' : row.slo_applied ? 'applied' : 'neutral'}
            variant={row.slo_degraded ? 'warning' : row.slo_applied ? 'success' : 'secondary'}
          />
        ) : null,
        width: 140,
      },
      { id: 'created', header: t('Created'), cell: (row) => row.kind === 'routing' ? formatDateTime(row.created_at_ms) : null, width: 180 },
    ];
    tableCopy = { description: t('Provider selection, fallback evidence, compiled snapshots, and SLO posture remain visible for every routing decision.'), emptyDescription: t('Broaden the query or widen the time window to inspect more routing decisions.'), emptyTitle: t('No routing decisions match the current filters'), title: t('Routing decision log') };
    metrics = [
      { label: t('Decisions'), value: formatCount(routingRows.length) },
      { label: t('SLO applied'), value: formatCount(routingRows.filter((row) => row.slo_applied).length) },
      { label: t('SLO degraded'), value: formatCount(routingRows.filter((row) => row.slo_degraded).length) },
      { label: t('Providers'), value: formatCount(new Set(routingRows.map((row) => row.selected_provider_id)).size) },
    ];
  } else if (viewMode === 'billing') {
    rows = billingRows;
    columns = [
      {
        id: 'project',
        header: t('Project'),
        cell: (row) => row.kind === 'billing' ? (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">{row.project_id}</div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">{row.quota_policy_id ?? t('No quota policy')}</div>
          </div>
        ) : null,
      },
      { id: 'events', align: 'right', header: t('Billing events'), cell: (row) => row.kind === 'billing' ? formatCount(row.event_count) : null, width: 140 },
      { id: 'requests', align: 'right', header: t('Requests'), cell: (row) => row.kind === 'billing' ? formatCount(row.request_count) : null, width: 120 },
      { id: 'charge', align: 'right', header: t('Customer charge'), cell: (row) => row.kind === 'billing' ? formatCurrency(row.total_customer_charge, 4) : null, width: 150 },
      {
        id: 'quota',
        header: t('Quota'),
        cell: (row) => row.kind === 'billing' ? (
          <StatusBadge
            label={row.exhausted ? t('Exhausted') : t('Healthy')}
            showIcon
            status={row.exhausted ? 'exhausted' : 'healthy'}
            variant={row.exhausted ? 'warning' : 'success'}
          />
        ) : null,
        width: 140,
      },
    ];
    tableCopy = { description: t('Billing events summarize project chargeback, request volume, and quota posture in one view.'), emptyDescription: t('Try a broader query to inspect more billing events.'), emptyTitle: t('No billing events match the current filters'), title: t('Billing summary') };
    metrics = [
      { label: t('Projects'), value: formatCount(billingRows.length) },
      { label: t('Billing events'), value: formatCount(filteredBillingEventSummary.total_events) },
      { label: t('Customer charge'), value: formatCurrency(filteredBillingEventSummary.total_customer_charge, 4) },
      { label: t('Upstream cost'), value: formatCurrency(filteredBillingEventSummary.total_upstream_cost, 4) },
    ];
  } else if (viewMode === 'users') {
    rows = userRows;
    columns = [
      {
        id: 'user',
        header: t('Portal user'),
        cell: (row) => row.kind === 'users' ? (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">{row.display_name}</div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">{row.email}</div>
          </div>
        ) : null,
      },
      {
        id: 'workspace',
        header: t('Workspace project'),
        cell: (row) => row.kind === 'users' ? (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{row.workspace_tenant_id ?? t('Unassigned tenant')}</div>
            <div>{row.workspace_project_id ?? t('Unassigned project')}</div>
          </div>
        ) : null,
      },
      { id: 'requests', align: 'right', header: t('Requests'), cell: (row) => row.kind === 'users' ? formatCount(row.filtered_request_count) : null, width: 120 },
      { id: 'tokens', align: 'right', header: t('Tokens'), cell: (row) => row.kind === 'users' ? formatCount(row.filtered_total_tokens) : null, width: 140 },
      {
        id: 'status',
        header: t('Status'),
        cell: (row) => row.kind === 'users' ? (
          <StatusBadge
            label={row.active ? t('Active') : t('Disabled')}
            showIcon
            status={row.active ? 'active' : 'disabled'}
            variant={row.active ? 'success' : 'secondary'}
          />
        ) : null,
        width: 140,
      },
    ];
    tableCopy = { description: t('Portal identities ranked by visible request volume, token load, and amount.'), emptyDescription: t('Try a broader query or wider time window to inspect more portal users.'), emptyTitle: t('No portal users match the current filters'), title: t('User traffic leaderboard') };
    metrics = [
      { label: t('Visible users'), value: formatCount(userRows.length) },
      { label: t('Active users'), value: formatCount(userRows.filter((row) => row.active).length) },
      { label: t('Requests'), value: formatCount(userRows.reduce((sum, row) => sum + row.filtered_request_count, 0)) },
      { label: t('Amount'), value: formatCurrency(userRows.reduce((sum, row) => sum + row.filtered_amount, 0), 4) },
    ];
  } else if (viewMode === 'projects') {
    rows = projectRows;
    columns = [
      {
        id: 'project',
        header: t('Project'),
        cell: (row) => row.kind === 'projects' ? (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">{row.project_id}</div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">{t('Hotspot candidate')}</div>
          </div>
        ) : null,
      },
      { id: 'requests', align: 'right', header: t('Requests'), cell: (row) => row.kind === 'projects' ? formatCount(row.request_count) : null, width: 120 },
      { id: 'tokens', align: 'right', header: t('Tokens'), cell: (row) => row.kind === 'projects' ? formatCount(row.total_tokens) : null, width: 140 },
      { id: 'units', align: 'right', header: t('Units'), cell: (row) => row.kind === 'projects' ? formatCount(row.total_units) : null, width: 120 },
      { id: 'amount', align: 'right', header: t('Amount'), cell: (row) => row.kind === 'projects' ? formatCurrency(row.total_amount, 4) : null, width: 140 },
    ];
    tableCopy = { description: t('Projects ranked by visible request volume, token load, and amount.'), emptyDescription: t('Try a broader query or wider time window to surface more hotspot projects.'), emptyTitle: t('No hotspot projects match the current filters'), title: t('Project hotspots') };
    metrics = [
      { label: t('Visible hotspots'), value: formatCount(projectRows.length) },
      { label: t('Requests'), value: formatCount(projectRows.reduce((sum, row) => sum + row.request_count, 0)) },
      { label: t('Tokens'), value: formatCount(projectRows.reduce((sum, row) => sum + row.total_tokens, 0)) },
      { label: t('Amount'), value: formatCurrency(projectRows.reduce((sum, row) => sum + row.total_amount, 0), 4) },
    ];
  }

  const totalPages = Math.max(1, Math.ceil(rows.length / pageSize));
  const safePage = Math.min(page, totalPages);
  const startIndex = (safePage - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const pagedRows = rows.slice(startIndex, endIndex);

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  useEffect(() => {
    if (!pagedRows.length) {
      if (selectedRowId !== null) {
        setSelectedRowId(null);
      }
      setIsDetailDrawerOpen(false);
      return;
    }

    if (selectedRowId && pagedRows.some((row) => rowId(row) === selectedRowId)) {
      return;
    }

    setSelectedRowId(rowId(pagedRows[0]));
    setIsDetailDrawerOpen(false);
  }, [pagedRows, selectedRowId]);

  const selectedRow = pagedRows.find((row) => rowId(row) === selectedRowId) ?? pagedRows[0] ?? null;
  const detail = selectedRow ? buildDetail(selectedRow, t) : null;
  const currentViewLabel = localizedViewModeOptions.find((option) => option.value === viewMode)?.label ?? t('Usage');

  function openDetailDrawer(row: TrafficRow) {
    setSelectedRowId(rowId(row));
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
  }

  function clearFilters() {
    setSearch('');
    setRecentWindow('all');
    setPage(1);
  }

  function exportCurrentCsv() {
    if (viewMode === 'routing') {
      return downloadCsv('sdkwork-router-routing-logs.csv', ['decision_id', 'selected_provider_id', 'capability', 'route_key', 'strategy', 'selection_reason', 'fallback_reason', 'compiled_routing_snapshot_id', 'requested_region', 'slo_applied', 'slo_degraded', 'created_at'], routingRows.map((row) => [row.decision_id, row.selected_provider_id, row.capability, row.route_key, row.strategy ?? '', row.selection_reason ?? '', row.fallback_reason ?? '', row.compiled_routing_snapshot_id ?? '', row.requested_region ?? '', row.slo_applied, row.slo_degraded, new Date(row.created_at_ms).toISOString()]));
    }
    if (viewMode === 'billing') {
      return downloadCsv('sdkwork-router-billing-summary.csv', ['project_id', 'event_count', 'request_count', 'total_units', 'total_customer_charge', 'total_upstream_cost', 'quota_policy_id', 'remaining_units', 'status'], billingRows.map((row) => [row.project_id, row.event_count, row.request_count, row.total_units, row.total_customer_charge.toFixed(4), row.total_upstream_cost.toFixed(4), row.quota_policy_id ?? '', row.remaining_units ?? '', row.exhausted ? 'exhausted' : 'healthy']));
    }
    if (viewMode === 'users') {
      return downloadCsv('sdkwork-router-user-traffic.csv', ['user_id', 'email', 'project_id', 'request_count', 'total_tokens', 'usage_units', 'amount', 'status'], userRows.map((row) => [row.id, row.email, row.workspace_project_id ?? '', row.filtered_request_count, row.filtered_total_tokens, row.filtered_usage_units, row.filtered_amount.toFixed(4), row.active ? 'active' : 'disabled']));
    }
    if (viewMode === 'projects') {
      return downloadCsv('sdkwork-router-project-hotspots.csv', ['project_id', 'request_count', 'total_tokens', 'total_units', 'total_amount'], projectRows.map((row) => [row.project_id, row.request_count, row.total_tokens, row.total_units, row.total_amount.toFixed(4)]));
    }
    return downloadCsv('sdkwork-router-usage-records.csv', ['project_id', 'model', 'provider', 'input_tokens', 'output_tokens', 'total_tokens', 'units', 'amount', 'created_at'], usageRows.map((row) => [row.project_id, row.model, row.provider, row.input_tokens, row.output_tokens, row.total_tokens, row.units, row.amount.toFixed(4), new Date(row.created_at_ms).toISOString()]));
  }

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <Card className="shrink-0">
          <CardContent className="p-4">
            <form className="flex flex-wrap items-center gap-3" onSubmit={(event) => event.preventDefault()}>
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="traffic-search">{t('Search traffic')}</Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="traffic-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) => {
                      setSearch(event.target.value);
                      setPage(1);
                    }}
                    placeholder={t('project, provider, model, route, user')}
                    value={search}
                  />
                </div>
              </div>
              <div className="min-w-[12rem]">
                <SelectField
                  label={t('Recent window')}
                  labelVisibility="sr-only"
                  onValueChange={(value: RecentWindow) => {
                    setRecentWindow(value);
                    setPage(1);
                  }}
                  options={localizedRecentWindowOptions}
                  value={recentWindow}
                />
              </div>
              <div className="min-w-[18rem] flex-[1.1]">
                <div className="space-y-0">
                  <Label className="sr-only">{t('Traffic view')}</Label>
                  <SegmentedControl
                    onValueChange={(value: string) => {
                      setViewMode(value as ViewMode);
                      setPage(1);
                      setSelectedRowId(null);
                      setIsDetailDrawerOpen(false);
                    }}
                    options={localizedViewModeOptions}
                    size="sm"
                    value={viewMode}
                  />
                </div>
              </div>
              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatCount(rows.length) })}
                  {' | '}
                  {currentViewLabel}
                  {' | '}
                  {localizedRecentWindowOptions.find((option) => option.value === recentWindow)?.label}
                </div>
                <Button onClick={clearFilters} type="button" variant="outline">{t('Reset filters')}</Button>
                <Button disabled={!rows.length} onClick={exportCurrentCsv} type="button" variant="primary">{t('Export CSV')}</Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="grid gap-4 xl:grid-cols-4">
          <Card>
            <CardContent className="space-y-3 p-4">
              <div className="space-y-1">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">{t('Billing events')}</div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">{t('Billing-event analytics stay visible across all traffic lenses.')}</div>
              </div>
              <div className="space-y-1">
                <div className="text-2xl font-semibold text-[var(--sdk-color-text-primary)]">{formatCount(filteredBillingEventSummary.total_events)}</div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">{t('{count} requests', { count: formatCount(filteredBillingEventSummary.total_request_count) })}</div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">{t('{amount} charge', { amount: formatCurrency(filteredBillingEventSummary.total_customer_charge, 4) })}</div>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="space-y-3 p-4">
              <div className="space-y-1">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">{t('Group chargeback')}</div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">{t('Top API key groups by visible customer charge.')}</div>
              </div>
              <SpotlightList emptyLabel={t('No group chargeback data is visible for this slice.')} items={groupChargebackSpotlight.map((group) => ({ key: group.api_key_group_id ?? 'ungrouped', label: group.api_key_group_id ?? t('Ungrouped'), value: formatCurrency(group.total_customer_charge, 4) }))} />
            </CardContent>
          </Card>
          <Card>
            <CardContent className="space-y-3 p-4">
              <div className="space-y-1">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">{t('Capability mix')}</div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">{t('Top billed capabilities in the active time slice.')}</div>
              </div>
              <SpotlightList emptyLabel={t('No capability mix is visible for this slice.')} items={capabilityMixSpotlight.map((capability) => ({ key: capability.capability, label: capability.capability, value: formatCurrency(capability.total_customer_charge, 4) }))} />
            </CardContent>
          </Card>
          <Card>
            <CardContent className="space-y-3 p-4">
              <div className="space-y-1">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">{t('Accounting mode')}</div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">{t('Platform credit, BYOK, and passthrough mix remain visible.')}</div>
              </div>
              <SpotlightList emptyLabel={t('No accounting-mode mix is visible for this slice.')} items={accountingModeSpotlight.map((mode) => ({ key: mode.accounting_mode, label: formatAccountingMode(mode.accounting_mode), value: formatCurrency(mode.total_customer_charge, 4) }))} />
            </CardContent>
          </Card>
        </div>

        <Card className="min-h-0 flex-1 overflow-hidden p-0">
          <DataTable
            className={embeddedAdminDataTableClassName}
            columns={columns}
            emptyDescription={tableCopy.emptyDescription}
            emptyTitle={tableCopy.emptyTitle}
            getRowId={rowId}
            getRowProps={buildEmbeddedAdminSingleSelectRowProps(selectedRowId, rowId)}
            onRowClick={openDetailDrawer}
            rowActions={(row: TrafficRow) => (
              <Button
                onClick={(event) => {
                  event.stopPropagation();
                  openDetailDrawer(row);
                }}
                size="sm"
                type="button"
                variant="ghost"
              >
                {t('Inspect')}
              </Button>
            )}
            rows={pagedRows}
            slotProps={embeddedAdminDataTableSlotProps}
            stickyHeader
          />

          <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="min-w-0">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">{tableCopy.title}</div>
                <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">{tableCopy.description}</div>
              </div>
              <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">{t('Page {page} of {totalPages}', { page: safePage, totalPages })}</div>
            </div>

            <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
              {metrics.map((metric) => (
                <span key={metric.label}>
                  <span className="font-medium text-[var(--sdk-color-text-primary)]">{metric.value}</span>
                  {' '}
                  {metric.label}
                </span>
              ))}
            </div>

            {viewMode === 'billing' && billingRows.some((row) => row.exhausted) ? (
              <InlineAlert
                description={t('{count} project(s) currently sit at the quota ceiling and may require intervention.', { count: formatCount(billingRows.filter((row) => row.exhausted).length) })}
                title={t('Quota exhaustion detected')}
                tone="warning"
              />
            ) : null}

            {rows.length > 0 ? (
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Showing {start} - {end} of {total}', { start: startIndex + 1, end: Math.min(endIndex, rows.length), total: formatCount(rows.length) })}
                </div>
                <Pagination>
                  <PaginationContent>
                    <PaginationItem>
                      <PaginationPrevious className={safePage <= 1 ? 'pointer-events-none opacity-50' : 'cursor-pointer'} onClick={() => setPage((current) => Math.max(1, current - 1))} />
                    </PaginationItem>
                    {Array.from({ length: Math.min(5, totalPages) }, (_, index) => {
                      let pageNumber: number;
                      if (totalPages <= 5) {
                        pageNumber = index + 1;
                      } else if (safePage <= 3) {
                        pageNumber = index + 1;
                      } else if (safePage >= totalPages - 2) {
                        pageNumber = totalPages - 4 + index;
                      } else {
                        pageNumber = safePage - 2 + index;
                      }

                      return (
                        <PaginationItem key={pageNumber}>
                          <PaginationLink className="cursor-pointer" isActive={safePage === pageNumber} onClick={() => setPage(pageNumber)}>
                            {pageNumber}
                          </PaginationLink>
                        </PaginationItem>
                      );
                    })}
                    <PaginationItem>
                      <PaginationNext className={safePage >= totalPages ? 'pointer-events-none opacity-50' : 'cursor-pointer'} onClick={() => setPage((current) => Math.min(totalPages, current + 1))} />
                    </PaginationItem>
                  </PaginationContent>
                </Pagination>
              </div>
            ) : null}
          </div>
        </Card>
      </div>

      <Drawer open={isDetailDrawerOpen} onOpenChange={handleDetailDrawerOpenChange}>
        <DrawerContent side="right" size="xl">
          {detail ? (
            <>
              <DrawerHeader>
                <div className="space-y-3">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="space-y-1">
                      <DrawerTitle>{detail.title}</DrawerTitle>
                      <DrawerDescription>{detail.description}</DrawerDescription>
                    </div>
                    <StatusBadge label={currentViewLabel} showIcon status="active" variant="secondary" />
                  </div>
                </div>
              </DrawerHeader>

              <DrawerBody className="space-y-4">
                <InlineAlert description={detail.alertDescription} title={detail.alertTitle} tone={detail.alertTone} />
                <DetailGrid items={detail.items} />
              </DrawerBody>

              <DrawerFooter className="text-xs text-[var(--sdk-color-text-secondary)]">
                {t('Traffic inspection keeps the active lens and filter window attached to the selected row.')}
              </DrawerFooter>
            </>
          ) : null}
        </DrawerContent>
      </Drawer>
    </>
  );
}
