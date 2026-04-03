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
  InlineAlert,
  Input,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import { DialogField, SelectField } from './shared';

type ProjectDialogProps = {
  draft: { tenant_id: string; id: string; name: string };
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  selectedProjectBilling:
    | AdminPageProps['snapshot']['billingSummary']['projects'][number]
    | undefined;
  selectedProjectTokens: number;
  selectedProjectUsage:
    | AdminPageProps['snapshot']['usageSummary']['projects'][number]
    | undefined;
  setDraft: Dispatch<
    SetStateAction<{ tenant_id: string; id: string; name: string }>
  >;
  snapshot: AdminPageProps['snapshot'];
};

export function ProjectDialog({
  draft,
  onOpenChange,
  onSubmit,
  open,
  selectedProjectBilling,
  selectedProjectTokens,
  selectedProjectUsage,
  setDraft,
  snapshot,
}: ProjectDialogProps) {
  const { formatNumber, t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,46rem)]">
        <DialogHeader>
          <DialogTitle>{draft.id ? t('Edit project') : t('New project')}</DialogTitle>
          <DialogDescription>
            {t(
              'Projects remain the routing, usage, and billing ownership boundary, so edits belong in their own dialog.',
            )}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t(
              'Projects inherit tenant scope and become the primary boundary for usage, billing, and key issuance.',
            )}
            title={t('Project profile')}
          >
            <FormGrid columns={2}>
              <div className="space-y-2">
                {snapshot.tenants.length ? (
                  <SelectField
                    label={t('Tenant id')}
                    onValueChange={(value) =>
                      setDraft((current) => ({ ...current, tenant_id: value }))
                    }
                    options={snapshot.tenants.map((tenant) => ({
                      label: `${tenant.name} (${tenant.id})`,
                      value: tenant.id,
                    }))}
                    value={draft.tenant_id}
                  />
                ) : (
                  <DialogField label={t('Tenant id')}>
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

              <DialogField htmlFor="project-id" label={t('Project id')}>
                <Input
                  id="project-id"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({ ...current, id: event.target.value }))
                  }
                  required
                  value={draft.id}
                />
              </DialogField>

              <DialogField htmlFor="project-name" label={t('Project name')}>
                <Input
                  id="project-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({ ...current, name: event.target.value }))
                  }
                  required
                  value={draft.name}
                />
              </DialogField>
            </FormGrid>

            <InlineAlert
              description={t(
                'Requests: {requests} | Usage units: {units} | Tokens: {tokens}',
                {
                  requests: formatNumber(selectedProjectUsage?.request_count ?? 0),
                  tokens: formatNumber(selectedProjectTokens),
                  units: formatNumber(selectedProjectBilling?.used_units ?? 0),
                },
              )}
              showIcon
              title={t('Selected project posture')}
              tone="info"
            />
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
              {draft.id ? t('Save project') : t('Create project')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
