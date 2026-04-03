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

import { DialogField } from './shared';

type TenantDialogProps = {
  draft: { id: string; name: string };
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setDraft: Dispatch<SetStateAction<{ id: string; name: string }>>;
};

export function TenantDialog({
  draft,
  onOpenChange,
  onSubmit,
  open,
  setDraft,
}: TenantDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,42rem)]">
        <DialogHeader>
          <DialogTitle>{draft.id ? t('Edit tenant') : t('New tenant')}</DialogTitle>
          <DialogDescription>
            {t(
              'Tenant creation and editing happen in a dedicated dialog so the registry stays primary on the page.',
            )}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t(
              'Tenant identifiers should remain stable because projects, portal users, and keys inherit this boundary.',
            )}
            title={t('Tenant profile')}
          >
            <FormGrid columns={2}>
              <DialogField htmlFor="tenant-id" label={t('Tenant id')}>
                <Input
                  id="tenant-id"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({ ...current, id: event.target.value }))
                  }
                  required
                  value={draft.id}
                />
              </DialogField>
              <DialogField htmlFor="tenant-name" label={t('Tenant name')}>
                <Input
                  id="tenant-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({ ...current, name: event.target.value }))
                  }
                  required
                  value={draft.name}
                />
              </DialogField>
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
              {draft.id ? t('Save tenant') : t('Create tenant')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
