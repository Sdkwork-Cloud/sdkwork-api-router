import { useEffect, useState } from 'react';

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
} from 'sdkwork-router-portal-commons/framework/display';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import {
  Card,
  CardContent,
} from 'sdkwork-router-portal-commons/framework/layout';
import { WorkspacePanel } from 'sdkwork-router-portal-commons/framework/workspace';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import { loadPortalSettlementsWorkspace } from '../repository';
import { buildPortalSettlementsViewModel } from '../services';
import type {
  PortalSettlementsPageProps,
  PortalSettlementsViewModel,
  PortalSettlementsWorkspaceData,
} from '../types';

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;

function formatStatusLabel(value: string): string {
  return value
    .split(/[-_\s]+/g)
    .filter(Boolean)
    .map((segment) =>
      segment.length > 1
        ? `${segment.slice(0, 1).toUpperCase()}${segment.slice(1)}`
        : segment.toUpperCase())
    .join(' ');
}

function settlementStatusVariant(
  status: string,
): 'success' | 'warning' | 'secondary' {
  switch (status) {
    case 'captured':
      return 'success';
    case 'failed':
    case 'refunded':
      return 'warning';
    default:
      return 'secondary';
  }
}

function holdStatusVariant(status: string): 'success' | 'warning' | 'secondary' {
  switch (status) {
    case 'captured':
      return 'success';
    case 'held':
    case 'partially_released':
      return 'warning';
    default:
      return 'secondary';
  }
}

function commercialPricingChargeUnitLabel(
  chargeUnit: PortalSettlementsViewModel['primary_rate_charge_unit'],
  t: TranslateFn,
): string {
  switch (chargeUnit) {
    case 'input_token':
      return t('Input token');
    case 'output_token':
      return t('Output token');
    case 'cache_read_token':
      return t('Cache read token');
    case 'cache_write_token':
      return t('Cache write token');
    case 'request':
      return t('Request');
    case 'image':
      return t('Image');
    case 'audio_second':
      return t('Audio second');
    case 'audio_minute':
      return t('Audio minute');
    case 'video_second':
      return t('Video second');
    case 'video_minute':
      return t('Video minute');
    case 'music_track':
      return t('Music track');
    case 'character':
      return t('Character');
    case 'storage_mb_day':
      return t('Storage MB day');
    case 'tool_call':
      return t('Tool call');
    case 'unit':
      return t('Unit');
    default:
      return t('n/a');
  }
}

function commercialPricingMethodLabel(
  pricingMethod: PortalSettlementsViewModel['primary_rate_pricing_method'],
  t: TranslateFn,
): string {
  switch (pricingMethod) {
    case 'per_unit':
      return t('Per unit');
    case 'flat':
      return t('Flat');
    case 'step':
      return t('Step');
    case 'included_then_per_unit':
      return t('Included then per unit');
    default:
      return t('n/a');
  }
}

function commercialPricingDisplayUnit(
  displayPriceUnit: PortalSettlementsViewModel['primary_rate_display_price_unit'],
  chargeUnit: PortalSettlementsViewModel['primary_rate_charge_unit'],
  t: TranslateFn,
): string {
  if (displayPriceUnit?.trim()) {
    return displayPriceUnit;
  }

  switch (chargeUnit) {
    case 'input_token':
      return t('USD / 1M input tokens');
    case 'image':
      return t('USD / image');
    case 'music_track':
      return t('USD / music track');
    default:
      return t('n/a');
  }
}

