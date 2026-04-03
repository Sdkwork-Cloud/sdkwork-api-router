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
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import { DialogField, SelectField, type CredentialDraft } from './shared';

type CatalogCredentialDialogProps = {
  credentialDraft: CredentialDraft;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setCredentialDraft: Dispatch<SetStateAction<CredentialDraft>>;
  snapshot: AdminPageProps['snapshot'];
};

export function CatalogCredentialDialog({
  credentialDraft,
  onOpenChange,
  onSubmit,
  open,
  setCredentialDraft,
  snapshot,
}: CatalogCredentialDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,44rem)]">
        <DialogHeader>
          <DialogTitle>{t('Credential')}</DialogTitle>
          <DialogDescription>
            {t('Rotate or create encrypted provider credentials without leaving the catalog workbench.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection title={t('Credential details')}>
            <FormGrid columns={2}>
              <SelectField
                label={t('Tenant')}
                onValueChange={(value) =>
                  setCredentialDraft((current) => ({
                    ...current,
                    tenant_id: value,
                  }))
                }
                options={snapshot.tenants.map((tenant) => ({
                  label: `${tenant.name} (${tenant.id})`,
                  value: tenant.id,
                }))}
                value={credentialDraft.tenant_id}
              />
              <SelectField
                label={t('Provider')}
                onValueChange={(value) =>
                  setCredentialDraft((current) => ({
                    ...current,
                    provider_id: value,
                  }))
                }
                options={snapshot.providers.map((provider) => ({
                  label: `${provider.display_name} (${provider.id})`,
                  value: provider.id,
                }))}
                value={credentialDraft.provider_id}
              />
              <DialogField htmlFor="credential-reference" label={t('Key reference')}>
                <Input
                  id="credential-reference"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setCredentialDraft((current) => ({
                      ...current,
                      key_reference: event.target.value,
                    }))
                  }
                  required
                  value={credentialDraft.key_reference}
                />
              </DialogField>
              <DialogField htmlFor="credential-secret" label={t('Secret value')}>
                <Textarea
                  id="credential-secret"
                  onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                    setCredentialDraft((current) => ({
                      ...current,
                      secret_value: event.target.value,
                    }))
                  }
                  required
                  rows={4}
                  value={credentialDraft.secret_value}
                />
              </DialogField>
            </FormGrid>
          </FormSection>
          <FormActions>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {t('Save credential')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
