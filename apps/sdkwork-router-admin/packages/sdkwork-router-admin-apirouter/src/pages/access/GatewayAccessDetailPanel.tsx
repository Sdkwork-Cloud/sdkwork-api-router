import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  InlineAlert,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  ApiKeyGroupRecord,
  GatewayApiKeyRecord,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

import type { GatewayModelMappingRecord } from '../../services/gatewayOverlayStore';
import { readGatewayApiKeyOverlay } from '../../services/gatewayOverlayStore';
import {
  copyToClipboard,
  formatAccountingModeLabel,
  formatEnvironmentLabel,
  formatTimestamp,
  resolvePlaintextForKey,
} from './shared';

type GatewayAccessDetailPanelProps = {
  groupById: Map<string, ApiKeyGroupRecord>;
  groupPolicyTitle: string;
  mappingById: Map<string, GatewayModelMappingRecord>;
  providerById: Map<string, ProxyProviderRecord>;
  selectedKey: GatewayApiKeyRecord;
};

export function GatewayAccessDetailPanel({
  groupById,
  groupPolicyTitle,
  mappingById,
  providerById,
  selectedKey,
}: GatewayAccessDetailPanelProps) {
  const { t } = useAdminI18n();
  const overlay = readGatewayApiKeyOverlay(selectedKey.hashed_key);
  const provider = overlay.route_provider_id
    ? providerById.get(overlay.route_provider_id)
    : null;
  const mapping = overlay.model_mapping_id
    ? mappingById.get(overlay.model_mapping_id)
    : null;
  const group = selectedKey.api_key_group_id
    ? groupById.get(selectedKey.api_key_group_id)
    : null;
  const plaintextKey = resolvePlaintextForKey(selectedKey);

  return (
    <div className="space-y-4">
      <DescriptionList columns={2}>
        <DescriptionItem>
          <DescriptionTerm>{t('Tenant')}</DescriptionTerm>
          <DescriptionDetails>{selectedKey.tenant_id}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Project')}</DescriptionTerm>
          <DescriptionDetails>{selectedKey.project_id}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Environment')}</DescriptionTerm>
          <DescriptionDetails>{formatEnvironmentLabel(selectedKey.environment, t)}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Expires at')}</DescriptionTerm>
          <DescriptionDetails>
            {formatTimestamp(selectedKey.expires_at_ms)}
          </DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Hashed key')}</DescriptionTerm>
          <DescriptionDetails mono>{selectedKey.hashed_key}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Last used')}</DescriptionTerm>
          <DescriptionDetails>
            {formatTimestamp(selectedKey.last_used_at_ms)}
          </DescriptionDetails>
        </DescriptionItem>
      </DescriptionList>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{groupPolicyTitle}</CardTitle>
          <CardDescription>
            {t('Group defaults and inherited posture bound to this key.')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <DescriptionList columns={1}>
            <DescriptionItem>
              <DescriptionTerm>{t('API key group')}</DescriptionTerm>
              <DescriptionDetails>
                {group?.name ?? selectedKey.api_key_group_id ?? t('No group assigned')}
              </DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Default scope')}</DescriptionTerm>
              <DescriptionDetails>
                {group?.default_capability_scope ?? t('No default scope')}
              </DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Accounting mode')}</DescriptionTerm>
              <DescriptionDetails>
                {formatAccountingModeLabel(group?.default_accounting_mode, t)}
              </DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Routing profile')}</DescriptionTerm>
              <DescriptionDetails>
                {group?.default_routing_profile_id ?? t('No routing profile')}
              </DescriptionDetails>
            </DescriptionItem>
          </DescriptionList>
          {group?.description ? (
            <div className="rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3 text-sm text-[var(--sdk-color-text-secondary)]">
              {group.description}
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('Route posture')}</CardTitle>
          <CardDescription>
            {t('Selected provider mode and mapping for this key.')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <DescriptionList columns={1}>
            <DescriptionItem>
              <DescriptionTerm>{t('Provider mode')}</DescriptionTerm>
              <DescriptionDetails>
                {overlay.route_mode === 'custom'
                  ? provider?.display_name ?? provider?.id ?? t('Custom provider')
                  : t('SDKWork gateway default')}
              </DescriptionDetails>
            </DescriptionItem>
            <DescriptionItem>
              <DescriptionTerm>{t('Model mapping')}</DescriptionTerm>
              <DescriptionDetails>
                {mapping?.name ?? t('No model mapping')}
              </DescriptionDetails>
            </DescriptionItem>
          </DescriptionList>
          {plaintextKey ? (
            <Button
              onClick={() => void copyToClipboard(plaintextKey)}
              type="button"
              variant="outline"
            >
              {t('Copy plaintext API key')}
            </Button>
          ) : (
            <InlineAlert
              description={t(
                'Plaintext is not visible on this device. Create a replacement if you need to copy it again.',
              )}
              title={t('Plaintext not available')}
              tone="warning"
            />
          )}
        </CardContent>
      </Card>
    </div>
  );
}
