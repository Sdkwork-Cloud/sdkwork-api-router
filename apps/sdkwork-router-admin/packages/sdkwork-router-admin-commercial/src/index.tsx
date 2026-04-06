import { useEffect, useState, type ChangeEvent, type FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
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
  Input,
  Label,
  ManagementWorkbench,
  StatCard,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { getCommerceOrderAudit } from 'sdkwork-router-admin-admin-api';
import {
  countCurrentlyEffectiveCommercialPricingPlans,
  commercialPricingChargeUnitLabel,
  commercialPricingDisplayUnit,
  commercialPricingMethodLabel,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  formatAdminDateTime,
  selectPrimaryCommercialPricingPlan,
  selectPrimaryCommercialPricingRate,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  AdminPageProps,
  CommerceOrderAuditRecord,
  CommercePaymentEventRecord,
} from 'sdkwork-router-admin-types';
import {
  buildCommercialLedgerTimelineRows,
  buildCommercialRefundTimelineRows,
  type CommercialLedgerTimelineRow,
} from './ledgerTimeline';
import {
  buildCommercialOrderPaymentAuditRows,
  buildCommercialRefundAuditRows,
  type CommercialOrderPaymentAuditRow,
} from './orderPaymentAudit';
import {
  hasCommercialOrderAuditLookupValue,
  normalizeCommercialOrderAuditLookupValue,
} from './orderAuditLookup';

type CommercialFact = {
  label: string;
  value: string;
  detail: string;
  tone?: 'success' | 'warning' | 'secondary';
};

function formatStatusLabel(value: string) {
  return value
    .split(/[-_\s]+/g)
    .filter(Boolean)
    .map((segment) =>
      segment.length > 1
        ? `${segment.slice(0, 1).toUpperCase()}${segment.slice(1)}`
        : segment.toUpperCase())
    .join(' ');
}

function formatLedgerEntryTypeLabel(
  value: CommercialLedgerTimelineRow['entry_type'],
) {
  return formatStatusLabel(value);
}

function formatOrderAuditEventLabel(
  row: CommercialOrderPaymentAuditRow,
) {
  if (row.event_type) {
    return formatStatusLabel(row.event_type);
  }

  return formatStatusLabel(row.order_status);
}

function latestObservedPaymentEvent(
  paymentEvents: CommercePaymentEventRecord[],
) {
  return [...paymentEvents].sort((left, right) =>
    (right.processed_at_ms ?? right.received_at_ms)
    - (left.processed_at_ms ?? left.received_at_ms)
    || right.payment_event_id.localeCompare(left.payment_event_id),
  )[0] ?? null;
}

export function CommercialPage({ snapshot }: AdminPageProps) {
  const { formatCurrency, formatNumber, t } = useAdminI18n();

  const activeAccounts = snapshot.commercialAccounts.filter(
    (record) => record.account.status === 'active',
  ).length;
  const suspendedAccounts = snapshot.commercialAccounts.filter(
    (record) => record.account.status === 'suspended',
  ).length;
  const availableBalance = snapshot.commercialAccounts.reduce(
    (sum, record) => sum + record.available_balance,
    0,
  );
  const heldBalance = snapshot.commercialAccounts.reduce(
    (sum, record) => sum + record.held_balance,
    0,
  );
  const openHolds = snapshot.commercialAccountHolds.filter(
    (hold) =>
      hold.status === 'held'
      || hold.status === 'captured'
      || hold.status === 'partially_released',
  );
  const latestSettlements = [...snapshot.commercialRequestSettlements]
    .sort((left, right) => right.settled_at_ms - left.settled_at_ms)
    .slice(0, 5);
  const activePricingPlans = countCurrentlyEffectiveCommercialPricingPlans(
    snapshot.commercialPricingPlans,
  );
  const pricedMetrics = new Set(
    snapshot.commercialPricingRates.map((rate) => rate.metric_code),
  );
  const primaryPricingPlan = selectPrimaryCommercialPricingPlan(
    snapshot.commercialPricingPlans,
  );
  const primaryPricingRate = selectPrimaryCommercialPricingRate(
    snapshot.commercialPricingRates,
    primaryPricingPlan,
  );
  const commercialLedgerTimeline = buildCommercialLedgerTimelineRows(
    snapshot.commercialAccountLedger,
    snapshot.commercialRequestSettlements,
  );
  const recentLedgerTimeline = commercialLedgerTimeline.slice(0, 8);
  const refundTimelineRows = buildCommercialRefundTimelineRows(
    commercialLedgerTimeline,
  ).slice(0, 6);
  const commercialOrderPaymentAuditRows = buildCommercialOrderPaymentAuditRows(
    snapshot.commerceOrders,
    snapshot.commercePaymentEvents,
  );
  const recentOrderPaymentAuditRows = commercialOrderPaymentAuditRows.slice(0, 8);
  const refundAuditRows = buildCommercialRefundAuditRows(
    commercialOrderPaymentAuditRows,
  ).slice(0, 6);
  const rejectedPaymentEvents = snapshot.commercePaymentEvents.filter((event) =>
    event.processing_status === 'rejected'
    || event.processing_status === 'failed',
  ).length;
  const [orderAuditLookupValue, setOrderAuditLookupValue] = useState('');
  const [orderAuditLookupError, setOrderAuditLookupError] = useState<string | null>(null);
  const [selectedOrderAuditId, setSelectedOrderAuditId] = useState<string | null>(null);
  const [selectedOrderAudit, setSelectedOrderAudit] =
    useState<CommerceOrderAuditRecord | null>(null);
  const [isOrderAuditLoading, setIsOrderAuditLoading] = useState(false);
  const [orderAuditError, setOrderAuditError] = useState<string | null>(null);
  const selectedOrderFromSnapshot = selectedOrderAuditId
    ? snapshot.commerceOrders.find((order) => order.order_id === selectedOrderAuditId) ?? null
    : null;
  const selectedOrderRecord = selectedOrderAudit?.order ?? selectedOrderFromSnapshot ?? null;
  const selectedOrderPaymentEvents = selectedOrderAudit?.payment_events
    ?? (selectedOrderAuditId
      ? snapshot.commercePaymentEvents.filter((event) => event.order_id === selectedOrderAuditId)
      : []);
  const latestOrderPaymentEvent = latestObservedPaymentEvent(selectedOrderPaymentEvents);
  const orderAuditOpen = selectedOrderAuditId != null;

  useEffect(() => {
    if (!selectedOrderAuditId) {
      setSelectedOrderAudit(null);
      setOrderAuditError(null);
      setIsOrderAuditLoading(false);
      return;
    }

    let cancelled = false;
    setSelectedOrderAudit(null);
    setOrderAuditError(null);
    setIsOrderAuditLoading(true);

    getCommerceOrderAudit(selectedOrderAuditId)
      .then((audit) => {
        if (!cancelled) {
          setSelectedOrderAudit(audit);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setOrderAuditError(
            error instanceof Error ? error.message : 'Failed to load order audit detail.',
          );
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsOrderAuditLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedOrderAuditId]);

  function openOrderAudit(orderId: string) {
    const normalizedOrderId = normalizeCommercialOrderAuditLookupValue(orderId);
    setOrderAuditLookupValue(normalizedOrderId);
    setOrderAuditLookupError(null);
    setSelectedOrderAuditId(normalizedOrderId);
  }

  function handleOrderAuditOpenChange(open: boolean) {
    if (!open) {
      setSelectedOrderAuditId(null);
    }
  }

  function handleOrderAuditLookupSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!hasCommercialOrderAuditLookupValue(orderAuditLookupValue)) {
      setOrderAuditLookupError(t('Enter an order id to open order audit detail.'));
      return;
    }

    openOrderAudit(orderAuditLookupValue);
  }

  const summaryCards = [
    {
      label: t('Commercial accounts'),
      value: formatNumber(snapshot.commercialAccounts.length),
      description: t('Canonical payable accounts currently discoverable by the commercial control plane.'),
    },
    {
      label: t('Available balance'),
      value: formatNumber(availableBalance),
      description: t('Spendable credit still available across the commercial account inventory.'),
    },
    {
      label: t('Settlement explorer'),
      value: formatNumber(snapshot.commercialRequestSettlements.length),
      description: t('Captured, released, and refunded request settlements ready for operator investigation.'),
    },
    {
      label: t('Order payment audit'),
      value: formatNumber(snapshot.commerceOrders.length),
      description: t('Recent commerce orders stay linked to provider callbacks and operator-visible payment evidence.'),
    },
    {
      label: t('Pricing governance'),
      value: formatNumber(snapshot.commercialPricingRates.length),
      description: t('Live metric-rate rows currently shaping canonical commercial charging.'),
    },
  ];
  const settlementLedgerColumns: Array<DataTableColumn<CommercialLedgerTimelineRow>> = [
    {
      id: 'account',
      header: t('Account'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {t('Account #{id}', { id: row.account_id })}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.request_id != null
              ? t('Request #{id}', { id: row.request_id })
              : t('No linked request')}
          </div>
        </div>
      ),
    },
    {
      id: 'entry_type',
      header: t('Entry'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={formatLedgerEntryTypeLabel(row.entry_type)}
          variant={row.entry_type === 'refund' ? 'warning' : 'secondary'}
        />
      ),
      width: 180,
    },
    {
      id: 'credits',
      header: t('Credits'),
      cell: (row) => formatNumber(row.amount),
      width: 120,
    },
    {
      id: 'settlement',
      header: t('Settlement'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={row.settlement_status ? formatStatusLabel(row.settlement_status) : t('Unlinked')}
          variant={
            row.settlement_status === 'captured'
              ? 'success'
              : row.settlement_status === 'refunded'
                ? 'warning'
                : 'secondary'
          }
        />
      ),
      width: 160,
    },
    {
      id: 'retail_charge',
      header: t('Retail charge'),
      cell: (row) =>
        row.request_settlement_id != null
          ? formatCurrency(row.retail_charge_amount)
          : t('n/a'),
      width: 140,
    },
    {
      id: 'observed',
      header: t('Observed'),
      cell: (row) => formatAdminDateTime(row.created_at_ms),
      width: 180,
    },
  ];
  const refundTimelineColumns: Array<DataTableColumn<CommercialLedgerTimelineRow>> = [
    {
      id: 'account',
      header: t('Account'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {t('Account #{id}', { id: row.account_id })}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.hold_id != null
              ? t('Hold #{id}', { id: row.hold_id })
              : t('No linked hold')}
          </div>
        </div>
      ),
    },
    {
      id: 'request',
      header: t('Request'),
      cell: (row) =>
        row.request_id != null ? t('Request #{id}', { id: row.request_id }) : t('Unlinked'),
      width: 140,
    },
    {
      id: 'refund_credits',
      header: t('Refund credits'),
      cell: (row) => formatNumber(row.refunded_amount || row.amount),
      width: 140,
    },
    {
      id: 'retail_charge',
      header: t('Retail charge'),
      cell: (row) =>
        row.request_settlement_id != null
          ? formatCurrency(row.retail_charge_amount)
          : t('n/a'),
      width: 140,
    },
    {
      id: 'provider_cost',
      header: t('Provider cost'),
      cell: (row) =>
        row.request_settlement_id != null
          ? formatCurrency(row.provider_cost_amount)
          : t('n/a'),
      width: 140,
    },
    {
      id: 'observed',
      header: t('Observed'),
      cell: (row) => formatAdminDateTime(row.created_at_ms),
      width: 180,
    },
  ];
  const orderPaymentAuditColumns: Array<DataTableColumn<CommercialOrderPaymentAuditRow>> = [
    {
      id: 'order',
      header: t('Order'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {t('Order #{id}', { id: row.order_id })}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.target_name}
          </div>
        </div>
      ),
    },
    {
      id: 'event',
      header: t('Event'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={formatOrderAuditEventLabel(row)}
          variant={row.event_type === 'refunded' ? 'warning' : 'secondary'}
        />
      ),
      width: 180,
    },
    {
      id: 'provider',
      header: t('Provider'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {formatStatusLabel(row.provider)}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.provider_event_id ?? t('No provider event id')}
          </div>
        </div>
      ),
      width: 220,
    },
    {
      id: 'processing',
      header: t('Processing'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={row.processing_status ? formatStatusLabel(row.processing_status) : t('Pending evidence')}
          variant={
            row.processing_status === 'processed'
              ? 'success'
              : row.processing_status === 'rejected' || row.processing_status === 'failed'
                ? 'danger'
                : 'secondary'
          }
        />
      ),
      width: 180,
    },
    {
      id: 'amount',
      header: t('Amount'),
      cell: (row) => row.payable_price_label,
      width: 140,
    },
    {
      id: 'observed',
      header: t('Observed'),
      cell: (row) => formatAdminDateTime(row.observed_at_ms),
      width: 180,
    },
    {
      id: 'detail',
      header: t('Investigation'),
      cell: (row) => (
        <Button
          onClick={() => openOrderAudit(row.order_id)}
          size="sm"
          type="button"
          variant="outline"
        >
          {t('View order audit')}
        </Button>
      ),
      width: 180,
    },
  ];
  const orderRefundAuditColumns: Array<DataTableColumn<CommercialOrderPaymentAuditRow>> = [
    {
      id: 'order',
      header: t('Order'),
      cell: (row) => (
        <div className="space-y-1">
          <div className="font-medium text-[var(--sdk-color-text-primary)]">
            {t('Order #{id}', { id: row.order_id })}
          </div>
          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
            {row.target_name}
          </div>
        </div>
      ),
    },
    {
      id: 'provider',
      header: t('Provider'),
      cell: (row) => formatStatusLabel(row.provider),
      width: 160,
    },
    {
      id: 'refund_state',
      header: t('Refund state'),
      cell: (row) => (
        <StatusBadge
          showIcon
          status={formatOrderAuditEventLabel(row)}
          variant="warning"
        />
      ),
      width: 160,
    },
    {
      id: 'amount',
      header: t('Amount'),
      cell: (row) => row.payable_price_label,
      width: 140,
    },
    {
      id: 'observed',
      header: t('Observed'),
      cell: (row) => formatAdminDateTime(row.observed_at_ms),
      width: 180,
    },
    {
      id: 'detail',
      header: t('Investigation'),
      cell: (row) => (
        <Button
          onClick={() => openOrderAudit(row.order_id)}
          size="sm"
          type="button"
          variant="outline"
        >
          {t('View order audit')}
        </Button>
      ),
      width: 180,
    },
  ];

  const accountFacts: CommercialFact[] = [
    {
      label: t('Active accounts'),
      value: formatNumber(activeAccounts),
      detail: t('Accounts currently able to receive holds and settlement capture.'),
      tone: 'success',
    },
    {
      label: t('Suspended accounts'),
      value: formatNumber(suspendedAccounts),
      detail: t('Accounts blocked from new commercial admission until operator review.'),
      tone: suspendedAccounts > 0 ? 'warning' : 'secondary',
    },
    {
      label: t('Held balance'),
      value: formatNumber(heldBalance),
      detail: t('Credit currently reserved by request admission and pending settlement flows.'),
      tone: heldBalance > 0 ? 'warning' : 'secondary',
    },
  ];

  const settlementFacts: CommercialFact[] = [
    {
      label: t('Open holds'),
      value: formatNumber(openHolds.length),
      detail: t('Commercial holds that still need capture, release, expiry, or operator intervention.'),
      tone: openHolds.length > 0 ? 'warning' : 'secondary',
    },
    {
      label: t('Captured settlements'),
      value: formatNumber(
        snapshot.commercialRequestSettlements.filter((record) => record.status === 'captured').length,
      ),
      detail: t('Settlements already converted into captured commercial liability evidence.'),
      tone: 'success',
    },
    {
      label: t('Refunded settlements'),
      value: formatNumber(
        snapshot.commercialRequestSettlements.filter((record) => record.status === 'refunded').length,
      ),
      detail: t('Refund posture keeps correction flows visible inside the settlement explorer.'),
      tone: 'secondary',
    },
    {
      label: t('Rejected callbacks'),
      value: formatNumber(rejectedPaymentEvents),
      detail: t('Rejected or failed provider callbacks stay visible before they drift into silent payment reconciliation gaps.'),
      tone: rejectedPaymentEvents > 0 ? 'warning' : 'secondary',
    },
  ];

  const pricingFacts: CommercialFact[] = [
    {
      label: t('Active pricing plans'),
      value: formatNumber(activePricingPlans),
      detail: t('Commercial pricing plans that are active and currently effective in the control plane.'),
      tone: activePricingPlans > 0 ? 'success' : 'warning',
    },
    {
      label: t('Priced metrics'),
      value: formatNumber(pricedMetrics.size),
      detail: t('Distinct metric codes already governed by canonical pricing rates.'),
      tone: 'secondary',
    },
    {
      label: t('Primary plan'),
      value: primaryPricingPlan?.display_name ?? t('No active plan'),
      detail: t('The first active pricing plan remains the quickest operator reference point.'),
      tone: primaryPricingPlan ? 'success' : 'warning',
    },
    {
      label: t('Charge unit'),
      value: commercialPricingChargeUnitLabel(primaryPricingRate?.charge_unit, t),
      detail: t('Primary metered unit keeps settlement granularity explicit for operator review.'),
      tone: primaryPricingRate ? 'success' : 'secondary',
    },
    {
      label: t('Billing method'),
      value: commercialPricingMethodLabel(primaryPricingRate?.pricing_method, t),
      detail: t('Settlement method shows whether the primary rate charges per unit, flat, or step-based.'),
      tone: primaryPricingRate ? 'success' : 'secondary',
    },
    {
      label: t('Price unit'),
      value: commercialPricingDisplayUnit(primaryPricingRate, t),
      detail: t('Display unit makes the commercial rate readable for token and multimodal pricing review.'),
      tone: primaryPricingRate ? 'success' : 'secondary',
    },
  ];

  return (
    <>
      <ManagementWorkbench
      description={t('Commercial accounts, settlement explorer, and pricing governance now live as a first-class admin module.')}
      eyebrow={t('Revenue')}
      main={{
        title: t('Commercial control plane'),
        description: t('Operators can audit commercial accounts, request settlement posture, and pricing governance without leaving a dedicated module.'),
        children: (
            <div className="space-y-6">
            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
              {summaryCards.map((metric) => (
                <StatCard
                  description={metric.description}
                  key={metric.label}
                  label={metric.label}
                  value={metric.value}
                />
              ))}
            </div>

            <div className="grid gap-4 xl:grid-cols-3">
              <Card>
                <CardHeader>
                  <CardTitle>{t('Commercial accounts')}</CardTitle>
                  <CardDescription>
                    {t('Account posture keeps status, held balance, and admission readiness visible in one surface.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  {accountFacts.map((fact) => (
                    <div className="flex items-start justify-between gap-3" key={fact.label}>
                      <div className="space-y-1">
                        <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                          {fact.label}
                        </div>
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {fact.detail}
                        </div>
                      </div>
                      <StatusBadge
                        showIcon
                        status={fact.value}
                        variant={fact.tone ?? 'secondary'}
                      />
                    </div>
                  ))}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>{t('Settlement explorer')}</CardTitle>
                  <CardDescription>
                    {t('Settlement explorer highlights open holds, captured requests, and correction posture from canonical settlement records.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  {settlementFacts.map((fact) => (
                    <div className="flex items-start justify-between gap-3" key={fact.label}>
                      <div className="space-y-1">
                        <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                          {fact.label}
                        </div>
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {fact.detail}
                        </div>
                      </div>
                      <StatusBadge
                        showIcon
                        status={fact.value}
                        variant={fact.tone ?? 'secondary'}
                      />
                    </div>
                  ))}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>{t('Pricing governance')}</CardTitle>
                  <CardDescription>
                    {t('Pricing governance keeps commercial plan activation and metric-rate coverage visible for operator review.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  {pricingFacts.map((fact) => (
                    <div className="flex items-start justify-between gap-3" key={fact.label}>
                      <div className="space-y-1">
                        <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                          {fact.label}
                        </div>
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {fact.detail}
                        </div>
                      </div>
                      <StatusBadge
                        showIcon
                        status={fact.value}
                        variant={fact.tone ?? 'secondary'}
                      />
                    </div>
                  ))}
                </CardContent>
              </Card>
            </div>

            <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
              <Card className="min-h-0 flex flex-col overflow-hidden p-0">
                <CardHeader>
                  <CardTitle>{t('Settlement ledger')}</CardTitle>
                  <CardDescription>
                    {t('Settlement ledger keeps capture and refund entries linked to request settlements so operators can audit credits, retail charge, and final correction posture without leaving the commercial module.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="min-h-0 flex-1 p-0">
                  <DataTable
                    className={embeddedAdminDataTableClassName}
                    columns={settlementLedgerColumns}
                    emptyDescription={t('Settlement ledger entries will appear here once commercial account history begins landing for the selected control-plane slice.')}
                    emptyTitle={t('No settlement ledger entries yet')}
                    getRowId={(row: CommercialLedgerTimelineRow) => row.id}
                    rows={recentLedgerTimeline}
                    slotProps={embeddedAdminDataTableSlotProps}
                    stickyHeader
                  />
                </CardContent>
              </Card>

              <Card className="min-h-0 flex flex-col overflow-hidden p-0">
                <CardHeader>
                  <CardTitle>{t('Refund timeline')}</CardTitle>
                  <CardDescription>
                    {t('Refund timeline isolates correction entries so support and finance can verify credited quantity, linked request, and refund cost posture at a glance.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="min-h-0 flex-1 p-0">
                  <DataTable
                    className={embeddedAdminDataTableClassName}
                    columns={refundTimelineColumns}
                    emptyDescription={t('Refund activity will appear here once commercial refunds are posted into the account ledger history.')}
                    emptyTitle={t('No refunds recorded yet')}
                    getRowId={(row: CommercialLedgerTimelineRow) => row.id}
                    rows={refundTimelineRows}
                    slotProps={embeddedAdminDataTableSlotProps}
                    stickyHeader
                  />
                </CardContent>
              </Card>
            </div>

            <div className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
              <Card className="min-h-0 flex flex-col overflow-hidden p-0">
                <CardHeader className="space-y-4">
                  <CardTitle>{t('Order payment audit')}</CardTitle>
                  <CardDescription>
                    {t('Order payment audit keeps recent commercial orders linked to payment callbacks, provider evidence, and operator-visible processing posture without loading unbounded order history into the commercial module.')}
                  </CardDescription>
                  <form
                    className="flex flex-wrap items-end gap-3"
                    onSubmit={handleOrderAuditLookupSubmit}
                  >
                    <div className="min-w-[18rem] flex-[1.3] space-y-2">
                      <Label htmlFor="commercial-order-audit-lookup">
                        {t('Find order audit')}
                      </Label>
                      <Input
                        aria-invalid={orderAuditLookupError != null}
                        autoComplete="off"
                        id="commercial-order-audit-lookup"
                        onChange={(event: ChangeEvent<HTMLInputElement>) => {
                          setOrderAuditLookupValue(event.target.value);
                          if (orderAuditLookupError) {
                            setOrderAuditLookupError(null);
                          }
                        }}
                        placeholder={t('Enter an order id to open order audit detail.')}
                        value={orderAuditLookupValue}
                      />
                      {orderAuditLookupError ? (
                        <div className="text-xs text-[var(--sdk-color-status-danger)]">
                          {orderAuditLookupError}
                        </div>
                      ) : null}
                    </div>

                    <Button type="submit" variant="outline">
                      {t('Inspect')}
                    </Button>
                  </form>
                </CardHeader>
                <CardContent className="min-h-0 flex-1 p-0">
                  <DataTable
                    className={embeddedAdminDataTableClassName}
                    columns={orderPaymentAuditColumns}
                    emptyDescription={t('Recent commerce orders will appear here once checkout, webhook, and settlement evidence starts landing in the commercial audit stream.')}
                    emptyTitle={t('No order payment evidence yet')}
                    getRowId={(row: CommercialOrderPaymentAuditRow) => row.id}
                    rows={recentOrderPaymentAuditRows}
                    slotProps={embeddedAdminDataTableSlotProps}
                    stickyHeader
                  />
                </CardContent>
              </Card>

              <Card className="min-h-0 flex flex-col overflow-hidden p-0">
                <CardHeader>
                  <CardTitle>{t('Order refund audit')}</CardTitle>
                  <CardDescription>
                    {t('Order refund audit keeps explicit refund callbacks and refunded-order fallback evidence visible so operators can spot missing callback closure before it becomes a reconciliation blind spot.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="min-h-0 flex-1 p-0">
                  <DataTable
                    className={embeddedAdminDataTableClassName}
                    columns={orderRefundAuditColumns}
                    emptyDescription={t('Refund audit rows will appear here once commercial orders begin entering explicit refund or refunded-order-state correction flows.')}
                    emptyTitle={t('No refund evidence yet')}
                    getRowId={(row: CommercialOrderPaymentAuditRow) => row.id}
                    rows={refundAuditRows}
                    slotProps={embeddedAdminDataTableSlotProps}
                    stickyHeader
                  />
                </CardContent>
              </Card>
            </div>
          </div>
        ),
      }}
      detail={{
        title: t('Latest settlements'),
        description: t('The right rail keeps the most recent commercial settlement evidence in view for rapid operator triage.'),
        children: (
          <div className="space-y-4">
            {latestSettlements.length ? (
              latestSettlements.map((settlement) => (
                <Card key={settlement.request_settlement_id}>
                  <CardHeader className="space-y-2">
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <CardTitle className="text-base">
                          {t('Request #{id}', { id: settlement.request_id })}
                        </CardTitle>
                        <CardDescription>
                          {t('Account #{id}', { id: settlement.account_id })}
                        </CardDescription>
                      </div>
                      <StatusBadge
                        showIcon
                        status={formatStatusLabel(settlement.status)}
                        variant={
                          settlement.status === 'captured'
                            ? 'success'
                            : settlement.status === 'failed'
                              ? 'danger'
                              : 'secondary'
                        }
                      />
                    </div>
                  </CardHeader>
                  <CardContent className="grid gap-1 text-sm text-[var(--sdk-color-text-secondary)]">
                    <div>
                      {t('Retail charge: {amount}', {
                        amount: formatCurrency(settlement.retail_charge_amount),
                      })}
                    </div>
                    <div>
                      {t('Provider cost: {amount}', {
                        amount: formatCurrency(settlement.provider_cost_amount),
                      })}
                    </div>
                    <div>
                      {t('Captured credits: {count}', {
                        count: formatNumber(settlement.captured_credit_amount),
                      })}
                    </div>
                  </CardContent>
                </Card>
              ))
            ) : (
              <Card>
                <CardHeader>
                  <CardTitle>{t('No settlement evidence yet')}</CardTitle>
                  <CardDescription>
                    {t('Latest settlements will appear here once request settlement records start landing from the canonical commercial kernel.')}
                  </CardDescription>
                </CardHeader>
              </Card>
            )}
          </div>
        ),
      }}
      title={t('Commercial')}
      />
      <Drawer open={orderAuditOpen} onOpenChange={handleOrderAuditOpenChange}>
        <DrawerContent side="right" size="xl">
          {orderAuditOpen ? (
            <>
              <DrawerHeader>
                <div className="space-y-3">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="space-y-1">
                      <DrawerTitle>{t('Order audit detail')}</DrawerTitle>
                      <DrawerDescription>
                        {selectedOrderRecord
                          ? t('Order #{id}', { id: selectedOrderRecord.order_id })
                          : t('Loading selected order')}
                      </DrawerDescription>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <StatusBadge
                        showIcon
                        status={selectedOrderRecord ? formatStatusLabel(selectedOrderRecord.status) : t('Loading')}
                        variant={
                          selectedOrderRecord?.status === 'fulfilled'
                            ? 'success'
                            : selectedOrderRecord?.status === 'refunded'
                              ? 'warning'
                              : 'secondary'
                        }
                      />
                      <StatusBadge
                        showIcon
                        status={latestOrderPaymentEvent ? formatStatusLabel(latestOrderPaymentEvent.provider) : t('No payment evidence')}
                        variant={latestOrderPaymentEvent ? 'secondary' : 'warning'}
                      />
                      {selectedOrderRecord?.applied_coupon_code ? (
                        <StatusBadge
                          showIcon
                          status={selectedOrderRecord.applied_coupon_code}
                          variant="secondary"
                        />
                      ) : null}
                    </div>
                  </div>
                </div>
              </DrawerHeader>

              <DrawerBody className="space-y-4">
                {isOrderAuditLoading ? (
                  <Card>
                    <CardHeader>
                      <CardTitle>{t('Loading order audit evidence')}</CardTitle>
                      <CardDescription>
                        {t('Payment, coupon, and campaign evidence is being loaded for the selected order.')}
                      </CardDescription>
                    </CardHeader>
                  </Card>
                ) : null}

                {!isOrderAuditLoading && orderAuditError ? (
                  <Card>
                    <CardHeader>
                      <CardTitle>{t('Order audit detail unavailable')}</CardTitle>
                      <CardDescription>{orderAuditError}</CardDescription>
                    </CardHeader>
                  </Card>
                ) : null}

                {!isOrderAuditLoading && !orderAuditError && selectedOrderRecord ? (
                  <>
                    <Card>
                      <CardHeader>
                        <CardTitle>{t('Order audit detail')}</CardTitle>
                        <CardDescription>
                          {t('Commercial order, checkout, and coupon evidence stay bundled here so operators can reconstruct fulfillment and refund posture without switching modules.')}
                        </CardDescription>
                      </CardHeader>
                      <CardContent>
                        <DescriptionList columns={2}>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Project')}</DescriptionTerm>
                            <DescriptionDetails>{selectedOrderRecord.project_id}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('User')}</DescriptionTerm>
                            <DescriptionDetails>{selectedOrderRecord.user_id}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Target')}</DescriptionTerm>
                            <DescriptionDetails>{selectedOrderRecord.target_name}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Target kind')}</DescriptionTerm>
                            <DescriptionDetails>{formatStatusLabel(selectedOrderRecord.target_kind)}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('List price')}</DescriptionTerm>
                            <DescriptionDetails>{selectedOrderRecord.list_price_label}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Payable price')}</DescriptionTerm>
                            <DescriptionDetails>{selectedOrderRecord.payable_price_label}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Granted units')}</DescriptionTerm>
                            <DescriptionDetails>{formatNumber(selectedOrderRecord.granted_units)}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Bonus units')}</DescriptionTerm>
                            <DescriptionDetails>{formatNumber(selectedOrderRecord.bonus_units)}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Coupon code')}</DescriptionTerm>
                            <DescriptionDetails>{selectedOrderRecord.applied_coupon_code ?? t('No coupon applied')}</DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Observed')}</DescriptionTerm>
                            <DescriptionDetails>{formatAdminDateTime(selectedOrderRecord.updated_at_ms)}</DescriptionDetails>
                          </DescriptionItem>
                        </DescriptionList>
                      </CardContent>
                    </Card>

                    <Card>
                      <CardHeader>
                        <CardTitle>{t('Payment evidence timeline')}</CardTitle>
                        <CardDescription>
                          {t('Provider callbacks remain ordered here so operators can verify settlement, rejection, and refund sequencing for the selected order.')}
                        </CardDescription>
                      </CardHeader>
                      <CardContent className="space-y-3">
                        {selectedOrderPaymentEvents.length ? (
                          selectedOrderPaymentEvents.map((event) => (
                            <Card className="border-[var(--sdk-color-border-subtle)] shadow-none" key={event.payment_event_id}>
                              <CardHeader className="space-y-2">
                                <div className="flex flex-wrap items-start justify-between gap-3">
                                  <div className="space-y-1">
                                    <CardTitle className="text-sm">
                                      {formatStatusLabel(event.event_type)}
                                    </CardTitle>
                                    <CardDescription>
                                      {event.provider_event_id ?? t('No provider event id')}
                                    </CardDescription>
                                  </div>
                                  <div className="flex flex-wrap gap-2">
                                    <StatusBadge
                                      showIcon
                                      status={formatStatusLabel(event.provider)}
                                      variant="secondary"
                                    />
                                    <StatusBadge
                                      showIcon
                                      status={formatStatusLabel(event.processing_status)}
                                      variant={
                                        event.processing_status === 'processed'
                                          ? 'success'
                                          : event.processing_status === 'rejected'
                                            || event.processing_status === 'failed'
                                            ? 'danger'
                                            : 'secondary'
                                      }
                                    />
                                  </div>
                                </div>
                              </CardHeader>
                              <CardContent>
                                <DescriptionList columns={2}>
                                  <DescriptionItem>
                                    <DescriptionTerm>{t('Observed')}</DescriptionTerm>
                                    <DescriptionDetails>
                                      {formatAdminDateTime(event.processed_at_ms ?? event.received_at_ms)}
                                    </DescriptionDetails>
                                  </DescriptionItem>
                                  <DescriptionItem>
                                    <DescriptionTerm>{t('Order status after')}</DescriptionTerm>
                                    <DescriptionDetails>
                                      {event.order_status_after
                                        ? formatStatusLabel(event.order_status_after)
                                        : t('No derived order status')}
                                    </DescriptionDetails>
                                  </DescriptionItem>
                                  <DescriptionItem>
                                    <DescriptionTerm>{t('Payment event id')}</DescriptionTerm>
                                    <DescriptionDetails>{event.payment_event_id}</DescriptionDetails>
                                  </DescriptionItem>
                                  <DescriptionItem>
                                    <DescriptionTerm>{t('Dedupe key')}</DescriptionTerm>
                                    <DescriptionDetails>{event.dedupe_key}</DescriptionDetails>
                                  </DescriptionItem>
                                </DescriptionList>
                                {event.processing_message ? (
                                  <div className="mt-3 text-sm text-[var(--sdk-color-text-secondary)]">
                                    {event.processing_message}
                                  </div>
                                ) : null}
                              </CardContent>
                            </Card>
                          ))
                        ) : (
                          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                            {t('No payment evidence has been recorded for this order yet.')}
                          </div>
                        )}
                      </CardContent>
                    </Card>

                    <Card>
                      <CardHeader>
                        <CardTitle>{t('Coupon evidence chain')}</CardTitle>
                        <CardDescription>
                          {t('Reservation, redemption, rollback, code, template, and campaign evidence stays attached so discount posture can be audited together with payment callbacks.')}
                        </CardDescription>
                      </CardHeader>
                      <CardContent>
                        <DescriptionList columns={2}>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Reservation')}</DescriptionTerm>
                            <DescriptionDetails>
                              {selectedOrderAudit?.coupon_reservation
                                ? `${selectedOrderAudit.coupon_reservation.coupon_reservation_id} (${formatStatusLabel(selectedOrderAudit.coupon_reservation.reservation_status)})`
                                : t('No reservation evidence')}
                            </DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Redemption')}</DescriptionTerm>
                            <DescriptionDetails>
                              {selectedOrderAudit?.coupon_redemption
                                ? `${selectedOrderAudit.coupon_redemption.coupon_redemption_id} (${formatStatusLabel(selectedOrderAudit.coupon_redemption.redemption_status)})`
                                : t('No redemption evidence')}
                            </DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Rollback count')}</DescriptionTerm>
                            <DescriptionDetails>
                              {formatNumber(selectedOrderAudit?.coupon_rollbacks.length ?? 0)}
                            </DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Coupon code')}</DescriptionTerm>
                            <DescriptionDetails>
                              {selectedOrderAudit?.coupon_code?.code_value ?? t('No coupon code evidence')}
                            </DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Coupon template')}</DescriptionTerm>
                            <DescriptionDetails>
                              {selectedOrderAudit?.coupon_template?.display_name ?? t('No template evidence')}
                            </DescriptionDetails>
                          </DescriptionItem>
                          <DescriptionItem>
                            <DescriptionTerm>{t('Marketing campaign')}</DescriptionTerm>
                            <DescriptionDetails>
                              {selectedOrderAudit?.marketing_campaign?.display_name ?? t('No campaign evidence')}
                            </DescriptionDetails>
                          </DescriptionItem>
                        </DescriptionList>
                      </CardContent>
                    </Card>

                    <Card>
                      <CardHeader>
                        <CardTitle>{t('Coupon rollback timeline')}</CardTitle>
                        <CardDescription>
                          {t('Rollback evidence confirms whether coupon subsidy and inventory were restored during refund handling.')}
                        </CardDescription>
                      </CardHeader>
                      <CardContent className="space-y-3">
                        {selectedOrderAudit?.coupon_rollbacks.length ? (
                          selectedOrderAudit.coupon_rollbacks.map((rollback) => (
                            <Card className="border-[var(--sdk-color-border-subtle)] shadow-none" key={rollback.coupon_rollback_id}>
                              <CardHeader className="space-y-2">
                                <div className="flex flex-wrap items-start justify-between gap-3">
                                  <div className="space-y-1">
                                    <CardTitle className="text-sm">
                                      {rollback.coupon_rollback_id}
                                    </CardTitle>
                                    <CardDescription>
                                      {formatAdminDateTime(rollback.updated_at_ms)}
                                    </CardDescription>
                                  </div>
                                  <div className="flex flex-wrap gap-2">
                                    <StatusBadge
                                      showIcon
                                      status={formatStatusLabel(rollback.rollback_type)}
                                      variant="warning"
                                    />
                                    <StatusBadge
                                      showIcon
                                      status={formatStatusLabel(rollback.rollback_status)}
                                      variant={
                                        rollback.rollback_status === 'completed'
                                          ? 'success'
                                          : rollback.rollback_status === 'failed'
                                            ? 'danger'
                                            : 'secondary'
                                      }
                                    />
                                  </div>
                                </div>
                              </CardHeader>
                              <CardContent>
                                <DescriptionList columns={2}>
                                  <DescriptionItem>
                                    <DescriptionTerm>{t('Restored budget')}</DescriptionTerm>
                                    <DescriptionDetails>{formatNumber(rollback.restored_budget_minor)}</DescriptionDetails>
                                  </DescriptionItem>
                                  <DescriptionItem>
                                    <DescriptionTerm>{t('Restored inventory')}</DescriptionTerm>
                                    <DescriptionDetails>{formatNumber(rollback.restored_inventory_count)}</DescriptionDetails>
                                  </DescriptionItem>
                                </DescriptionList>
                              </CardContent>
                            </Card>
                          ))
                        ) : (
                          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                            {t('No coupon rollback evidence has been recorded for this order.')}
                          </div>
                        )}
                      </CardContent>
                    </Card>
                  </>
                ) : null}
              </DrawerBody>

              <DrawerFooter className="text-xs text-[var(--sdk-color-text-secondary)]">
                {t(
                  'Order audit detail keeps payment callbacks and coupon lifecycle evidence scoped to the selected order so reconciliation triage stays deterministic.',
                )}
              </DrawerFooter>
            </>
          ) : null}
        </DrawerContent>
      </Drawer>
    </>
  );
}
