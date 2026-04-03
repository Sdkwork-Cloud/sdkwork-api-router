import type { Dispatch, SetStateAction } from 'react';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  FormSection,
  InlineAlert,
  StatCard,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  GatewayApiKeyRecord,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

import {
  buildApiKeyCurlSnippet,
  type ApiKeyQuickSetupPlan,
  type ApiKeySetupClientId,
  type ApiKeySetupInstance,
} from '../../services/gatewayApiKeyAccessService';
import type {
  GatewayApiKeyOverlayRecord,
  GatewayModelMappingRecord,
} from '../../services/gatewayOverlayStore';
import {
  QUICK_SETUP_CLIENT_LABELS,
  QUICK_SETUP_CLIENT_ORDER,
  copyToClipboard,
} from './shared';

type GatewayApiKeyUsageDialogProps = {
  applyingClientId: ApiKeySetupClientId | null;
  gatewayBaseUrl: string;
  loadingInstances: boolean;
  mappingById: Map<string, GatewayModelMappingRecord>;
  onApplySetup: (plan: ApiKeyQuickSetupPlan) => void;
  onOpenChange: (open: boolean) => void;
  openClawInstances: ApiKeySetupInstance[];
  providerById: Map<string, ProxyProviderRecord>;
  quickSetupPlans: ApiKeyQuickSetupPlan[];
  selectedClientId: ApiKeySetupClientId;
  selectedInstanceIds: string[];
  setSelectedClientId: (clientId: ApiKeySetupClientId) => void;
  setSelectedInstanceIds: Dispatch<SetStateAction<string[]>>;
  usageKey: GatewayApiKeyRecord | null;
  usageOverlay: GatewayApiKeyOverlayRecord | null;
  usagePlaintext: string | null;
  usageStatus: string;
};

