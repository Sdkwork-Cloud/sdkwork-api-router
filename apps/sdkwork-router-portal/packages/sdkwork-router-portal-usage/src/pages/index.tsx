import { RefreshCw } from 'lucide-react';
import { startTransition, useEffect, useMemo, useState } from 'react';

import {
  formatCurrency,
  formatDateTime,
  formatUnits,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  DataTable,
  StatCard,
} from 'sdkwork-router-portal-commons/framework/display';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from 'sdkwork-router-portal-commons/framework/entry';
import {
  FilterBar,
  FilterBarActions,
  FilterBarSection,
  FilterField,
} from 'sdkwork-router-portal-commons/framework/form';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { GatewayApiKeyRecord, UsageRecord } from 'sdkwork-router-portal-types';

import { loadUsageWorkbenchData } from '../repository';
import { buildPortalUsageViewModel } from '../services';
import type { PortalUsagePageProps, UsageFilters } from '../types';

const PAGE_SIZE = 12;
const DEFAULT_FILTERS: UsageFilters = {
  api_key_hash: 'all',
  channel_id: 'all',
  model: 'all',
  time_range: '30d',
};

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;

function clampPage(page: number, totalPages: number): number {
  return Math.min(Math.max(page, 1), Math.max(totalPages, 1));
}

function formatLatency(latencyMs: number | null, t: TranslateFn): string {
  if (latencyMs === null || latencyMs === undefined) {
    return t('Pending');
  }

  if (latencyMs >= 1000) {
    return `${(latencyMs / 1000).toFixed(latencyMs >= 10_000 ? 0 : 1)}s`;
  }

  return `${formatUnits(latencyMs)} ms`;
}

function buildUsageRecordKey(record: UsageRecord, index: number): string {
  return [
    record.project_id,
    record.api_key_hash ?? 'workspace',
    record.model,
    record.created_at_ms,
    index,
  ].join(':');
}

