import type { ChangeEvent, Dispatch, FormEvent, SetStateAction } from 'react';
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
import type {
  AdminWorkspaceSnapshot,
  ProjectRecord,
} from 'sdkwork-router-admin-types';

import type { GatewayModelMappingRecord, GatewayRouteMode } from '../../services/gatewayOverlayStore';
import { DialogField, SelectField } from '../shared';
import { filterApiKeyGroupsByScope, type CreateDraft } from './shared';

type GatewayApiKeyCreateDialogProps = {
  availableProjects: ProjectRecord[];
  draft: CreateDraft;
  modelMappings: GatewayModelMappingRecord[];
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setDraft: Dispatch<SetStateAction<CreateDraft>>;
  snapshot: AdminWorkspaceSnapshot;
};

export function GatewayApiKeyCreateDialog({
  availableProjects,
  draft,
  modelMappings,
  onOpenChange,
  onSubmit,
  open,
  setDraft,
  snapshot,
}: GatewayApiKeyCreateDialogProps) {
  const { t } = useAdminI18n();
  const availableGroups = filterApiKeyGroupsByScope(snapshot.apiKeyGroups, {
    tenant_id: draft.tenant_id,
    project_id: draft.project_id,
    environment: draft.environment,
  }).filter((group) => group.active);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,68rem)]">
        <DialogHeader>
          <DialogTitle>{t('Create API key')}</DialogTitle>
          <DialogDescription>
            {t('Issue a new API key, set its workspace scope, and define the initial route posture in one flow.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t('Select the tenant, project, and environment that own the new key.')}
            title={t('Workspace scope')}
          >
            <FormGrid columns={2}>
              {snapshot.tenants.length ? (
                <SelectField
                  label={t('Tenant')}
                  onValueChange={(value) =>
                    setDraft((current) => ({
                      ...current,
                      tenant_id: value,
                      project_id:
                        snapshot.projects.find(
                          (project) => project.tenant_id === value,
                        )?.id ?? current.project_id,
                      api_key_group_id: '',
                    }))
                  }
                  options={snapshot.tenants.map((tenant) => ({
                    label: `${tenant.name} (${tenant.id})`,
                    value: tenant.id,
                  }))}
                  value={draft.tenant_id}
                />
              ) : (
                <DialogField htmlFor="create-tenant" label={t('Tenant')}>
                  <Input
                    id="create-tenant"
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
              {availableProjects.length ? (
                <SelectField
                  label={t('Project')}
                onValueChange={(value) =>
                    setDraft((current) => ({
                      ...current,
                      project_id: value,
                      api_key_group_id: '',
                    }))
                  }
                  options={availableProjects.map((project) => ({
                    label: `${project.name} (${project.id})`,
                    value: project.id,
                  }))}
                  value={draft.project_id}
                />
              ) : (
                <DialogField htmlFor="create-project" label={t('Project')}>
                  <Input
                    id="create-project"
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
              <SelectField<'live' | 'staging' | 'test'>
                label={t('Environment')}
                onValueChange={(value) =>
                  setDraft((current) => ({
                    ...current,
                    environment: value,
                    api_key_group_id: '',
                  }))
                }
                options={[
                  { label: t('Live'), value: 'live' },
                  { label: t('Staging'), value: 'staging' },
                  { label: t('Test'), value: 'test' },
                ]}
                value={draft.environment as 'live' | 'staging' | 'test'}
              />
              <SelectField
                label={t('API key group')}
                onValueChange={(value) =>
                  setDraft((current) => ({
                    ...current,
                    api_key_group_id: value,
                  }))
                }
                options={[
                  { label: t('No group'), value: '' },
                  ...availableGroups.map((group) => ({
                    label: `${group.name} (${group.slug})`,
                    value: group.group_id,
                  })),
                ]}
                value={draft.api_key_group_id}
              />
              <DialogField htmlFor="create-label" label={t('Label')}>
                <Input
                  id="create-label"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      label: event.target.value,
                    }))
                  }
                  placeholder={t('Workspace default API key')}
                  value={draft.label}
                />
              </DialogField>
            </FormGrid>
          </FormSection>
          <FormSection
            description={t(
              'Control expiration, notes, and whether the plaintext key is generated or provided manually.',
            )}
            title={t('Key lifecycle')}
          >
            <FormGrid columns={2}>
              <DialogField htmlFor="create-expiry" label={t('Expires at')}>
                <Input
                  id="create-expiry"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      expires_at: event.target.value,
                    }))
                  }
                  type="datetime-local"
                  value={draft.expires_at}
                />
              </DialogField>
              <DialogField
                description={t('Leave empty to let the gateway generate the plaintext.')}
                htmlFor="create-plaintext"
                label={t('Custom API key')}
              >
                <Input
                  id="create-plaintext"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      plaintext_key: event.target.value,
                    }))
                  }
                  placeholder={t('sk-router-live-demo')}
                  value={draft.plaintext_key}
                />
              </DialogField>
              <DialogField htmlFor="create-notes" label={t('Notes')}>
                <Textarea
                  id="create-notes"
                  onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                    setDraft((current) => ({
                      ...current,
                      notes: event.target.value,
                    }))
                  }
                  rows={4}
                  value={draft.notes}
                />
              </DialogField>
            </FormGrid>
          </FormSection>
          <FormSection
            description={t(
              'Set the initial route mode, optional provider pinning, and optional model mapping.',
            )}
            title={t('Route policy')}
          >
            <FormGrid columns={2}>
              <SelectField<GatewayRouteMode>
                label={t('Route mode')}
                onValueChange={(value) =>
                  setDraft((current) => ({ ...current, route_mode: value }))
                }
                options={[
                  {
                    label: t('SDKWork gateway default'),
                    value: 'sdkwork-remote',
                  },
                  { label: t('Custom provider'), value: 'custom' },
                ]}
                value={draft.route_mode}
              />
              <SelectField
                disabled={draft.route_mode !== 'custom'}
                label={t('Pinned provider')}
                onValueChange={(value) =>
                  setDraft((current) => ({
                    ...current,
                    route_provider_id: value,
                  }))
                }
                options={[
                  { label: t('Gateway default'), value: '' },
                  ...snapshot.providers.map((provider) => ({
                    label: `${provider.display_name} (${provider.id})`,
                    value: provider.id,
                  })),
                ]}
                value={draft.route_provider_id}
              />
              <SelectField
                label={t('Model mapping')}
                onValueChange={(value) =>
                  setDraft((current) => ({
                    ...current,
                    model_mapping_id: value,
                  }))
                }
                options={[
                  { label: t('No mapping'), value: '' },
                  ...modelMappings.map((mapping) => ({
                    label: mapping.name,
                    value: mapping.id,
                  })),
                ]}
                value={draft.model_mapping_id}
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
              {t('Create API key')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
