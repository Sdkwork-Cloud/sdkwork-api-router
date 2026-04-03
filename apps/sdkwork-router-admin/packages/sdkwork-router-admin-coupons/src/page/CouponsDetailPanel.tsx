import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { CouponRecord } from 'sdkwork-router-admin-types';

import { expiryDetail, quotaHealth } from './shared';

type CouponsDetailPanelProps = {
  selectedCoupon: CouponRecord;
};

export function CouponsDetailPanel({
  selectedCoupon,
}: CouponsDetailPanelProps) {
  const { formatNumber, t } = useAdminI18n();
  const health = quotaHealth(selectedCoupon);

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between gap-3">
            <CardTitle className="text-base">{t('Campaign posture')}</CardTitle>
            <StatusBadge
              showIcon
              status={selectedCoupon.active ? 'active' : 'archived'}
              variant={selectedCoupon.active ? 'success' : 'secondary'}
            />
          </div>
          <CardDescription>{selectedCoupon.note}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <DescriptionList columns={2}>
            <DescriptionItem>
              <DescriptionTerm>{t('Audience')}</DescriptionTerm>
              <DescriptionDetails>{selectedCoupon.audience}</DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Discount')}</DescriptionTerm>
              <DescriptionDetails>{selectedCoupon.discount_label}</DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Remaining quota')}</DescriptionTerm>
              <DescriptionDetails>{formatNumber(selectedCoupon.remaining)}</DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Expiry')}</DescriptionTerm>
              <DescriptionDetails>{selectedCoupon.expires_on}</DescriptionDetails>
            </DescriptionItem>
          </DescriptionList>
          <div className="grid gap-3 md:grid-cols-2">
            <Card className="border-[var(--sdk-color-border-subtle)] shadow-none">
              <CardHeader className="space-y-1">
                <CardTitle className="text-sm">{t('Quota health')}</CardTitle>
                <CardDescription>{health.detail}</CardDescription>
              </CardHeader>
              <CardContent className="pt-0">
                <StatusBadge showIcon status={health.label} variant={health.variant} />
              </CardContent>
            </Card>
            <Card className="border-[var(--sdk-color-border-subtle)] shadow-none">
              <CardHeader className="space-y-1">
                <CardTitle className="text-sm">{t('Expiry window')}</CardTitle>
                <CardDescription>{expiryDetail(selectedCoupon)}</CardDescription>
              </CardHeader>
              <CardContent className="pt-0 text-sm text-[var(--sdk-color-text-secondary)]">
                {t(
                  'Support and campaign operators can use this window to stage renewals or retire the offer.',
                )}
              </CardContent>
            </Card>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
