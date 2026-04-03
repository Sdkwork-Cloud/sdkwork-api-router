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

import {
  defaultProjectId,
  DialogField,
  SelectField,
  type ApiKeyDraft,
} from './shared';

type ApiKeyDialogProps = {
  availableProjects: AdminPageProps['snapshot']['projects'];
  draft: ApiKeyDraft;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setDraft: Dispatch<SetStateAction<ApiKeyDraft>>;
  snapshot: AdminPageProps['snapshot'];
};

export function ApiKeyDialog({
  availableProjects,
  draft,
  onOpenChange,
  onSubmit,
  open,
  setDraft,
  snapshot,
}: ApiKeyDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,52rem)]">
        <DialogHeader>
          <DialogTitle>{t('Issue gateway key')}</DialogTitle>
          <DialogDescription>
            {t(
              'Mint a project-scoped API key in a focused dialog, then reveal the plaintext once for secure handoff.',
            )}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t('Scope determines which tenant and project own the new key.')}
            title={t('Key scope')}
          >
            <FormGrid columns={2}>
              <div className="space-y-2">
                {snapshot.tenants.length ? (
                  <SelectField
                    label={t('Tenant')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        tenant_id: value,
                        project_id: defaultProjectId(snapshot, value),
                      }))
                    }
                    options={snapshot.tenants.map((tenant) => ({
                      label: `${tenant.name} (${tenant.id})`,
                      value: tenant.id,
                    }))}
                    value={draft.tenant_id}
                  />
                ) : (
                  <DialogField label={t('Tenant')}>
                    <Input
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          tenant_id: event.target.value,
                        }))
                      }
                      required
                      value={draft.tenant_id}
                    />
                  </DialogField>
                )}
              </div>

              <div className="space-y-2">
                {availableProjects.length ? (
                  <SelectField
                    label={t('Project')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        project_id: value,
                      }))
                    }
                    options={availableProjects.map((project) => ({
                      label: `${project.name} (${project.id})`,
                      value: project.id,
                    }))}
                    value={draft.project_id}
                  />
                ) : (
                  <DialogField label={t('Project')}>
                    <Input
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          project_id: event.target.value,
                        }))
                      }
                      required
                      value={draft.project_id}
                    />
                  </DialogField>
                )}
              </div>

              <SelectField<'production' | 'staging' | 'development'>
                label={t('Environment')}
                onValueChange={(value) =>
                  setDraft((current) => ({ ...current, environment: value }))
                }
                options={[
                  { label: t('Production'), value: 'production' },
                  { label: t('Staging'), value: 'staging' },
                  { label: t('Development'), value: 'development' },
                ]}
                value={draft.environment as 'production' | 'staging' | 'development'}
              />
            </FormGrid>
          </FormSection>

          <FormSection
            description={t('Labels and notes keep the key inventory readable for operators.')}
            title={t('Inventory metadata')}
          >
            <FormGrid columns={2}>
              <DialogField htmlFor="key-label" label={t('Key label')}>
                <Input
                  id="key-label"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({ ...current, label: event.target.value }))
                  }
                  placeholder={t('Production App Key')}
                  value={draft.label}
                />
              </DialogField>

              <DialogField htmlFor="expires-at" label={t('Expires at (ms)')}>
                <Input
                  id="expires-at"
                  inputMode="numeric"
                  min="0"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      expires_at_ms: event.target.value,
                    }))
                  }
                  placeholder={t('4102444800000')}
                  step="1"
                  type="number"
                  value={draft.expires_at_ms}
                />
              </DialogField>

              <DialogField label={t('Notes')}>
                <Textarea
                  onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                    setDraft((current) => ({ ...current, notes: event.target.value }))
                  }
                  placeholder={t('Retained for admin inventory')}
                  rows={3}
                  value={draft.notes}
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
              {t('Issue gateway key')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
