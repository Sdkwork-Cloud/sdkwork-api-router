import {
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import {
  cn,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import { Badge } from 'sdkwork-router-portal-commons/framework/display';
import {
  Card,
  CardContent,
} from 'sdkwork-router-portal-commons/framework/layout';

import type {
  DashboardSpendTrendPoint,
  DashboardTrafficTrendPoint,
} from '../types';

type DashboardTrendSeriesKey = 'total_tokens' | 'input_tokens' | 'output_tokens';

interface DashboardTrendSeries {
  key: DashboardTrendSeriesKey;
  label: string;
  dotClassName: string;
  strokeClassName: string;
}

type DashboardMetricBreakdown = {
  label: string;
  value: string;
};

function buildLinePath(points: Array<{ x: number; y: number }>) {
  return points
    .map((point, index) => `${index === 0 ? 'M' : 'L'} ${point.x.toFixed(2)} ${point.y.toFixed(2)}`)
    .join(' ');
}

function buildAreaPath(points: Array<{ x: number; y: number }>, baseline: number) {
  if (!points.length) {
    return '';
  }

  const firstPoint = points[0];
  const lastPoint = points[points.length - 1];
  const linePath = buildLinePath(points);

  return `${linePath} L ${lastPoint.x.toFixed(2)} ${baseline.toFixed(2)} L ${firstPoint.x.toFixed(2)} ${baseline.toFixed(2)} Z`;
}

function useChartFrameWidth(minWidth: number) {
  const chartFrameRef = useRef<HTMLDivElement | null>(null);
  const [chartWidth, setChartWidth] = useState(0);

  useEffect(() => {
    const frame = chartFrameRef.current;
    if (!frame) {
      return;
    }

    const syncWidth = (nextWidth: number) => {
      const roundedWidth = Math.round(nextWidth);
      setChartWidth((currentWidth) => (
        currentWidth === roundedWidth ? currentWidth : roundedWidth
      ));
    };

    syncWidth(frame.clientWidth);

    if (typeof ResizeObserver === 'undefined') {
      const handleResize = () => syncWidth(frame.clientWidth);
      window.addEventListener('resize', handleResize);
      return () => window.removeEventListener('resize', handleResize);
    }

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry) {
        syncWidth(entry.contentRect.width);
      }
    });

    observer.observe(frame);

    return () => observer.disconnect();
  }, []);

  return {
    chartFrameRef,
    width: Math.max(chartWidth, minWidth),
  };
}

function getTrendSeriesValue(
  point: DashboardTrafficTrendPoint,
  key: DashboardTrendSeriesKey,
) {
  return point[key];
}

export function DashboardSummaryCard({
  eyebrow,
  title,
  description,
  accent,
  changeLabel,
  children,
}: {
  eyebrow: string;
  title: string;
  description: string;
  accent?: ReactNode;
  changeLabel?: string;
  children: ReactNode;
}) {
  return (
    <section className="relative overflow-hidden rounded-[1.9rem] border border-white/70 bg-white/82 p-5 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-white/6 dark:bg-zinc-900/82">
      <div className="pointer-events-none absolute right-5 top-2 h-24 w-24 rounded-full bg-primary-500/10 blur-3xl" />
      <div className="relative flex items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="text-[11px] font-semibold uppercase tracking-[0.24em] text-zinc-500 dark:text-zinc-400">
            {eyebrow}
          </p>
          <h3 className="mt-3 text-lg font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
            {title}
          </h3>
          <p className="mt-1 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
            {description}
          </p>
        </div>
        <div className="flex shrink-0 flex-col items-end gap-3">
          {changeLabel ? (
            <span className="inline-flex items-center rounded-full border border-primary-500/20 bg-primary-500/10 px-2.5 py-1 text-[11px] font-semibold text-primary-700 dark:text-primary-200">
              {changeLabel}
            </span>
          ) : null}
          {accent}
        </div>
      </div>

      <div className="relative mt-5">{children}</div>
    </section>
  );
}

