import type {
  ChangeEvent,
  Dispatch,
  FormEvent,
  SetStateAction,
} from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  FormActions,
  FormGrid,
  FormSection,
  Input,
  Textarea,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { CouponRecord } from 'sdkwork-router-admin-types';

import { DialogField, SelectField } from './shared';

type CouponDialogProps = {
  draft: CouponRecord;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setDraft: Dispatch<SetStateAction<CouponRecord>>;
};

export function CouponDialog({
  draft,
  onOpenChange,
  onSubmit,
  open,
  setDraft,
}: CouponDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,52rem)]">
        <DialogHeader>
          <DialogTitle>
            {draft.id ? t('Edit coupon campaign') : t('Create coupon')}
          </DialogTitle>
          <DialogDescription>
            {t(
              'Use one modal for both launch and revision so the roster always stays primary in the workspace.',
            )}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t(
              'Campaign identifiers, offer copy, and audience targeting live together for easier operator review.',
            )}
            title={t('Campaign profile')}
          >
            <FormGrid columns={2}>
              <DialogField
                description={t(
                  'Stored in uppercase for consistency across support and redemption flows.',
                )}
                htmlFor="coupon-code"
                label={t('Coupon code')}
              >
                <Input
                  id="coupon-code"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      code: event.target.value.toUpperCase(),
                    }))
                  }
                  required
                  value={draft.code}
                />
              </DialogField>
              <DialogField htmlFor="discount-label" label={t('Discount label')}>
                <Input
                  id="discount-label"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      discount_label: event.target.value,
                    }))
                  }
                  required
                  value={draft.discount_label}
                />
              </DialogField>
              <DialogField htmlFor="audience" label={t('Audience')}>
                <Input
                  id="audience"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      audience: event.target.value,
                    }))
                  }
                  required
                  value={draft.audience}
                />
              </DialogField>
              <DialogField htmlFor="remaining" label={t('Remaining quota')}>
                <Input
                  id="remaining"
                  min="0"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      remaining: Number(event.target.value),
                    }))
                  }
                  required
                  type="number"
                  value={String(draft.remaining)}
                />
              </DialogField>
              <DialogField htmlFor="expires-on" label={t('Expires on')}>
                <Input
                  id="expires-on"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      expires_on: event.target.value,
                    }))
                  }
                  required
                  type="date"
                  value={draft.expires_on}
                />
              </DialogField>
              <SelectField<'active' | 'archived'>
                label={t('Status')}
                onValueChange={(value) =>
                  setDraft((current) => ({
                    ...current,
                    active: value === 'active',
                  }))
                }
                options={[
                  { label: t('Active'), value: 'active' },
                  { label: t('Archived'), value: 'archived' },
                ]}
                value={draft.active ? 'active' : 'archived'}
              />
            </FormGrid>
          </FormSection>

          <FormSection
            description={t(
              'Operator notes capture campaign intent, guardrails, and support context.',
            )}
            title={t('Operator note')}
          >
            <DialogField label={t('Operator note')}>
              <Textarea
                onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                  setDraft((current) => ({
                    ...current,
                    note: event.target.value,
                  }))
                }
                required
                rows={4}
                value={draft.note}
              />
            </DialogField>
          </FormSection>

          <FormActions>
            <Button
              onClick={() => onOpenChange(false)}
              type="button"
              variant="outline"
            >
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {draft.id ? t('Save coupon') : t('Create coupon')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