export function GatewayApiKeyUsageDialog({
  applyingClientId,
  gatewayBaseUrl,
  loadingInstances,
  mappingById,
  onApplySetup,
  onOpenChange,
  openClawInstances,
  providerById,
  quickSetupPlans,
  selectedClientId,
  selectedInstanceIds,
  setSelectedClientId,
  setSelectedInstanceIds,
  usageKey,
  usageOverlay,
  usagePlaintext,
  usageStatus,
}: GatewayApiKeyUsageDialogProps) {
  const { t } = useAdminI18n();
  const selectedPlan =
    quickSetupPlans.find((plan) => plan.id === selectedClientId)
    ?? quickSetupPlans[0]
    ?? null;

  return (
    <Dialog open={Boolean(usageKey)} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,70rem)]">
        <DialogHeader>
          <DialogTitle>{t('Usage method')}</DialogTitle>
          <DialogDescription>
            {t(
              'Quick setup keeps the API key workbench aligned with real gateway compatibility endpoints and local client bootstrapping flows.',
            )}
          </DialogDescription>
        </DialogHeader>
        {usageKey ? (
          <div className="space-y-6">
            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
              <StatCard
                description={t('Selected key label or workspace fallback.')}
                label={t('Key')}
                value={usageKey.label || usageKey.project_id}
              />
              <StatCard
                description={t('Gateway compatibility endpoint for OpenAI-style clients.')}
                label={t('Gateway endpoint')}
                value={`${gatewayBaseUrl}/v1`}
              />
              <StatCard
                description={t('Current route provider mode for the selected key.')}
                label={t('Route mode')}
                value={
                  usageOverlay?.route_mode === 'custom'
                    ? providerById.get(
                        usageOverlay.route_provider_id ?? '',
                      )?.display_name ?? t('Custom provider')
                    : t('Gateway default')
                }
              />
              <StatCard
                description={t('Model mapping applied to this key, if any.')}
                label={t('Model mapping')}
                value={
                  usageOverlay?.model_mapping_id
                    ? mappingById.get(usageOverlay.model_mapping_id)?.name
                      ?? usageOverlay.model_mapping_id
                    : t('No mapping')
                }
              />
            </div>
            <DescriptionList columns={2}>
              <DescriptionItem>
                <DescriptionTerm>{t('Authorization header')}</DescriptionTerm>
                <DescriptionDetails mono>
                  {usagePlaintext
                    ? t('Authorization: Bearer {token}', {
                        token: usagePlaintext,
                      })
                    : t('Authorization: Bearer {token}', {
                        token: '<rotate-to-reveal-a-new-api-key>',
                      })}
                </DescriptionDetails>
              </DescriptionItem>
              <DescriptionItem>
                <DescriptionTerm>{t('cURL smoke test')}</DescriptionTerm>
                <DescriptionDetails mono>
                  {buildApiKeyCurlSnippet(
                    gatewayBaseUrl,
                    usagePlaintext || '<rotate-to-reveal-a-new-api-key>',
                  )}
                </DescriptionDetails>
              </DescriptionItem>
            </DescriptionList>
            <FormSection
              description={t('Pick a client profile to view and apply the generated setup snippets.')}
              title={t('Quick setup')}
            >
              <div className="flex flex-wrap gap-2">
                {QUICK_SETUP_CLIENT_ORDER.map((clientId) => {
                  const plan = quickSetupPlans.find((item) => item.id === clientId);

                  if (!plan) return null;

                  return (
                    <Button
                      key={plan.id}
                      onClick={() => setSelectedClientId(plan.id)}
                      type="button"
                      variant={
                        selectedClientId === plan.id ? 'primary' : 'outline'
                      }
                    >
                      {t(QUICK_SETUP_CLIENT_LABELS[plan.id] ?? plan.label)}
                    </Button>
                  );
                })}
              </div>
              {selectedPlan ? (
                <div className="space-y-4">
                  <InlineAlert
                    description={t(selectedPlan.description)}
                    title={t(selectedPlan.label)}
                    tone="info"
                  />
                  {selectedPlan.requiresInstances ? (
                    <Card>
                      <CardHeader>
                        <CardTitle className="text-base">
                          {t('OpenClaw instances')}
                        </CardTitle>
                        <CardDescription>
                          {t('Choose the local instances that should receive the generated setup.')}
                        </CardDescription>
                      </CardHeader>
                      <CardContent className="space-y-3">
                        {loadingInstances ? (
                          <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                            {t('Loading local instances...')}
                          </div>
                        ) : openClawInstances.length ? (
                          openClawInstances.map((instance) => (
                            <label
                              className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3"
                              key={instance.id}
                            >
                              <Checkbox
                                checked={selectedInstanceIds.includes(
                                  instance.id,
                                )}
                                onCheckedChange={(
                                  nextChecked: boolean | 'indeterminate',
                                ) =>
                                  setSelectedInstanceIds((current) =>
                                    nextChecked === true
                                      ? Array.from(
                                          new Set([...current, instance.id]),
                                        )
                                      : current.filter(
                                          (item) => item !== instance.id,
                                        ),
                                  )
                                }
                              />
                              <div className="space-y-1">
                                <div className="font-medium text-[var(--sdk-color-text-primary)]">
                                  {instance.label}
                                </div>
                                {instance.detail ? (
                                  <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                                    {instance.detail}
                                  </div>
                                ) : null}
                              </div>
                            </label>
                          ))
                        ) : (
                          <InlineAlert
                            description={t('No OpenClaw instances were detected on this machine yet.')}
                            title={t('Instance inventory is empty')}
                            tone="warning"
                          />
                        )}
                      </CardContent>
                    </Card>
                  ) : null}
                  <div className="grid gap-4 xl:grid-cols-2">
                    {selectedPlan.snippets.map((snippet) => (
                      <Card key={snippet.id}>
                        <CardHeader>
                          <CardTitle className="text-base">
                            {t(snippet.title)}
                          </CardTitle>
                          <CardDescription>{t(snippet.target)}</CardDescription>
                        </CardHeader>
                        <CardContent>
                          <pre className="overflow-x-auto rounded-[var(--sdk-radius-control)] bg-[var(--sdk-color-surface-panel-muted)] p-3 text-xs text-[var(--sdk-color-text-secondary)]">
                            <code>{snippet.content}</code>
                          </pre>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                  <DialogFooter>
                    {usagePlaintext ? (
                      <Button
                        onClick={() => void copyToClipboard(usagePlaintext)}
                        type="button"
                        variant="outline"
                      >
                        {t('Copy API key')}
                      </Button>
                    ) : null}
                    <Button
                      disabled={
                        applyingClientId === selectedPlan.id
                        || !usagePlaintext
                        || (
                          selectedPlan.requiresInstances
                          && !selectedInstanceIds.length
                        )
                      }
                      onClick={() => onApplySetup(selectedPlan)}
                      type="button"
                      variant="primary"
                    >
                      {applyingClientId === selectedPlan.id
                        ? t('Applying...')
                        : t('Apply setup')}
                    </Button>
                  </DialogFooter>
                </div>
              ) : null}
              {usageStatus ? (
                <InlineAlert
                  description={t(usageStatus)}
                  title={t('Setup status')}
                  tone={usageStatus.startsWith('Applied') ? 'success' : 'warning'}
                />
              ) : null}
            </FormSection>
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
