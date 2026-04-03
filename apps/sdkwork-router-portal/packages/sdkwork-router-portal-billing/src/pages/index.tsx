import { useDeferredValue, useEffect, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';

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
import {
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from 'sdkwork-router-portal-commons/framework/entry';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import {
  FilterBar,
  FilterBarActions,
  FilterBarSection,
  FilterField,
  SearchInput,
  SettingsField,
} from 'sdkwork-router-portal-commons/framework/form';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from 'sdkwork-router-portal-commons/framework/overlays';
import { WorkspacePanel } from 'sdkwork-router-portal-commons/framework/workspace';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  BillingAccountingMode,
  BillingEventCapabilitySummary,
  BillingEventRecord,
  BillingEventSummary,
  PortalCommerceCheckoutSession,
  PortalCommerceCheckoutSessionMethod,
  PortalCommerceCheckoutSessionStatus,
  PortalCommerceMembership,
  PortalCommerceOrder,
  PortalCommerceOrderStatus,
  PortalCommercePaymentEventType,
  PortalCommerceQuoteKind,
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import { BillingRecommendationCard } from '../components';
import {
  cancelBillingOrder,
  createBillingOrder,
  getBillingCheckoutSession,
  loadBillingPageData,
  previewBillingCheckout,
  sendBillingPaymentEvent,
  settleBillingOrder,
} from '../repository';
import {
  buildBillingEventAnalytics,
  buildBillingEventCsvDocument,
  isRecommendedPack,
  isRecommendedPlan,
  recommendBillingChange,
} from '../services';
import type {
  BillingEventAnalyticsViewModel,
  BillingCheckoutPreview,
  BillingPageData,
  PortalBillingPageProps,
} from '../types';

const emptySummary: ProjectBillingSummary = {
  project_id: '',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  exhausted: false,
};

const emptyBillingEventSummary: BillingEventSummary = {
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

type BillingSelection =
  | {
      kind: 'subscription_plan';
      target: SubscriptionPlan;
    }
  | {
      kind: 'recharge_pack';
      target: RechargePack;
    };

type OrderWorkbenchLane = 'all' | 'pending_payment' | 'failed' | 'timeline';

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;
type CsvValue = string | number | boolean | null | undefined;

function csvValue(value: CsvValue): string {
  const normalized = value == null ? '' : String(value);
  return `"${normalized.replaceAll('"', '""')}"`;
}

function downloadCsv(
  filename: string,
  headers: string[],
  rows: Array<CsvValue[]>,
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

function targetKindLabel(
  kind: BillingSelection['kind'] | PortalCommerceQuoteKind,
  t: TranslateFn,
): string {
  switch (kind) {
    case 'subscription_plan':
      return t('Subscription plan');
    case 'recharge_pack':
      return t('Recharge pack');
    default:
      return t('Offer');
  }
}

function orderStatusLabel(status: PortalCommerceOrderStatus, t: TranslateFn): string {
  switch (status) {
    case 'pending_payment':
      return t('Payment pending');
    case 'fulfilled':
      return t('Fulfilled');
    case 'canceled':
      return t('Canceled');
    case 'failed':
      return t('Failed');
    default:
      return t('Status');
  }
}

function checkoutSessionStatusLabel(
  status: PortalCommerceCheckoutSessionStatus,
  t: TranslateFn,
): string {
  switch (status) {
    case 'open':
      return t('Open');
    case 'settled':
      return t('Settled');
    case 'canceled':
      return t('Canceled');
    case 'failed':
      return t('Failed');
    case 'not_required':
      return t('Not required');
    case 'closed':
      return t('Closed');
    default:
      return t('Status');
  }
}

function checkoutMethodActionLabel(
  action: PortalCommerceCheckoutSessionMethod['action'],
  t: TranslateFn,
): string {
  switch (action) {
    case 'settle_order':
      return t('Settle order');
    case 'cancel_order':
      return t('Cancel order');
    case 'provider_handoff':
      return t('Provider handoff');
    default:
      return t('Actions');
  }
}

function checkoutMethodAvailabilityLabel(
  availability: PortalCommerceCheckoutSessionMethod['availability'],
  t: TranslateFn,
): string {
  switch (availability) {
    case 'available':
      return t('Available');
    case 'planned':
      return t('Planned');
    case 'closed':
      return t('Closed');
    default:
      return t('Status');
  }
}

function checkoutMethodAvailabilityTone(
  availability: PortalCommerceCheckoutSessionMethod['availability'],
): 'success' | 'warning' | 'secondary' {
  switch (availability) {
    case 'available':
      return 'success';
    case 'planned':
      return 'warning';
    case 'closed':
      return 'secondary';
    default:
      return 'secondary';
  }
}

function membershipDescription(
  membership: PortalCommerceMembership | null,
  t: TranslateFn,
): string {
  if (membership) {
    return t(
      '{planName} is the active workspace membership and defines the current subscription entitlement baseline.',
      {
        planName: membership.plan_name,
      },
    );
  }

  return t(
    'No active membership is recorded yet. Settle a subscription order to activate monthly entitlement posture.',
  );
}

function orderMatchesSearch(order: PortalCommerceOrder, search: string, t: TranslateFn): boolean {
  if (!search) {
    return true;
  }

  const haystack = [
    order.order_id,
    order.target_name,
    order.target_kind,
    targetKindLabel(order.target_kind, t),
    order.status,
    orderStatusLabel(order.status, t),
    order.applied_coupon_code ?? '',
    order.payable_price_label,
  ]
    .join(' ')
    .toLowerCase();

  return haystack.includes(search);
}

function selectionLabel(selection: BillingSelection | null): string | null {
  if (!selection) {
    return null;
  }

  return selection.kind === 'subscription_plan'
    ? selection.target.name
    : selection.target.label;
}

function isPendingPaymentOrder(order: PortalCommerceOrder): boolean {
  return order.status === 'pending_payment';
}

function matchesOrderLane(
  order: PortalCommerceOrder,
  lane: OrderWorkbenchLane,
): boolean {
  switch (lane) {
    case 'pending_payment':
      return isPendingPaymentOrder(order);
    case 'failed':
      return order.status === 'failed';
    case 'timeline':
      return !isPendingPaymentOrder(order);
    default:
      return true;
  }
}

function orderWorkbenchDetail(lane: OrderWorkbenchLane, t: TranslateFn): string {
  switch (lane) {
    case 'pending_payment':
      return t(
        'Pending payment queue keeps unpaid or unfulfilled orders visible until the workspace settles or cancels them.',
      );
    case 'failed':
      return t(
        'Failed payment isolates checkout attempts that need coupon, payment rail, or provider callback review.',
      );
    case 'timeline':
      return t('Order timeline shows completed or closed outcomes after checkout attempts resolve.');
    default:
      return t(
        'Switch between pending payment queue, failed payment, and order timeline without leaving the main order table.',
      );
  }
}

function checkoutModeLabel(
  session: PortalCommerceCheckoutSession | null,
  t: TranslateFn,
): string {
  switch (session?.mode) {
    case 'operator_settlement':
      return t('Operator settlement');
    case 'instant_fulfillment':
      return t('Instant fulfillment');
    default:
      return t('Closed checkout');
  }
}

function hasProviderHandoff(session: PortalCommerceCheckoutSession | null): boolean {
  return Boolean(session?.methods.some((method) => method.action === 'provider_handoff'));
}

function orderStatusTone(
  status: PortalCommerceOrder['status'],
): 'secondary' | 'default' | 'success' | 'warning' {
  switch (status) {
    case 'fulfilled':
      return 'success';
    case 'failed':
      return 'warning';
    case 'pending_payment':
      return 'default';
    default:
      return 'secondary';
  }
}

function accountingModeLabel(mode: BillingAccountingMode, t: TranslateFn): string {
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

function accountingModeTone(
  mode: BillingAccountingMode,
): 'default' | 'secondary' | 'success' | 'warning' {
  switch (mode) {
    case 'platform_credit':
      return 'default';
    case 'byok':
      return 'success';
    case 'passthrough':
      return 'warning';
    default:
      return 'secondary';
  }
}

function groupChargebackLabel(groupId: string | null | undefined, t: TranslateFn): string {
  return groupId?.trim() ? groupId : t('Unassigned');
}

function billingCapabilitySignalLabel(
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

  return signals.length
    ? signals.join(' · ')
    : t('{count} requests', { count: formatUnits(capability.request_count) });
}

function billingEventSignalLabel(event: BillingEventRecord, t: TranslateFn): string {
  const signals: string[] = [];

  if (event.total_tokens > 0) {
    signals.push(t('{count} tokens', { count: formatUnits(event.total_tokens) }));
  }
  if (event.image_count > 0) {
    signals.push(t('{count} images', { count: formatUnits(event.image_count) }));
  }
  if (event.audio_seconds > 0) {
    signals.push(t('{count} audio sec', { count: formatUnits(event.audio_seconds) }));
  }
  if (event.video_seconds > 0) {
    signals.push(t('{count} video sec', { count: formatUnits(event.video_seconds) }));
  }
  if (event.music_seconds > 0) {
    signals.push(t('{count} music sec', { count: formatUnits(event.music_seconds) }));
  }

  return signals.length
    ? signals.join(' · ')
    : t('{count} requests', { count: formatUnits(event.request_count) });
}

function capabilitySignalText(
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

  return signals.length
    ? signals.join(' / ')
    : t('{count} requests', { count: formatUnits(capability.request_count) });
}

function eventSignalText(event: BillingEventRecord, t: TranslateFn): string {
  const signals: string[] = [];

  if (event.total_tokens > 0) {
    signals.push(t('{count} tokens', { count: formatUnits(event.total_tokens) }));
  }
  if (event.image_count > 0) {
    signals.push(t('{count} images', { count: formatUnits(event.image_count) }));
  }
  if (event.audio_seconds > 0) {
    signals.push(t('{count} audio sec', { count: formatUnits(event.audio_seconds) }));
  }
  if (event.video_seconds > 0) {
    signals.push(t('{count} video sec', { count: formatUnits(event.video_seconds) }));
  }
  if (event.music_seconds > 0) {
    signals.push(t('{count} music sec', { count: formatUnits(event.music_seconds) }));
  }

  return signals.length
    ? signals.join(' / ')
    : t('{count} requests', { count: formatUnits(event.request_count) });
}

function downloadBillingEventsCsv(events: BillingEventRecord[]): void {
  const document = buildBillingEventCsvDocument(events);
  downloadCsv('sdkwork-router-billing-events.csv', document.headers, document.rows);
}

export function PortalBillingPage({ onNavigate }: PortalBillingPageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [billingEventSummary, setBillingEventSummary] = useState<BillingEventSummary>(
    emptyBillingEventSummary,
  );
  const [billingEvents, setBillingEvents] = useState<BillingEventRecord[]>([]);
  const [usageRecords, setUsageRecords] = useState<UsageRecord[]>([]);
  const [plans, setPlans] = useState<SubscriptionPlan[]>([]);
  const [packs, setPacks] = useState<RechargePack[]>([]);
  const [orders, setOrders] = useState<PortalCommerceOrder[]>([]);
  const [membership, setMembership] = useState<PortalCommerceMembership | null>(null);
  const [status, setStatus] = useState(t('Loading billing posture...'));
  const [searchQuery, setSearchQuery] = useState('');
  const [orderLane, setOrderLane] = useState<OrderWorkbenchLane>('all');
  const [checkoutOpen, setCheckoutOpen] = useState(false);
  const [checkoutSelection, setCheckoutSelection] = useState<BillingSelection | null>(null);
  const [couponCode, setCouponCode] = useState('');
  const [checkoutPreview, setCheckoutPreview] = useState<BillingCheckoutPreview | null>(null);
  const [checkoutStatus, setCheckoutStatus] = useState(
    t('Choose a plan or recharge path to price the next checkout.'),
  );
  const [previewLoading, setPreviewLoading] = useState(false);
  const [orderLoading, setOrderLoading] = useState(false);
  const [queueActionOrderId, setQueueActionOrderId] = useState<string | null>(null);
  const [queueActionType, setQueueActionType] = useState<'settle' | 'cancel' | null>(null);
  const [checkoutSession, setCheckoutSession] = useState<PortalCommerceCheckoutSession | null>(null);
  const [checkoutSessionOrderId, setCheckoutSessionOrderId] = useState<string | null>(null);
  const [providerEventOrderId, setProviderEventOrderId] = useState<string | null>(null);
  const [providerEventType, setProviderEventType] = useState<PortalCommercePaymentEventType | null>(
    null,
  );
  const [checkoutSessionStatus, setCheckoutSessionStatus] = useState(
    t('Open session from Pending payment queue to inspect the payment rail.'),
  );
  const [checkoutSessionLoading, setCheckoutSessionLoading] = useState(false);
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  const recommendation = recommendBillingChange(summary, plans, packs, usageRecords);
  const billingEventAnalytics: BillingEventAnalyticsViewModel = buildBillingEventAnalytics(
    billingEventSummary,
    billingEvents,
  );

  useEffect(() => {
    let cancelled = false;

    void loadBillingPageData()
      .then((data) => {
        if (cancelled) {
          return;
        }

        applyBillingPageData(data);
        setStatus(
          t(
            'Billing posture now combines live quota evidence, checkout state, and the payment lifecycle timeline.',
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

  function applyBillingPageData(data: BillingPageData) {
    setSummary(data.summary);
    setBillingEventSummary(data.billing_event_summary);
    setBillingEvents(data.billing_events);
    setUsageRecords(data.usage_records);
    setPlans(data.plans);
    setPacks(data.packs);
    setOrders(data.orders);
    setMembership(data.membership);
  }

  useEffect(() => {
    if (checkoutSessionOrderId && orders.some((order) => order.order_id === checkoutSessionOrderId)) {
      return;
    }

    const nextPendingOrder = orders.find((order) => isPendingPaymentOrder(order));
    if (!nextPendingOrder) {
      setCheckoutSessionOrderId(null);
      setCheckoutSession(null);
      setCheckoutSessionStatus(t('Open session from Pending payment queue to inspect the payment rail.'));
      return;
    }

    void loadCheckoutSession(nextPendingOrder.order_id);
  }, [orders, checkoutSessionOrderId]);

  async function refreshBillingPage(nextStatus?: string): Promise<void> {
    const data = await loadBillingPageData();
    applyBillingPageData(data);
    if (nextStatus) {
      setStatus(nextStatus);
    }
  }

  function currentRemainingUnits(): number | null {
    return summary.remaining_units ?? null;
  }

  function handleBillingEventExport(): void {
    if (!billingEvents.length) {
      return;
    }

    downloadBillingEventsCsv(billingEvents);
  }

  async function loadCheckoutSession(orderId: string): Promise<void> {
    setCheckoutSessionOrderId(orderId);
    setCheckoutSessionLoading(true);
    setCheckoutSessionStatus(t('Loading checkout session for {orderId}...', { orderId }));

    try {
      const session = await getBillingCheckoutSession(orderId);
      setCheckoutSession(session);
      setCheckoutSessionStatus(
        t(
          '{reference} maps this order into the current payment rail in {mode} mode.',
          {
            reference: session.reference,
            mode: checkoutModeLabel(session, t),
          },
        ),
      );
    } catch (error) {
      setCheckoutSession(null);
      setCheckoutSessionStatus(portalErrorMessage(error));
    } finally {
      setCheckoutSessionLoading(false);
    }
  }

  async function loadCheckoutPreview(
    selection: BillingSelection,
    nextCouponCode = couponCode,
  ): Promise<void> {
    setPreviewLoading(true);
    setCheckoutStatus(
      t('Loading live checkout pricing for {targetId}...', {
        targetId: selection.target.id,
      }),
    );
    setCheckoutPreview(null);

    try {
      const quote = await previewBillingCheckout({
        target_kind: selection.kind,
        target_id: selection.target.id,
        coupon_code: nextCouponCode.trim() ? nextCouponCode.trim().toUpperCase() : null,
        current_remaining_units: currentRemainingUnits(),
      });
      setCheckoutPreview(quote);
      setCheckoutStatus(
        t(
          '{targetName} is priced by the live commerce quote service and ready to create as a pending payment order.',
          {
            targetName: quote.target_name,
          },
        ),
      );
    } catch (error) {
      setCheckoutStatus(portalErrorMessage(error));
    } finally {
      setPreviewLoading(false);
    }
  }

  function openCheckout(selection: BillingSelection) {
    setCheckoutSelection(selection);
    setCheckoutOpen(true);
    setCouponCode('');
    setCheckoutPreview(null);
    void loadCheckoutPreview(selection, '');
  }

  async function handleCheckoutPreviewRefresh(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!checkoutSelection) {
      return;
    }

    await loadCheckoutPreview(checkoutSelection);
  }

  async function placeOrder() {
    if (!checkoutSelection) {
      return;
    }

    setOrderLoading(true);
    setCheckoutStatus(
      t('Creating a checkout order for {targetId}...', {
        targetId: checkoutSelection.target.id,
      }),
    );

    try {
      const order = await createBillingOrder({
        target_kind: checkoutSelection.kind,
        target_id: checkoutSelection.target.id,
        coupon_code: couponCode.trim() ? couponCode.trim().toUpperCase() : null,
      });
      await refreshBillingPage(
        t(
          '{targetName} was queued in Pending payment queue. Settle it before quota or membership changes are applied.',
          {
            targetName: order.target_name,
          },
        ),
      );
      await loadCheckoutSession(order.order_id);
      setCheckoutOpen(false);
      setCheckoutPreview(null);
      setCouponCode('');
      setCheckoutSelection(null);
    } catch (error) {
      setCheckoutStatus(portalErrorMessage(error));
    } finally {
      setOrderLoading(false);
    }
  }

  async function handleQueueAction(
    order: PortalCommerceOrder,
    action: 'settle' | 'cancel',
  ): Promise<void> {
    setQueueActionOrderId(order.order_id);
    setQueueActionType(action);
    setStatus(
      action === 'settle'
        ? t('Settling {targetName} into active workspace quota...', {
            targetName: order.target_name,
          })
        : t('Canceling {targetName} before fulfillment is applied...', {
            targetName: order.target_name,
          }),
    );

    try {
      const nextOrder =
        action === 'settle'
          ? await settleBillingOrder(order.order_id)
          : await cancelBillingOrder(order.order_id);
      await refreshBillingPage(
        action === 'settle'
          ? t('{targetName} was settled and moved into Order timeline.', {
              targetName: nextOrder.target_name,
            })
          : t('{targetName} was canceled and left out of quota fulfillment.', {
              targetName: nextOrder.target_name,
            }),
      );
      await loadCheckoutSession(nextOrder.order_id);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setQueueActionOrderId(null);
      setQueueActionType(null);
    }
  }

  async function handleProviderEvent(eventType: PortalCommercePaymentEventType): Promise<void> {
    if (!checkoutSessionOrderId) {
      return;
    }

    setProviderEventOrderId(checkoutSessionOrderId);
    setProviderEventType(eventType);
    setStatus(
      eventType === 'settled'
        ? t('Replaying provider settlement for {orderId}...', {
            orderId: checkoutSessionOrderId,
          })
        : eventType === 'failed'
          ? t('Replaying provider failure for {orderId}...', {
              orderId: checkoutSessionOrderId,
            })
          : t('Replaying provider cancellation for {orderId}...', {
              orderId: checkoutSessionOrderId,
            }),
    );

    try {
      const nextOrder = await sendBillingPaymentEvent(checkoutSessionOrderId, {
        event_type: eventType,
      });
      await refreshBillingPage(
        eventType === 'settled'
          ? t('{targetName} was settled through the provider callback flow.', {
              targetName: nextOrder.target_name,
            })
          : eventType === 'failed'
            ? t('{targetName} was marked failed and left out of fulfillment.', {
                targetName: nextOrder.target_name,
              })
            : t('{targetName} was canceled through the provider callback flow.', {
                targetName: nextOrder.target_name,
              }),
      );
      await loadCheckoutSession(nextOrder.order_id);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setProviderEventOrderId(null);
      setProviderEventType(null);
    }
  }

  const remainingUnitsLabel =
    summary.remaining_units === null || summary.remaining_units === undefined
      ? t('Unlimited')
      : formatUnits(summary.remaining_units);
  const searchedOrders = orders.filter((order) => orderMatchesSearch(order, deferredSearch, t));
  const pendingOrders = orders.filter((order) => isPendingPaymentOrder(order));
  const failedOrders = orders.filter((order) => order.status === 'failed');
  const timelineOrders = orders.filter((order) => !isPendingPaymentOrder(order));
  const visibleOrders = searchedOrders.filter((order) => matchesOrderLane(order, orderLane));
  const pendingPaymentCount = orders.filter((order) => isPendingPaymentOrder(order)).length;
  const failedPaymentCount = failedOrders.length;
  const timelineOrderCount = timelineOrders.length;
  const selectedTargetLabel = selectionLabel(checkoutSelection);
  const selectedTargetKindLabel = checkoutSelection
    ? targetKindLabel(checkoutSelection.kind, t)
    : null;
  const orderWorkbenchCopy = orderWorkbenchDetail(orderLane, t);
  const membershipPanelDescription = membershipDescription(membership, t);
  const orderEmptyTitle =
    orderLane === 'pending_payment'
      ? t('No pending payment orders for this slice')
      : orderLane === 'failed'
        ? t('No failed payment orders for this slice')
        : orderLane === 'timeline'
          ? t('No timeline orders for this slice')
          : t('No orders for this slice');
  const orderEmptyDetail = orders.length
    ? orderLane === 'pending_payment'
      ? t('Adjust the search or switch Order lane to reveal a different pending checkout.')
      : orderLane === 'failed'
        ? t('No failed payment orders match the current search or lane selection.')
        : orderLane === 'timeline'
          ? t('Adjust the search or switch Order lane to reveal a different settled or canceled order.')
          : t('Adjust the search or switch Order lane to reveal a different checkout.')
    : t('Create the first subscription or recharge checkout and it will appear here.');
  const detailCardClassName =
    'rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80';
  const catalogCardClassName =
    'rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50';
  const decisionSupportFacts = [
    {
      detail: t('Live quota still available to the current workspace.'),
      label: t('Remaining units'),
      value: remainingUnitsLabel,
    },
    {
      detail: t('Total token units already consumed.'),
      label: t('Used units'),
      value: formatUnits(summary.used_units),
    },
    {
      detail: t('Live amount visible in the billing summary.'),
      label: t('Booked amount'),
      value: formatCurrency(summary.booked_amount),
    },
    {
      detail: t('Open checkout orders that still need settlement before quota changes apply.'),
      label: t('Pending payment'),
      value: String(pendingPaymentCount),
    },
    {
      detail: t('Payment attempts that closed on the failure path and need a fresh checkout decision.'),
      label: t('Failed payment'),
      value: String(failedPaymentCount),
    },
  ];
  const membershipFacts = [
    {
      label: t('Plan'),
      value: membership?.plan_name ?? t('No membership'),
    },
    {
      label: t('Cadence'),
      value: membership?.cadence ?? t('n/a'),
    },
    {
      label: t('Included units'),
      value: membership ? formatUnits(membership.included_units) : t('n/a'),
    },
    {
      label: t('Status'),
      value: resolveMembershipStatusLabel(membership?.status, t),
    },
  ];
  const runwayFacts = [
    {
      label: t('Projected coverage'),
      value: recommendation.runway.label,
    },
    {
      label: t('Daily burn'),
      value: recommendation.runway.daily_units
        ? t('{units} / day', { units: formatUnits(recommendation.runway.daily_units) })
        : t('Needs data'),
    },
    {
      label: t('Quota posture'),
      value: summary.exhausted ? t('Exhausted') : t('Active'),
    },
  ];
  const recommendedBundleFacts = [
    {
      label: t('Subscription'),
      value: recommendation.plan?.name ?? t('None'),
    },
    {
      label: t('Recharge buffer'),
      value: recommendation.pack?.label ?? t('Optional'),
    },
    {
      label: t('Bundle posture'),
      value: recommendation.bundle.title,
    },
  ];
  const multimodalFacts = [
    {
      detail: t('Workspace-scoped billing requests recorded by the Billing 2.0 event ledger.'),
      label: t('Requests'),
      value: formatUnits(billingEventAnalytics.totals.total_request_count),
    },
    {
      detail: t('Text tokens charged across chat, responses, and other token-driven routes.'),
      label: t('Tokens'),
      value: formatUnits(billingEventAnalytics.totals.total_tokens),
    },
    {
      detail: t('Generated image count tracked by event-level metering.'),
      label: t('Images'),
      value: formatUnits(billingEventAnalytics.totals.total_image_count),
    },
    {
      detail: t('Audio seconds tracked for speech, transcription, or audio generation routes.'),
      label: t('Audio sec'),
      value: formatUnits(billingEventAnalytics.totals.total_audio_seconds),
    },
    {
      detail: t('Video seconds tracked for multimodal video generation or relay traffic.'),
      label: t('Video sec'),
      value: formatUnits(billingEventAnalytics.totals.total_video_seconds),
    },
    {
      detail: t('Music seconds tracked for music generation workloads.'),
      label: t('Music sec'),
      value: formatUnits(billingEventAnalytics.totals.total_music_seconds),
    },
  ];

  return (
    <>
      <Dialog open={checkoutOpen} onOpenChange={setCheckoutOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('Checkout preview')}</DialogTitle>
            <DialogDescription>{checkoutStatus}</DialogDescription>
          </DialogHeader>

          <form className="grid gap-4" onSubmit={(event) => void handleCheckoutPreviewRefresh(event)}>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50">
                <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                  {t('Selected offer')}
                </p>
                <h3 className="mt-3 text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                  {selectedTargetLabel ?? t('Checkout preview')}
                </h3>
                <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                  {checkoutPreview
                    ? t('{kind} / {price}', {
                        kind: targetKindLabel(checkoutPreview.target_kind, t),
                        price: checkoutPreview.payable_price_label,
                      })
                    : t('Preview the live quote before creating a checkout.')}
                </p>
                <div className="mt-4 flex flex-wrap gap-2">
                  {selectedTargetKindLabel ? (
                    <Badge variant="default">{selectedTargetKindLabel}</Badge>
                  ) : null}
                  {checkoutPreview?.applied_coupon ? (
                    <Badge variant="warning">{checkoutPreview.applied_coupon.code}</Badge>
                  ) : null}
                </div>
              </div>

              <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50">
                <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                  {t('Order impact')}
                </p>
                <div className="mt-3 grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Payable price')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview?.payable_price_label ?? t('Pending')}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Granted units')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview ? formatUnits(checkoutPreview.granted_units) : t('Pending')}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Projected remaining')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview?.projected_remaining_units === null
                      || checkoutPreview?.projected_remaining_units === undefined
                        ? remainingUnitsLabel
                        : formatUnits(checkoutPreview.projected_remaining_units)}
                    </strong>
                  </div>
                </div>
              </div>
            </div>

            <SettingsField
              description={t('Optional coupon codes are priced by the live quote service before checkout creation.')}
              label={t('Coupon code')}
              layout="vertical"
            >
              <Input
                onChange={(event: ChangeEvent<HTMLInputElement>) => {
                  setCouponCode(event.target.value);
                  setCheckoutPreview(null);
                }}
                placeholder={t('SPRING20')}
                value={couponCode}
              />
            </SettingsField>

            <DialogFooter>
              <Button onClick={() => setCheckoutOpen(false)} type="button" variant="ghost">
                {t('Close')}
              </Button>
              <Button onClick={() => onNavigate('credits')} type="button" variant="secondary">
                {t('Open redeem')}
              </Button>
              <Button disabled={previewLoading || orderLoading} type="submit" variant="secondary">
                {previewLoading ? t('Loading preview...') : t('Refresh preview')}
              </Button>
              <Button
                disabled={!checkoutPreview || previewLoading || orderLoading}
                onClick={() => void placeOrder()}
                type="button"
              >
                {orderLoading ? t('Creating checkout...') : t('Create checkout')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="grid gap-4">
        <section
          data-slot="portal-billing-toolbar"
          className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
        >
          <FilterBar>
            <FilterBarSection className="min-w-[15rem] flex-[0_1_20rem]" grow={false}>
              <FilterField
                className="w-full"
                controlClassName="min-w-0"
                label={t('Search order lifecycle')}
              >
                <SearchInput
                  value={searchQuery}
                  onChange={(event) => setSearchQuery(event.target.value)}
                  placeholder={t('Search order lifecycle')}
                />
              </FilterField>
            </FilterBarSection>
            <FilterBarSection className="min-w-[12rem] shrink-0" grow={false}>
              <FilterField className="w-full" label={t('Order lane')}>
                <Select
                  value={orderLane}
                  onValueChange={(value) => setOrderLane(value as OrderWorkbenchLane)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('Order lane')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">{t('All orders')}</SelectItem>
                    <SelectItem value="pending_payment">{t('Pending payment queue')}</SelectItem>
                    <SelectItem value="failed">{t('Failed payment')}</SelectItem>
                    <SelectItem value="timeline">{t('Order timeline')}</SelectItem>
                  </SelectContent>
                </Select>
              </FilterField>
            </FilterBarSection>
            <FilterBarActions className="gap-2.5 whitespace-nowrap">
              <Button type="button" onClick={() => onNavigate('credits')}>
                {t('Open redeem')}
              </Button>
              <Button onClick={() => onNavigate('usage')} variant="secondary">
                {t('Open usage')}
              </Button>
              <Button onClick={() => onNavigate('account')} variant="secondary">
                {t('Open account')}
              </Button>
            </FilterBarActions>
          </FilterBar>
        </section>

        <section className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
          <WorkspacePanel description={status} title={t('Decision support')}>
            <div className="grid gap-4">
              <BillingRecommendationCard recommendation={recommendation} />
              <div className="grid gap-3 md:grid-cols-2">
                {decisionSupportFacts.map((item) => (
                  <article className={detailCardClassName} key={item.label}>
                    <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                      {item.label}
                    </p>
                    <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                      {item.value}
                    </strong>
                    <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">{item.detail}</p>
                  </article>
                ))}
              </div>
            </div>
          </WorkspacePanel>

          <div className="grid gap-4">
            <WorkspacePanel
              description={membershipPanelDescription}
              title={t('Active membership')}
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                {membershipFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <span>{item.label}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                  </div>
                ))}
              </div>
            </WorkspacePanel>

            <WorkspacePanel
              description={recommendation.runway.detail}
              title={t('Estimated runway')}
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                {runwayFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <span>{item.label}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                  </div>
                ))}
              </div>
            </WorkspacePanel>

            <WorkspacePanel
              description={recommendation.bundle.detail}
              title={t('Recommended bundle')}
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                {recommendedBundleFacts.map((item) => (
                  <div className="flex items-center justify-between gap-3" key={item.label}>
                    <span>{item.label}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{item.value}</strong>
                  </div>
                ))}
              </div>
            </WorkspacePanel>
          </div>
        </section>

        <section className="grid gap-4 xl:grid-cols-[1.15fr_0.85fr]">
          <WorkspacePanel
            actions={(
              <div className="flex flex-wrap gap-2">
                <Badge variant="default">
                  {t('{count} billing events', {
                    count: formatUnits(billingEventAnalytics.totals.total_events),
                  })}
                </Badge>
                <Badge variant="secondary">
                  {t('{amount} customer charge', {
                    amount: formatCurrency(billingEventAnalytics.totals.total_customer_charge),
                  })}
                </Badge>
              </div>
            )}
            description={t(
              'Billing event analytics turns route-level metering into multimodal, group, and accounting evidence for commercial reviews.',
            )}
            title={t('Billing event analytics')}
          >
            {billingEventAnalytics.totals.total_events ? (
              <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                {multimodalFacts.map((item) => (
                  <article className={detailCardClassName} key={item.label}>
                    <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                      {item.label}
                    </p>
                    <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                      {item.value}
                    </strong>
                    <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">{item.detail}</p>
                  </article>
                ))}
              </div>
            ) : (
              <EmptyState
                description={t(
                  'Billing event analytics appears after the workspace records routed usage, multimodal traffic, or chargeback activity.',
                )}
                title={t('No billing event analytics yet')}
              />
            )}
          </WorkspacePanel>

          <div className="grid gap-4">
            <WorkspacePanel
              description={t(
                'Top capabilities show where customer charge is landing across text, image, audio, video, and music traffic.',
              )}
              title={t('Capability mix')}
            >
              {billingEventAnalytics.top_capabilities.length ? (
                <div className="grid gap-3">
                  {billingEventAnalytics.top_capabilities.map((item) => (
                    <article className={catalogCardClassName} key={item.capability}>
                      <div className="flex flex-wrap items-start justify-between gap-3">
                        <div className="grid gap-2">
                          <div className="flex flex-wrap gap-2">
                            <Badge variant="default">{titleCaseToken(item.capability)}</Badge>
                            <Badge variant="secondary">
                              {t('{count} events', { count: formatUnits(item.event_count) })}
                            </Badge>
                          </div>
                          <strong className="text-base text-zinc-950 dark:text-zinc-50">
                            {capabilitySignalText(item, t)}
                          </strong>
                          <p className="text-sm text-zinc-600 dark:text-zinc-300">
                            {t('{count} requests routed through this capability slice.', {
                              count: formatUnits(item.request_count),
                            })}
                          </p>
                        </div>
                        <div className="grid gap-1 text-right text-sm">
                          <span className="text-zinc-500 dark:text-zinc-400">
                            {t('Customer charge')}
                          </span>
                          <strong className="text-zinc-950 dark:text-zinc-50">
                            {formatCurrency(item.total_customer_charge)}
                          </strong>
                          <span className="text-zinc-500 dark:text-zinc-400">
                            {t('Upstream cost')}: {formatCurrency(item.total_upstream_cost)}
                          </span>
                        </div>
                      </div>
                    </article>
                  ))}
                </div>
              ) : (
                <EmptyState
                  description={t(
                    'Capability charge mix will appear after the workspace records billing events.',
                  )}
                  title={t('No capability mix yet')}
                />
              )}
            </WorkspacePanel>

            <WorkspacePanel
              description={t(
                'API key group chargeback keeps environment and tenant-level billing accountability visible without leaving the billing workspace.',
              )}
              title={t('API key group chargeback')}
            >
              {billingEventAnalytics.group_chargeback.length ? (
                <div className="grid gap-3">
                  {billingEventAnalytics.group_chargeback.map((item) => (
                    <article
                      className="flex items-center justify-between gap-3 border-b border-zinc-200/80 pb-3 last:border-b-0 last:pb-0 dark:border-zinc-800/80"
                      key={item.api_key_group_id ?? 'unassigned'}
                    >
                      <div className="grid gap-2">
                        <div className="flex flex-wrap gap-2">
                          <Badge variant={item.api_key_group_id ? 'default' : 'secondary'}>
                            {groupChargebackLabel(item.api_key_group_id, t)}
                          </Badge>
                          <Badge variant="secondary">
                            {t('{count} requests', { count: formatUnits(item.request_count) })}
                          </Badge>
                        </div>
                        <p className="text-sm text-zinc-600 dark:text-zinc-300">
                          {t('{count} billing events contributed to this group chargeback slice.', {
                            count: formatUnits(item.event_count),
                          })}
                        </p>
                      </div>
                      <div className="grid gap-1 text-right text-sm">
                        <strong className="text-zinc-950 dark:text-zinc-50">
                          {formatCurrency(item.total_customer_charge)}
                        </strong>
                        <span className="text-zinc-500 dark:text-zinc-400">
                          {t('Upstream cost')}: {formatCurrency(item.total_upstream_cost)}
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              ) : (
                <EmptyState
                  description={t(
                    'Group chargeback will appear once billing events are attributed to API key groups.',
                  )}
                  title={t('No group chargeback yet')}
                />
              )}
            </WorkspacePanel>

            <WorkspacePanel
              description={t(
                'Accounting mode mix separates platform credit, BYOK, and passthrough consumption in one billing review slice.',
              )}
              title={t('Accounting mode mix')}
            >
              {billingEventAnalytics.accounting_mode_mix.length ? (
                <div className="grid gap-3">
                  {billingEventAnalytics.accounting_mode_mix.map((item) => (
                    <article
                      className="flex items-center justify-between gap-3 border-b border-zinc-200/80 pb-3 last:border-b-0 last:pb-0 dark:border-zinc-800/80"
                      key={item.accounting_mode}
                    >
                      <div className="grid gap-2">
                        <div className="flex flex-wrap gap-2">
                          <Badge variant={accountingModeTone(item.accounting_mode)}>
                            {accountingModeLabel(item.accounting_mode, t)}
                          </Badge>
                          <Badge variant="secondary">
                            {t('{count} requests', { count: formatUnits(item.request_count) })}
                          </Badge>
                        </div>
                        <p className="text-sm text-zinc-600 dark:text-zinc-300">
                          {t('{count} billing events used this accounting mode.', {
                            count: formatUnits(item.event_count),
                          })}
                        </p>
                      </div>
                      <div className="grid gap-1 text-right text-sm">
                        <strong className="text-zinc-950 dark:text-zinc-50">
                          {formatCurrency(item.total_customer_charge)}
                        </strong>
                        <span className="text-zinc-500 dark:text-zinc-400">
                          {t('Upstream cost')}: {formatCurrency(item.total_upstream_cost)}
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              ) : (
                <EmptyState
                  description={t(
                    'Accounting mode evidence appears after billing events record platform credit, BYOK, or passthrough traffic.',
                  )}
                  title={t('No accounting mode mix yet')}
                />
              )}
            </WorkspacePanel>

            <WorkspacePanel
              description={t(
                'Inspect the compiled routing evidence for this workspace after policy, project defaults, and API key group profile overlays are combined.',
              )}
              title={t('Routing evidence')}
            >
              <div className="grid gap-3 md:grid-cols-3">
                <article className={detailCardClassName}>
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    {t('Applied routing profile')}
                  </p>
                  <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                    {formatUnits(billingEventAnalytics.routing_evidence.events_with_profile)}
                  </strong>
                  <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {t('{count} billing events retained an applied profile id.', {
                      count: formatUnits(billingEventAnalytics.routing_evidence.events_with_profile),
                    })}
                  </p>
                </article>

                <article className={detailCardClassName}>
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    {t('Compiled snapshot')}
                  </p>
                  <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                    {formatUnits(
                      billingEventAnalytics.routing_evidence.events_with_compiled_snapshot,
                    )}
                  </strong>
                  <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {t('{count} billing events retained a compiled snapshot id.', {
                      count: formatUnits(
                        billingEventAnalytics.routing_evidence.events_with_compiled_snapshot,
                      ),
                    })}
                  </p>
                </article>

                <article className={detailCardClassName}>
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    {t('Fallback reason')}
                  </p>
                  <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
                    {formatUnits(
                      billingEventAnalytics.routing_evidence.events_with_fallback_reason,
                    )}
                  </strong>
                  <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {t(
                      'Fallback reasoning is preserved so operators can distinguish degraded routing from normal preference selection.',
                    )}
                  </p>
                </article>
              </div>
            </WorkspacePanel>
          </div>
        </section>

        <section>
          <WorkspacePanel
            actions={(
              <div className="flex flex-wrap gap-2">
                <Badge variant="secondary">
                  {t('{count} capabilities', {
                    count: formatUnits(billingEventSummary.capability_count),
                  })}
                </Badge>
                <Badge variant="secondary">
                  {t('{count} groups', {
                    count: formatUnits(billingEventSummary.group_count),
                  })}
                </Badge>
                <Button
                  disabled={!billingEvents.length}
                  onClick={handleBillingEventExport}
                  type="button"
                  variant="secondary"
                >
                  {t('Export billing events CSV')}
                </Button>
              </div>
            )}
            description={t(
              'Recent billing events keep multimodal chargeback, provider cost, and routing evidence in one finance-ready table.',
            )}
            title={t('Recent billing events')}
          >
            <DataTable
              columns={[
                {
                  id: 'event',
                  header: t('Event'),
                  cell: (row) => row.event_id,
                },
                {
                  id: 'capability',
                  header: t('Capability'),
                  cell: (row) => titleCaseToken(row.capability),
                },
                {
                  id: 'group',
                  header: t('Group'),
                  cell: (row) => groupChargebackLabel(row.api_key_group_id, t),
                },
                {
                  id: 'signals',
                  header: t('Signals'),
                  cell: (row) => eventSignalText(row, t),
                },
                {
                  id: 'accounting',
                  header: t('Accounting'),
                  cell: (row) => (
                    <Badge variant={accountingModeTone(row.accounting_mode)}>
                      {accountingModeLabel(row.accounting_mode, t)}
                    </Badge>
                  ),
                },
                {
                  id: 'applied_routing_profile_id',
                  header: t('Applied routing profile'),
                  cell: (row) => (
                    <div className="max-w-[12rem] truncate">
                      {row.applied_routing_profile_id ?? t('Not recorded')}
                    </div>
                  ),
                },
                {
                  id: 'compiled_routing_snapshot_id',
                  header: t('Compiled snapshot'),
                  cell: (row) => (
                    <div className="max-w-[12rem] truncate">
                      {row.compiled_routing_snapshot_id ?? t('Not recorded')}
                    </div>
                  ),
                },
                {
                  id: 'fallback_reason',
                  header: t('Fallback reason'),
                  cell: (row) => (
                    <div className="max-w-[14rem] truncate">
                      {row.fallback_reason ?? t('None')}
                    </div>
                  ),
                },
                {
                  id: 'customer_charge',
                  header: t('Customer charge'),
                  cell: (row) => formatCurrency(row.customer_charge),
                },
                {
                  id: 'upstream_cost',
                  header: t('Upstream cost'),
                  cell: (row) => formatCurrency(row.upstream_cost),
                },
                {
                  id: 'time',
                  header: t('Created'),
                  cell: (row) => formatDateTime(row.created_at_ms),
                },
              ]}
              emptyState={(
                <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                  <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {t('No recent billing events yet')}
                  </strong>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">
                    {t(
                      'Recent billing events appear once the workspace records billable routed traffic.',
                    )}
                  </p>
                </div>
              )}
              getRowId={(row) => row.event_id}
              rows={billingEventAnalytics.recent_events}
            />
          </WorkspacePanel>
        </section>

        <section className="grid gap-4 xl:grid-cols-2">
          <WorkspacePanel
            description={t('Choose the monthly posture that best matches expected gateway demand.')}
            title={t('Plan catalog')}
          >
            <div className="grid gap-3">
              {plans.map((plan) => (
                <article
                  key={plan.id}
                  className={catalogCardClassName}
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="grid gap-2">
                      <div className="flex flex-wrap gap-2">
                        <Badge variant={isRecommendedPlan(plan, recommendation) ? 'success' : 'secondary'}>
                          {plan.name}
                        </Badge>
                        <Badge variant="default">{plan.price_label}</Badge>
                      </div>
                      <strong className="text-lg text-zinc-950 dark:text-zinc-50">
                        {t('{units} included units', {
                          units: formatUnits(plan.included_units),
                        })}
                      </strong>
                      <p className="text-sm text-zinc-600 dark:text-zinc-300">{plan.highlight}</p>
                    </div>
                    <Button
                      type="button"
                      onClick={() =>
                        openCheckout({
                          kind: 'subscription_plan',
                          target: plan,
                        })
                      }
                    >
                      {plan.cta}
                    </Button>
                  </div>
                </article>
              ))}
            </div>
          </WorkspacePanel>

          <WorkspacePanel
            description={t('Use top-ups to restore headroom without changing the base plan.')}
            title={t('Recharge packs')}
          >
            <div className="grid gap-3">
              {packs.map((pack) => (
                <article
                  key={pack.id}
                  className={catalogCardClassName}
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="grid gap-2">
                      <div className="flex flex-wrap gap-2">
                        <Badge variant={isRecommendedPack(pack, recommendation) ? 'warning' : 'secondary'}>
                          {pack.label}
                        </Badge>
                        <Badge variant="default">{pack.price_label}</Badge>
                      </div>
                      <strong className="text-lg text-zinc-950 dark:text-zinc-50">
                        {t('{units} units', {
                          units: formatUnits(pack.points),
                        })}
                      </strong>
                      <p className="text-sm text-zinc-600 dark:text-zinc-300">{pack.note}</p>
                    </div>
                    <Button
                      type="button"
                      onClick={() =>
                        openCheckout({
                          kind: 'recharge_pack',
                          target: pack,
                        })
                      }
                    >
                      {t('Create checkout')}
                    </Button>
                  </div>
                </article>
              ))}
            </div>
          </WorkspacePanel>
        </section>

        <section>
          <WorkspacePanel
            actions={(
              <div className="flex flex-wrap gap-2">
                <Badge variant={orderLane === 'all' ? 'default' : 'secondary'}>
                  {t('{count} all orders', { count: orders.length })}
                </Badge>
                <Badge variant={orderLane === 'pending_payment' ? 'default' : 'secondary'}>
                  {t('{count} Pending payment queue', { count: pendingPaymentCount })}
                </Badge>
                <Badge variant={orderLane === 'failed' ? 'warning' : 'secondary'}>
                  {t('{count} Failed payment', { count: failedPaymentCount })}
                </Badge>
                <Badge variant={orderLane === 'timeline' ? 'success' : 'secondary'}>
                  {t('{count} Order timeline', { count: timelineOrderCount })}
                </Badge>
              </div>
            )}
            description={orderWorkbenchCopy}
            title={t('Order workbench')}
          >
            <div className="grid gap-4">
              <DataTable
                columns={[
                  {
                    id: 'offer',
                    header: t('Offer'),
                    cell: (row) => row.target_name,
                  },
                  {
                    id: 'kind',
                    header: t('Kind'),
                    cell: (row) => targetKindLabel(row.target_kind, t),
                  },
                  {
                    id: 'coupon',
                    header: t('Coupon'),
                    cell: (row) => row.applied_coupon_code ?? t('None'),
                  },
                  {
                    id: 'payable',
                    header: t('Payable'),
                    cell: (row) => row.payable_price_label,
                  },
                  {
                    id: 'units',
                    header: t('Granted units'),
                    cell: (row) => formatUnits(row.granted_units + row.bonus_units),
                  },
                  {
                    id: 'status',
                    header: t('Status'),
                    cell: (row) => (
                      <Badge variant={orderStatusTone(row.status)}>
                        {orderStatusLabel(row.status, t)}
                      </Badge>
                    ),
                  },
                  {
                    id: 'time',
                    header: t('Created'),
                    cell: (row) => formatDateTime(row.created_at_ms),
                  },
                  {
                    id: 'actions',
                    header: t('Actions'),
                    cell: (row) => (
                      <div className="flex flex-wrap gap-2">
                        <Button
                          disabled={checkoutSessionLoading}
                          onClick={() => void loadCheckoutSession(row.order_id)}
                          variant="secondary"
                        >
                          {checkoutSessionLoading && checkoutSessionOrderId === row.order_id
                            ? t('Loading session...')
                            : t('Open session')}
                        </Button>
                        {isPendingPaymentOrder(row) ? (
                          <>
                            <Button
                              disabled={queueActionOrderId !== null}
                              onClick={() => void handleQueueAction(row, 'settle')}
                              variant="primary"
                            >
                              {queueActionOrderId === row.order_id && queueActionType === 'settle'
                                ? t('Settling...')
                                : t('Settle order')}
                            </Button>
                            <Button
                              disabled={queueActionOrderId !== null}
                              onClick={() => void handleQueueAction(row, 'cancel')}
                              variant="secondary"
                            >
                              {queueActionOrderId === row.order_id && queueActionType === 'cancel'
                                ? t('Canceling...')
                                : t('Cancel order')}
                            </Button>
                          </>
                        ) : null}
                      </div>
                    ),
                  },
                ]}
                emptyState={(
                  <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                    <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                      {orderEmptyTitle}
                    </strong>
                    <p className="text-sm text-zinc-500 dark:text-zinc-400">
                      {orderEmptyDetail}
                    </p>
                  </div>
                )}
                getRowId={(row) => row.order_id}
                rows={visibleOrders}
              />
            </div>
          </WorkspacePanel>
        </section>

        <section className="grid gap-4 xl:grid-cols-[0.95fr_1.05fr]">
          <WorkspacePanel
            description={checkoutSessionStatus}
            title={t('Checkout session')}
          >
            {checkoutSession ? (
              <div className="grid gap-4">
                <div className="grid gap-3 md:grid-cols-2 text-sm text-zinc-600 dark:text-zinc-300">
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Reference')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutSession.reference}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Payment rail')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutSession.provider}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Checkout mode')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutModeLabel(checkoutSession, t)}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>{t('Session status')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutSessionStatusLabel(checkoutSession.session_status, t)}
                    </strong>
                  </div>
                </div>

                <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    {t('Guidance')}
                  </p>
                  <p className="mt-3 text-sm text-zinc-600 dark:text-zinc-300">
                    {checkoutSession.guidance}
                  </p>
                </div>

                <div className="grid gap-3">
                  {checkoutSession.methods.length ? (
                    checkoutSession.methods.map((method) => (
                      <article
                        key={method.id}
                        className={catalogCardClassName}
                      >
                        <div className="flex flex-wrap items-start justify-between gap-3">
                          <div className="grid gap-2">
                            <div className="flex flex-wrap gap-2">
                              <Badge variant="default">{method.label}</Badge>
                              <Badge variant={checkoutMethodAvailabilityTone(method.availability)}>
                                {checkoutMethodAvailabilityLabel(method.availability, t)}
                              </Badge>
                            </div>
                            <p className="text-sm text-zinc-600 dark:text-zinc-300">{method.detail}</p>
                          </div>
                          <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                            {checkoutMethodActionLabel(method.action, t)}
                          </strong>
                        </div>
                      </article>
                    ))
                  ) : (
                    <EmptyState
                      description={t('This checkout session is already closed, so there are no remaining payment actions.')}
                      title={t('No checkout methods remain')}
                    />
                  )}
                </div>

                    {hasProviderHandoff(checkoutSession) ? (
                  <div className={detailCardClassName}>
                    <div className="flex flex-wrap items-start justify-between gap-3">
                      <div className="grid gap-2">
                        <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                          {t('Provider callbacks')}
                        </p>
                        <p className="text-sm text-zinc-600 dark:text-zinc-300">
                          {t(
                            'Simulate hosted payment callbacks so server mode can rehearse settlement, failure, and cancellation before a live payment provider is connected.',
                          )}
                        </p>
                      </div>
                      <Badge variant="warning">{t('Provider webhooks')}</Badge>
                    </div>
                    <div className="mt-4 flex flex-wrap gap-2">
                      <Button
                        disabled={providerEventOrderId !== null}
                        onClick={() => void handleProviderEvent('settled')}
                        variant="primary"
                      >
                        {providerEventOrderId === checkoutSessionOrderId
                        && providerEventType === 'settled'
                          ? t('Applying settlement...')
                          : t('Simulate provider settlement')}
                      </Button>
                      <Button
                        disabled={providerEventOrderId !== null}
                        onClick={() => void handleProviderEvent('failed')}
                        variant="secondary"
                      >
                        {providerEventOrderId === checkoutSessionOrderId
                        && providerEventType === 'failed'
                          ? t('Applying failure...')
                          : t('Simulate provider failure')}
                      </Button>
                      <Button
                        disabled={providerEventOrderId !== null}
                        onClick={() => void handleProviderEvent('canceled')}
                        variant="secondary"
                      >
                        {providerEventOrderId === checkoutSessionOrderId
                        && providerEventType === 'canceled'
                          ? t('Applying cancel...')
                          : t('Simulate provider cancel')}
                      </Button>
                    </div>
                  </div>
                ) : null}
              </div>
            ) : (
                <EmptyState
                  description={t('Open session from Pending payment queue to inspect the checkout flow for the selected order.')}
                  title={t('No checkout session selected')}
                />
              )}
          </WorkspacePanel>

          <WorkspacePanel
            description={t('One checkout flow keeps local desktop mode and server-hosted payment providers aligned under one payment rail.')}
            title={t('Payment rail')}
          >
            <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
              <div className="flex items-center justify-between gap-3">
                <span>{t('Local desktop mode')}</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {t('Operator settlement')}
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span>{t('Server mode handoff')}</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {t('Provider handoff')}
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span>{t('Current selected reference')}</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {checkoutSession?.reference ?? t('Awaiting pending order')}
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span>{t('Payable price')}</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {checkoutSession?.payable_price_label ?? t('n/a')}
                </strong>
              </div>
            </div>
          </WorkspacePanel>
        </section>
      </div>
    </>
  );
}





