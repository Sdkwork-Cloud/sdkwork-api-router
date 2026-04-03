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
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import { DialogField, SelectField, type OperatorDraft } from './shared';

type OperatorUserDialogProps = {
  draft: OperatorDraft;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setDraft: Dispatch<SetStateAction<OperatorDraft>>;
};

export function OperatorUserDialog({
  draft,
  onOpenChange,
  onSubmit,
  open,
  setDraft,
}: OperatorUserDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,46rem)]">
        <DialogHeader>
          <DialogTitle>{draft.id ? t('Edit operator') : t('Create operator')}</DialogTitle>
          <DialogDescription>
            {t(
              'Operators manage catalog, traffic, and runtime posture. Keep this population tightly controlled.',
            )}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t('Operator accounts stay minimal, high-trust, and easy to audit.')}
            title={t('Operator profile')}
          >
            <FormGrid columns={2}>
              <DialogField htmlFor="operator-name" label={t('Display name')}>
                <Input
                  id="operator-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      display_name: event.target.value,
                    }))
                  }
                  required
                  value={draft.display_name}
                />
              </DialogField>

              <DialogField htmlFor="operator-email" label={t('Email')}>
                <Input
                  id="operator-email"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      email: event.target.value,
                    }))
                  }
                  required
                  type="email"
                  value={draft.email}
                />
              </DialogField>

              <DialogField
                htmlFor="operator-password"
                label={draft.id ? t('New password') : t('Password')}
                description={
                  draft.id
                    ? t('Leave blank to preserve the current password.')
                    : t('Set a strong operator password.')
                }
              >
                <Input
                  id="operator-password"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      password: event.target.value,
                    }))
                  }
                  required={!draft.id}
                  type="password"
                  value={draft.password}
                />
              </DialogField>

              <SelectField<'active' | 'disabled'>
                label={t('Status')}
                onValueChange={(value) =>
                  setDraft((current) => ({
                    ...current,
                    active: value === 'active',
                  }))
                }
                options={[
                  { label: t('Active'), value: 'active' },
                  { label: t('Disabled'), value: 'disabled' },
                ]}
                value={draft.active ? 'active' : 'disabled'}
              />
            </FormGrid>
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
              {draft.id ? t('Save operator') : t('Create operator')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
