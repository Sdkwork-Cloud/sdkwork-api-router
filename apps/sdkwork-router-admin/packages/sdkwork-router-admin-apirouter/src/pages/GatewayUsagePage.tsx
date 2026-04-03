import { startTransition, useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  DataTable,
  Input,
  Label,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Search } from 'lucide-react';
import {
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  AdminPageProps,
  BillingAccountingMode,
  BillingEventRecord,
  GatewayApiKeyRecord,
  UsageRecord,
} from 'sdkwork-router-admin-types';

import {
  buildBillingEventCsvDocument,
  buildGatewayBillingEventAnalytics,
  filterBillingEvents,
  summarizeBillingEvents,
} from './billingEventAnalytics';
import { SelectField } from './shared';
import { GatewayUsageDetailDrawer } from './usage/GatewayUsageDetailDrawer';
import { GatewayUsageRegistrySection } from './usage/GatewayUsageRegistrySection';
import {
  buildUsageRecordKey,
  compareUsageRecords,
  downloadCsv,
  formatCurrency,
  formatDateTime,
  formatNumber,
  recentWindowCutoff,
  PAGE_SIZE,
  type TimeRangePreset,
} from './usage/shared';

type GatewayUsagePageProps = AdminPageProps & {
  onRefreshWorkspace: () => Promise<void>;
};

function accountingModeLabel(mode: BillingAccountingMode): string {
  switch (mode) {
    case 'platform_credit':
      return 'Platform credit';
    case 'byok':
      return 'BYOK';
    case 'passthrough':
      return 'Passthrough';
    default:
      return mode;
  }
}

function billingEventSignalLabel(event: BillingEventRecord): string {
  const signals: string[] = [];

  if (event.total_tokens > 0) {
    signals.push(`${formatNumber(event.total_tokens)} tokens`);
  }
  if (event.image_count > 0) {
    signals.push(`${formatNumber(event.image_count)} images`);
  }
  if (event.audio_seconds > 0) {
    signals.push(`${formatNumber(event.audio_seconds)} audio sec`);
  }
  if (event.video_seconds > 0) {
    signals.push(`${formatNumber(event.video_seconds)} video sec`);
  }
  if (event.music_seconds > 0) {
    signals.push(`${formatNumber(event.music_seconds)} music sec`);
  }

  return signals.length
    ? signals.join(' / ')
    : `${formatNumber(event.request_count)} requests`;
}

function capabilitySignalLabel(
  capability: {
    request_count: number;
    total_tokens: number;
    image_count: number;
    audio_seconds: number;
    video_seconds: number;
    music_seconds: number;
  },
): string {
  const signals: string[] = [];

  if (capability.total_tokens > 0) {
    signals.push(`${formatNumber(capability.total_tokens)} tokens`);
  }
  if (capability.image_count > 0) {
    signals.push(`${formatNumber(capability.image_count)} images`);
  }
  if (capability.audio_seconds > 0) {
    signals.push(`${formatNumber(capability.audio_seconds)} audio sec`);
  }
  if (capability.video_seconds > 0) {
    signals.push(`${formatNumber(capability.video_seconds)} video sec`);
  }
  if (capability.music_seconds > 0) {
    signals.push(`${formatNumber(capability.music_seconds)} music sec`);
  }

  return signals.length
    ? signals.join(' / ')
    : `${formatNumber(capability.request_count)} requests`;
}