export function PortalUsagePage({ onNavigate: _onNavigate }: PortalUsagePageProps) {
  const { t } = usePortalI18n();
  const loadingStatus = t('Loading request telemetry...');
  const syncedStatus = t(
    'Usage cards and ledger-grade request facts reflect the current filtered workspace slice.',
  );
  const [apiKeys, setApiKeys] = useState<GatewayApiKeyRecord[]>([]);
  const [records, setRecords] = useState<UsageRecord[]>([]);
  const [filters, setFilters] = useState<UsageFilters>(DEFAULT_FILTERS);
  const [page, setPage] = useState(1);
  const [status, setStatus] = useState(loadingStatus);

  async function refreshUsageWorkbench(): Promise<void> {
    setStatus(loadingStatus);

    try {
      const data = await loadUsageWorkbenchData();
      setApiKeys(data.apiKeys);
      setRecords(data.records);
      setStatus(syncedStatus);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    }
  }

  useEffect(() => {
    void refreshUsageWorkbench();
  }, [loadingStatus, syncedStatus]);

  const viewModel = useMemo(
    () =>
      buildPortalUsageViewModel({
        apiKeys,
        records,
        filters,
        page,
        page_size: PAGE_SIZE,
      }),
    [apiKeys, filters, page, records, t],
  );
  const totalPages = viewModel.pagination.total_pages;
  const currentPage = viewModel.pagination.page;
  const totalItems = viewModel.pagination.total_items;

  useEffect(() => {
    setPage((current) => clampPage(current, totalPages));
  }, [totalPages]);

  function updateFilters(nextFilters: Partial<UsageFilters>): void {
    startTransition(() => {
      setFilters((current) => ({
        ...current,
        ...nextFilters,
      }));
      setPage(1);
    });
  }

  function clearFilters(): void {
    startTransition(() => {
      setFilters(DEFAULT_FILTERS);
      setPage(1);
    });
  }

  const hasActiveFilters = Object.entries(filters).some(([key, value]) => {
    const defaultValue = DEFAULT_FILTERS[key as keyof UsageFilters];
    return value !== defaultValue;
  });
  const pageStatus = status !== syncedStatus ? status : '';
  const showingStart = totalItems === 0 ? 0 : (currentPage - 1) * PAGE_SIZE + 1;
  const showingEnd = totalItems === 0 ? 0 : Math.min(currentPage * PAGE_SIZE, totalItems);

  return (
    <div className="space-y-4" data-slot="portal-usage-page">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          className="h-full"
          description={t(
            'Total request count after API key, time range, channel, and model filters are applied.',
          )}
          label={t('Total requests')}
          value={formatUnits(viewModel.summary.total_requests)}
        />
        <StatCard
          className="h-full"
          description={t('Input {input} / Output {output}', {
            input: formatUnits(viewModel.summary.input_tokens),
            output: formatUnits(viewModel.summary.output_tokens),
          })}
          label={t('Total tokens')}
          value={formatUnits(viewModel.summary.total_tokens)}
        />
        <StatCard
          className="h-full"
          description={t('{actual} / {reference} actual deduction / reference original price.', {
            actual: formatCurrency(viewModel.summary.actual_amount),
            reference: formatCurrency(viewModel.summary.reference_amount),
          })}
          label={t('Total spend')}
          value={`${formatCurrency(viewModel.summary.actual_amount)} / ${formatCurrency(viewModel.summary.reference_amount)}`}
        />
        <StatCard
          className="h-full"
          description={t('Average latency across usage rows that already contain request latency facts.')}
          label={t('Average latency')}
          value={formatLatency(viewModel.summary.average_latency_ms, t)}
        />
      </div>

      <FilterBar data-slot="portal-usage-filter-bar" wrap={false}>
        <FilterBarSection className="min-w-[12rem] flex-[1_1_15rem]" grow={false} wrap={false}>
          <FilterField className="w-full" label={t('API key')}>
            <Select
              value={filters.api_key_hash}
              onValueChange={(value) => updateFilters({ api_key_hash: value })}
            >
              <SelectTrigger>
                <SelectValue placeholder={t('API key')} />
              </SelectTrigger>
              <SelectContent>
                {viewModel.filter_options.api_keys.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {t(option.label)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </FilterField>
        </FilterBarSection>

        <FilterBarSection className="min-w-[11rem] shrink-0" grow={false} wrap={false}>
          <FilterField className="w-full" label={t('Time range')}>
            <Select
              value={filters.time_range}
              onValueChange={(value) =>
                updateFilters({
                  time_range: value as UsageFilters['time_range'],
                })
              }
            >
              <SelectTrigger>
                <SelectValue placeholder={t('Time range')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="24h">{t('Last 24 hours')}</SelectItem>
                <SelectItem value="7d">{t('Last 7 days')}</SelectItem>
                <SelectItem value="30d">{t('Last 30 days')}</SelectItem>
                <SelectItem value="all">{t('All time')}</SelectItem>
              </SelectContent>
            </Select>
          </FilterField>
        </FilterBarSection>

        <FilterBarSection className="min-w-[11rem] shrink-0" grow={false} wrap={false}>
          <FilterField className="w-full" label={t('Channel')}>
            <Select
              value={filters.channel_id}
              onValueChange={(value) => updateFilters({ channel_id: value })}
            >
              <SelectTrigger>
                <SelectValue placeholder={t('Channel')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t('All channels')}</SelectItem>
                {viewModel.filter_options.channels
                  .filter((option) => option !== 'all')
                  .map((option) => (
                    <SelectItem key={option} value={option}>
                      {option}
                    </SelectItem>
                  ))}
              </SelectContent>
            </Select>
          </FilterField>
        </FilterBarSection>

        <FilterBarSection className="min-w-[12rem] flex-[1_1_15rem]" grow={false} wrap={false}>
          <FilterField className="w-full" label={t('Model')}>
            <Select
              value={filters.model}
              onValueChange={(value) => updateFilters({ model: value })}
            >
              <SelectTrigger>
                <SelectValue placeholder={t('Model')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t('All models')}</SelectItem>
                {viewModel.filter_options.models
                  .filter((option) => option !== 'all')
                  .map((option) => (
                    <SelectItem key={option} value={option}>
                      {option}
                    </SelectItem>
                  ))}
              </SelectContent>
            </Select>
          </FilterField>
        </FilterBarSection>

        <FilterBarActions className="gap-2.5 whitespace-nowrap shrink-0" wrap={false}>
          <Button disabled={!hasActiveFilters} onClick={clearFilters} variant="secondary">
            {t('Clear filters')}
          </Button>
          <Button onClick={() => void refreshUsageWorkbench()} variant="secondary">
            <RefreshCw className="h-4 w-4" />
            {t('Refresh')}
          </Button>
        </FilterBarActions>
      </FilterBar>

      {pageStatus ? (
        <div
          className="rounded-2xl border border-zinc-200 bg-zinc-50/85 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300"
          data-slot="portal-usage-feedback"
          role="status"
        >
          {pageStatus}
        </div>
      ) : null}

      <DataTable
        className="rounded-[28px] border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950"
        columns={[
          { id: 'api-key', header: t('API key'), cell: (row) => <strong>{row.api_key_label}</strong> },
          { id: 'channel', header: t('Channel'), cell: (row) => row.channel_label },
          { id: 'model', header: t('Model'), cell: (row) => <strong>{row.model}</strong> },
          { id: 'input', header: t('Input tokens'), cell: (row) => formatUnits(row.input_tokens) },
          { id: 'output', header: t('Output tokens'), cell: (row) => formatUnits(row.output_tokens) },
          { id: 'total', header: t('Total tokens'), cell: (row) => formatUnits(row.total_tokens) },
          { id: 'actual', header: t('Actual spend'), cell: (row) => formatCurrency(row.amount) },
          {
            id: 'reference',
            header: t('Reference price'),
            cell: (row) => formatCurrency(row.reference_amount),
          },
          { id: 'latency', header: t('Latency'), cell: (row) => formatLatency(row.latency_ms, t) },
          { id: 'time', header: t('Recorded'), cell: (row) => formatDateTime(row.created_at_ms) },
        ]}
        data-slot="portal-usage-table"
        emptyState={(
          <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
            <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
              {t('No usage records for this slice')}
            </strong>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">
              {records.length
                ? t('Adjust the API key, time range, channel, or model filter to reveal more request facts.')
                : status}
            </p>
          </div>
        )}
        footer={(
          <div
            className="flex flex-col gap-3 rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/50 lg:flex-row lg:items-center lg:justify-between"
            data-slot="portal-usage-pagination"
          >
            <div className="text-sm text-zinc-600 dark:text-zinc-300">
              {t('Showing {start}-{end} of {total} records', {
                end: showingEnd,
                start: showingStart,
                total: totalItems,
              })}
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Button
                disabled={currentPage <= 1}
                onClick={() =>
                  startTransition(() => {
                    setPage((current) => clampPage(current - 1, totalPages));
                  })
                }
                variant="secondary"
              >
                {t('Previous page')}
              </Button>
              <span className="min-w-[8rem] text-center text-sm font-medium text-zinc-600 dark:text-zinc-300">
                {t('Page {page} of {total}', { page: currentPage, total: totalPages })}
              </span>
              <Button
                disabled={currentPage >= totalPages}
                onClick={() =>
                  startTransition(() => {
                    setPage((current) => clampPage(current + 1, totalPages));
                  })
                }
                variant="secondary"
              >
                {t('Next page')}
              </Button>
            </div>
          </div>
        )}
        getRowId={(row, index) => buildUsageRecordKey(row, index)}
        rows={viewModel.rows}
      />
    </div>
  );
}