export function PortalSettlementsPage({ onNavigate }: PortalSettlementsPageProps) {
  const { t } = usePortalI18n();
  const [workspaceData, setWorkspaceData] = useState<PortalSettlementsWorkspaceData | null>(null);
  const [status, setStatus] = useState(t('Loading settlement explorer...'));

  useEffect(() => {
    let cancelled = false;

    void loadPortalSettlementsWorkspace()
      .then((data) => {
        if (cancelled) {
          return;
        }

        setWorkspaceData(data);
        setStatus(
          t(
            'Settlement explorer now keeps account balance, credit holds, request settlements, and pricing evidence on one workspace surface.',
          ),
        );
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [t]);

  const viewModel: PortalSettlementsViewModel | null = workspaceData
    ? buildPortalSettlementsViewModel(workspaceData)
    : null;

  if (!viewModel) {
    return (
      <Card
        className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
        data-slot="portal-settlements-page"
      >
        <CardContent className="p-8">
          <EmptyState
            description={t(
              'Settlement explorer will appear after the portal loads canonical balance, hold, settlement, and pricing evidence.',
            )}
            title={status || t('Preparing settlement explorer')}
          />
        </CardContent>
      </Card>
    );
  }

  const summaryCards = [
    {
      label: t('Request settlements'),
      value: formatUnits(viewModel.settlement_count),
      detail: t('All canonical request settlements currently visible for this workspace.'),
    },
    {
      label: t('Credit holds'),
      value: formatUnits(viewModel.open_hold_count),
      detail: t('Open hold posture keeps reserved credit visible before capture or release.'),
    },
    {
      label: t('Available balance'),
      value: formatUnits(viewModel.available_balance),
      detail: t('Spendable credit that remains after current hold reservations.'),
    },
    {
      label: t('Pricing evidence'),
      value: formatUnits(viewModel.priced_metric_count),
      detail: t('Distinct priced metrics currently backing settlement and balance decisions.'),
    },
  ];

  const balancePosture = [
    {
      label: t('Account'),
      value: viewModel.account_id === null ? t('n/a') : String(viewModel.account_id),
    },
    {
      label: t('Status'),
      value: viewModel.account_status ? formatStatusLabel(viewModel.account_status) : t('n/a'),
    },
    {
      label: t('Held balance'),
      value: formatUnits(viewModel.held_balance),
    },
    {
      label: t('Grant balance'),
      value: formatUnits(viewModel.grant_balance),
    },
  ];

  const pricingEvidence = [
    {
      label: t('Primary plan'),
      value: viewModel.primary_plan_display_name ?? t('No active plan'),
    },
    {
      label: t('Primary metric'),
      value: viewModel.primary_rate_metric_code ?? t('No priced metric'),
    },
    {
      label: t('Charge unit'),
      value: commercialPricingChargeUnitLabel(viewModel.primary_rate_charge_unit, t),
    },
    {
      label: t('Billing method'),
      value: commercialPricingMethodLabel(viewModel.primary_rate_pricing_method, t),
    },
    {
      label: t('Price unit'),
      value: commercialPricingDisplayUnit(
        viewModel.primary_rate_display_price_unit,
        viewModel.primary_rate_charge_unit,
        t,
      ),
    },
    {
      label: t('Priced metrics'),
      value: formatUnits(viewModel.priced_metric_count),
    },
    {
      label: t('Active lots'),
      value: formatUnits(viewModel.active_benefit_lot_count),
    },
  ];

  return (
    <div className="space-y-4" data-slot="portal-settlements-page">
      <div
        data-slot="portal-settlements-toolbar"
        className="flex flex-wrap items-start justify-between gap-3 rounded-[24px] border border-zinc-200/80 bg-zinc-50/85 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/45"
      >
        <p className="min-w-[16rem] flex-1 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
          {status}
        </p>
        <div className="flex flex-wrap items-center gap-2">
          <Button onClick={() => onNavigate('account')} variant="secondary">
            {t('Open account')}
          </Button>
          <Button onClick={() => onNavigate('recharge')}>
            {t('Recharge')}
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {summaryCards.map((metric) => (
          <Card
            className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
            key={metric.label}
          >
            <CardContent className="space-y-3 p-5">
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {metric.label}
              </p>
              <strong className="block text-3xl font-semibold text-zinc-950 dark:text-zinc-50">
                {metric.value}
              </strong>
              <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {metric.detail}
              </p>
            </CardContent>
          </Card>
        ))}
      </div>

      <section className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
        <WorkspacePanel
          description={t(
            'Canonical request settlements show captured credits, refunds, retail charge, and provider cost on one table.',
          )}
          title={t('Request settlements')}
        >
          <DataTable
            columns={[
              {
                id: 'request',
                header: t('Request'),
                cell: (row) => `#${row.request_id}`,
              },
              {
                id: 'status',
                header: t('Status'),
                cell: (row) => (
                  <Badge variant={settlementStatusVariant(row.status)}>
                    {formatStatusLabel(row.status)}
                  </Badge>
                ),
              },
              {
                id: 'credits',
                header: t('Credits'),
                cell: (row) => (
                  <div className="flex flex-col gap-1">
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {formatUnits(row.captured_credit_amount)}
                    </strong>
                    <span className="text-xs text-zinc-500 dark:text-zinc-400">
                      {t('Refunded {value}', {
                        value: formatUnits(row.refunded_amount),
                      })}
                    </span>
                  </div>
                ),
              },
              {
                id: 'charge',
                header: t('Charge'),
                cell: (row) => (
                  <div className="flex flex-col gap-1">
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {formatCurrency(row.retail_charge_amount)}
                    </strong>
                    <span className="text-xs text-zinc-500 dark:text-zinc-400">
                      {t('Provider {value}', {
                        value: formatCurrency(row.provider_cost_amount),
                      })}
                    </span>
                  </div>
                ),
              },
              {
                id: 'settled',
                header: t('Settled'),
                cell: (row) => formatDateTime(row.settled_at_ms),
              },
            ]}
            emptyState={(
              <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('No request settlements yet')}
                </strong>
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  {t(
                    'Request settlement rows will appear once routed traffic starts creating canonical commercial settlement evidence.',
                  )}
                </p>
              </div>
            )}
            getRowId={(row) => String(row.request_settlement_id)}
            rows={viewModel.request_settlements}
          />
        </WorkspacePanel>

        <div className="grid gap-4">
          <WorkspacePanel
            description={t(
              'Credit holds reveal reserved balance, capture posture, and hold expiry without leaving the explorer.',
            )}
            title={t('Credit holds')}
          >
            {viewModel.open_holds.length ? (
              <div className="grid gap-3">
                {viewModel.open_holds.map((hold) => (
                  <Card
                    className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
                    key={hold.hold_id}
                  >
                    <CardContent className="space-y-3 p-4">
                      <div className="flex items-start justify-between gap-3">
                        <div>
                          <strong className="block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {t('Hold #{id}', { id: hold.hold_id })}
                          </strong>
                          <span className="text-xs text-zinc-500 dark:text-zinc-400">
                            {t('Request #{id}', { id: hold.request_id })}
                          </span>
                        </div>
                        <Badge variant={holdStatusVariant(hold.status)}>
                          {formatStatusLabel(hold.status)}
                        </Badge>
                      </div>

                      <div className="grid gap-2 text-sm text-zinc-600 dark:text-zinc-300">
                        <div className="flex items-center justify-between gap-3">
                          <span>{t('Estimated')}</span>
                          <strong className="text-zinc-950 dark:text-zinc-50">
                            {formatUnits(hold.estimated_quantity)}
                          </strong>
                        </div>
                        <div className="flex items-center justify-between gap-3">
                          <span>{t('Captured')}</span>
                          <strong className="text-zinc-950 dark:text-zinc-50">
                            {formatUnits(hold.captured_quantity)}
                          </strong>
                        </div>
                        <div className="flex items-center justify-between gap-3">
                          <span>{t('Released')}</span>
                          <strong className="text-zinc-950 dark:text-zinc-50">
                            {formatUnits(hold.released_quantity)}
                          </strong>
                        </div>
                        <div className="flex items-center justify-between gap-3">
                          <span>{t('Expires')}</span>
                          <strong className="text-zinc-950 dark:text-zinc-50">
                            {formatDateTime(hold.expires_at_ms)}
                          </strong>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            ) : (
              <EmptyState
                description={t(
                  'Credit holds will appear here once canonical admission begins reserving balance before settlement capture.',
                )}
                title={t('No credit holds yet')}
              />
            )}
          </WorkspacePanel>

          <WorkspacePanel
            description={t(
              'Pricing evidence aligns active commercial plans, primary metrics, and balance posture for settlement review.',
            )}
            title={t('Pricing evidence')}
          >
            <div className="grid gap-3">
              <Card className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950">
                <CardContent className="grid gap-3 p-4">
                  {pricingEvidence.map((item) => (
                    <div
                      className="flex items-center justify-between gap-3"
                      key={item.label}
                    >
                      <span className="text-sm text-zinc-600 dark:text-zinc-300">
                        {item.label}
                      </span>
                      <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                        {item.value}
                      </strong>
                    </div>
                  ))}
                </CardContent>
              </Card>

              <Card className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950">
                <CardContent className="space-y-3 p-4">
                  <div className="grid gap-2">
                    {balancePosture.map((item) => (
                      <div
                        className="flex items-center justify-between gap-3"
                        key={item.label}
                      >
                        <span className="text-sm text-zinc-600 dark:text-zinc-300">
                          {item.label}
                        </span>
                        <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                          {item.value}
                        </strong>
                      </div>
                    ))}
                  </div>

                  <div className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/60">
                    <p className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      {t('Benefit lots')}
                    </p>
                    <div className="mt-3 space-y-2">
                      {viewModel.active_benefit_lots.length ? (
                        viewModel.active_benefit_lots.map((lot) => (
                          <div
                            className="flex items-center justify-between gap-3 text-sm"
                            key={lot.lot_id}
                          >
                            <span className="text-zinc-600 dark:text-zinc-300">
                              {t('{type} #{id}', {
                                type: formatStatusLabel(lot.benefit_type),
                                id: lot.lot_id,
                              })}
                            </span>
                            <strong className="text-zinc-950 dark:text-zinc-50">
                              {formatUnits(lot.remaining_quantity)}
                            </strong>
                          </div>
                        ))
                      ) : (
                        <div className="text-sm text-zinc-500 dark:text-zinc-400">
                          {t('No active benefit lots yet')}
                        </div>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          </WorkspacePanel>
        </div>
      </section>
    </div>
  );
}
