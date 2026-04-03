import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  Input,
  Label,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Plus, Search } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps, CouponRecord } from 'sdkwork-router-admin-types';

import { CouponDialog } from './page/CouponDialog';
import { CouponsDetailDrawer } from './page/CouponsDetailDrawer';
import { CouponsDetailPanel } from './page/CouponsDetailPanel';
import { CouponsRegistrySection } from './page/CouponsRegistrySection';
import {
  ConfirmActionDialog,
  SelectField,
  createEmptyCouponDraft,
  daysUntilExpiry,
  expiryDetail,
  isCouponAtRisk,
  isCouponExpiringSoon,
  quotaHealth,
  type CouponStatusFilter,
} from './page/shared';

type CouponsPageProps = AdminPageProps & {
  onSaveCoupon: (coupon: CouponRecord) => Promise<void> | void;
  onToggleCoupon: (coupon: CouponRecord) => Promise<void> | void;
  onDeleteCoupon: (couponId: string) => Promise<void> | void;
};

export function CouponsPage({
  snapshot,
  onSaveCoupon,
  onToggleCoupon,
  onDeleteCoupon,
}: CouponsPageProps) {
  const { formatNumber, t } = useAdminI18n();
  const [draft, setDraft] = useState<CouponRecord>(createEmptyCouponDraft());
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<CouponStatusFilter>('all');
  const [selectedCouponId, setSelectedCouponId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const [isCouponDialogOpen, setIsCouponDialogOpen] = useState(false);
  const [pendingDeleteCoupon, setPendingDeleteCoupon] = useState<CouponRecord | null>(null);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  const activeCoupons = snapshot.coupons.filter((coupon) => coupon.active);
  const archivedCoupons = snapshot.coupons.filter((coupon) => !coupon.active);
  const atRiskCoupons = activeCoupons.filter(isCouponAtRisk);
  const expiringSoonCoupons = activeCoupons.filter(isCouponExpiringSoon);
  const remainingQuota = activeCoupons.reduce(
    (total, coupon) => total + Math.max(coupon.remaining, 0),
    0,
  );
  const coveredAudiences = new Set(
    activeCoupons
      .map((coupon) => coupon.audience.trim().toLowerCase())
      .filter(Boolean),
  );
  const nextExpiringCoupon =
    activeCoupons
      .map((coupon) => ({
        coupon,
        days: daysUntilExpiry(coupon.expires_on),
      }))
      .filter((item) => item.days !== null && item.days >= 0)
      .sort(
        (left, right) =>
          (left.days ?? Number.MAX_SAFE_INTEGER)
          - (right.days ?? Number.MAX_SAFE_INTEGER),
      )[0] ?? null;

  const filteredCoupons = useMemo(
    () =>
      snapshot.coupons.filter((coupon) => {
        const matchesStatus =
          statusFilter === 'all'
          || (statusFilter === 'active' && coupon.active)
          || (statusFilter === 'archived' && !coupon.active)
          || (statusFilter === 'at_risk' && isCouponAtRisk(coupon));

        if (!matchesStatus) {
          return false;
        }

        const haystack = [
          coupon.code,
          coupon.discount_label,
          coupon.audience,
          coupon.note,
          coupon.expires_on,
        ]
          .join(' ')
          .toLowerCase();

        return haystack.includes(deferredQuery);
      }),
    [deferredQuery, snapshot.coupons, statusFilter],
  );

  useEffect(() => {
    if (selectedCouponId && !filteredCoupons.some((coupon) => coupon.id === selectedCouponId)) {
      setSelectedCouponId(null);
      setIsDetailDrawerOpen(false);
    }
  }, [filteredCoupons, selectedCouponId]);

  const selectedCoupon =
    filteredCoupons.find((coupon) => coupon.id === selectedCouponId) ?? null;

  const columns = useMemo<DataTableColumn<CouponRecord>[]>(
    () => [
      {
        id: 'campaign',
        header: t('Campaign'),
        cell: (coupon) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {coupon.code}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {coupon.note}
            </div>
          </div>
        ),
      },
      {
        id: 'offer',
        header: t('Offer'),
        cell: (coupon) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{coupon.discount_label}</div>
            <div>{coupon.audience}</div>
          </div>
        ),
      },
      {
        id: 'remaining',
        header: t('Remaining quota'),
        align: 'right',
        cell: (coupon) => formatNumber(coupon.remaining),
        width: 140,
      },
      {
        id: 'quota-health',
        header: t('Quota health'),
        cell: (coupon) => {
          const health = quotaHealth(coupon);
          return (
            <div className="space-y-1">
              <StatusBadge showIcon status={health.label} variant={health.variant} />
              <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                {health.detail}
              </div>
            </div>
          );
        },
      },
      {
        id: 'expires',
        header: t('Expiry'),
        cell: (coupon) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{coupon.expires_on}</div>
            <div>{expiryDetail(coupon)}</div>
          </div>
        ),
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (coupon) => (
          <StatusBadge
            showIcon
            status={coupon.active ? 'active' : 'archived'}
            variant={coupon.active ? 'success' : 'secondary'}
          />
        ),
        width: 140,
      },
    ],
    [formatNumber, t],
  );

  function resetCouponDialog() {
    setIsCouponDialogOpen(false);
    setDraft(createEmptyCouponDraft());
  }

  function handleCouponDialogOpenChange(open: boolean) {
    if (!open) {
      resetCouponDialog();
      return;
    }

    setIsCouponDialogOpen(true);
  }

  function openCouponDialog(coupon?: CouponRecord) {
    setDraft(coupon ? { ...coupon } : createEmptyCouponDraft());
    setIsCouponDialogOpen(true);
  }

  function openDetailDrawer(coupon: CouponRecord) {
    setSelectedCouponId(coupon.id);
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedCouponId(null);
    }
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveCoupon({
      ...draft,
      id: draft.id || `coupon_${Date.now().toString(16)}`,
      code: draft.code.trim().toUpperCase(),
      note: draft.note.trim(),
      audience: draft.audience.trim(),
      discount_label: draft.discount_label.trim(),
      expires_on: draft.expires_on.trim(),
    });
    resetCouponDialog();
  }

  async function handleDeleteCoupon() {
    if (!pendingDeleteCoupon) {
      return;
    }

    await onDeleteCoupon(pendingDeleteCoupon.id);
    setPendingDeleteCoupon(null);
    setSelectedCouponId(null);
    setIsDetailDrawerOpen(false);
  }

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="coupon-search">
                  {t('Search campaigns')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="coupon-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setSearch(event.target.value)
                    }
                    placeholder={t('code, audience, note')}
                    value={search}
                  />
                </div>
              </div>

              <div className="min-w-[12rem]">
                <SelectField
                  label={t('Campaign state')}
                  labelVisibility="sr-only"
                  onValueChange={setStatusFilter}
                  options={[
                    { label: t('All campaigns'), value: 'all' },
                    { label: t('Active'), value: 'active' },
                    { label: t('At risk'), value: 'at_risk' },
                    { label: t('Archived'), value: 'archived' },
                  ]}
                  placeholder={t('Campaign state')}
                  value={statusFilter}
                />
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatNumber(filteredCoupons.length) })}
                  {' | '}
                  {t('{count} live', { count: formatNumber(activeCoupons.length) })}
                  {' | '}
                  {t('{count} at risk', { count: formatNumber(atRiskCoupons.length) })}
                </div>
                <Button onClick={() => openCouponDialog()} type="button" variant="primary">
                  <Plus className="w-4 h-4" />
                  {t('New coupon')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <CouponsRegistrySection
            activeCoupons={activeCoupons}
            archivedCoupons={archivedCoupons}
            atRiskCoupons={atRiskCoupons}
            columns={columns}
            coveredAudiencesCount={coveredAudiences.size}
            expiringSoonCoupons={expiringSoonCoupons}
            filteredCoupons={filteredCoupons}
            nextExpiringCoupon={nextExpiringCoupon}
            onDeleteCoupon={setPendingDeleteCoupon}
            onEditCoupon={openCouponDialog}
            onSelectCoupon={openDetailDrawer}
            onToggleCoupon={onToggleCoupon}
            remainingQuota={remainingQuota}
            selectedCouponId={selectedCouponId}
          />
        </div>
      </div>

      <CouponsDetailDrawer
        onDelete={() => {
          if (!selectedCoupon) {
            return;
          }
          setPendingDeleteCoupon(selectedCoupon);
        }}
        onEdit={() => {
          if (!selectedCoupon) {
            return;
          }
          setIsDetailDrawerOpen(false);
          openCouponDialog(selectedCoupon);
        }}
        onOpenChange={handleDetailDrawerOpenChange}
        onToggleStatus={() => {
          if (!selectedCoupon) {
            return;
          }
          void onToggleCoupon(selectedCoupon);
        }}
        open={isDetailDrawerOpen}
        selectedCoupon={selectedCoupon}
      />

      <CouponDialog
        draft={draft}
        onOpenChange={handleCouponDialogOpenChange}
        onSubmit={(event) => void handleSubmit(event)}
        open={isCouponDialogOpen}
        setDraft={setDraft}
      />

      <ConfirmActionDialog
        confirmLabel={t('Delete coupon')}
        description={
          pendingDeleteCoupon
            ? t(
                'Remove {code} from the campaign roster. This permanently deletes the offer from the admin control plane.',
                { code: pendingDeleteCoupon.code },
              )
            : ''
        }
        onConfirm={() => void handleDeleteCoupon()}
        onOpenChange={(open) => {
          if (!open) {
            setPendingDeleteCoupon(null);
          }
        }}
        open={Boolean(pendingDeleteCoupon)}
        title={t('Delete coupon campaign')}
      />
    </>
  );
}
