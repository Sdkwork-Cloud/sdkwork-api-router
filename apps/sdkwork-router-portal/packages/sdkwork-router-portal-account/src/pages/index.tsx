import { startTransition, useDeferredValue, useEffect, useMemo, useState } from 'react';

import {
  formatCurrency,
  formatDateTime,
  formatUnits,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  Badge,
  DataTable,
  Tabs,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons/framework/display';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import { SearchInput } from 'sdkwork-router-portal-commons/framework/form';
import {
  Card,
  CardContent,
} from 'sdkwork-router-portal-commons/framework/layout';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  BillingAccountingMode,
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  LedgerEntry,
  PortalCommerceMembership,
  ProjectBillingSummary,
  UsageRecord,
  UsageSummary,
} from 'sdkwork-router-portal-types';

import {
  getPortalBillingEventSummary,
  getPortalBillingSummary,
  getPortalCommerceMembership,
  getPortalUsageSummary,
  listPortalBillingLedger,
  listPortalUsageRecords,
} from '../repository';
import { buildPortalAccountViewModel } from '../services';
import type {
  AccountMetricSummary,
  PortalAccountHistoryRow,
  PortalAccountHistoryView,
  PortalAccountPageProps,
} from '../types';

const ACCOUNT_PAGE_SIZE = 8;

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;
type MetricBreakdown = { label: string; value: string };
type FinancialBreakdownItem = { label: string; value: string; detail: string };
type FinancialMetric = { label: string; value: string };

function clampPercentage(value: number): number {
  return Math.min(100, Math.max(0, value));
}

function formatAverageSpend(
  summary: AccountMetricSummary,
  t: TranslateFn,
): string {
  return summary.request_count > 0 ? formatCurrency(summary.average_booked_spend) : t('n/a');
}

