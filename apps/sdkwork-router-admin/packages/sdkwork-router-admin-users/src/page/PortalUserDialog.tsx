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

import {
  defaultProjectId,
  DialogField,
  SelectField,
  type PortalDraft,
} from './shared';

type PortalUserDialogProps = {
  availableProjects: AdminPageProps['snapshot']['projects'];
  draft: PortalDraft;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  selectedProject: AdminPageProps['snapshot']['projects'][number] | undefined;
  selectedProjectBilling:
    | AdminPageProps['snapshot']['billingSummary']['projects'][number]
    | undefined;
  selectedProjectTokens: number;
  selectedProjectTraffic:
    | AdminPageProps['snapshot']['usageSummary']['projects'][number]
    | undefined;
  setDraft: Dispatch<SetStateAction<PortalDraft>>;
  snapshot: AdminPageProps['snapshot'];
};

export function PortalUserDialog({
  availableProjects,
  draft,
  onOpenChange,
  onSubmit,
  open,
  selectedProject,
  selectedProjectBilling,
  selectedProjectTokens,
  selectedProjectTraffic,
  setDraft,
  snapshot,
}: PortalUserDialogProps) {
  const { formatNumber, t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,56rem)]">
        <DialogHeader>
          <DialogTitle>
            {draft.id ? t('Edit portal user') : t('Create portal user')}
          </DialogTitle>
          <DialogDescription>
            {t(
              'Portal identities are scoped to a tenant and project so usage, billing, and request posture remain attributable.',
            )}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t(
              'Capture the user identity first, then bind it to a specific workspace.',
            )}
            title={t('Portal identity')}
          >
            <FormGrid columns={2}>
              <DialogField htmlFor="portal-name" label={t('Display name')}>
                <Input
                  id="portal-name"
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

              <DialogField htmlFor="portal-email" label={t('Email')}>
                <Input
                  id="portal-email"
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
                htmlFor="portal-password"
                label={draft.id ? t('New password') : t('Password')}
                description={
                  draft.id
                    ? t('Leave blank to keep the current secret.')
                    : t('Set an initial portal password.')
                }
              >
                <Input
                  id="portal-password"
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

          <FormSection
            description={t(
              'Tenant and project scope determine where traffic and billing attribution land.',
            )}
            title={t('Workspace binding')}
          >
            <FormGrid columns={2}>
              <div className="space-y-2">
                {snapshot.tenants.length ? (
                  <SelectField
                    label={t('Workspace tenant')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        workspace_tenant_id: value,
                        workspace_project_id: defaultProjectId(snapshot, value),
                      }))
                    }
                    options={snapshot.tenants.map((tenant) => ({
                      label: `${tenant.name} (${tenant.id})`,
                      value: tenant.id,
                    }))}
                    value={draft.workspace_tenant_id}
                  />
                ) : (
                  <DialogField label={t('Workspace tenant')}>
                    <Input
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          workspace_tenant_id: event.target.value,
                        }))
                      }
                      required
                      value={draft.workspace_tenant_id}
                    />
                  </DialogField>
                )}
              </div>

              <div className="space-y-2">
                {availableProjects.length ? (
                  <SelectField
                    label={t('Workspace project')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        workspace_project_id: value,
                      }))
                    }
                    options={availableProjects.map((project) => ({
                      label: `${project.name} (${project.id})`,
                      value: project.id,
                    }))}
                    value={draft.workspace_project_id}
                  />
                ) : (
                  <DialogField label={t('Workspace project')}>
                    <Input
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          workspace_project_id: event.target.value,
                        }))
                      }
                      required
                      value={draft.workspace_project_id}
                    />
                  </DialogField>
                )}
              </div>
            </FormGrid>

            <InlineAlert
              description={t(
                '{workspace} | Requests: {requests} | Usage units: {units} | Tokens: {tokens}',
                {
                  requests: formatNumber(selectedProjectTraffic?.request_count ?? 0),
                  tokens: formatNumber(selectedProjectTokens),
                  units: formatNumber(selectedProjectBilling?.used_units ?? 0),
                  workspace: selectedProject?.name ?? t('Unassigned workspace'),
                },
              )}
              showIcon
              title={t('Selected workspace posture')}
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
              {draft.id ? t('Save portal user') : t('Create portal user')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
