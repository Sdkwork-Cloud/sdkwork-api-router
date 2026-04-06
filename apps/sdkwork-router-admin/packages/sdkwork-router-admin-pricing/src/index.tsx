import { useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Label,
  ManagementWorkbench,
  StatCard,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import {
  commercialPricingChargeUnitLabel,
  commercialPricingDisplayUnit,
  commercialPricingMethodLabel,
  useAdminI18n,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';
import type {
  AdminPageProps,
  CommercialPricingChargeUnit,
  CommercialPricingLifecycleSynchronizationReport,
  CommercialPricingMethod,
  CommercialPricingPlanCreateInput,
  CommercialPricingPlanRecord,
  CommercialPricingRateCreateInput,
  CommercialPricingRateRecord,
} from 'sdkwork-router-admin-types';

type PricingTranslateFn = (message: string, values?: Record<string, string | number>) => string;

function buildChargeUnitOptions(t: PricingTranslateFn): Array<{
  value: CommercialPricingChargeUnit;
  label: string;
  detail: string;
}> {
  return [
    {
      value: 'input_token',
      label: commercialPricingChargeUnitLabel('input_token', t),
      detail: t('Prompt and ingestion token pricing.'),
    },
    {
      value: 'output_token',
      label: commercialPricingChargeUnitLabel('output_token', t),
      detail: t('Completion and generation token pricing.'),
    },
    {
      value: 'cache_read_token',
      label: commercialPricingChargeUnitLabel('cache_read_token', t),
      detail: t('Read-side cached token pricing.'),
    },
    {
      value: 'cache_write_token',
      label: commercialPricingChargeUnitLabel('cache_write_token', t),
      detail: t('Write-side cache population pricing.'),
    },
    {
      value: 'request',
      label: commercialPricingChargeUnitLabel('request', t),
      detail: t('Flat request admission or invocation pricing.'),
    },
    {
      value: 'image',
      label: commercialPricingChargeUnitLabel('image', t),
      detail: t('Per-image generation pricing.'),
    },
    {
      value: 'audio_second',
      label: commercialPricingChargeUnitLabel('audio_second', t),
      detail: t('Per-second audio processing pricing.'),
    },
    {
      value: 'audio_minute',
      label: commercialPricingChargeUnitLabel('audio_minute', t),
      detail: t('Minute-based audio processing pricing.'),
    },
    {
      value: 'video_second',
      label: commercialPricingChargeUnitLabel('video_second', t),
      detail: t('Per-second video generation pricing.'),
    },
    {
      value: 'video_minute',
      label: commercialPricingChargeUnitLabel('video_minute', t),
      detail: t('Minute-based video generation pricing.'),
    },
    {
      value: 'music_track',
      label: commercialPricingChargeUnitLabel('music_track', t),
      detail: t('Per-track music generation pricing.'),
    },
    {
      value: 'character',
      label: commercialPricingChargeUnitLabel('character', t),
      detail: t('Per-character text or OCR pricing.'),
    },
    {
      value: 'storage_mb_day',
      label: commercialPricingChargeUnitLabel('storage_mb_day', t),
      detail: t('Storage footprint pricing over time.'),
    },
    {
      value: 'tool_call',
      label: commercialPricingChargeUnitLabel('tool_call', t),
      detail: t('Per tool or function invocation pricing.'),
    },
    {
      value: 'unit',
      label: commercialPricingChargeUnitLabel('unit', t),
      detail: t('Fallback commercial unit when no specialized unit applies.'),
    },
  ];
}

function buildPricingMethodOptions(t: PricingTranslateFn): Array<{
  value: CommercialPricingMethod;
  label: string;
  detail: string;
}> {
  return [
    {
      value: 'per_unit',
      label: commercialPricingMethodLabel('per_unit', t),
      detail: t('Quantity times unit price.'),
    },
    {
      value: 'flat',
      label: commercialPricingMethodLabel('flat', t),
      detail: t('One fixed charge per matched operation.'),
    },
    {
      value: 'step',
      label: commercialPricingMethodLabel('step', t),
      detail: t('Charge by normalized quantity steps.'),
    },
    {
      value: 'included_then_per_unit',
      label: commercialPricingMethodLabel('included_then_per_unit', t),
      detail: t('Burn included usage before overage pricing.'),
    },
  ];
}

function buildRoundingModeOptions(t: PricingTranslateFn): Array<{
  value: CommercialPricingRateCreateInput['rounding_mode'];
  label: string;
}> {
  return [
    { value: 'none', label: t('No rounding') },
    { value: 'ceil', label: t('Round up') },
    { value: 'floor', label: t('Round down') },
    { value: 'half_up', label: t('Round half up') },
  ];
}

function parseNumber(value: string, fallback: number) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function parseDateTimeLocalInputValue(value: string): number | null {
  const normalizedValue = value.trim();
  if (!normalizedValue) {
    return null;
  }

  const parsed = new Date(normalizedValue).getTime();
  return Number.isFinite(parsed) ? parsed : null;
}

function toDateTimeLocalInputValue(value?: number | null): string {
  if (!value || value <= 0) {
    return '';
  }

  const localTimestamp = value - new Date(value).getTimezoneOffset() * 60_000;
  return new Date(localTimestamp).toISOString().slice(0, 16);
}

function buildPlanDraft(snapshot: AdminPageProps['snapshot']): CommercialPricingPlanCreateInput {
  const firstPlan = snapshot.commercialPricingPlans[0];
  return {
    tenant_id: firstPlan?.tenant_id ?? 1001,
    organization_id: firstPlan?.organization_id ?? 0,
    plan_code: '',
    plan_version: 1,
    display_name: '',
    currency_code: firstPlan?.currency_code ?? 'USD',
    credit_unit_code: firstPlan?.credit_unit_code ?? 'credit',
    status: 'draft',
    effective_from_ms: firstPlan?.effective_from_ms ?? 0,
    effective_to_ms: firstPlan?.effective_to_ms ?? null,
  };
}

function buildPlanDraftFromRecord(
  plan: CommercialPricingPlanRecord,
): CommercialPricingPlanCreateInput {
  return {
    tenant_id: plan.tenant_id,
    organization_id: plan.organization_id,
    plan_code: plan.plan_code,
    plan_version: plan.plan_version,
    display_name: plan.display_name,
    currency_code: plan.currency_code,
    credit_unit_code: plan.credit_unit_code,
    status: plan.status,
    effective_from_ms: plan.effective_from_ms,
    effective_to_ms: plan.effective_to_ms ?? null,
  };
}

function buildRateDraft(snapshot: AdminPageProps['snapshot']): CommercialPricingRateCreateInput {
  const firstPlan = snapshot.commercialPricingPlans[0];
  return {
    tenant_id: firstPlan?.tenant_id ?? 1001,
    organization_id: firstPlan?.organization_id ?? 0,
    pricing_plan_id: firstPlan?.pricing_plan_id ?? 0,
    metric_code: '',
    capability_code: '',
    model_code: '',
    provider_code: '',
    charge_unit: 'input_token',
    pricing_method: 'per_unit',
    quantity_step: 1000000,
    unit_price: 0,
    display_price_unit: 'USD / 1M input tokens',
    minimum_billable_quantity: 0,
    minimum_charge: 0,
    rounding_increment: 1,
    rounding_mode: 'ceil',
    included_quantity: 0,
    priority: 100,
    notes: '',
    status: 'draft',
  };
}

function buildRateDraftFromRecord(
  rate: CommercialPricingRateRecord,
): CommercialPricingRateCreateInput {
  return {
    tenant_id: rate.tenant_id,
    organization_id: rate.organization_id,
    pricing_plan_id: rate.pricing_plan_id,
    metric_code: rate.metric_code,
    capability_code: rate.capability_code ?? '',
    model_code: rate.model_code ?? '',
    provider_code: rate.provider_code ?? '',
    charge_unit: rate.charge_unit,
    pricing_method: rate.pricing_method,
    quantity_step: rate.quantity_step,
    unit_price: rate.unit_price,
    display_price_unit: rate.display_price_unit,
    minimum_billable_quantity: rate.minimum_billable_quantity,
    minimum_charge: rate.minimum_charge,
    rounding_increment: rate.rounding_increment,
    rounding_mode: rate.rounding_mode,
    included_quantity: rate.included_quantity,
    priority: rate.priority,
    notes: rate.notes ?? '',
    status: rate.status,
  };
}

function normalizePlanDraft(
  draft: CommercialPricingPlanCreateInput,
): CommercialPricingPlanCreateInput {
  return {
    ...draft,
    plan_code: draft.plan_code.trim(),
    display_name: draft.display_name.trim(),
    currency_code: draft.currency_code.trim() || 'USD',
    credit_unit_code: draft.credit_unit_code.trim() || 'credit',
    status: draft.status.trim() || 'draft',
    effective_from_ms: Math.max(0, Math.trunc(draft.effective_from_ms)),
    effective_to_ms: draft.effective_to_ms == null
      ? null
      : Math.max(0, Math.trunc(draft.effective_to_ms)),
  };
}

function normalizeRateDraft(
  draft: CommercialPricingRateCreateInput,
): CommercialPricingRateCreateInput {
  return {
    ...draft,
    metric_code: draft.metric_code.trim(),
    capability_code: draft.capability_code?.trim() || null,
    model_code: draft.model_code?.trim() || null,
    provider_code: draft.provider_code?.trim() || null,
    display_price_unit: draft.display_price_unit.trim(),
    notes: draft.notes?.trim() || null,
    status: draft.status.trim() || 'draft',
  };
}

function isDuePlannedPricingPlan(
  plan: Pick<CommercialPricingPlanRecord, 'status' | 'effective_from_ms' | 'effective_to_ms'>,
  nowMs: number,
) {
  return plan.status === 'planned'
    && plan.effective_from_ms <= nowMs
    && (plan.effective_to_ms == null || plan.effective_to_ms >= nowMs);
}

function describeLifecycleReport(
  report: CommercialPricingLifecycleSynchronizationReport | null,
  formatNumber: (value: number) => string,
  t: PricingTranslateFn,
) {
  if (!report) {
    return t(
      'Force lifecycle convergence when due planned versions should become active before the next automatic pricing read.',
    );
  }

  if (report.changed) {
    return t(
      'Last sync activated {planCount} plan versions and {rateCount} pricing rows.',
      {
        planCount: formatNumber(report.activated_plan_count),
        rateCount: formatNumber(report.activated_rate_count),
      },
    );
  }

  if (report.skipped_plan_count > 0) {
    return t(
      'Last sync skipped {count} due planned versions because no rate rows were attached.',
      {
        count: formatNumber(report.skipped_plan_count),
      },
    );
  }

  return t('Last sync found no due planned versions that required lifecycle changes.');
}

export function PricingPage({ snapshot }: AdminPageProps) {
  const {
    handleCreateCommercialPricingPlan,
    handleCreateCommercialPricingRate,
    handleCloneCommercialPricingPlan,
    handlePublishCommercialPricingPlan,
    handleScheduleCommercialPricingPlan,
    handleRetireCommercialPricingPlan,
    handleSynchronizeCommercialPricingLifecycle,
    handleUpdateCommercialPricingPlan,
    handleUpdateCommercialPricingRate,
    loading,
    status,
  } = useAdminWorkbench();
  const { formatCurrency, formatDateTime, formatNumber, t } = useAdminI18n();
  const [planDraft, setPlanDraft] = useState(() => buildPlanDraft(snapshot));
  const [rateDraft, setRateDraft] = useState(() => buildRateDraft(snapshot));
  const [editingPlanId, setEditingPlanId] = useState<number | null>(null);
  const [editingRateId, setEditingRateId] = useState<number | null>(null);
  const [savingPlan, setSavingPlan] = useState(false);
  const [savingRate, setSavingRate] = useState(false);
  const [synchronizingLifecycle, setSynchronizingLifecycle] = useState(false);
  const [lastLifecycleReport, setLastLifecycleReport] =
    useState<CommercialPricingLifecycleSynchronizationReport | null>(null);

  useEffect(() => {
    if (editingPlanId === null) {
      setPlanDraft(buildPlanDraft(snapshot));
    }
  }, [snapshot, editingPlanId]);

  useEffect(() => {
    if (editingRateId === null) {
      setRateDraft(buildRateDraft(snapshot));
    }
  }, [snapshot, editingRateId]);

  const tokenPricing = useMemo(
    () => snapshot.commercialPricingRates.filter((rate) => rate.charge_unit.includes('token')),
    [snapshot.commercialPricingRates],
  );
  const mediaPricing = useMemo(
    () =>
      snapshot.commercialPricingRates.filter((rate) =>
        [
          'image',
          'audio_second',
          'audio_minute',
          'video_second',
          'video_minute',
          'music_track',
        ].includes(rate.charge_unit),
      ),
    [snapshot.commercialPricingRates],
  );
  const chargeUnitCount = useMemo(
    () => new Set(snapshot.commercialPricingRates.map((rate) => rate.charge_unit)).size,
    [snapshot.commercialPricingRates],
  );
  const chargeUnitOptions = useMemo(() => buildChargeUnitOptions(t), [t]);
  const billingMethodCount = useMemo(
    () => new Set(snapshot.commercialPricingRates.map((rate) => rate.pricing_method)).size,
    [snapshot.commercialPricingRates],
  );
  const pricingMethodOptions = useMemo(() => buildPricingMethodOptions(t), [t]);
  const duePlannedPlanCount = useMemo(() => {
    const nowMs = Date.now();
    return snapshot.commercialPricingPlans.filter((plan) => isDuePlannedPricingPlan(plan, nowMs))
      .length;
  }, [snapshot.commercialPricingPlans]);
  const hasPricingPlan = snapshot.commercialPricingPlans.length > 0 || editingRateId !== null;
  const roundingModeOptions = useMemo(() => buildRoundingModeOptions(t), [t]);
  const lifecycleSummary = useMemo(
    () => describeLifecycleReport(lastLifecycleReport, formatNumber, t),
    [formatNumber, lastLifecycleReport, t],
  );

  function resetPlanComposer() {
    setEditingPlanId(null);
    setPlanDraft(buildPlanDraft(snapshot));
  }

  function resetRateComposer() {
    setEditingRateId(null);
    setRateDraft(buildRateDraft(snapshot));
  }

  function startPlanEdit(plan: CommercialPricingPlanRecord) {
    setEditingPlanId(plan.pricing_plan_id);
    setPlanDraft(buildPlanDraftFromRecord(plan));
  }

  async function clonePlan(plan: CommercialPricingPlanRecord) {
    setSavingPlan(true);
    try {
      const clonedPlan = await handleCloneCommercialPricingPlan(plan.pricing_plan_id);
      setEditingPlanId(clonedPlan.pricing_plan_id);
      setPlanDraft(buildPlanDraftFromRecord(clonedPlan));
    } finally {
      setSavingPlan(false);
    }
  }

  async function publishPlan(plan: CommercialPricingPlanRecord) {
    setSavingPlan(true);
    try {
      const publishedPlan = await handlePublishCommercialPricingPlan(plan.pricing_plan_id);
      if (editingPlanId === publishedPlan.pricing_plan_id) {
        setPlanDraft(buildPlanDraftFromRecord(publishedPlan));
      }
    } finally {
      setSavingPlan(false);
    }
  }

  async function schedulePlan(plan: CommercialPricingPlanRecord) {
    setSavingPlan(true);
    try {
      const scheduledPlan = await handleScheduleCommercialPricingPlan(plan.pricing_plan_id);
      if (editingPlanId === scheduledPlan.pricing_plan_id) {
        setPlanDraft(buildPlanDraftFromRecord(scheduledPlan));
      }
    } finally {
      setSavingPlan(false);
    }
  }

  async function retirePlan(plan: CommercialPricingPlanRecord) {
    setSavingPlan(true);
    try {
      const retiredPlan = await handleRetireCommercialPricingPlan(plan.pricing_plan_id);
      if (editingPlanId === retiredPlan.pricing_plan_id) {
        setPlanDraft(buildPlanDraftFromRecord(retiredPlan));
      }
    } finally {
      setSavingPlan(false);
    }
  }

  async function synchronizeLifecycle() {
    setSynchronizingLifecycle(true);
    try {
      const report = await handleSynchronizeCommercialPricingLifecycle();
      setLastLifecycleReport(report);
    } finally {
      setSynchronizingLifecycle(false);
    }
  }

  function startRateEdit(rate: CommercialPricingRateRecord) {
    setEditingRateId(rate.pricing_rate_id);
    setRateDraft(buildRateDraftFromRecord(rate));
  }

  async function submitPlan(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSavingPlan(true);
    try {
      const normalizedDraft = normalizePlanDraft(planDraft);
      if (editingPlanId === null) {
        await handleCreateCommercialPricingPlan(normalizedDraft);
      } else {
        await handleUpdateCommercialPricingPlan(editingPlanId, normalizedDraft);
      }
      resetPlanComposer();
    } finally {
      setSavingPlan(false);
    }
  }

  async function submitRate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSavingRate(true);
    try {
      const normalizedDraft = normalizeRateDraft(rateDraft);
      if (editingRateId === null) {
        await handleCreateCommercialPricingRate(normalizedDraft);
      } else {
        await handleUpdateCommercialPricingRate(editingRateId, normalizedDraft);
      }
      resetRateComposer();
    } finally {
      setSavingRate(false);
    }
  }

  return (
    <ManagementWorkbench
      description={t('A dedicated pricing module keeps settlement-facing pricing governance separate from catalog market prices.')}
      eyebrow={t('Finops')}
      title={t('Pricing governance')}
      main={{
        title: t('Pricing governance'),
        description: t('Pricing plans, charge units, and billing methods are maintained here for token, image, audio, video, and music APIs.'),
        children: (
          <div className="space-y-6">
            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
              <StatCard
                label={t('Pricing plans')}
                value={formatNumber(snapshot.commercialPricingPlans.length)}
                description={t('Versioned commercial plan headers available to operators.')}
              />
              <StatCard
                label={t('Charge units')}
                value={formatNumber(chargeUnitCount)}
                description={t('Distinct units already represented in canonical pricing rows.')}
              />
              <StatCard
                label={t('Billing methods')}
                value={formatNumber(billingMethodCount)}
                description={t('Settlement methods visible in active pricing definitions.')}
              />
              <StatCard
                label={t('Due planned versions')}
                value={formatNumber(duePlannedPlanCount)}
                description={t('Planned versions already inside their effective window and eligible for lifecycle convergence.')}
              />
              <StatCard
                label={t('Pricing rates')}
                value={formatNumber(snapshot.commercialPricingRates.length)}
                description={t('Token pricing and media pricing rows currently maintained.')}
              />
            </div>

            <div className="grid gap-4 xl:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>{t('Pricing plans')}</CardTitle>
                  <CardDescription>
                    {t('Operators define versioned commercial plans before adding rate rows.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex flex-wrap items-start justify-between gap-3 rounded-xl border border-[var(--sdk-color-border-primary)] p-3">
                    <div className="space-y-1">
                      <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                        {t('Synchronize lifecycle')}
                      </div>
                      <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                        {lifecycleSummary}
                      </div>
                    </div>
                    <Button
                      disabled={synchronizingLifecycle || savingPlan || loading}
                      onClick={synchronizeLifecycle}
                      type="button"
                    >
                      {synchronizingLifecycle ? t('Synchronizing...') : t('Synchronize lifecycle')}
                    </Button>
                  </div>

                  <form className="grid gap-3 md:grid-cols-2" onSubmit={submitPlan}>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-plan-code">{t('Plan code')}</Label>
                      <Input
                        id="pricing-plan-code"
                        value={planDraft.plan_code}
                        onChange={(event) =>
                          setPlanDraft((current) => ({
                            ...current,
                            plan_code: event.target.value,
                          }))}
                        placeholder={t('Example: retail-pro')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-plan-version">{t('Plan version')}</Label>
                      <Input
                        id="pricing-plan-version"
                        type="number"
                        value={String(planDraft.plan_version)}
                        onChange={(event) =>
                          setPlanDraft((current) => ({
                            ...current,
                            plan_version: parseNumber(event.target.value, current.plan_version),
                          }))}
                      />
                    </div>
                    <div className="space-y-2 md:col-span-2">
                      <Label htmlFor="pricing-plan-display-name">{t('Display name')}</Label>
                      <Input
                        id="pricing-plan-display-name"
                        value={planDraft.display_name}
                        onChange={(event) =>
                          setPlanDraft((current) => ({
                            ...current,
                            display_name: event.target.value,
                          }))}
                        placeholder={t('Example: Retail Pro')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-plan-currency-code">{t('Currency code')}</Label>
                      <Input
                        id="pricing-plan-currency-code"
                        value={planDraft.currency_code}
                        onChange={(event) =>
                          setPlanDraft((current) => ({
                            ...current,
                            currency_code: event.target.value,
                          }))}
                        placeholder={t('Example: USD')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-plan-credit-unit">{t('Credit unit code')}</Label>
                      <Input
                        id="pricing-plan-credit-unit"
                        value={planDraft.credit_unit_code}
                        onChange={(event) =>
                          setPlanDraft((current) => ({
                            ...current,
                            credit_unit_code: event.target.value,
                          }))}
                        placeholder={t('Example: credit')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-plan-status">{t('Status')}</Label>
                      <select
                        className="h-10 w-full rounded-md border border-[var(--sdk-color-border-primary)] bg-[var(--sdk-color-bg-primary)] px-3 text-sm"
                        id="pricing-plan-status"
                        value={planDraft.status}
                        onChange={(event) =>
                          setPlanDraft((current) => ({
                            ...current,
                            status: event.target.value,
                          }))}
                      >
                        <option value="draft">{t('Draft')}</option>
                        <option value="planned">{t('Planned')}</option>
                        <option value="active">{t('Active')}</option>
                        <option value="archived">{t('Archived')}</option>
                      </select>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-plan-effective-from">{t('Effective from')}</Label>
                      <Input
                        id="pricing-plan-effective-from"
                        type="datetime-local"
                        value={toDateTimeLocalInputValue(planDraft.effective_from_ms)}
                        onChange={(event) =>
                          setPlanDraft((current) => ({
                            ...current,
                            effective_from_ms: parseDateTimeLocalInputValue(event.target.value) ?? 0,
                          }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-plan-effective-to">{t('Effective to')}</Label>
                      <Input
                        id="pricing-plan-effective-to"
                        type="datetime-local"
                        value={toDateTimeLocalInputValue(planDraft.effective_to_ms)}
                        onChange={(event) =>
                          setPlanDraft((current) => ({
                            ...current,
                            effective_to_ms: parseDateTimeLocalInputValue(event.target.value),
                          }))}
                      />
                    </div>
                    <div className="flex items-end gap-2">
                      <Button disabled={savingPlan || loading} type="submit">
                        {savingPlan
                          ? t('Saving...')
                          : editingPlanId === null
                            ? t('Create plan')
                            : t('Update plan')}
                      </Button>
                      {editingPlanId !== null ? (
                        <Button
                          disabled={savingPlan || loading}
                          onClick={resetPlanComposer}
                          type="button"
                        >
                          {t('Create new plan')}
                        </Button>
                      ) : null}
                    </div>
                  </form>

                  <div className="space-y-3">
                    {snapshot.commercialPricingPlans.slice(0, 5).map((plan) => {
                      const isFutureDatedPlan = plan.effective_from_ms > Date.now();
                      const canSchedule =
                        isFutureDatedPlan &&
                        plan.status !== 'active' &&
                        plan.status !== 'archived' &&
                        plan.status !== 'planned';
                      const canPublish = !isFutureDatedPlan && plan.status !== 'active';

                      return (
                        <div
                          className="flex items-start justify-between gap-3 rounded-xl border border-[var(--sdk-color-border-primary)] p-3"
                          key={plan.pricing_plan_id}
                        >
                          <div className="space-y-1">
                            <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                              {plan.display_name || plan.plan_code}
                            </div>
                            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                              {plan.plan_code} v{plan.plan_version} | {plan.currency_code}
                            </div>
                            <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                              {t('Effective from')}: {plan.effective_from_ms > 0
                                ? formatDateTime(plan.effective_from_ms)
                                : t('Immediate')}
                            </div>
                            <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                              {t('Effective to')}: {plan.effective_to_ms != null
                                ? formatDateTime(plan.effective_to_ms)
                                : t('Open ended')}
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            <StatusBadge
                              showIcon
                              status={plan.status}
                              variant={plan.status === 'active' ? 'success' : 'secondary'}
                            />
                            {canSchedule ? (
                              <Button
                                disabled={savingPlan || loading}
                                onClick={() => schedulePlan(plan)}
                                type="button"
                              >
                                {t('Schedule plan')}
                              </Button>
                            ) : null}
                            {canPublish ? (
                              <Button
                                disabled={savingPlan || loading}
                                onClick={() => publishPlan(plan)}
                                type="button"
                              >
                                {t('Publish plan')}
                              </Button>
                            ) : null}
                            {plan.status !== 'archived' ? (
                              <Button
                                disabled={savingPlan || loading}
                                onClick={() => retirePlan(plan)}
                                type="button"
                              >
                                {t('Retire plan')}
                              </Button>
                            ) : null}
                            <Button
                              disabled={savingPlan || loading}
                              onClick={() => clonePlan(plan)}
                              type="button"
                            >
                              {t('Clone plan')}
                            </Button>
                            <Button onClick={() => startPlanEdit(plan)} type="button">
                              {t('Edit plan')}
                            </Button>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>{t('Pricing rate composer')}</CardTitle>
                  <CardDescription>
                    {t('Create commercial pricing rows with explicit charge units, billing methods, rounding, and minimums.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <form className="grid gap-3 md:grid-cols-2" onSubmit={submitRate}>
                    <div className="space-y-2 md:col-span-2">
                      <Label htmlFor="pricing-rate-plan">{t('Pricing plan')}</Label>
                      <select
                        className="h-10 w-full rounded-md border border-[var(--sdk-color-border-primary)] bg-[var(--sdk-color-bg-primary)] px-3 text-sm"
                        id="pricing-rate-plan"
                        value={String(rateDraft.pricing_plan_id)}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            pricing_plan_id: parseNumber(
                              event.target.value,
                              current.pricing_plan_id,
                            ),
                          }))}
                      >
                        {snapshot.commercialPricingPlans.length === 0 ? (
                          <option value="0">{t('No pricing plan available')}</option>
                        ) : (
                          snapshot.commercialPricingPlans.map((plan) => (
                            <option key={plan.pricing_plan_id} value={plan.pricing_plan_id}>
                              {plan.display_name || plan.plan_code}
                            </option>
                          ))
                        )}
                      </select>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-metric">{t('Metric code')}</Label>
                      <Input
                        id="pricing-rate-metric"
                        value={rateDraft.metric_code}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            metric_code: event.target.value,
                          }))}
                        placeholder={t('Example: token.input')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-capability">{t('Capability code')}</Label>
                      <Input
                        id="pricing-rate-capability"
                        value={rateDraft.capability_code ?? ''}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            capability_code: event.target.value,
                          }))}
                        placeholder={t('Example: responses')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-model">{t('Model code')}</Label>
                      <Input
                        id="pricing-rate-model"
                        value={rateDraft.model_code ?? ''}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            model_code: event.target.value,
                          }))}
                        placeholder={t('Example: gpt-4.1')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-provider">{t('Provider code')}</Label>
                      <Input
                        id="pricing-rate-provider"
                        value={rateDraft.provider_code ?? ''}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            provider_code: event.target.value,
                          }))}
                        placeholder={t('Example: provider-openai-official')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-charge-unit">{t('Charge units')}</Label>
                      <select
                        className="h-10 w-full rounded-md border border-[var(--sdk-color-border-primary)] bg-[var(--sdk-color-bg-primary)] px-3 text-sm"
                        id="pricing-rate-charge-unit"
                        value={rateDraft.charge_unit}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            charge_unit: event.target.value as CommercialPricingChargeUnit,
                          }))}
                      >
                        {chargeUnitOptions.map((option) => (
                          <option key={option.value} value={option.value}>
                            {option.label}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-method">{t('Billing methods')}</Label>
                      <select
                        className="h-10 w-full rounded-md border border-[var(--sdk-color-border-primary)] bg-[var(--sdk-color-bg-primary)] px-3 text-sm"
                        id="pricing-rate-method"
                        value={rateDraft.pricing_method}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            pricing_method: event.target.value as CommercialPricingMethod,
                          }))}
                      >
                        {pricingMethodOptions.map((option) => (
                          <option key={option.value} value={option.value}>
                            {option.label}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-quantity-step">{t('Quantity step')}</Label>
                      <Input
                        id="pricing-rate-quantity-step"
                        type="number"
                        value={String(rateDraft.quantity_step)}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            quantity_step: parseNumber(
                              event.target.value,
                              current.quantity_step,
                            ),
                          }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-unit-price">{t('Unit price')}</Label>
                      <Input
                        id="pricing-rate-unit-price"
                        type="number"
                        value={String(rateDraft.unit_price)}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            unit_price: parseNumber(event.target.value, current.unit_price),
                          }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-display-unit">{t('Display unit')}</Label>
                      <Input
                        id="pricing-rate-display-unit"
                        value={rateDraft.display_price_unit}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            display_price_unit: event.target.value,
                          }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-minimum-quantity">
                        {t('Minimum billable quantity')}
                      </Label>
                      <Input
                        id="pricing-rate-minimum-quantity"
                        type="number"
                        value={String(rateDraft.minimum_billable_quantity)}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            minimum_billable_quantity: parseNumber(
                              event.target.value,
                              current.minimum_billable_quantity,
                            ),
                          }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-minimum-charge">{t('Minimum charge')}</Label>
                      <Input
                        id="pricing-rate-minimum-charge"
                        type="number"
                        value={String(rateDraft.minimum_charge)}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            minimum_charge: parseNumber(
                              event.target.value,
                              current.minimum_charge,
                            ),
                          }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-rounding-increment">
                        {t('Rounding increment')}
                      </Label>
                      <Input
                        id="pricing-rate-rounding-increment"
                        type="number"
                        value={String(rateDraft.rounding_increment)}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            rounding_increment: parseNumber(
                              event.target.value,
                              current.rounding_increment,
                            ),
                          }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-rounding">{t('Rounding')}</Label>
                      <select
                        className="h-10 w-full rounded-md border border-[var(--sdk-color-border-primary)] bg-[var(--sdk-color-bg-primary)] px-3 text-sm"
                        id="pricing-rate-rounding"
                        value={rateDraft.rounding_mode}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            rounding_mode: event.target.value,
                          }))}
                      >
                        {roundingModeOptions.map((option) => (
                          <option key={option.value} value={option.value}>
                            {option.label}
                          </option>
                        ))}
                      </select>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-included-quantity">{t('Included quantity')}</Label>
                      <Input
                        id="pricing-rate-included-quantity"
                        type="number"
                        value={String(rateDraft.included_quantity)}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            included_quantity: parseNumber(
                              event.target.value,
                              current.included_quantity,
                            ),
                          }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-priority">{t('Priority')}</Label>
                      <Input
                        id="pricing-rate-priority"
                        type="number"
                        value={String(rateDraft.priority)}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            priority: parseNumber(event.target.value, current.priority),
                          }))}
                      />
                    </div>
                    <div className="space-y-2 md:col-span-2">
                      <Label htmlFor="pricing-rate-notes">{t('Notes')}</Label>
                      <Input
                        id="pricing-rate-notes"
                        value={rateDraft.notes ?? ''}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            notes: event.target.value,
                          }))}
                        placeholder={t('Example: Retail text input pricing')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="pricing-rate-status">{t('Status')}</Label>
                      <select
                        className="h-10 w-full rounded-md border border-[var(--sdk-color-border-primary)] bg-[var(--sdk-color-bg-primary)] px-3 text-sm"
                        id="pricing-rate-status"
                        value={rateDraft.status}
                        onChange={(event) =>
                          setRateDraft((current) => ({
                            ...current,
                            status: event.target.value,
                          }))}
                      >
                        <option value="draft">{t('Draft')}</option>
                        <option value="planned">{t('Planned')}</option>
                        <option value="active">{t('Active')}</option>
                        <option value="archived">{t('Archived')}</option>
                      </select>
                    </div>
                    <div className="flex items-end gap-2">
                      <Button
                        disabled={!hasPricingPlan || savingRate || loading}
                        type="submit"
                      >
                        {savingRate
                          ? t('Saving...')
                          : editingRateId === null
                            ? t('Create pricing rate')
                            : t('Update rate')}
                      </Button>
                      {editingRateId !== null ? (
                        <Button
                          disabled={savingRate || loading}
                          onClick={resetRateComposer}
                          type="button"
                        >
                          {t('Create new rate')}
                        </Button>
                      ) : null}
                    </div>
                  </form>
                  <div className="space-y-2 text-sm text-[var(--sdk-color-text-secondary)]">
                    {!hasPricingPlan ? t('Create a pricing plan before maintaining pricing rates.') : null}
                    <div>{status}</div>
                  </div>
                </CardContent>
              </Card>
            </div>

            <div className="grid gap-4 xl:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>{t('Token pricing')}</CardTitle>
                  <CardDescription>
                    {t('Token pricing stays explicit for input, output, and cache-related usage.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  {tokenPricing.slice(0, 6).map((rate) => (
                    <div
                      className="flex items-start justify-between gap-3 rounded-xl border border-[var(--sdk-color-border-primary)] p-3"
                      key={rate.pricing_rate_id}
                    >
                      <div className="space-y-1">
                        <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                          {rate.metric_code}
                        </div>
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {commercialPricingDisplayUnit(rate, t)}
                          {' | '}
                          {commercialPricingMethodLabel(rate.pricing_method, t)}
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <StatusBadge
                          showIcon
                          status={formatCurrency(rate.unit_price)}
                          variant={rate.status === 'active' ? 'success' : 'secondary'}
                        />
                        <Button onClick={() => startRateEdit(rate)} type="button">
                          {t('Edit rate')}
                        </Button>
                      </div>
                    </div>
                  ))}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>{t('Media pricing')}</CardTitle>
                  <CardDescription>
                    {t('Media pricing covers images, audio, video, and music with modality-native units.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  {mediaPricing.slice(0, 6).map((rate) => (
                    <div
                      className="flex items-start justify-between gap-3 rounded-xl border border-[var(--sdk-color-border-primary)] p-3"
                      key={rate.pricing_rate_id}
                    >
                      <div className="space-y-1">
                        <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                          {rate.capability_code || rate.metric_code}
                        </div>
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {commercialPricingDisplayUnit(rate, t)}
                          {' | '}
                          {commercialPricingChargeUnitLabel(rate.charge_unit, t)}
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <StatusBadge
                          showIcon
                          status={formatCurrency(rate.unit_price)}
                          variant={rate.status === 'active' ? 'success' : 'secondary'}
                        />
                        <Button onClick={() => startRateEdit(rate)} type="button">
                          {t('Edit rate')}
                        </Button>
                      </div>
                    </div>
                  ))}
                </CardContent>
              </Card>
            </div>

            <div className="grid gap-4 xl:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>{t('Charge units')}</CardTitle>
                  <CardDescription>
                    {t('Charge units define what quantity gets billed in the commercial settlement layer.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  {chargeUnitOptions.map((option) => (
                    <div className="flex items-start justify-between gap-3" key={option.value}>
                      <div className="space-y-1">
                        <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                          {option.label}
                        </div>
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {option.detail}
                        </div>
                      </div>
                      <StatusBadge showIcon status={option.value} variant="secondary" />
                    </div>
                  ))}
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>{t('Billing methods')}</CardTitle>
                  <CardDescription>
                    {t('Billing methods stay standardized so settlement logic can evolve without schema churn.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  {pricingMethodOptions.map((option) => (
                    <div className="flex items-start justify-between gap-3" key={option.value}>
                      <div className="space-y-1">
                        <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                          {option.label}
                        </div>
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {option.detail}
                        </div>
                      </div>
                      <StatusBadge showIcon status={option.value} variant="secondary" />
                    </div>
                  ))}
                </CardContent>
              </Card>
            </div>
          </div>
        ),
      }}
    />
  );
}