function AccountMetricCard({
  breakdowns,
  description,
  label,
  value,
}: {
  breakdowns: MetricBreakdown[];
  description: string;
  label: string;
  value: string;
}) {
  return (
    <Card className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950">
      <CardContent className="p-5">
        <div className="space-y-4">
          <div className="space-y-2">
            <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
              {label}
            </p>
            <strong className="block text-3xl font-semibold text-zinc-950 dark:text-zinc-50">
              {value}
            </strong>
            <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">
              {description}
            </p>
          </div>

          <div className="grid grid-cols-3 gap-2">
            {breakdowns.map((item) => (
              <div
                key={item.label}
                className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-3 py-3 dark:border-zinc-800 dark:bg-zinc-900/60"
              >
                <span className="text-[11px] font-medium uppercase tracking-[0.14em] text-zinc-500 dark:text-zinc-400">
                  {item.label}
                </span>
                <strong className="mt-1 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {item.value}
                </strong>
              </div>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function AccountBalanceCard({
  balanceValue,
  description,
  onRecharge,
  onRedeem,
  planValue,
  quotaLimitValue,
  statusLabel,
  t,
  usedBreakdowns,
  usedUnitsValue,
  utilizationPercent,
}: {
  balanceValue: string;
  description: string;
  onRecharge: () => void;
  onRedeem: () => void;
  planValue: string;
  quotaLimitValue: string;
  statusLabel: string;
  t: TranslateFn;
  usedBreakdowns: MetricBreakdown[];
  usedUnitsValue: string;
  utilizationPercent: number | null;
}) {
  return (
    <Card className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950 xl:col-span-2">
      <CardContent className="p-6">
        <div className="space-y-5">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div className="min-w-0 flex-1 space-y-3">
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Balance')}
              </p>
              <div
                className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between"
                data-slot="portal-account-balance-primary"
              >
                <strong className="block text-4xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                  {balanceValue}
                </strong>
                <div
                  className="flex flex-wrap gap-2"
                  data-slot="portal-account-balance-actions"
                >
                  <Button onClick={onRecharge}>{t('Recharge')}</Button>
                  <Button onClick={onRedeem} variant="secondary">
                    {t('Redeem')}
                  </Button>
                </div>
              </div>
              <p className="max-w-2xl text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {description}
              </p>
            </div>
            <Badge variant="secondary">{statusLabel}</Badge>
          </div>

          <div className="h-2.5 overflow-hidden rounded-full bg-zinc-200/80 dark:bg-zinc-800/80">
            <div
              className="h-full rounded-full bg-zinc-900 transition-all dark:bg-zinc-100"
              style={{ width: `${utilizationPercent ?? 32}%` }}
            />
          </div>

          <div className="grid gap-3 sm:grid-cols-3">
            <AccountSnapshotItem
              breakdowns={usedBreakdowns}
              breakdownsSlot="portal-account-used-breakdowns"
              label={t('Used units')}
              value={usedUnitsValue}
              valueSlot="portal-account-used-total"
            />
            <AccountSnapshotItem label={t('Quota limit')} value={quotaLimitValue} />
            <AccountSnapshotItem label={t('Plan')} value={planValue} />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function AccountSnapshotItem({
  breakdowns,
  breakdownsSlot,
  label,
  value,
  valueSlot,
}: {
  breakdowns?: MetricBreakdown[];
  breakdownsSlot?: string;
  label: string;
  value: string;
  valueSlot?: string;
}) {
  return (
    <div className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-4 dark:border-zinc-800 dark:bg-zinc-900/60">
      <span className="text-xs font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
        {label}
      </span>
      <strong
        className="mt-2 block text-lg font-semibold text-zinc-950 dark:text-zinc-50"
        data-slot={valueSlot}
      >
        {value}
      </strong>
      {breakdowns?.length ? (
        <div
          className="mt-3 grid grid-cols-3 gap-2"
          data-slot={breakdownsSlot}
        >
          {breakdowns.map((item) => (
            <div
              key={item.label}
              className="rounded-xl border border-zinc-200 bg-white px-2.5 py-2 dark:border-zinc-800 dark:bg-zinc-950"
            >
              <span className="block text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500 dark:text-zinc-400">
                {item.label}
              </span>
              <strong className="mt-1 block text-xs font-semibold text-zinc-950 dark:text-zinc-50">
                {item.value}
              </strong>
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function AccountFinancialListCard({
  dataSlot,
  description,
  emptyDescription,
  emptyTitle,
  items,
  title,
}: {
  dataSlot: string;
  description: string;
  emptyDescription: string;
  emptyTitle: string;
  items: FinancialBreakdownItem[];
  title: string;
}) {
  return (
    <Card
      className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
      data-slot={dataSlot}
    >
      <CardContent className="space-y-4 p-5">
        <div className="space-y-2">
          <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
            {title}
          </h2>
          <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">
            {description}
          </p>
        </div>

        {items.length ? (
          <div className="space-y-3">
            {items.map((item) => (
              <div
                key={`${title}:${item.label}`}
                className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-4 dark:border-zinc-800 dark:bg-zinc-900/60"
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <strong className="block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                      {item.label}
                    </strong>
                    <p className="mt-1 text-xs leading-5 text-zinc-500 dark:text-zinc-400">
                      {item.detail}
                    </p>
                  </div>
                  <span className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {item.value}
                  </span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <EmptyState description={emptyDescription} title={emptyTitle} />
        )}
      </CardContent>
    </Card>
  );
}

function AccountFinancialMetricCard({
  dataSlot,
  description,
  metrics,
  title,
}: {
  dataSlot: string;
  description: string;
  metrics: FinancialMetric[];
  title: string;
}) {
  return (
    <Card
      className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
      data-slot={dataSlot}
    >
      <CardContent className="space-y-4 p-5">
        <div className="space-y-2">
          <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
            {title}
          </h2>
          <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">
            {description}
          </p>
        </div>

        <div className="grid gap-3 sm:grid-cols-2">
          {metrics.map((metric) => (
            <div
              key={`${title}:${metric.label}`}
              className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-4 dark:border-zinc-800 dark:bg-zinc-900/60"
            >
              <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                {metric.label}
              </span>
              <strong className="mt-2 block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                {metric.value}
              </strong>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function buildMetricBreakdowns(
  t: TranslateFn,
  today: string,
  trailing7d: string,
  currentMonth: string,
): MetricBreakdown[] {
  return [
    { label: t('Today'), value: today },
    { label: t('7 days'), value: trailing7d },
    { label: t('This month'), value: currentMonth },
  ];
}

function titleCaseToken(value: string): string {
  return value
    .split(/[-_\s]+/g)
    .filter(Boolean)
    .map((segment) =>
      segment.length <= 3
        ? segment.toUpperCase()
        : `${segment.slice(0, 1).toUpperCase()}${segment.slice(1)}`,
    )
    .join(' ');
}

function capabilityLabel(
  capability: string,
  t: TranslateFn,
): string {
  switch (capability.trim().toLowerCase()) {
    case 'responses':
      return t('Responses');
    case 'images':
      return t('Images');
    case 'audio':
      return t('Audio');
    case 'video':
      return t('Video');
    case 'music':
      return t('Music');
    default:
      return titleCaseToken(capability);
  }
}

function accountingModeLabel(
  mode: BillingAccountingMode,
  t: TranslateFn,
): string {
  switch (mode) {
    case 'platform_credit':
      return t('Platform credit');
    case 'byok':
      return t('BYOK');
    case 'passthrough':
      return t('Passthrough');
    default:
      return t('Accounting mode');
  }
}

function capabilitySignalLabel(
  capability: BillingEventCapabilitySummary,
  t: TranslateFn,
): string {
  const signals: string[] = [];

  if (capability.total_tokens > 0) {
    signals.push(t('{count} tokens', { count: formatUnits(capability.total_tokens) }));
  }
  if (capability.image_count > 0) {
    signals.push(t('{count} images', { count: formatUnits(capability.image_count) }));
  }
  if (capability.audio_seconds > 0) {
    signals.push(t('{count} audio sec', { count: formatUnits(capability.audio_seconds) }));
  }
  if (capability.video_seconds > 0) {
    signals.push(t('{count} video sec', { count: formatUnits(capability.video_seconds) }));
  }
  if (capability.music_seconds > 0) {
    signals.push(t('{count} music sec', { count: formatUnits(capability.music_seconds) }));
  }

  signals.push(
    t('{count} requests', { count: formatUnits(capability.request_count) }),
  );

  return signals.join(' / ');
}

function accountingModeSignalLabel(
  summary: BillingEventAccountingModeSummary,
  t: TranslateFn,
): string {
  return t('{requests} requests / {events} events', {
    requests: formatUnits(summary.request_count),
    events: formatUnits(summary.event_count),
  });
}

function buildCapabilityMixItems(
  capabilities: BillingEventCapabilitySummary[],
  t: TranslateFn,
): FinancialBreakdownItem[] {
  return capabilities.map((capability) => ({
    label: capabilityLabel(capability.capability, t),
    value: formatCurrency(capability.total_customer_charge),
    detail: capabilitySignalLabel(capability, t),
  }));
}

function buildAccountingModeItems(
  accountingModes: BillingEventAccountingModeSummary[],
  t: TranslateFn,
): FinancialBreakdownItem[] {
  return accountingModes.map((summary) => ({
    label: accountingModeLabel(summary.accounting_mode, t),
    value: formatCurrency(summary.total_customer_charge),
    detail: accountingModeSignalLabel(summary, t),
  }));
}

function buildMultimodalUsageMetrics(
  summary: {
    image_count: number;
    audio_seconds: number;
    video_seconds: number;
    music_seconds: number;
  },
  t: TranslateFn,
): FinancialMetric[] {
  return [
    { label: t('Images'), value: formatUnits(summary.image_count) },
    { label: t('Audio'), value: formatUnits(summary.audio_seconds) },
    { label: t('Video'), value: formatUnits(summary.video_seconds) },
    { label: t('Music'), value: formatUnits(summary.music_seconds) },
  ];
}

function formatSignedCurrency(
  amount: number,
  kind: PortalAccountHistoryRow['kind'],
): string {
  return `${kind === 'expense' ? '-' : '+'}${formatCurrency(amount)}`;
}

function formatSignedUnits(
  units: number,
  kind: PortalAccountHistoryRow['kind'],
): string {
  return `${kind === 'expense' ? '-' : '+'}${formatUnits(units)}`;
}

function resolveHistoryRecordedLabel(
  row: PortalAccountHistoryRow,
  t: TranslateFn,
): string {
  return row.occurred_at_ms ? formatDateTime(row.occurred_at_ms) : t('Ledger snapshot');
}

function resolveMembershipStatusLabel(
  status: string | null | undefined,
  t: TranslateFn,
): string {
  const normalized = status?.trim().toLowerCase() ?? '';

  switch (normalized) {
    case 'active':
      return t('Active');
    case 'inactive':
      return t('Inactive');
    case 'canceled':
    case 'cancelled':
      return t('Canceled');
    case 'past_due':
    case 'past-due':
      return t('Past due');
    case 'grace_period':
    case 'grace-period':
      return t('Grace period');
    case 'paused':
      return t('Paused');
    default:
      return status?.trim() ? titleCaseToken(status) : t('Inactive');
  }
}

function resolveQuotaPolicyLabel(
  quotaPolicyId: string | null | undefined,
  t: TranslateFn,
): string {
  const normalized = quotaPolicyId?.trim().toLowerCase() ?? '';

  if (!normalized) {
    return t('Workspace default');
  }

  if (normalized === 'quota-enterprise' || normalized === 'enterprise') {
    return t('Enterprise quota');
  }

  return titleCaseToken(quotaPolicyId ?? '');
}

function resolveHistoryTitle(
  row: PortalAccountHistoryRow,
  t: TranslateFn,
): string {
  if (row.source === 'usage') {
    return row.model ?? t('Usage record');
  }

  return t('Ledger snapshot');
}

function resolveHistoryProjectLabel(
  row: PortalAccountHistoryRow,
  t: TranslateFn,
): string {
  return row.scope === 'current' ? t('Current workspace') : t('Linked project');
}

function resolveHistoryProviderLabel(
  provider: string | null | undefined,
  t: TranslateFn,
): string {
  const normalized = provider?.trim().toLowerCase() ?? '';

  if (!normalized) {
    return t('Unassigned');
  }

  if (normalized.includes('openai')) {
    return t('OpenAI');
  }

  if (normalized.includes('anthropic')) {
    return t('Anthropic');
  }

  if (normalized.includes('gemini')) {
    return t('Gemini');
  }

  return provider?.trim() ?? t('Unassigned');
}

function resolveHistoryChannelLabel(
  channelId: string | null | undefined,
  t: TranslateFn,
): string {
  const normalized = channelId?.trim().toLowerCase() ?? '';

  switch (normalized) {
    case 'openai':
      return t('OpenAI-compatible');
    case 'anthropic':
      return t('Anthropic Messages');
    case 'gemini':
      return t('Gemini');
    default:
      return channelId?.trim() || t('Unassigned');
  }
}

function resolveHistoryDetail(
  row: PortalAccountHistoryRow,
  t: TranslateFn,
): string {
  if (row.source === 'usage') {
    return [
      `${t('Provider')}: ${resolveHistoryProviderLabel(row.provider, t)}`,
      `${t('Channel')}: ${resolveHistoryChannelLabel(row.channel_id, t)}`,
      resolveHistoryProjectLabel(row, t),
    ]
      .filter(Boolean)
      .join(' / ');
  }

  return resolveHistoryProjectLabel(row, t);
}

function resolveHistoryEmptyState(
  historyView: PortalAccountHistoryView,
  searchQuery: string,
  status: string,
  t: TranslateFn,
): { title: string; description: string } {
  const normalizedSearch = searchQuery.trim();

  if (normalizedSearch) {
    return {
      title: t('No account history for this slice'),
      description: t('Adjust the search or switch history tabs to reveal more records.'),
    };
  }

  if (historyView === 'expense') {
    return {
      title: t('No expense history for this slice'),
      description: t('Expense history will appear once workspace usage starts generating billable activity.'),
    };
  }

  if (historyView === 'revenue') {
    return {
      title: t('No revenue entries for this slice'),
      description: t('Revenue history will appear once billing ledger movements reach this account.'),
    };
  }

  return {
    title: t('No account history for this slice'),
    description: status,
  };
}

export function PortalAccountPage({ onNavigate }: PortalAccountPageProps) {
  const { t } = usePortalI18n();
  const loadingStatus = t('Loading account analytics...');
  const syncedStatus = t('Account analytics are synced with the latest billing and usage evidence.');
  const [summary, setSummary] = useState<ProjectBillingSummary | null>(null);
  const [billingEventSummary, setBillingEventSummary] = useState<BillingEventSummary | null>(null);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [membership, setMembership] = useState<PortalCommerceMembership | null>(null);
  const [usageSummary, setUsageSummary] = useState<UsageSummary | null>(null);
  const [usageRecords, setUsageRecords] = useState<UsageRecord[]>([]);
  const [status, setStatus] = useState(loadingStatus);
  const [historyView, setHistoryView] = useState<PortalAccountHistoryView>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [page, setPage] = useState(1);
  const deferredSearch = useDeferredValue(searchQuery);

  useEffect(() => {
    let cancelled = false;

    setStatus(loadingStatus);

    void Promise.all([
      getPortalBillingSummary(),
      getPortalBillingEventSummary(),
      listPortalBillingLedger(),
      getPortalCommerceMembership(),
      getPortalUsageSummary(),
      listPortalUsageRecords(),
    ])
      .then(
        ([
          nextSummary,
          nextBillingEventSummary,
          nextLedger,
          nextMembership,
          nextUsageSummary,
          nextUsageRecords,
        ]) => {
          if (cancelled) {
            return;
          }

          setSummary(nextSummary);
          setBillingEventSummary(nextBillingEventSummary);
          setLedger(nextLedger);
          setMembership(nextMembership);
          setUsageSummary(nextUsageSummary);
          setUsageRecords(nextUsageRecords);
          setStatus(syncedStatus);
        },
      )
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [loadingStatus, syncedStatus]);

  const viewModel = useMemo(
    () =>
      summary && usageSummary && billingEventSummary
        ? buildPortalAccountViewModel({
            summary,
            membership,
            usageSummary,
            usageRecords,
            ledger,
            billingEventSummary,
            historyView,
            page,
            pageSize: ACCOUNT_PAGE_SIZE,
            searchQuery: deferredSearch,
          })
        : null,
    [
      billingEventSummary,
      deferredSearch,
      historyView,
      ledger,
      membership,
      page,
      summary,
      usageRecords,
      usageSummary,
    ],
  );

  useEffect(() => {
    if (viewModel && viewModel.pagination.page !== page) {
      setPage(viewModel.pagination.page);
    }
  }, [page, viewModel]);

  if (!viewModel) {
    return (
      <Card className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950">
        <CardContent className="p-8">
          <EmptyState
            description={t(
              'Account analytics will appear after the portal loads billing, membership, and usage evidence.',
            )}
            title={status || t('Preparing account')}
          />
        </CardContent>
      </Card>
    );
  }

  const remainingUnits = viewModel.balance.remaining_units;
  const quotaRatio = viewModel.balance.utilization_ratio === null
    ? null
    : clampPercentage(viewModel.balance.utilization_ratio * 100);
  const postureStatusLabel = viewModel.billing_summary.exhausted ? t('Exhausted') : t('Healthy');
  const pageStatus = status !== syncedStatus ? status : '';
  const capabilityMixItems = buildCapabilityMixItems(
    viewModel.financial_breakdown.top_capabilities,
    t,
  );
  const accountingModeItems = buildAccountingModeItems(
    viewModel.financial_breakdown.accounting_mode_mix,
    t,
  );
  const multimodalMetrics = buildMultimodalUsageMetrics(
    viewModel.financial_breakdown.multimodal_totals,
    t,
  );

  return (
    <div className="space-y-4" data-slot="portal-account-page">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <AccountBalanceCard
          balanceValue={remainingUnits === null ? t('Unlimited') : formatUnits(remainingUnits)}
          description={t('Available workspace balance before the quota guardrail is reached.')}
          onRecharge={() => onNavigate('recharge')}
          onRedeem={() => onNavigate('credits')}
          planValue={viewModel.membership?.plan_name ?? t('No membership')}
          quotaLimitValue={
            viewModel.balance.quota_limit_units === null
              ? t('Unlimited')
              : formatUnits(viewModel.balance.quota_limit_units)
          }
          statusLabel={postureStatusLabel}
          t={t}
          usedBreakdowns={buildMetricBreakdowns(
            t,
            formatUnits(viewModel.today.used_units),
            formatUnits(viewModel.trailing_7d.used_units),
            formatUnits(viewModel.current_month.used_units),
          )}
          usedUnitsValue={formatUnits(viewModel.balance.used_units)}
          utilizationPercent={quotaRatio}
        />
        <AccountMetricCard
          breakdowns={buildMetricBreakdowns(
            t,
            formatCurrency(viewModel.today.revenue),
            formatCurrency(viewModel.trailing_7d.revenue),
            formatCurrency(viewModel.current_month.revenue),
          )}
          description={t('Total booked revenue and recent realized income stay aligned on one account surface.')}
          label={t('Revenue')}
          value={formatCurrency(viewModel.totals.revenue)}
        />
        <AccountMetricCard
          breakdowns={buildMetricBreakdowns(
            t,
            formatUnits(viewModel.today.request_count),
            formatUnits(viewModel.trailing_7d.request_count),
            formatUnits(viewModel.current_month.request_count),
          )}
          description={t('Call volume shows total routed demand plus recent billing windows.')}
          label={t('Total requests')}
          value={formatUnits(viewModel.totals.request_count)}
        />
        <AccountMetricCard
          breakdowns={buildMetricBreakdowns(
            t,
            formatAverageSpend(viewModel.today, t),
            formatAverageSpend(viewModel.trailing_7d, t),
            formatAverageSpend(viewModel.current_month, t),
          )}
          description={t('Average value per request keeps account efficiency visible across billing periods.')}
          label={t('Average booked spend')}
          value={formatAverageSpend(viewModel.totals, t)}
        />
      </div>

      <Card
        className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
        data-slot="portal-account-financial-breakdown"
      >
        <CardContent className="space-y-4 p-5">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
            <div className="space-y-2">
              <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                {t('Financial breakdown')}
              </h2>
              <p className="max-w-3xl text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {t(
                  'Billing event evidence adds capability mix, accounting mode posture, and multimodal totals to the account surface.',
                )}
              </p>
            </div>

            <div className="grid gap-3 sm:grid-cols-2">
              <AccountSnapshotItem
                label={t('Billing events')}
                value={formatUnits(viewModel.financial_breakdown.total_events)}
              />
              <AccountSnapshotItem
                label={t('Customer charge')}
                value={formatCurrency(viewModel.financial_breakdown.total_customer_charge)}
              />
            </div>
          </div>

          <div className="grid gap-4 xl:grid-cols-3">
            <AccountFinancialListCard
              dataSlot="portal-account-capability-mix"
              description={t(
                'Capability mix keeps the highest-charge API capabilities visible beside the account ledger.',
              )}
              emptyDescription={t(
                'Capability mix appears after billing events attribute charge across routed API capabilities.',
              )}
              emptyTitle={t('No capability mix yet')}
              items={capabilityMixItems}
              title={t('Capability mix')}
            />
            <AccountFinancialListCard
              dataSlot="portal-account-accounting-mode-mix"
              description={t(
                'Accounting mode mix separates platform credit, BYOK, and passthrough consumption in one account slice.',
              )}
              emptyDescription={t(
                'Accounting mode evidence appears after billing events record platform credit, BYOK, or passthrough traffic.',
              )}
              emptyTitle={t('No accounting mode mix yet')}
              items={accountingModeItems}
              title={t('Accounting mode mix')}
            />
            <AccountFinancialMetricCard
              dataSlot="portal-account-multimodal-usage"
              description={t(
                'Images, audio, video, and music totals keep multimodal demand visible on the same financial surface.',
              )}
              metrics={multimodalMetrics}
              title={t('Multimodal usage')}
            />
          </div>
        </CardContent>
      </Card>

      <Card
        className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
        data-slot="portal-account-history"
      >
        <CardContent className="space-y-4 p-5">
          <div
            className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between"
            data-slot="portal-account-history-header"
          >
            <div className="flex min-w-0 flex-col gap-3 lg:flex-row lg:items-center lg:gap-4">
              <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                {t('Account history')}
              </h2>

              <Tabs
                className="min-w-0"
                onValueChange={(value) =>
                  startTransition(() => {
                    setHistoryView(value as PortalAccountHistoryView);
                    setPage(1);
                  })}
                value={historyView}
              >
                <TabsList
                  className="inline-flex w-auto justify-start"
                  data-slot="portal-account-history-tabs"
                >
                  <TabsTrigger value="all">
                    <span className="inline-flex items-center gap-2">
                      <span>{t('All')}</span>
                      <span className="text-xs text-zinc-500 dark:text-zinc-400">
                        {formatUnits(viewModel.history_counts.all)}
                      </span>
                    </span>
                  </TabsTrigger>
                  <TabsTrigger value="expense">
                    <span className="inline-flex items-center gap-2">
                      <span>{t('Expense')}</span>
                      <span className="text-xs text-zinc-500 dark:text-zinc-400">
                        {formatUnits(viewModel.history_counts.expense)}
                      </span>
                    </span>
                  </TabsTrigger>
                  <TabsTrigger value="revenue">
                    <span className="inline-flex items-center gap-2">
                      <span>{t('Revenue')}</span>
                      <span className="text-xs text-zinc-500 dark:text-zinc-400">
                        {formatUnits(viewModel.history_counts.revenue)}
                      </span>
                    </span>
                  </TabsTrigger>
                </TabsList>
              </Tabs>
            </div>

            <div
              className="w-full xl:w-[22rem]"
              data-slot="portal-account-toolbar"
            >
              <SearchInput
                className="w-full"
                data-slot="portal-account-history-search"
                inputClassName="w-full"
                onChange={(event) =>
                  startTransition(() => {
                    setSearchQuery(event.target.value);
                    setPage(1);
                  })}
                placeholder={t('Search account history')}
                value={searchQuery}
              />
            </div>
          </div>

          {pageStatus ? (
            <div
              className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/50 dark:text-zinc-300"
              role="status"
            >
              {pageStatus}
            </div>
          ) : null}

          <DataTable
            className="rounded-2xl border border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950"
            columns={[
              {
                id: 'recorded',
                header: t('Recorded'),
                cell: (row) => resolveHistoryRecordedLabel(row, t),
              },
              {
                id: 'type',
                header: t('Type'),
                cell: (row) => (
                  <Badge variant={row.kind === 'expense' ? 'warning' : 'success'}>
                    {row.kind === 'expense' ? t('Expense') : t('Revenue')}
                  </Badge>
                ),
              },
              {
                id: 'details',
                header: t('Details'),
                cell: (row) => (
                  <div className="flex flex-col gap-1">
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {resolveHistoryTitle(row, t)}
                    </strong>
                    <span className="text-xs text-zinc-500 dark:text-zinc-400">
                      {resolveHistoryDetail(row, t)}
                    </span>
                  </div>
                ),
              },
              {
                id: 'units',
                header: t('Units'),
                cell: (row) => formatSignedUnits(row.units, row.kind),
              },
              {
                id: 'amount',
                header: t('Amount'),
                cell: (row) => formatSignedCurrency(row.amount, row.kind),
              },
            ]}
            data-slot="portal-account-table"
            emptyState={(() => {
              const emptyState = resolveHistoryEmptyState(historyView, searchQuery, status, t);

              return (
                <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                  <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {emptyState.title}
                  </strong>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">
                    {emptyState.description}
                  </p>
                </div>
              );
            })()}
            footer={(
              <div
                className="flex flex-col gap-3 rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/50 sm:flex-row sm:items-center sm:justify-between"
                data-slot="portal-account-pagination"
              >
                <span className="text-sm font-medium text-zinc-600 dark:text-zinc-300">
                  {t('Page {page} of {total}', {
                    page: viewModel.pagination.page,
                    total: viewModel.pagination.total_pages,
                  })}
                </span>
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    disabled={viewModel.pagination.page <= 1}
                    onClick={() =>
                      startTransition(() => {
                        setPage((current) => Math.max(1, current - 1));
                      })}
                    variant="secondary"
                  >
                    {t('Previous page')}
                  </Button>
                  <Button
                    disabled={viewModel.pagination.page >= viewModel.pagination.total_pages}
                    onClick={() =>
                      startTransition(() => {
                        setPage((current) => current + 1);
                      })}
                    variant="secondary"
                  >
                    {t('Next page')}
                  </Button>
                </div>
              </div>
            )}
            getRowId={(row) => row.id}
            rows={viewModel.visible_history}
          />
        </CardContent>
      </Card>
    </div>
  );
}
