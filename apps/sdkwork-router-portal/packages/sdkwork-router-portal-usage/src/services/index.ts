import { formatCurrency, formatDateTime, formatUnits } from 'sdkwork-router-portal-commons';
import type { UsageRecord, UsageSummary } from 'sdkwork-router-portal-types';

import type {
  UsageDateRange,
  UsageDiagnostic,
  UsageFilters,
  UsageHighlight,
  UsageProfileItem,
  UsageWorkbenchViewModel,
} from '../types';

function sortedUnique(values: string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function dateRangeCutoff(dateRange: UsageDateRange): number | null {
  const now = Date.now();

  switch (dateRange) {
    case '24h':
      return now - 24 * 60 * 60 * 1000;
    case '7d':
      return now - 7 * 24 * 60 * 60 * 1000;
    case '30d':
      return now - 30 * 24 * 60 * 60 * 1000;
    case 'all':
      return null;
  }
}

function filteredUsageRecords(records: UsageRecord[], filters: UsageFilters): UsageRecord[] {
  const cutoff = dateRangeCutoff(filters.date_range);

  return records.filter((record) => {
    if (filters.model && record.model !== filters.model) {
      return false;
    }
    if (filters.provider && record.provider !== filters.provider) {
      return false;
    }
    if (cutoff !== null && record.created_at_ms < cutoff) {
      return false;
    }
    return true;
  });
}

function buildUsageHighlights(records: UsageRecord[]): UsageHighlight[] {
  const totalUnits = records.reduce((sum, record) => sum + record.units, 0);
  const totalAmount = records.reduce((sum, record) => sum + record.amount, 0);
  const heaviestRequest = [...records].sort((left, right) => right.units - left.units)[0];
  const averageUnits = records.length ? Math.round(totalUnits / records.length) : 0;

  return [
    {
      id: 'filtered-requests',
      label: 'Filtered requests',
      value: formatUnits(records.length),
      detail: 'Requests visible after the current model/provider filters are applied.',
    },
    {
      id: 'avg-units',
      label: 'Average token units',
      value: formatUnits(averageUnits),
      detail: 'Average token-unit volume per visible request.',
    },
    {
      id: 'peak-request',
      label: 'Peak request',
      value: heaviestRequest ? formatUnits(heaviestRequest.units) : '0',
      detail: heaviestRequest
        ? `${heaviestRequest.model} via ${heaviestRequest.provider}`
        : 'A heavy request spotlight will appear after traffic starts.',
    },
    {
      id: 'booked',
      label: 'Filtered booked amount',
      value: formatCurrency(totalAmount),
      detail: 'Booked amount associated with the currently visible request slice.',
    },
  ];
}

function buildTrafficProfile(records: UsageRecord[]): UsageProfileItem[] {
  const latestRequest = [...records].sort((left, right) => right.created_at_ms - left.created_at_ms)[0];
  const providerCounts = new Map<string, number>();
  const modelCounts = new Map<string, number>();

  for (const record of records) {
    providerCounts.set(record.provider, (providerCounts.get(record.provider) ?? 0) + 1);
    modelCounts.set(record.model, (modelCounts.get(record.model) ?? 0) + 1);
  }

  const primaryProvider = [...providerCounts.entries()].sort((left, right) => right[1] - left[1])[0];
  const primaryModel = [...modelCounts.entries()].sort((left, right) => right[1] - left[1])[0];

  return [
    {
      id: 'primary-provider',
      label: 'Primary provider',
      value: primaryProvider ? primaryProvider[0] : 'Waiting for traffic',
      detail: primaryProvider
        ? `${formatUnits(primaryProvider[1])} visible request(s) are currently routed through this provider path.`
        : 'Provider preference will appear after the first request lands.',
    },
    {
      id: 'primary-model',
      label: 'Primary model',
      value: primaryModel ? primaryModel[0] : 'Waiting for traffic',
      detail: primaryModel
        ? `${formatUnits(primaryModel[1])} visible request(s) currently target this model.`
        : 'Model concentration will appear once requests are recorded.',
    },
    {
      id: 'latest-request',
      label: 'Latest request',
      value: latestRequest ? formatDateTime(latestRequest.created_at_ms) : 'Pending',
      detail: latestRequest
        ? `${latestRequest.model} via ${latestRequest.provider}.`
        : 'The latest-call marker will appear once request telemetry is present.',
    },
  ];
}

function buildSpendWatch(records: UsageRecord[]): UsageProfileItem[] {
  const totalUnits = records.reduce((sum, record) => sum + record.units, 0);
  const totalAmount = records.reduce((sum, record) => sum + record.amount, 0);
  const heaviestRequest = [...records].sort((left, right) => right.units - left.units)[0];
  const averageAmount = records.length ? totalAmount / records.length : 0;
  const dailyBurn = records.length ? Math.round(totalUnits / Math.max(1, records.length / 3)) : 0;

  return [
    {
      id: 'booked-amount',
      label: 'Visible booked amount',
      value: formatCurrency(totalAmount),
      detail: 'Booked amount associated with the currently filtered request slice.',
    },
    {
      id: 'avg-booked',
      label: 'Average booked per request',
      value: formatCurrency(averageAmount),
      detail: 'A quick proxy for how expensive each visible request currently is.',
    },
    {
      id: 'daily-burn',
      label: 'Observed burn pace',
      value: formatUnits(dailyBurn),
      detail: 'A rough request-slice burn estimate derived from visible token-unit usage.',
    },
    {
      id: 'peak-request',
      label: 'Largest request cost',
      value: heaviestRequest ? formatCurrency(heaviestRequest.amount) : '$0.00',
      detail: heaviestRequest
        ? `${formatUnits(heaviestRequest.units)} token units in the heaviest visible request.`
        : 'Peak cost will appear once traffic is recorded.',
    },
  ];
}

function buildDiagnostics(records: UsageRecord[]): UsageDiagnostic[] {
  if (!records.length) {
    return [
      {
        id: 'first-request',
        title: 'Send the first request to unlock diagnostics',
        detail: 'Traffic profile, spend watch, and anomaly signals all depend on the first visible request slice.',
        tone: 'accent',
      },
    ];
  }

  const diagnostics: UsageDiagnostic[] = [];
  const totalUnits = records.reduce((sum, record) => sum + record.units, 0);
  const averageUnits = totalUnits / records.length;
  const byProvider = new Map<string, number>();
  const byModel = new Map<string, number>();

  for (const record of records) {
    byProvider.set(record.provider, (byProvider.get(record.provider) ?? 0) + 1);
    byModel.set(record.model, (byModel.get(record.model) ?? 0) + 1);
  }

  const dominantProvider = [...byProvider.entries()].sort((left, right) => right[1] - left[1])[0];
  const dominantModel = [...byModel.entries()].sort((left, right) => right[1] - left[1])[0];
  const peakRequest = [...records].sort((left, right) => right.units - left.units)[0];

  if (dominantProvider && dominantProvider[1] / records.length >= 0.8) {
    diagnostics.push({
      id: 'provider-concentration',
      title: 'Provider concentration is high',
      detail: `${dominantProvider[0]} currently carries ${formatUnits(dominantProvider[1])} of ${formatUnits(records.length)} visible requests. Consider whether this is intentional before traffic scales.`,
      tone: 'warning',
    });
  }

  if (dominantModel && dominantModel[1] / records.length >= 0.8) {
    diagnostics.push({
      id: 'model-concentration',
      title: 'Model mix is narrowly concentrated',
      detail: `${dominantModel[0]} dominates the visible request slice. Keep an eye on reliability and cost if this becomes the only production path.`,
      tone: 'accent',
    });
  }

  if (peakRequest && peakRequest.units > averageUnits * 3) {
    diagnostics.push({
      id: 'token-spike',
      title: 'Token spikes are visible in the request mix',
      detail: `The heaviest visible request used ${formatUnits(peakRequest.units)} token units, more than 3x the current average. Investigate prompt or payload size before launch.`,
      tone: 'warning',
    });
  }

  if (!diagnostics.length) {
    diagnostics.push({
      id: 'healthy-slice',
      title: 'Current request slice looks stable',
      detail: 'No obvious provider concentration or token-spike signal stands out in the current filtered traffic view.',
      tone: 'positive',
    });
  }

  return diagnostics;
}

export function buildUsageWorkbenchViewModel(
  summary: UsageSummary,
  records: UsageRecord[],
  filters: UsageFilters,
): UsageWorkbenchViewModel {
  const filtered_records = filteredUsageRecords(records, filters);

  return {
    summary,
    filtered_records,
    total_units: filtered_records.reduce((sum, record) => sum + record.units, 0),
    total_amount: filtered_records.reduce((sum, record) => sum + record.amount, 0),
    model_options: sortedUnique(records.map((record) => record.model)),
    provider_options: sortedUnique(records.map((record) => record.provider)),
    highlights: buildUsageHighlights(filtered_records),
    traffic_profile: buildTrafficProfile(filtered_records),
    spend_watch: buildSpendWatch(filtered_records),
    diagnostics: buildDiagnostics(filtered_records),
  };
}
