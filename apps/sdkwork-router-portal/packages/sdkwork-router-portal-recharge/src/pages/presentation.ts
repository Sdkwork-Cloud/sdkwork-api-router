import type { PortalCommerceOrder } from 'sdkwork-router-portal-types';

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;

type HandoffMode = 'billing_handoff' | 'create_order';

export interface PortalRechargePrimaryActionState {
  mode: HandoffMode;
  disabled: boolean;
  label: string;
}

export interface PortalRechargeMobileActionState {
  mode: HandoffMode;
  amountLabel: string;
  eyebrow: string;
  supportingText: string;
  buttonLabel: string;
  disabled: boolean;
}

export type PortalRechargeFlowTrackerStepStatus =
  | 'current'
  | 'complete'
  | 'pending'
  | 'attention';

export interface PortalRechargeFlowTrackerStep {
  id: 'choose_amount' | 'create_order' | 'complete_payment';
  label: string;
  detail: string;
  status: PortalRechargeFlowTrackerStepStatus;
}

export interface PortalRechargeFlowTrackerState {
  title: string;
  steps: PortalRechargeFlowTrackerStep[];
}

export interface PortalRechargePendingPaymentSpotlightLike {
  latestOrder: Pick<PortalCommerceOrder, 'order_id'>;
}

function resolvePortalRechargeActionDisabled(input: {
  postOrderHandoffActive: boolean;
  quoteLoading: boolean;
  createLoading: boolean;
  hasSelection: boolean;
}) {
  if (input.postOrderHandoffActive) {
    return false;
  }

  return input.quoteLoading || input.createLoading || !input.hasSelection;
}

export function resolvePortalRechargePostOrderHandoffActive(input: {
  lastCreatedOrderId: string | null;
  pendingPaymentSpotlight: PortalRechargePendingPaymentSpotlightLike | null;
}) {
  const { lastCreatedOrderId, pendingPaymentSpotlight } = input;
  return Boolean(
    lastCreatedOrderId
      && pendingPaymentSpotlight
      && pendingPaymentSpotlight.latestOrder.order_id === lastCreatedOrderId,
  );
}

export function buildPortalRechargePrimaryActionState(input: {
  postOrderHandoffActive: boolean;
  quoteLoading: boolean;
  createLoading: boolean;
  hasSelection: boolean;
  t: TranslateFn;
}): PortalRechargePrimaryActionState {
  const { postOrderHandoffActive, quoteLoading, createLoading, hasSelection, t } = input;

  if (postOrderHandoffActive) {
    return {
      mode: 'billing_handoff',
      disabled: false,
      label: t('Continue in billing'),
    };
  }

  return {
    mode: 'create_order',
    disabled: resolvePortalRechargeActionDisabled({
      postOrderHandoffActive,
      quoteLoading,
      createLoading,
      hasSelection,
    }),
    label: createLoading ? t('Creating...') : t('Create recharge order'),
  };
}

export function buildPortalRechargeMobileActionState(input: {
  postOrderHandoffActive: boolean;
  selectedAmountLabel: string;
  grantedUnitsLabel: string;
  quoteLoading: boolean;
  createLoading: boolean;
  hasSelection: boolean;
  t: TranslateFn;
}): PortalRechargeMobileActionState {
  const {
    postOrderHandoffActive,
    selectedAmountLabel,
    grantedUnitsLabel,
    quoteLoading,
    createLoading,
    hasSelection,
    t,
  } = input;

  if (postOrderHandoffActive) {
    return {
      mode: 'billing_handoff',
      amountLabel: selectedAmountLabel,
      eyebrow: t('Order ready for payment'),
      supportingText: t('Continue in billing'),
      buttonLabel: t('Continue in billing'),
      disabled: false,
    };
  }

  return {
    mode: 'create_order',
    amountLabel: selectedAmountLabel,
    eyebrow: t('Create order in billing'),
    supportingText: t('Granted units: {units}', { units: grantedUnitsLabel }),
    buttonLabel: createLoading ? t('Creating...') : t('Create order in billing'),
    disabled: resolvePortalRechargeActionDisabled({
      postOrderHandoffActive,
      quoteLoading,
      createLoading,
      hasSelection,
    }),
  };
}

export function buildPortalRechargeFlowTrackerState(input: {
  hasSelection: boolean;
  hasQuote: boolean;
  postOrderHandoffActive: boolean;
  pendingPaymentCount: number;
  t: TranslateFn;
}): PortalRechargeFlowTrackerState {
  const {
    hasSelection,
    hasQuote,
    postOrderHandoffActive,
    pendingPaymentCount,
    t,
  } = input;

  const selectionReady = hasSelection && hasQuote;

  return {
    title: t('Funding flow'),
    steps: [
      {
        id: 'choose_amount',
        label: t('Choose amount'),
        detail: selectionReady
          ? t('Live quote ready for the selected recharge path.')
          : t('Pick a package or custom amount to unlock the live quote.'),
        status: selectionReady ? 'complete' : 'current',
      },
      {
        id: 'create_order',
        label: t('Create order'),
        detail: postOrderHandoffActive
          ? t('Recharge order created and recorded.')
          : selectionReady
            ? t('Create the recharge order to hand settlement into billing.')
            : t('Order creation unlocks once an amount is ready.'),
        status: postOrderHandoffActive
          ? 'complete'
          : selectionReady
            ? 'current'
            : 'pending',
      },
      {
        id: 'complete_payment',
        label: t('Complete payment in billing'),
        detail: postOrderHandoffActive
          ? t('Continue in billing to finish payment capture.')
          : pendingPaymentCount > 0
            ? t('Pending settlement queue already needs billing follow-up.')
            : t('Settlement starts after the recharge order is created.'),
        status: postOrderHandoffActive
          ? 'current'
          : pendingPaymentCount > 0
            ? 'attention'
            : 'pending',
      },
    ],
  };
}
