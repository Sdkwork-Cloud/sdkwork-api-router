import type { ReactNode } from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@sdkwork/ui-pc-react';
import {
  formatAdminNumber,
  translateAdminText,
} from 'sdkwork-router-admin-core';
import type { CouponRecord } from 'sdkwork-router-admin-types';

export type CouponStatusFilter = 'all' | 'active' | 'at_risk' | 'archived';

const EXPIRING_SOON_WINDOW_DAYS = 14;
const LOW_QUOTA_THRESHOLD = 25;

export function createEmptyCouponDraft(): CouponRecord {
  return {
    id: '',
    code: '',
    discount_label: '10% off first bill',
    audience: 'new_signup',
    remaining: 100,
    active: true,
    note: 'Launch campaign',
    expires_on: '2026-12-31',
  };
}

export function daysUntilExpiry(expiresOn: string): number | null {
  const expiryValue = Date.parse(expiresOn);
  if (Number.isNaN(expiryValue)) {
    return null;
  }

  const now = new Date();
  const startOfTodayUtc = Date.UTC(
    now.getUTCFullYear(),
    now.getUTCMonth(),
    now.getUTCDate(),
  );

  return Math.ceil((expiryValue - startOfTodayUtc) / 86_400_000);
}

export function isCouponExpiringSoon(coupon: CouponRecord): boolean {
  if (!coupon.active) {
    return false;
  }

  const days = daysUntilExpiry(coupon.expires_on);
  return days !== null && days >= 0 && days <= EXPIRING_SOON_WINDOW_DAYS;
}

export function isCouponAtRisk(coupon: CouponRecord): boolean {
  if (!coupon.active) {
    return false;
  }

  const days = daysUntilExpiry(coupon.expires_on);
  return (
    coupon.remaining <= LOW_QUOTA_THRESHOLD
    || (days !== null && days <= EXPIRING_SOON_WINDOW_DAYS)
  );
}

export function quotaHealth(coupon: CouponRecord): {
  label: string;
  variant: 'default' | 'success' | 'warning' | 'danger' | 'secondary';
  detail: string;
} {
  if (!coupon.active) {
    return {
      label: translateAdminText('Archived'),
      variant: 'secondary',
      detail: translateAdminText('Campaign is disabled for new redemptions.'),
    };
  }

  const days = daysUntilExpiry(coupon.expires_on);
  if (days !== null && days < 0) {
    return {
      label: translateAdminText('Expired'),
      variant: 'danger',
      detail: translateAdminText('Expiry date has already passed and needs operator review.'),
    };
  }

  if (coupon.remaining <= LOW_QUOTA_THRESHOLD) {
    return {
      label: translateAdminText('At risk'),
      variant: 'danger',
      detail: translateAdminText('{remaining} units remaining before depletion.', {
        remaining: coupon.remaining,
      }),
    };
  }

  if (isCouponExpiringSoon(coupon)) {
    return {
      label: translateAdminText('Expiring soon'),
      variant: 'warning',
      detail: translateAdminText('{days} days remain before campaign expiry.', {
        days: days ?? 0,
      }),
    };
  }

  return {
    label: translateAdminText('Healthy'),
    variant: 'success',
    detail: translateAdminText('{remaining} units available for redemptions.', {
      remaining: coupon.remaining,
    }),
  };
}

export function expiryDetail(coupon: CouponRecord): string {
  const days = daysUntilExpiry(coupon.expires_on);
  if (days === null) {
    return translateAdminText('Expiry date is not available.');
  }

  if (days < 0) {
    return translateAdminText('{days} days overdue.', {
      days: Math.abs(days),
    });
  }

  if (days === 0) {
    return translateAdminText('Expires today.');
  }

  if (days <= EXPIRING_SOON_WINDOW_DAYS) {
    return translateAdminText('{days} days left in the current window.', {
      days,
    });
  }

  return translateAdminText('{days} days of runway remain.', {
    days,
  });
}

export function formatNumber(value: number) {
  return formatAdminNumber(value);
}

export function DialogField({
  children,
  description,
  htmlFor,
  label,
}: {
  children: ReactNode;
  description?: ReactNode;
  htmlFor?: string;
  label: ReactNode;
}) {
  return (
    <div className="space-y-2">
      <Label htmlFor={htmlFor}>{label}</Label>
      {children}
      {description ? (
        <div className="text-xs text-[var(--sdk-color-text-secondary)]">
          {description}
        </div>
      ) : null}
    </div>
  );
}

export function SelectField<T extends string>({
  label,
  labelVisibility = 'visible',
  onValueChange,
  options,
  placeholder,
  value,
}: {
  label: ReactNode;
  labelVisibility?: 'visible' | 'sr-only';
  onValueChange: (value: T) => void;
  options: Array<{ label: ReactNode; value: T }>;
  placeholder?: string;
  value: T;
}) {
  const isHiddenLabel = labelVisibility === 'sr-only';

  return (
    <div className={isHiddenLabel ? 'space-y-0' : 'space-y-2'}>
      <Label className={isHiddenLabel ? 'sr-only' : undefined}>{label}</Label>
      <Select onValueChange={(nextValue: string) => onValueChange(nextValue as T)} value={value}>
        <SelectTrigger>
          <SelectValue placeholder={placeholder ?? String(label)} />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

export function ConfirmActionDialog({
  confirmLabel = translateAdminText('Confirm'),
  description,
  onConfirm,
  onOpenChange,
  open,
  title,
}: {
  confirmLabel?: string;
  description: ReactNode;
  onConfirm: () => void | Promise<void>;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  title: ReactNode;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,28rem)]">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
            {translateAdminText('Cancel')}
          </Button>
          <Button onClick={() => void onConfirm()} type="button" variant="danger">
            {confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
