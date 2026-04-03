import type { Dispatch, FormEvent, SetStateAction } from 'react';
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
import type {
  AdminWorkspaceSnapshot,
  GatewayApiKeyRecord,
} from 'sdkwork-router-admin-types';

import type {
  GatewayModelMappingRecord,
  GatewayRouteMode,
} from '../../services/gatewayOverlayStore';
import { DialogField, SelectField } from '../shared';
import type { RouteDraft } from './shared';

type GatewayApiKeyRouteDialogProps = {
  modelMappings: GatewayModelMappingRecord[];
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  routeDraft: RouteDraft;
  routeKey: GatewayApiKeyRecord | null;
  setRouteDraft: Dispatch<SetStateAction<RouteDraft>>;
  snapshot: AdminWorkspaceSnapshot;
};

export function GatewayApiKeyRouteDialog({
  modelMappings,
  onOpenChange,
  onSubmit,
  routeDraft,
  routeKey,
  setRouteDraft,
  snapshot,
}: GatewayApiKeyRouteDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={Boolean(routeKey)} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,42rem)]">
        <DialogHeader>
          <DialogTitle>{t('Route config')}</DialogTitle>
          <DialogDescription>
            {t(
              'Keep per-key route mode, provider pinning, and model mapping aligned with the local overlay behavior.',
            )}
          </DialogDescription>
        </DialogHeader>
        {routeKey ? (
          <form className="space-y-6" onSubmit={onSubmit}>
            <FormSection title={t('Route overlay')}>
              <FormGrid columns={2}>
                <DialogField label={t('API key')}>
                  <Input
                    disabled
                    value={`${routeKey.label || routeKey.project_id} (${routeKey.environment})`}
                  />
                </DialogField>
                <SelectField<'system-generated' | 'custom'>
                  label={t('Source')}
                  onValueChange={(value) =>
                    setRouteDraft((current) => ({ ...current, source: value }))
                  }
                  options={[
                    { label: t('System generated'), value: 'system-generated' },
                    { label: t('Custom'), value: 'custom' },
                  ]}
                  value={routeDraft.source}
                />
                <SelectField<GatewayRouteMode>
                  label={t('Route mode')}
                  onValueChange={(value) =>
                    setRouteDraft((current) => ({
                      ...current,
                      route_mode: value,
                    }))
                  }
                  options={[
                    {
                      label: t('SDKWork gateway default'),
                      value: 'sdkwork-remote',
                    },
                    { label: t('Custom provider'), value: 'custom' },
                  ]}
                  value={routeDraft.route_mode}
                />
                <SelectField
                  disabled={routeDraft.route_mode !== 'custom'}
                  label={t('Pinned provider')}
                  onValueChange={(value) =>
                    setRouteDraft((current) => ({
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
                  value={routeDraft.route_provider_id}
                />
                <SelectField
                  label={t('Model mapping')}
                  onValueChange={(value) =>
                    setRouteDraft((current) => ({
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
                  value={routeDraft.model_mapping_id}
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
                {t('Save route config')}
              </Button>
            </FormActions>
          </form>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
