import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  LedgerEntry,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import type {
  AccountMetricSummary,
  BuildPortalAccountViewModelInput,
  PortalAccountFinancialBreakdown,
  PortalAccountBalanceSummary,
  PortalAccountHistoryRow,
  PortalAccountHistoryView,
  PortalAccountViewModel,
} from '../types';

function clampPage(page: number, totalPages: number): number {
  return Math.min(Math.max(page, 1), Math.max(totalPages, 1));
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

function summarizeUsageRecords(records: UsageRecord[]): AccountMetricSummary {
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

function buildBalanceSummary(
  input: BuildPortalAccountViewModelInput,
): PortalAccountBalanceSummary {
  const quotaLimitUnits = input.summary.quota_limit_units ?? null;
  const usedUnits = input.summary.used_units;

  return {
    remaining_units: input.summary.remaining_units ?? null,
    quota_limit_units: quotaLimitUnits,
    used_units: usedUnits,
    utilization_ratio:
      quotaLimitUnits && quotaLimitUnits > 0
        ? Math.min(1, Math.max(0, usedUnits / quotaLimitUnits))
        : null,
  };
}

function sortCapabilityMix(
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

function sortAccountingModeMix(
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

function buildFinancialBreakdown(
  summary: BillingEventSummary | null | undefined,
): PortalAccountFinancialBreakdown {
  const safeSummary = summary ?? emptyBillingEventSummary();

  return {
    total_events: safeSummary.total_events,
    total_request_count: safeSummary.total_request_count,
    total_customer_charge: safeSummary.total_customer_charge,
    top_capabilities: sortCapabilityMix(safeSummary.capabilities).slice(0, 3),
    accounting_mode_mix: sortAccountingModeMix(safeSummary.accounting_modes).slice(0, 3),
    multimodal_totals: {
      image_count: safeSummary.total_image_count,
      audio_seconds: safeSummary.total_audio_seconds,
      video_seconds: safeSummary.total_video_seconds,
      music_seconds: safeSummary.total_music_seconds,
    },
  };
}

function ledgerSortKey(projectId: string, row: LedgerEntry): [number, number, string] {
  return [
    row.project_id === projectId ? 0 : 1,
    -Math.abs(row.amount),
    row.project_id,
  ];
}

function compareLedgerRows(projectId: string, left: LedgerEntry, right: LedgerEntry): number {
  const leftKey = ledgerSortKey(projectId, left);
  const rightKey = ledgerSortKey(projectId, right);

  if (leftKey[0] !== rightKey[0]) {
    return leftKey[0] - rightKey[0];
  }

  if (leftKey[1] !== rightKey[1]) {
    return leftKey[1] - rightKey[1];
  }

  return leftKey[2].localeCompare(rightKey[2]);
}

function classifyLedgerKind(row: LedgerEntry): PortalAccountHistoryRow['kind'] {
  return row.amount < 0 || row.units < 0 ? 'expense' : 'revenue';
}

function buildUsageHistoryRows(
  projectId: string,
  bookedAmount: number,
  usageRecords: UsageRecord[],
): PortalAccountHistoryRow[] {
  const denominator = bookedAmount > 0
    ? bookedAmount
    : usageRecords.reduce((sum, row) => sum + Math.abs(row.amount), 0);

  return usageRecords.map((record, index) => ({
    id: `usage:${record.project_id}:${record.created_at_ms}:${record.api_key_hash}:${index}`,
    kind: 'expense',
    source: 'usage',
    scope: record.project_id === projectId ? 'current' : 'linked',
    project_id: record.project_id,
    units: Math.abs(record.units),
    amount: Math.abs(record.amount),
    occurred_at_ms: record.created_at_ms,
    share_of_booked_amount:
      denominator > 0 ? Math.min(1, Math.abs(record.amount) / denominator) : 0,
    model: record.model,
    provider: record.provider,
    channel_id: record.channel_id,
    api_key_hash: record.api_key_hash,
  }));
}

function buildLedgerHistoryRows(
  projectId: string,
  bookedAmount: number,
  ledger: LedgerEntry[],
): PortalAccountHistoryRow[] {
  const denominator = bookedAmount > 0
    ? bookedAmount
    : ledger.reduce((sum, row) => sum + Math.abs(row.amount), 0);

  return ledger.map((row, index) => ({
    id: `ledger:${row.project_id}:${row.units}:${row.amount}:${index}`,
    ...row,
    kind: classifyLedgerKind(row),
    source: 'ledger',
    scope: row.project_id === projectId ? 'current' : 'linked',
    units: Math.abs(row.units),
    amount: Math.abs(row.amount),
    occurred_at_ms: null,
    share_of_booked_amount:
      denominator > 0 ? Math.min(1, Math.abs(row.amount) / denominator) : 0,
  }));
}

function matchesHistoryQuery(row: PortalAccountHistoryRow, normalizedQuery: string): boolean {
  if (!normalizedQuery) {
    return true;
  }

  return [
    row.project_id,
    row.scope,
    row.kind,
    row.source,
    row.model,
    row.provider,
    row.channel_id,
    row.api_key_hash,
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase()
    .includes(normalizedQuery);
}

function compareHistoryRows(
  left: PortalAccountHistoryRow,
  right: PortalAccountHistoryRow,
): number {
  const leftTimestamp = left.occurred_at_ms ?? -1;
  const rightTimestamp = right.occurred_at_ms ?? -1;

  if (leftTimestamp !== rightTimestamp) {
    return rightTimestamp - leftTimestamp;
  }

  if (left.kind !== right.kind) {
    return left.kind === 'expense' ? -1 : 1;
  }

  if (left.scope !== right.scope) {
    return left.scope === 'current' ? -1 : 1;
  }

  if (left.share_of_booked_amount !== right.share_of_booked_amount) {
    return right.share_of_booked_amount - left.share_of_booked_amount;
  }

  if (left.amount !== right.amount) {
    return right.amount - left.amount;
  }

  return left.id.localeCompare(right.id);
}

function filterHistoryRows(
  rows: PortalAccountHistoryRow[],
  historyView: PortalAccountHistoryView,
  normalizedQuery: string,
): PortalAccountHistoryRow[] {
  return rows
    .filter(
      (row) =>
        matchesHistoryQuery(row, normalizedQuery)
        && (historyView === 'all' || row.kind === historyView),
    )
    .sort(compareHistoryRows);
}

export function buildPortalAccountViewModel(
  input: BuildPortalAccountViewModelInput,
): PortalAccountViewModel {
  const normalizedQuery = input.searchQuery.trim().toLowerCase();
  const historyView = input.historyView ?? 'all';
  const now = input.now ?? Date.now();
  const billingEventSummary = input.billingEventSummary ?? emptyBillingEventSummary();
  const usageSummary = summarizeUsageRecords(input.usageRecords);
  const totalRevenue =
    input.summary.booked_amount > 0 ? input.summary.booked_amount : usageSummary.revenue;
  const totalRequests = input.usageSummary?.total_requests ?? usageSummary.request_count;
  const totalUsedUnits =
    input.summary.used_units > 0 ? input.summary.used_units : usageSummary.used_units;
  const todayStart = startOfDayMs(now);
  const trailing7dStart = startOfTrailing7dMs(now);
  const monthStart = startOfMonthMs(now);

  const todayRecords = input.usageRecords.filter((record) => record.created_at_ms >= todayStart);
  const trailing7dRecords = input.usageRecords.filter(
    (record) => record.created_at_ms >= trailing7dStart,
  );
  const currentMonthRecords = input.usageRecords.filter(
    (record) => record.created_at_ms >= monthStart,
  );

  const revenueLedger = [...input.ledger].sort((left, right) =>
    compareLedgerRows(input.summary.project_id, left, right));
  const accountHistoryRows = [
    ...buildUsageHistoryRows(input.summary.project_id, totalRevenue, input.usageRecords),
    ...buildLedgerHistoryRows(input.summary.project_id, totalRevenue, revenueLedger),
  ];
  const allHistoryRows = filterHistoryRows(accountHistoryRows, 'all', normalizedQuery);
  const expenseHistoryRows = allHistoryRows.filter((row) => row.kind === 'expense');
  const revenueHistoryRows = allHistoryRows.filter((row) => row.kind === 'revenue');
  const activeHistoryRows =
    historyView === 'expense'
      ? expenseHistoryRows
      : historyView === 'revenue'
        ? revenueHistoryRows
        : allHistoryRows;
  const totalPages = Math.max(1, Math.ceil(activeHistoryRows.length / input.pageSize));
  const page = clampPage(input.page, totalPages);
  const startIndex = (page - 1) * input.pageSize;
  const visibleHistory = activeHistoryRows.slice(startIndex, startIndex + input.pageSize);

  return {
    billing_summary: input.summary,
    membership: input.membership,
    balance: buildBalanceSummary(input),
    totals: {
      revenue: totalRevenue,
      request_count: totalRequests,
      used_units: totalUsedUnits,
      average_booked_spend: totalRequests > 0 ? totalRevenue / totalRequests : 0,
    },
    today: summarizeUsageRecords(todayRecords),
    trailing_7d: summarizeUsageRecords(trailing7dRecords),
    current_month: summarizeUsageRecords(currentMonthRecords),
    financial_breakdown: buildFinancialBreakdown(billingEventSummary),
    history_view: historyView,
    history_counts: {
      all: allHistoryRows.length,
      expense: expenseHistoryRows.length,
      revenue: revenueHistoryRows.length,
    },
    visible_history: visibleHistory,
    pagination: {
      page,
      total_items: activeHistoryRows.length,
      total_pages: totalPages,
    },
  };
}