function DashboardSnapshotItem({
  breakdowns,
  breakdownsSlot,
  label,
  value,
  valueSlot,
}: {
  breakdowns?: DashboardMetricBreakdown[];
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

export function DashboardMetricCard({
  breakdowns,
  description,
  label,
  value,
}: {
  breakdowns: DashboardMetricBreakdown[];
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

export function DashboardBalanceCard({
  balanceValue,
  description,
  onRecharge,
  onRedeem,
  planValue,
  quotaLimitValue,
  statusLabel,
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
  usedBreakdowns: DashboardMetricBreakdown[];
  usedUnitsValue: string;
  utilizationPercent: number | null;
}) {
  const { t } = usePortalI18n();

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
                data-slot="portal-dashboard-balance-primary"
              >
                <strong className="block text-4xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                  {balanceValue}
                </strong>
                <div
                  className="flex flex-wrap gap-2"
                  data-slot="portal-dashboard-balance-actions"
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
            <DashboardSnapshotItem
              breakdowns={usedBreakdowns}
              breakdownsSlot="portal-dashboard-used-breakdowns"
              label={t('Used units')}
              value={usedUnitsValue}
              valueSlot="portal-dashboard-used-total"
            />
            <DashboardSnapshotItem label={t('Quota limit')} value={quotaLimitValue} />
            <DashboardSnapshotItem label={t('Plan')} value={planValue} />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export function DashboardRevenueTrendChart({
  points = [],
  title,
  summaryLabel,
  summaryValue,
  peakLabel,
  yAxisFormatter = (value) => `${value}`,
}: {
  points?: DashboardSpendTrendPoint[];
  title: string;
  summaryLabel: string;
  summaryValue: string;
  peakLabel: string;
  yAxisFormatter?: (value: number) => string;
}) {
  const { t } = usePortalI18n();
  const { chartFrameRef, width } = useChartFrameWidth(720);
  const hasRenderableData = points.length > 0;
  const height = 352;
  const paddingTop = 18;
  const paddingBottom = 38;
  const chartPaddingX = 16;
  const yAxisLabelWidth = 42;
  const plotLeft = chartPaddingX + yAxisLabelWidth;
  const plotRight = width - chartPaddingX;
  const usableWidth = plotRight - plotLeft;
  const usableHeight = height - paddingTop - paddingBottom;
  const maxValue = hasRenderableData
    ? Math.max(...points.map((point) => point.amount), 1)
    : 1;
  const xAxisStep = usableWidth / Math.max(points.length - 1, 1);
  const coordinates = hasRenderableData
    ? points.map((point, index) => ({
        point,
        x: plotLeft + xAxisStep * index,
        y: paddingTop + usableHeight - (Math.max(point.amount, 0) / maxValue) * usableHeight,
      }))
    : [];
  const yAxisTicks = Array.from({ length: 5 }, (_, index) => {
    const ratio = 1 - index / 4;
    return Math.round(maxValue * ratio);
  });
  const labelStep = Math.max(1, Math.ceil(points.length / 8));
  const xAxisIndices = hasRenderableData
    ? Array.from(
        new Set(
          points
            .map((_, index) => index)
            .filter(
              (index) => index === 0 || index === points.length - 1 || index % labelStep === 0,
            ),
        ),
      )
    : [];
  const linePath = buildLinePath(coordinates);
  const areaPath = buildAreaPath(coordinates, height - paddingBottom);
  const lastPoint = coordinates[coordinates.length - 1];
  const peakPoint = hasRenderableData
    ? points.reduce((currentPeak, point) => (
        point.amount > currentPeak.amount ? point : currentPeak
      ), points[0]!)
    : null;

  return (
    <div className="overflow-hidden rounded-[1.6rem] border border-white/70 bg-white/65 shadow-[inset_0_1px_0_rgba(255,255,255,0.65)] dark:border-white/6 dark:bg-zinc-950/35">
      <div className="mx-4 mb-4 mt-4 flex flex-wrap items-center justify-between gap-3 rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/85 px-4 py-3 dark:border-white/6 dark:bg-white/[0.04]">
        <div className="flex items-center gap-3">
          <span className="h-2.5 w-2.5 rounded-full bg-emerald-500" />
          <div>
            <div className="text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
              {title}
            </div>
            <div className="mt-1 text-sm font-medium text-zinc-700 dark:text-zinc-200">
              {summaryLabel}: {summaryValue}
            </div>
          </div>
        </div>
        {peakPoint ? (
          <div className="rounded-full border border-emerald-500/20 bg-emerald-500/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-emerald-700 dark:text-emerald-200">
            {peakLabel} {peakPoint.label}
          </div>
        ) : null}
      </div>

      {hasRenderableData ? (
        <div ref={chartFrameRef} className="w-full">
          <svg
            viewBox={`0 0 ${width} ${height}`}
            className="h-[22rem] w-full"
            role="img"
            aria-label={title}
          >
            {yAxisTicks.map((value, index) => {
              const y = paddingTop + (usableHeight / Math.max(yAxisTicks.length - 1, 1)) * index;

              return (
                <g key={`${value}-${index}`}>
                  <line
                    x1={plotLeft}
                    y1={y}
                    x2={plotRight}
                    y2={y}
                    className="stroke-zinc-200/90 dark:stroke-zinc-800/85"
                    strokeDasharray="4 8"
                  />
                  <text
                    x={plotLeft - 12}
                    y={y + 4}
                    textAnchor="end"
                    className="fill-zinc-400 text-[11px] font-medium dark:fill-zinc-500"
                  >
                    {yAxisFormatter(value)}
                  </text>
                </g>
              );
            })}

            {xAxisIndices.map((index) => {
              const point = points[index]!;
              const x = plotLeft + xAxisStep * index;

              return (
                <g key={point.bucket_key}>
                  <line
                    x1={x}
                    y1={paddingTop}
                    x2={x}
                    y2={height - paddingBottom}
                    className="stroke-zinc-100 dark:stroke-zinc-900"
                  />
                  <text
                    x={x}
                    y={height - 8}
                    textAnchor="middle"
                    className="fill-zinc-400 text-[11px] font-medium dark:fill-zinc-500"
                  >
                    {point.label}
                  </text>
                </g>
              );
            })}

            <g className="text-emerald-500">
              <path d={areaPath} fill="currentColor" className="opacity-12" />
              <path
                d={linePath}
                fill="none"
                stroke="currentColor"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={3.2}
              />
            </g>

            {coordinates.map((coordinate, index) => (
              <circle
                key={`${coordinate.point.bucket_key}-${index}`}
                cx={coordinate.x}
                cy={coordinate.y}
                r={index === coordinates.length - 1 ? 5 : 3.5}
                fill="rgb(16 185 129)"
                className={index === coordinates.length - 1 ? 'drop-shadow-[0_0_10px_rgba(16,185,129,0.32)]' : ''}
              />
            ))}

            {lastPoint ? (
              <text
                x={Math.min(lastPoint.x + 10, width - 90)}
                y={Math.max(lastPoint.y - 12, paddingTop + 12)}
                className="fill-emerald-600 text-[12px] font-semibold dark:fill-emerald-300"
              >
                {yAxisFormatter(lastPoint.point.amount)}
              </text>
            ) : null}
          </svg>
        </div>
      ) : (
        <div className="flex h-[22rem] items-center justify-center text-sm text-zinc-500 dark:text-zinc-400">
          {t('No spend trend data yet')}
        </div>
      )}
    </div>
  );
}

export function DashboardTokenTrendChart({
  points = [],
  series = [],
  title,
  summary,
  yAxisFormatter = (value) => `${value}`,
}: {
  points?: DashboardTrafficTrendPoint[];
  series?: DashboardTrendSeries[];
  title: string;
  summary: string;
  yAxisFormatter?: (value: number) => string;
}) {
  const { t } = usePortalI18n();
  const { chartFrameRef, width } = useChartFrameWidth(720);
  const hasRenderableData = points.length > 0 && series.length > 0;
  const height = 352;
  const paddingTop = 18;
  const paddingBottom = 38;
  const chartPaddingX = 16;
  const yAxisLabelWidth = 36;
  const plotLeft = chartPaddingX + yAxisLabelWidth;
  const plotRight = width - chartPaddingX;
  const usableWidth = plotRight - plotLeft;
  const usableHeight = height - paddingTop - paddingBottom;
  const maxValue = hasRenderableData
    ? Math.max(...series.flatMap((item) => points.map((point) => getTrendSeriesValue(point, item.key))), 1)
    : 1;
  const xAxisStep = usableWidth / Math.max(points.length - 1, 1);
  const yForValue = (value: number) => (
    paddingTop + usableHeight - (Math.max(value, 0) / maxValue) * usableHeight
  );
  const coordinatesBySeries = (hasRenderableData
    ? Object.fromEntries(
        series.map((item) => [
          item.key,
          points.map((point, index) => ({
            point,
            x: plotLeft + xAxisStep * index,
            y: yForValue(getTrendSeriesValue(point, item.key)),
          })),
        ]),
      )
    : {}) as Record<
      DashboardTrendSeriesKey,
      Array<{ point: DashboardTrafficTrendPoint; x: number; y: number }>
    >;
  const yAxisTicks = Array.from({ length: 5 }, (_, index) => {
    const ratio = 1 - index / 4;
    return Math.round(maxValue * ratio);
  });
  const labelStep = Math.max(1, Math.ceil(points.length / 8));
  const xAxisIndices = hasRenderableData
    ? Array.from(
        new Set(
          points
            .map((_, index) => index)
            .filter(
              (index) => index === 0 || index === points.length - 1 || index % labelStep === 0,
            ),
        ),
      )
    : [];
  const totalSeriesPoints = coordinatesBySeries.total_tokens ?? [];
  const totalAreaPath = buildAreaPath(totalSeriesPoints, height - paddingBottom);

  return (
    <div className="overflow-hidden rounded-[1.6rem] border border-white/70 bg-white/65 shadow-[inset_0_1px_0_rgba(255,255,255,0.65)] dark:border-white/6 dark:bg-zinc-950/35">
      <div className="mx-4 mb-5 mt-4 rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/85 p-4 dark:border-white/6 dark:bg-white/[0.04]">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
          <div className="min-w-0 flex-1">
            <div className="text-[11px] font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
              {title}
            </div>
            <div className="mt-1 truncate text-sm font-medium text-zinc-700 dark:text-zinc-200">
              {summary}
            </div>
          </div>

          <div className="flex flex-wrap gap-2 xl:shrink-0">
            {series.map((item) => (
              <span
                key={item.key}
                className="inline-flex items-center gap-2 rounded-full border border-zinc-200/80 bg-white/90 px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-white/8 dark:bg-zinc-950/50 dark:text-zinc-300"
              >
                <span className={cn('h-2.5 w-2.5 rounded-full', item.dotClassName)} />
                {item.label}
              </span>
            ))}
          </div>
        </div>
      </div>

      {hasRenderableData ? (
        <div ref={chartFrameRef} className="w-full">
          <svg
            viewBox={`0 0 ${width} ${height}`}
            className="h-[22rem] w-full"
            role="img"
            aria-label={title}
          >
            {yAxisTicks.map((value, index) => {
              const y = paddingTop + (usableHeight / Math.max(yAxisTicks.length - 1, 1)) * index;

              return (
                <g key={`${value}-${index}`}>
                  <line
                    x1={plotLeft}
                    y1={y}
                    x2={plotRight}
                    y2={y}
                    className="stroke-zinc-200/90 dark:stroke-zinc-800/85"
                    strokeDasharray="4 8"
                  />
                  <text
                    x={plotLeft - 12}
                    y={y + 4}
                    textAnchor="end"
                    className="fill-zinc-400 text-[11px] font-medium dark:fill-zinc-500"
                  >
                    {yAxisFormatter(value)}
                  </text>
                </g>
              );
            })}

            {xAxisIndices.map((index) => {
              const point = points[index]!;
              const x = plotLeft + xAxisStep * index;

              return (
                <g key={point.bucket_key}>
                  <line
                    x1={x}
                    y1={paddingTop}
                    x2={x}
                    y2={height - paddingBottom}
                    className="stroke-zinc-100 dark:stroke-zinc-900"
                  />
                  <text
                    x={x}
                    y={height - 8}
                    textAnchor="middle"
                    className="fill-zinc-400 text-[11px] font-medium dark:fill-zinc-500"
                  >
                    {point.label}
                  </text>
                </g>
              );
            })}

            <g className="text-primary-500">
              <path d={totalAreaPath} fill="currentColor" className="opacity-10" />
            </g>

            {series.map((item) => {
              const coordinates = coordinatesBySeries[item.key] ?? [];
              const linePath = buildLinePath(coordinates);
              const lastPoint = coordinates[coordinates.length - 1];

              if (!lastPoint) {
                return null;
              }

              return (
                <g key={item.key} className={item.strokeClassName}>
                  <path
                    d={linePath}
                    fill="none"
                    stroke="currentColor"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={item.key === 'total_tokens' ? 3.2 : 2.2}
                  />
                  <circle
                    cx={lastPoint.x}
                    cy={lastPoint.y}
                    r={item.key === 'total_tokens' ? 5 : 4}
                    fill="currentColor"
                    className="drop-shadow-[0_0_10px_rgba(15,23,42,0.14)]"
                  />
                  <circle
                    cx={lastPoint.x}
                    cy={lastPoint.y}
                    r={item.key === 'total_tokens' ? 2.4 : 1.6}
                    className="fill-white dark:fill-zinc-950"
                  />
                </g>
              );
            })}
          </svg>
        </div>
      ) : (
        <div className="flex h-[22rem] items-center justify-center rounded-[1.5rem] border border-dashed border-zinc-300/80 bg-white/60 text-sm text-zinc-500 dark:border-zinc-700/70 dark:bg-zinc-950/35 dark:text-zinc-400">
          {t('No traffic trend data yet')}
        </div>
      )}
    </div>
  );
}

export function DashboardDistributionRingChart<T extends { id: string }>({
  rows,
  sliceClassNames,
  centerLabel,
  centerValue,
  ariaLabel,
  valueAccessor,
}: {
  rows: T[];
  sliceClassNames: string[];
  centerLabel: string;
  centerValue: string;
  ariaLabel: string;
  valueAccessor: (row: T) => number;
}) {
  const radius = 74;
  const circumference = 2 * Math.PI * radius;
  const strokeWidth = 26;
  const total = rows.reduce((sum, row) => sum + valueAccessor(row), 0);
  let cumulativeOffset = 0;

  return (
    <div className="rounded-[1.6rem] border border-white/70 bg-white/65 p-5 shadow-[inset_0_1px_0_rgba(255,255,255,0.65)] dark:border-white/6 dark:bg-zinc-950/35">
      <div className="flex items-center justify-center">
        <svg viewBox="0 0 220 220" className="h-[15rem] w-[15rem]" role="img" aria-label={ariaLabel}>
          <circle
            cx="110"
            cy="110"
            r={radius}
            fill="none"
            className="stroke-zinc-100 dark:stroke-zinc-900"
            strokeWidth={strokeWidth}
          />
          <g transform="rotate(-90 110 110)">
            {rows.map((row, index) => {
              const value = valueAccessor(row);
              const ratio = total === 0 ? 0 : value / total;
              const dashLength = Math.max(ratio * circumference - 3, 0);
              const dashOffset = -cumulativeOffset;

              cumulativeOffset += ratio * circumference;

              return (
                <g key={row.id} className={sliceClassNames[index % sliceClassNames.length]}>
                  <circle
                    cx="110"
                    cy="110"
                    r={radius}
                    fill="none"
                    stroke="currentColor"
                    strokeLinecap="round"
                    strokeWidth={strokeWidth}
                    strokeDasharray={`${dashLength} ${circumference}`}
                    strokeDashoffset={dashOffset}
                  />
                </g>
              );
            })}
          </g>
          <circle cx="110" cy="110" r="54" className="fill-white dark:fill-zinc-950" />
          <text
            x="110"
            y="98"
            textAnchor="middle"
            className="fill-zinc-400 text-[12px] font-semibold uppercase tracking-[0.26em] dark:fill-zinc-500"
          >
            {centerLabel}
          </text>
          <text
            x="110"
            y="122"
            textAnchor="middle"
            className="fill-zinc-950 text-[24px] font-semibold tracking-tight dark:fill-zinc-50"
          >
            {centerValue}
          </text>
        </svg>
      </div>
    </div>
  );
}

export function DashboardModelDistributionChart<T extends { id: string }>({
  rows,
  sliceClassNames,
  centerLabel,
  centerValue,
  ariaLabel,
  valueAccessor,
}: {
  rows: T[];
  sliceClassNames: string[];
  centerLabel: string;
  centerValue: string;
  ariaLabel: string;
  valueAccessor: (row: T) => number;
}) {
  return (
    <DashboardDistributionRingChart
      rows={rows}
      sliceClassNames={sliceClassNames}
      centerLabel={centerLabel}
      centerValue={centerValue}
      ariaLabel={ariaLabel}
      valueAccessor={valueAccessor}
    />
  );
}

export { StatusBadge } from 'sdkwork-router-portal-commons/framework/display';


