import { MetricCard } from 'sdkwork-router-portal-commons';

import type { UsageDateRange, UsageFilters, UsageHighlight } from '../types';

const usageDateRanges: Array<{ label: string; value: UsageDateRange }> = [
  { label: '24h', value: '24h' },
  { label: '7d', value: '7d' },
  { label: '30d', value: '30d' },
  { label: 'All time', value: 'all' },
];

export function UsageFiltersPanel({
  filters,
  modelOptions,
  providerOptions,
  onChange,
}: {
  filters: UsageFilters;
  modelOptions: string[];
  providerOptions: string[];
  onChange: (nextFilters: UsageFilters) => void;
}) {
  return (
    <div className="portalx-filter-bar">
      <label className="portalx-field">
        <span>Model filter</span>
        <select
          onChange={(event) => onChange({ ...filters, model: event.target.value })}
          value={filters.model}
        >
          <option value="">All models</option>
          {modelOptions.map((model) => (
            <option key={model} value={model}>
              {model}
            </option>
          ))}
        </select>
      </label>
      <label className="portalx-field">
        <span>Provider filter</span>
        <select
          onChange={(event) => onChange({ ...filters, provider: event.target.value })}
          value={filters.provider}
        >
          <option value="">All providers</option>
          {providerOptions.map((provider) => (
            <option key={provider} value={provider}>
              {provider}
            </option>
          ))}
        </select>
      </label>
      <label className="portalx-field">
        <span>Time range</span>
        <select
          onChange={(event) =>
            onChange({
              ...filters,
              date_range: event.target.value as UsageDateRange,
            })}
          value={filters.date_range}
        >
          {usageDateRanges.map((range) => (
            <option key={range.value} value={range.value}>
              {range.label}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
}

export function UsageHighlights({
  highlights,
}: {
  highlights: UsageHighlight[];
}) {
  return (
    <div className="portalx-summary-grid">
      {highlights.map((highlight) => (
        <MetricCard
          detail={highlight.detail}
          key={highlight.id}
          label={highlight.label}
          value={highlight.value}
        />
      ))}
    </div>
  );
}