export function GatewayUsagePage({
  snapshot,
  onRefreshWorkspace,
}: GatewayUsagePageProps) {
  const { formatNumber: formatLocalizedNumber, t } = useAdminI18n();
  const [search, setSearch] = useState('');
  const [selectedKey, setSelectedKey] = useState('all');
  const [timeRange, setTimeRange] = useState<TimeRangePreset>('30d');
  const [page, setPage] = useState(1);
  const [selectedRecordId, setSelectedRecordId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const keyByHashed = useMemo(
    () => new Map(snapshot.apiKeys.map((key) => [key.hashed_key, key])),
    [snapshot.apiKeys],
  );
  const selectedKeyRecord: GatewayApiKeyRecord | null =
    selectedKey === 'all' ? null : keyByHashed.get(selectedKey) ?? null;
  const presetCutoff = recentWindowCutoff(timeRange);
  const filteredBillingEvents = useMemo(
    () =>
      filterBillingEvents(snapshot.billingEvents, {
        cutoffMs: presetCutoff,
        hashedKey: selectedKeyRecord?.hashed_key,
        projectId: selectedKeyRecord?.project_id,
        query: deferredSearch,
      }),
    [
      deferredSearch,
      presetCutoff,
      selectedKeyRecord?.hashed_key,
      selectedKeyRecord?.project_id,
      snapshot.billingEvents,
    ],
  );
  const billingEventSummary = useMemo(
    () => summarizeBillingEvents(filteredBillingEvents),
    [filteredBillingEvents],
  );
  const billingEventAnalytics = useMemo(
    () => buildGatewayBillingEventAnalytics(billingEventSummary, filteredBillingEvents),
    [billingEventSummary, filteredBillingEvents],
  );

  const filteredRecords = useMemo(
    () =>
      snapshot.usageRecords.filter((record) => {
        if (selectedKeyRecord && record.project_id !== selectedKeyRecord.project_id) {
          return false;
        }

        if (presetCutoff && record.created_at_ms < presetCutoff) {
          return false;
        }

        if (!deferredSearch) {
          return true;
        }

        const haystack = [record.project_id, record.model, record.provider]
          .join(' ')
          .toLowerCase();

        return haystack.includes(deferredSearch);
      }),
    [deferredSearch, presetCutoff, selectedKeyRecord, snapshot.usageRecords],
  );

  const sortedRecords = useMemo(
    () => [...filteredRecords].sort(compareUsageRecords),
    [filteredRecords],
  );
  const totalPages = Math.max(1, Math.ceil(sortedRecords.length / PAGE_SIZE));
  const safePage = Math.min(page, totalPages);
  const pagedRecords = sortedRecords.slice(
    (safePage - 1) * PAGE_SIZE,
    safePage * PAGE_SIZE,
  );

  useEffect(() => {
    if (!pagedRecords.length) {
      if (selectedRecordId !== null) {
        setSelectedRecordId(null);
      }
      setIsDetailDrawerOpen(false);
      return;
    }

    if (
      selectedRecordId
      && pagedRecords.some(
        (record, index) => buildUsageRecordKey(record, index) === selectedRecordId,
      )
    ) {
      return;
    }

    setSelectedRecordId(buildUsageRecordKey(pagedRecords[0], 0));
    setIsDetailDrawerOpen(false);
  }, [pagedRecords, selectedRecordId]);

  const selectedRecord =
    pagedRecords.find(
      (record, index) => buildUsageRecordKey(record, index) === selectedRecordId,
    )
    ?? pagedRecords[0]
    ?? null;

  const totalTokens = filteredRecords.reduce(
    (sum, record) => sum + record.total_tokens,
    0,
  );
  const totalUnits = filteredRecords.reduce((sum, record) => sum + record.units, 0);
  const totalAmount = filteredRecords.reduce(
    (sum, record) => sum + record.amount,
    0,
  );
  const uniqueProjects = new Set(
    filteredRecords.map((record) => record.project_id),
  ).size;
  const topGroupChargeback = billingEventAnalytics.group_chargeback.slice(0, 3);
  const topCapabilityMix = billingEventAnalytics.top_capabilities.slice(0, 3);
  const topAccountingModes = billingEventAnalytics.accounting_mode_mix.slice(0, 3);

  const columns = useMemo<DataTableColumn<UsageRecord>[]>(
    () => [
      {
        id: 'project',
        header: t('Project'),
        cell: (record) => (
          <div className="space-y-1">
            <div className="font-semibold text-[var(--sdk-color-text-primary)]">
              {record.project_id}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {record.provider}
            </div>
          </div>
        ),
      },
      {
        id: 'model',
        header: t('Model'),
        cell: (record) => (
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {record.model}
          </div>
        ),
      },
      {
        id: 'input',
        align: 'right',
        header: t('Input tokens'),
        cell: (record) => formatNumber(record.input_tokens),
      },
      {
        id: 'output',
        align: 'right',
        header: t('Output tokens'),
        cell: (record) => formatNumber(record.output_tokens),
      },
      {
        id: 'total',
        align: 'right',
        header: t('Total tokens'),
        cell: (record) => formatNumber(record.total_tokens),
      },
      {
        id: 'units',
        align: 'right',
        header: t('Units'),
        cell: (record) => formatNumber(record.units),
      },
      {
        id: 'amount',
        align: 'right',
        header: t('Amount'),
        cell: (record) => formatCurrency(record.amount),
      },
      {
        id: 'time',
        header: t('Created'),
        cell: (record) => (
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {formatDateTime(record.created_at_ms)}
          </div>
        ),
      },
    ],
    [t],
  );
  const billingEventColumns = useMemo<DataTableColumn<BillingEventRecord>[]>(
    () => [
      {
        id: 'capability',
        header: t('Capability'),
        cell: (event) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {event.capability}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {event.provider_id}
              {' | '}
              {event.usage_model}
            </div>
          </div>
        ),
      },
      {
        id: 'signal',
        header: t('Signal'),
        cell: (event) => (
          <div className="max-w-[16rem] text-sm text-[var(--sdk-color-text-secondary)]">
            {billingEventSignalLabel(event)}
          </div>
        ),
      },
      {
        id: 'accounting_mode',
        header: t('Accounting mode'),
        cell: (event) => (
          <div className="text-sm text-[var(--sdk-color-text-primary)]">
            {t(accountingModeLabel(event.accounting_mode))}
          </div>
        ),
      },
      {
        id: 'applied_routing_profile_id',
        header: t('Applied routing profile'),
        cell: (event) => (
          <div className="max-w-[12rem] truncate text-sm text-[var(--sdk-color-text-secondary)]">
            {event.applied_routing_profile_id ?? t('Not recorded')}
          </div>
        ),
      },
      {
        id: 'compiled_routing_snapshot_id',
        header: t('Compiled snapshot'),
        cell: (event) => (
          <div className="max-w-[12rem] truncate text-sm text-[var(--sdk-color-text-secondary)]">
            {event.compiled_routing_snapshot_id ?? t('Not recorded')}
          </div>
        ),
      },
      {
        id: 'fallback_reason',
        header: t('Fallback reason'),
        cell: (event) => (
          <div className="max-w-[14rem] truncate text-sm text-[var(--sdk-color-text-secondary)]">
            {event.fallback_reason ?? t('None')}
          </div>
        ),
      },
      {
        id: 'customer_charge',
        align: 'right',
        header: t('Customer charge'),
        cell: (event) => formatCurrency(event.customer_charge),
      },
      {
        id: 'created_at_ms',
        header: t('Created'),
        cell: (event) => formatDateTime(event.created_at_ms),
      },
    ],
    [t],
  );

  function exportCsv(): void {
    if (!sortedRecords.length) {
      return;
    }

    downloadCsv(
      'sdkwork-router-gateway-usage.csv',
      [
        'project_id',
        'selected_hashed_key',
        'provider',
        'model',
        'input_tokens',
        'output_tokens',
        'total_tokens',
        'units',
        'amount',
        'created_at',
      ],
      sortedRecords.map((record) => [
        record.project_id,
        selectedKeyRecord?.hashed_key ?? '',
        record.provider,
        record.model,
        record.input_tokens,
        record.output_tokens,
        record.total_tokens,
        record.units,
        record.amount.toFixed(4),
        new Date(record.created_at_ms).toISOString(),
      ]),
    );
  }

  function exportBillingEventsCsv(): void {
    if (!filteredBillingEvents.length) {
      return;
    }

    const document = buildBillingEventCsvDocument(filteredBillingEvents);
    downloadCsv(
      'sdkwork-router-billing-events.csv',
      document.headers,
      document.rows,
    );
  }

  function clearFilters(): void {
    startTransition(() => {
      setSearch('');
      setSelectedKey('all');
      setTimeRange('30d');
      setPage(1);
    });
  }

  function openDetailDrawer(record: UsageRecord, index: number): void {
    setSelectedRecordId(buildUsageRecordKey(record, index));
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean): void {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedRecordId(null);
    }
  }

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="gateway-usage-search">
                  {t('Search usage')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="gateway-usage-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) => {
                      setSearch(event.target.value);
                      setPage(1);
                    }}
                    placeholder={t('project, model, provider')}
                    value={search}
                  />
                </div>
              </div>
              <div className="min-w-[14rem]">
                <SelectField
                  label={t('API key')}
                  labelVisibility="sr-only"
                  onValueChange={(value) => {
                    setSelectedKey(value);
                    setPage(1);
                  }}
                  options={[
                    { label: t('All API keys'), value: 'all' },
                    ...snapshot.apiKeys.map((key) => ({
                      label: `${key.label || key.project_id} (${key.environment})`,
                      value: key.hashed_key,
                    })),
                  ]}
                  placeholder={t('API key')}
                  value={selectedKey}
                />
              </div>
              <div className="min-w-[12rem]">
                <SelectField<TimeRangePreset>
                  label={t('Time range')}
                  labelVisibility="sr-only"
                  onValueChange={(value) => {
                    setTimeRange(value);
                    setPage(1);
                  }}
                  options={[
                    { label: t('All time'), value: 'all' },
                    { label: t('Last 24 hours'), value: '24h' },
                    { label: t('Last 7 days'), value: '7d' },
                    { label: t('Last 30 days'), value: '30d' },
                  ]}
                  placeholder={t('Time range')}
                  value={timeRange}
                />
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatLocalizedNumber(sortedRecords.length) })}
                  {' | '}
                  {t('{count} projects', { count: formatLocalizedNumber(uniqueProjects) })}
                  {' | '}
                  {t('{count} tokens', { count: formatLocalizedNumber(totalTokens) })}
                </div>
                <Button
                  onClick={() => void onRefreshWorkspace()}
                  type="button"
                  variant="outline"
                >
                  {t('Refresh workspace')}
                </Button>
                <Button
                  disabled={!sortedRecords.length}
                  onClick={exportCsv}
                  type="button"
                  variant="primary"
                >
                  {t('Export usage CSV')}
                </Button>
                <Button
                  disabled={!filteredBillingEvents.length}
                  onClick={exportBillingEventsCsv}
                  type="button"
                  variant="outline"
                >
                  {t('Export billing events CSV')}
                </Button>
                <Button onClick={clearFilters} type="button" variant="ghost">
                  {t('Clear filters')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <div className="flex h-full min-h-0 flex-col gap-4">
            <div className="grid gap-4 xl:grid-cols-4">
            <Card>
              <CardContent className="space-y-3 p-4">
                <div className="space-y-1">
                  <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                    {t('Billing events')}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('Event-level chargeback stays aligned with the active usage filters.')}
                  </div>
                </div>
                <div className="space-y-1">
                  <div className="text-2xl font-semibold text-[var(--sdk-color-text-primary)]">
                    {formatLocalizedNumber(billingEventAnalytics.totals.total_events)}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('{count} requests', {
                      count: formatLocalizedNumber(
                        billingEventAnalytics.totals.total_request_count,
                      ),
                    })}
                    {' | '}
                    {t('{amount} charge', {
                      amount: formatCurrency(
                        billingEventAnalytics.totals.total_customer_charge,
                      ),
                    })}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('{amount} upstream', {
                      amount: formatCurrency(
                        billingEventAnalytics.totals.total_upstream_cost,
                      ),
                    })}
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardContent className="space-y-3 p-4">
                <div className="space-y-1">
                  <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                    {t('Group chargeback')}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('Top API key groups by visible customer charge.')}
                  </div>
                </div>
                <div className="space-y-2 text-sm">
                  {topGroupChargeback.length ? (
                    topGroupChargeback.map((group) => (
                      <div
                        className="flex items-center justify-between gap-3"
                        key={group.api_key_group_id ?? 'ungrouped'}
                      >
                        <div className="truncate text-[var(--sdk-color-text-primary)]">
                          {group.api_key_group_id ?? t('Ungrouped')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {formatCurrency(group.total_customer_charge)}
                        </div>
                      </div>
                    ))
                  ) : (
                    <div className="text-[var(--sdk-color-text-secondary)]">
                      {t('No billing events match the current filters.')}
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardContent className="space-y-3 p-4">
                <div className="space-y-1">
                  <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                    {t('Capability mix')}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('Charge distribution across routed multimodal capabilities.')}
                  </div>
                </div>
                <div className="space-y-2 text-sm">
                  {topCapabilityMix.length ? (
                    topCapabilityMix.map((capability) => (
                      <div
                        className="flex items-center justify-between gap-3"
                        key={capability.capability}
                      >
                        <div className="min-w-0">
                          <div className="truncate text-[var(--sdk-color-text-primary)]">
                            {capability.capability}
                          </div>
                          <div className="truncate text-xs text-[var(--sdk-color-text-secondary)]">
                            {capabilitySignalLabel(capability)}
                          </div>
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {formatCurrency(capability.total_customer_charge)}
                        </div>
                      </div>
                    ))
                  ) : (
                    <div className="text-[var(--sdk-color-text-secondary)]">
                      {t('No capability billing mix is available yet.')}
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardContent className="space-y-3 p-4">
                <div className="space-y-1">
                  <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                    {t('Accounting mode')}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('Compare platform credit, BYOK, and passthrough posture.')}
                  </div>
                </div>
                <div className="space-y-2 text-sm">
                  {topAccountingModes.length ? (
                    topAccountingModes.map((mode) => (
                      <div
                        className="flex items-center justify-between gap-3"
                        key={mode.accounting_mode}
                      >
                        <div className="truncate text-[var(--sdk-color-text-primary)]">
                          {t(accountingModeLabel(mode.accounting_mode))}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {formatCurrency(mode.total_customer_charge)}
                        </div>
                      </div>
                    ))
                  ) : (
                    <div className="text-[var(--sdk-color-text-secondary)]">
                      {t('No accounting-mode breakdown is available yet.')}
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
            </div>

            <div className="grid gap-4 xl:grid-cols-[0.95fr_1.05fr]">
              <Card>
                <CardContent className="space-y-4 p-4">
                  <div className="space-y-1">
                    <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                      {t('Multimodal signals')}
                    </div>
                    <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                      {t('Track token, image, audio, video, and music exposure from routed billing events.')}
                    </div>
                  </div>
                  <div className="grid gap-3 md:grid-cols-2">
                    <div className="rounded-2xl border border-[var(--sdk-color-border-default)] p-3">
                      <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                        {t('Tokens')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-[var(--sdk-color-text-primary)]">
                        {formatLocalizedNumber(billingEventAnalytics.totals.total_tokens)}
                      </div>
                    </div>
                    <div className="rounded-2xl border border-[var(--sdk-color-border-default)] p-3">
                      <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                        {t('Images')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-[var(--sdk-color-text-primary)]">
                        {formatLocalizedNumber(billingEventAnalytics.totals.total_image_count)}
                      </div>
                    </div>
                    <div className="rounded-2xl border border-[var(--sdk-color-border-default)] p-3">
                      <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                        {t('Audio sec')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-[var(--sdk-color-text-primary)]">
                        {formatLocalizedNumber(
                          billingEventAnalytics.totals.total_audio_seconds,
                        )}
                      </div>
                    </div>
                    <div className="rounded-2xl border border-[var(--sdk-color-border-default)] p-3">
                      <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                        {t('Video sec')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-[var(--sdk-color-text-primary)]">
                        {formatLocalizedNumber(
                          billingEventAnalytics.totals.total_video_seconds,
                        )}
                      </div>
                    </div>
                    <div className="rounded-2xl border border-[var(--sdk-color-border-default)] p-3 md:col-span-2">
                      <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                        {t('Music sec')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-[var(--sdk-color-text-primary)]">
                        {formatLocalizedNumber(
                          billingEventAnalytics.totals.total_music_seconds,
                        )}
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardContent className="space-y-4 p-4">
                  <div className="space-y-1">
                    <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                      {t('Routing evidence')}
                    </div>
                    <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                      {t('Operators can audit profile application, compiled snapshots, and fallback posture without leaving usage review.')}
                    </div>
                  </div>
                  <div className="grid gap-3 md:grid-cols-3">
                    <div className="rounded-2xl border border-[var(--sdk-color-border-default)] p-3">
                      <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                        {t('Applied routing profile')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-[var(--sdk-color-text-primary)]">
                        {formatLocalizedNumber(
                          billingEventAnalytics.routing_evidence.events_with_profile,
                        )}
                      </div>
                    </div>
                    <div className="rounded-2xl border border-[var(--sdk-color-border-default)] p-3">
                      <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                        {t('Compiled snapshot')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-[var(--sdk-color-text-primary)]">
                        {formatLocalizedNumber(
                          billingEventAnalytics.routing_evidence.events_with_compiled_snapshot,
                        )}
                      </div>
                    </div>
                    <div className="rounded-2xl border border-[var(--sdk-color-border-default)] p-3">
                      <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                        {t('Fallback reason')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-[var(--sdk-color-text-primary)]">
                        {formatLocalizedNumber(
                          billingEventAnalytics.routing_evidence.events_with_fallback_reason,
                        )}
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>

            <Card className="overflow-hidden p-0">
              <CardContent className="space-y-4 p-4">
                <div className="space-y-1">
                  <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                    {t('Recent billing events')}
                  </div>
                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {t('Recent billing events keep multimodal chargeback, provider cost, and routing evidence in one operator review table.')}
                  </div>
                </div>
              </CardContent>
              <DataTable
                className={embeddedAdminDataTableClassName}
                columns={billingEventColumns}
                emptyDescription={t(
                  'Recent billing events appear once routed requests create billable multimodal traffic.',
                )}
                emptyTitle={t('No recent billing events yet')}
                getRowId={(event) => event.event_id}
                rows={billingEventAnalytics.recent_events}
                slotProps={embeddedAdminDataTableSlotProps}
                stickyHeader
              />
            </Card>

            <div className="min-h-0 flex-1">
              <GatewayUsageRegistrySection
                columns={columns}
                onNextPage={() =>
                  setPage((current) => Math.min(totalPages, current + 1))
                }
                onPreviousPage={() =>
                  setPage((current) => Math.max(1, current - 1))
                }
                onSelectRecord={openDetailDrawer}
                page={safePage}
                pagedRecords={pagedRecords}
                rowSelectionId={selectedRecordId}
                totalAmount={totalAmount}
                totalPages={totalPages}
                totalTokens={totalTokens}
                totalUnits={totalUnits}
                totalVisibleRecords={sortedRecords.length}
                uniqueProjects={uniqueProjects}
              />
            </div>
          </div>
        </div>
      </div>

      <GatewayUsageDetailDrawer
        onOpenChange={handleDetailDrawerOpenChange}
        open={isDetailDrawerOpen}
        selectedKeyRecord={selectedKeyRecord}
        selectedRecord={selectedRecord}
        timeRange={timeRange}
      />
    </>
  );
}
