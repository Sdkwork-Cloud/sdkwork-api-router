import type { UsageRecord } from 'sdkwork-router-admin-types';
import {
  formatAdminCurrency,
  formatAdminDateTime,
  formatAdminNumber,
  translateAdminText,
} from 'sdkwork-router-admin-core';

export type TimeRangePreset = 'all' | '24h' | '7d' | '30d';

export const PAGE_SIZE = 20;

export function formatNumber(value: number): string {
  return formatAdminNumber(value);
}

export function formatCurrency(value: number): string {
  return formatAdminCurrency(value, 4);
}

export function formatDateTime(value: number): string {
  return formatAdminDateTime(value);
}

function csvValue(value: string | number | boolean | null | undefined): string {
  const normalized = value == null ? '' : String(value);
  return `"${normalized.replaceAll('"', '""')}"`;
}

export function downloadCsv(
  filename: string,
  headers: string[],
  rows: Array<Array<string | number | boolean | null | undefined>>,
): void {
  const contents = [
    headers.map(csvValue).join(','),
    ...rows.map((row) => row.map(csvValue).join(',')),
  ].join('\n');
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

export function recentWindowCutoff(window: TimeRangePreset): number | null {
  const now = Date.now();

  switch (window) {
    case '24h':
      return now - 24 * 60 * 60 * 1000;
    case '7d':
      return now - 7 * 24 * 60 * 60 * 1000;
    case '30d':
      return now - 30 * 24 * 60 * 60 * 1000;
    case 'all':
    default:
      return null;
  }
}

export function buildUsageRecordKey(record: UsageRecord, index: number): string {
  return `${record.project_id}:${record.model}:${record.provider}:${record.created_at_ms}:${index}`;
}

export function compareUsageRecords(
  left: UsageRecord,
  right: UsageRecord,
): number {
  return (
    right.created_at_ms - left.created_at_ms
    || left.project_id.localeCompare(right.project_id)
    || left.provider.localeCompare(right.provider)
    || left.model.localeCompare(right.model)
  );
}

export function formatTimeRangeLabel(value: TimeRangePreset): string {
  switch (value) {
    case 'all':
      return translateAdminText('All time');
    case '24h':
      return translateAdminText('Last 24 hours');
    case '7d':
      return translateAdminText('Last 7 days');
    case '30d':
    default:
      return translateAdminText('Last 30 days');
  }
}
