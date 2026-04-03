import type { ChangeEvent, FormEvent } from 'react';
import { KeyRound, Sparkles } from 'lucide-react';
import {
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
} from 'sdkwork-router-portal-commons/framework/entry';
import { SettingsField } from 'sdkwork-router-portal-commons/framework/form';
import {
  Card,
  CardContent,
} from 'sdkwork-router-portal-commons/framework/layout';

import type {
  PortalApiKeyCreateFormState,
  PortalApiKeyGroupOption,
  PortalApiKeyCreateMode,
} from '../types';
import { ApiKeyManagedNoticeCard } from './ApiKeyManagedNoticeCard';
import { ApiKeyModeChoiceCard } from './ApiKeyModeChoiceCard';

export function PortalApiKeyCreateForm({
  formState,
  groupOptions,
  onChange,
  onSubmit,
  submitting,
}: {
  formState: PortalApiKeyCreateFormState;
  groupOptions: PortalApiKeyGroupOption[];
  onChange: (updater: (current: PortalApiKeyCreateFormState) => PortalApiKeyCreateFormState) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  submitting: boolean;
}) {
  const { t } = usePortalI18n();
  const isSystemGenerated = formState.keyMode === 'system-generated';
  const environmentOptions = [
    { value: 'live', label: t('Live') },
    { value: 'staging', label: t('Staging') },
    { value: 'test', label: t('Test') },
    { value: 'custom', label: t('Custom environment') },
  ] as const;
  const keyModeOptions: Array<{
    id: PortalApiKeyCreateMode;
    title: string;
    detail: string;
    icon: typeof Sparkles;
  }> = [
    {
      id: 'system-generated',
      title: t('System generated'),
      detail: t('Let Portal create a one-time plaintext secret that is stored in write-only mode.'),
      icon: Sparkles,
    },
    {
      id: 'custom',
      title: t('Custom key'),
      detail: t('Provide an exact plaintext value when rollout coordination requires a predefined credential.'),
      icon: KeyRound,
    },
  ];

  return (
    <form className="space-y-6" onSubmit={onSubmit}>
      <Card className="border-zinc-200 bg-zinc-50/80 shadow-none dark:border-zinc-800 dark:bg-zinc-900/50">
        <CardContent className="p-5">
          <div className="grid gap-5 lg:grid-cols-2">
            <SettingsField
              description={t(
                'Keep labels auditable for incident review, ownership review, and future rotation.',
              )}
              label={t('Key label')}
              layout="vertical"
            >
              <Input
                placeholder={t('Production rollout')}
                value={formState.label}
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  onChange((current) => ({ ...current, label: event.target.value }))
                }
              />
            </SettingsField>

            <SettingsField
              description={t('Choose which workspace boundary this key should protect.')}
              label={t('Environment boundary')}
              layout="vertical"
            >
              <Select
                value={formState.environment}
                onValueChange={(value) =>
                  onChange((current) => ({ ...current, environment: value }))
                }
              >
                <SelectTrigger>
                  <SelectValue placeholder={t('Environment boundary')} />
                </SelectTrigger>
                <SelectContent>
                  {environmentOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </SettingsField>

            <SettingsField
              description={t(
                'Bind this key to a reusable workspace group for shared routing, accounting, and rollout defaults.',
              )}
              label={t('API key group')}
              layout="vertical"
            >
              <Select
                value={formState.apiKeyGroupId}
                onValueChange={(value) =>
                  onChange((current) => ({ ...current, apiKeyGroupId: value }))
                }
              >
                <SelectTrigger>
                  <SelectValue placeholder={t('API key group')} />
                </SelectTrigger>
                <SelectContent>
                  {groupOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.value === 'none' ? t('No group binding') : option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </SettingsField>

            {formState.environment === 'custom' ? (
              <div className="lg:col-span-2">
                <SettingsField
                  description={t('Examples: canary, partner, sandbox-eu')}
                  label={t('Custom environment')}
                  layout="vertical"
                >
                  <Input
                    placeholder={t('Custom environment')}
                    value={formState.customEnvironment}
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      onChange((current) => ({ ...current, customEnvironment: event.target.value }))
                    }
                  />
                </SettingsField>
              </div>
            ) : null}

            <SettingsField
              className="lg:col-span-2"
              description={t(
                'Choose whether Portal generates the secret or stores a custom plaintext value for this workspace boundary.',
              )}
              label={t('Gateway key mode')}
              layout="vertical"
            >
              <div className="grid gap-3 md:grid-cols-2">
                {keyModeOptions.map((option) => (
                  <ApiKeyModeChoiceCard
                    key={option.id}
                    title={option.title}
                    detail={option.detail}
                    icon={option.icon}
                    selected={formState.keyMode === option.id}
                    onClick={() =>
                      onChange((current) => ({
                        ...current,
                        keyMode: option.id,
                        customKey: option.id === current.keyMode ? current.customKey : '',
                      }))
                    }
                  />
                ))}
              </div>
            </SettingsField>

            {isSystemGenerated ? (
              <div className="lg:col-span-2">
                <ApiKeyManagedNoticeCard />
              </div>
            ) : (
              <div className="lg:col-span-2">
                <SettingsField
                  description={t('Paste the exact plaintext value that should be stored in write-only mode.')}
                  label={t('API key')}
                  layout="vertical"
                >
                  <Input
                    autoComplete="off"
                    className="font-mono"
                    placeholder="skw_live_custom_portal_secret"
                    spellCheck={false}
                    value={formState.customKey}
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      onChange((current) => ({ ...current, customKey: event.target.value }))
                    }
                  />
                </SettingsField>
              </div>
            )}

            <SettingsField
              description={t('Optional. Leave empty to keep this key active until you revoke it.')}
              label={t('Expires at')}
              layout="vertical"
            >
              <Input
                type="date"
                value={formState.expiresAt}
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  onChange((current) => ({ ...current, expiresAt: event.target.value }))
                }
              />
            </SettingsField>

            <SettingsField
              description={t('Add operator context, ownership, or rollout details for future review.')}
              label={t('Notes')}
              layout="vertical"
            >
              <Textarea
                rows={5}
                placeholder={t('Operator-managed migration key')}
                value={formState.notes}
                onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                  onChange((current) => ({ ...current, notes: event.target.value }))
                }
              />
            </SettingsField>
          </div>
        </CardContent>
      </Card>

      <div className="flex flex-wrap justify-end gap-3">
        <Button disabled={submitting} variant="primary" type="submit">
          {submitting ? t('Creating API key...') : t('Create API key')}
        </Button>
      </div>
    </form>
  );
}


