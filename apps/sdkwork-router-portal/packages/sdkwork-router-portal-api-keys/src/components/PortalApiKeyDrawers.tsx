import type { FormEvent, ReactNode } from 'react';
import { KeyRound, Link2 } from 'lucide-react';

import {
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  Badge,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons/framework/display';
import { Checkbox } from 'sdkwork-router-portal-commons/framework/entry';
import {
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
} from 'sdkwork-router-portal-commons/framework/overlays';
import type {
  ApiKeyGroupRecord,
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
} from 'sdkwork-router-portal-types';

import type {
  PortalApiKeyCreateFormState,
  PortalApiKeyGroupOption,
  PortalApiKeyUsagePreview,
} from '../types';
import type {
  ApiKeyQuickSetupPlan,
  ApiKeySetupClientId,
  ApiKeySetupInstance,
} from '../services/quickSetup';
import { PortalApiKeyCreateForm } from './PortalApiKeyCreateForm';

function DrawerInfoCard({
  children,
  title,
}: {
  children: ReactNode;
  title: ReactNode;
}) {
  return (
    <article className="rounded-2xl border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950">
      <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">{title}</div>
      <div className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">{children}</div>
    </article>
  );
}

export function PortalApiKeyDrawers({
  apiKeyGroups,
  createFormState,
  createOpen,
  createStatus,
  createdKey,
  groupOptions,
  onChangeForm,
  onCloseCreate,
  onCloseUsage,
  onCopyPlaintext,
  onCreate,
  onApplySetup,
  onChangeInstanceSelection,
  onSelectClient,
  submitting,
  applyingClientId,
  gatewayBaseUrl,
  loadingInstances,
  openClawInstances,
  quickSetupPlans,
  selectedClientId,
  selectedInstanceIds,
  selectedPlan,
  usagePlaintext,
  usageStatus,
  usageKey,
  usagePreview,
}: {
  apiKeyGroups: ApiKeyGroupRecord[];
  createFormState: PortalApiKeyCreateFormState;
  createOpen: boolean;
  createStatus: string;
  createdKey: CreatedGatewayApiKey | null;
  groupOptions: PortalApiKeyGroupOption[];
  onChangeForm: (updater: (current: PortalApiKeyCreateFormState) => PortalApiKeyCreateFormState) => void;
  onCloseCreate: () => void;
  onCloseUsage: () => void;
  onCopyPlaintext: () => void;
  onCreate: (event: FormEvent<HTMLFormElement>) => void;
  onApplySetup: () => void;
  onChangeInstanceSelection: (nextValue: string[]) => void;
  onSelectClient: (clientId: ApiKeySetupClientId) => void;
  submitting: boolean;
  applyingClientId: ApiKeySetupClientId | null;
  gatewayBaseUrl: string;
  loadingInstances: boolean;
  openClawInstances: ApiKeySetupInstance[];
  quickSetupPlans: ApiKeyQuickSetupPlan[];
  selectedClientId: ApiKeySetupClientId;
  selectedInstanceIds: string[];
  selectedPlan: ApiKeyQuickSetupPlan | null;
  usagePlaintext: string | null;
  usageStatus: string;
  usageKey: GatewayApiKeyRecord | null;
  usagePreview: PortalApiKeyUsagePreview | null;
}) {
  const { locale, t } = usePortalI18n();
  const isLatestUsageKey = createdKey && usageKey ? createdKey.hashed === usageKey.hashed_key : false;
  const canCopyPlaintext = Boolean(usagePlaintext || isLatestUsageKey);
  const canApplySetup = Boolean(
    selectedPlan
      && selectedPlan.available
      && applyingClientId !== selectedClientId
      && (!selectedPlan.requiresInstances || selectedInstanceIds.length),
  );
  const usageGroup = usageKey?.api_key_group_id
    ? apiKeyGroups.find((group) => group.group_id === usageKey.api_key_group_id) ?? null
    : null;

  return (
    <>
      <Drawer onOpenChange={(open: boolean) => !open && onCloseCreate()} open={createOpen}>
        <DrawerContent
          className="max-w-2xl"
          data-slot="portal-api-key-create-drawer"
          side="right"
          size="lg"
        >
          <DrawerHeader>
            <DrawerTitle>{t('Create API key')}</DrawerTitle>
            <DrawerDescription>
              {t(
                'Recommended key setup starts with Key label ownership, any needed Custom environment override, and the Lifecycle policy that matches the rollout plan.',
              )}
            </DrawerDescription>
          </DrawerHeader>

          <DrawerBody className="px-4 pb-5 pt-0 xl:px-6">
            {createStatus ? (
              <div className="mb-4">
                <DrawerInfoCard title={t('Status')}>{createStatus}</DrawerInfoCard>
              </div>
            ) : null}
            <PortalApiKeyCreateForm
              formState={createFormState}
              groupOptions={groupOptions}
              onChange={onChangeForm}
              onSubmit={onCreate}
              submitting={submitting}
            />
          </DrawerBody>
        </DrawerContent>
      </Drawer>

      <Drawer onOpenChange={(open: boolean) => !open && onCloseUsage()} open={Boolean(usageKey)}>
        <DrawerContent
          className="max-w-5xl"
          data-slot="portal-api-key-detail-drawer"
          side="right"
          size="xl"
        >
          <DrawerHeader>
            <DrawerTitle>{t('API key details')}</DrawerTitle>
            <DrawerDescription>
              {usagePreview?.detail ? t(usagePreview.detail) : t('How to use this key')}
            </DrawerDescription>
          </DrawerHeader>

          {usageKey && usagePreview ? (
            <>
              <DrawerBody className="px-4 pb-4 pt-0 xl:px-6">
                <div className="grid gap-5">
                  <section className="rounded-[28px] border border-zinc-200 bg-zinc-50/75 p-5 dark:border-zinc-800 dark:bg-zinc-900/55">
                    <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                      <div className="min-w-0">
                      <div className="text-lg font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                          {usageKey.label}
                        </div>
                        <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                          {t(
                            'Use this key for the {environment} environment boundary and keep rollout verification inside the same workspace posture.',
                            { environment: usageKey.environment },
                          )}
                        </p>
                      </div>
                      <div className="flex flex-wrap gap-2">
                        <Badge variant="outline">{usageKey.environment}</Badge>
                        <Badge variant="outline">
                          {usageGroup?.name ?? t('No group binding')}
                        </Badge>
                        <Badge variant={usageKey.active ? 'success' : 'warning'}>
                          {usageKey.active ? t('Active') : t('Inactive')}
                        </Badge>
                        <Badge variant={canCopyPlaintext ? 'default' : 'secondary'}>
                          {canCopyPlaintext ? t('Latest plaintext available once') : t('Write-only')}
                        </Badge>
                      </div>
                    </div>
                  </section>

                  <div className="grid gap-4 xl:grid-cols-3">
                    <DrawerInfoCard title={(
                      <span className="inline-flex items-center gap-2">
                        <Link2 className="h-4 w-4 text-primary-500" />
                        {t('Portal endpoint')}
                      </span>
                    )}
                    >
                      {gatewayBaseUrl}/v1/models
                    </DrawerInfoCard>

                    <DrawerInfoCard title={(
                      <span className="inline-flex items-center gap-2">
                        <KeyRound className="h-4 w-4 text-primary-500" />
                        {t('Authorization header')}
                      </span>
                    )}
                    >
                      {usagePreview.authorizationHeader
                        ?? t('Plaintext unavailable. Rotate this key to obtain a new one-time secret.')}
                    </DrawerInfoCard>

                    <DrawerInfoCard title={t('Expires at')}>
                      {usageKey.expires_at_ms
                        ? t('This credential expires on {date}.', {
                            date: new Intl.DateTimeFormat(locale, {
                              year: 'numeric',
                              month: 'short',
                              day: 'numeric',
                            }).format(new Date(usageKey.expires_at_ms)),
                          })
                        : t('This credential has no expiry. Keep revocation ownership explicit.')}
                    </DrawerInfoCard>
                  </div>

                  <DrawerInfoCard title={t('Key group')}>
                    <div className="space-y-2">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {usageGroup?.name ?? t('No group binding')}
                      </div>
                      {usageKey.api_key_group_id ? (
                        <div className="font-mono text-xs text-zinc-500 dark:text-zinc-400">
                          {usageKey.api_key_group_id}
                        </div>
                      ) : null}
                      {usageGroup?.default_routing_profile_id ? (
                        <div>
                          {t('Routing profile')}: {usageGroup.default_routing_profile_id}
                        </div>
                      ) : (
                        <div>{t('No routing profile override')}</div>
                      )}
                    </div>
                  </DrawerInfoCard>

                  <DrawerInfoCard title={t('How to use this key')}>
                    {t(
                      'Use this key for the {environment} environment boundary and keep rollout verification inside the same workspace posture. If the plaintext is no longer visible, create a replacement instead of depending on the UI as secret storage.',
                      { environment: usageKey.environment },
                    )}
                  </DrawerInfoCard>

                  <article className="rounded-2xl border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950">
                    <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                      {t('Quick setup')}
                    </div>
                    <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                      {t(
                        'Apply setup directly on this device for Codex, Claude Code, OpenCode, Gemini, or OpenClaw, or copy the generated snippets into your preferred environment.',
                      )}
                    </p>

                    <Tabs
                      className="mt-4"
                      onValueChange={(value: string) => onSelectClient(value as ApiKeySetupClientId)}
                      value={selectedClientId}
                    >
                      <TabsList>
                        {quickSetupPlans.map((plan) => (
                          <TabsTrigger key={plan.id} value={plan.id}>
                            {plan.label}
                          </TabsTrigger>
                        ))}
                      </TabsList>

                      {quickSetupPlans.map((plan) => (
                        <TabsContent key={plan.id} value={plan.id}>
                          <div className="space-y-4">
                            <div className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
                              <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                                {plan.label}
                              </div>
                              <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                                {t(plan.description)}
                              </p>
                            </div>

                            {plan.availabilityDetail ? (
                              <DrawerInfoCard title={t('Status')}>
                                {plan.availabilityDetail}
                              </DrawerInfoCard>
                            ) : null}

                            {plan.requiresInstances ? (
                              <div className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
                                <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                                  {t('OpenClaw instances')}
                                </div>
                                {loadingInstances ? (
                                  <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                                    {t('Loading local instances...')}
                                  </p>
                                ) : openClawInstances.length ? (
                                  <div className="mt-3 grid gap-2">
                                    {openClawInstances.map((instance) => (
                                      <label
                                        className="flex items-center gap-2 text-sm text-zinc-600 dark:text-zinc-300"
                                        key={instance.id}
                                      >
                                        <Checkbox
                                          checked={selectedInstanceIds.includes(instance.id)}
                                          onCheckedChange={(checked) =>
                                            onChangeInstanceSelection(
                                              checked
                                                ? [...selectedInstanceIds, instance.id]
                                                : selectedInstanceIds.filter((item) => item !== instance.id),
                                            )
                                          }
                                        />
                                        <span>
                                          {instance.label}
                                          {instance.detail ? ` / ${instance.detail}` : ''}
                                        </span>
                                      </label>
                                    ))}
                                  </div>
                                ) : (
                                  <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                                    {t('No OpenClaw instances were detected on this machine.')}
                                  </p>
                                )}
                              </div>
                            ) : null}

                            {plan.available
                              ? plan.snippets.map((snippet) => (
                                  <div
                                    className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60"
                                    key={snippet.id}
                                  >
                                    <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                                      {t(snippet.title)}
                                    </div>
                                    <p className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
                                      {snippet.target}
                                    </p>
                                    <pre className="mt-3 overflow-x-auto rounded-2xl bg-zinc-950 p-4 text-sm leading-6 text-zinc-300">
                                      <code>{snippet.content}</code>
                                    </pre>
                                  </div>
                                ))
                              : null}
                          </div>
                        </TabsContent>
                      ))}
                    </Tabs>
                  </article>

                  {usageKey.notes ? (
                    <DrawerInfoCard title={t('Notes')}>{usageKey.notes}</DrawerInfoCard>
                  ) : null}

                  {usageStatus ? (
                    <DrawerInfoCard title={t('Status')}>{usageStatus}</DrawerInfoCard>
                  ) : null}

                  {usagePreview.curlExample ? (
                    <article className="rounded-2xl border border-zinc-800 bg-zinc-950 p-4">
                      <div className="text-sm font-semibold text-zinc-100">{t('Quickstart snippet')}</div>
                      <pre className="mt-4 overflow-x-auto text-sm leading-6 text-zinc-300">
                        <code>{usagePreview.curlExample}</code>
                      </pre>
                    </article>
                  ) : null}
                </div>
              </DrawerBody>

              <DrawerFooter className="flex flex-wrap items-center justify-between gap-3">
                <div className="text-sm text-zinc-500 dark:text-zinc-400">
                  {t('Lifecycle policy')}
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  {canCopyPlaintext ? (
                    <Button onClick={onCopyPlaintext} variant="secondary">
                      {t('Copy plaintext')}
                    </Button>
                  ) : null}
                  <Button onClick={onCloseUsage} variant="secondary">
                    {t('Close')}
                  </Button>
                  <Button disabled={!canApplySetup} onClick={onApplySetup}>
                    {applyingClientId === selectedClientId ? t('Applying...') : t('Apply setup')}
                  </Button>
                </div>
              </DrawerFooter>
            </>
          ) : null}
        </DrawerContent>
      </Drawer>
    </>
  );
}
