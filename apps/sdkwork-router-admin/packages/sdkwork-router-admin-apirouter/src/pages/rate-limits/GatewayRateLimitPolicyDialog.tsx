import type { ChangeEvent, Dispatch, FormEvent, SetStateAction } from 'react';
import {
  Button,
  Checkbox,
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
import type { AdminWorkspaceSnapshot } from 'sdkwork-router-admin-types';

import { formatApiKeyReferenceLabel } from '../access/shared';
import { DialogField, SelectField } from '../shared';
import type { RateLimitDraft } from './shared';

type GatewayRateLimitPolicyDialogProps = {
  draft: RateLimitDraft;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setDraft: Dispatch<SetStateAction<RateLimitDraft>>;
  snapshot: AdminWorkspaceSnapshot;
};

export function GatewayRateLimitPolicyDialog({
  draft,
  onOpenChange,
  onSubmit,
  open,
  setDraft,
  snapshot,
}: GatewayRateLimitPolicyDialogProps) {
  const { t } = useAdminI18n();

  const availableApiKeys = snapshot.apiKeys.filter(
    (apiKey) => apiKey.project_id === draft.project_id,
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,68rem)]">
        <DialogHeader>
          <DialogTitle>{t('Create policy')}</DialogTitle>
          <DialogDescription>
            {t('Define the rate envelope once and optionally narrow it by API key, route, or model.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t('Choose the project boundary, then optionally add a narrower key, route, or model scope.')}
            title={t('Applied scope')}
          >
            <FormGrid columns={2}>
              {snapshot.projects.length ? (
                <SelectField
                  label={t('Project')}
                  onValueChange={(value) =>
                    setDraft((current) => ({
                      ...current,
                      project_id: value,
                      api_key_hash:
                        current.project_id === value ? current.api_key_hash : '',
                    }))
                  }
                  options={snapshot.projects.map((project) => ({
                    label: `${project.name} (${project.id})`,
                    value: project.id,
                  }))}
                  value={draft.project_id}
                />
              ) : (
                <DialogField htmlFor="rate-limit-project-id" label={t('Project')}>
                  <Input
                    id="rate-limit-project-id"
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
              <SelectField
                label={t('API key')}
                onValueChange={(value) =>
                  setDraft((current) => ({ ...current, api_key_hash: value }))
                }
                options={[
                  { label: t('All API keys'), value: '' },
                  ...availableApiKeys.map((apiKey) => ({
                    label: formatApiKeyReferenceLabel(apiKey, t),
                    value: apiKey.hashed_key,
                  })),
                ]}
                value={draft.api_key_hash}
              />
              <DialogField
                description={t('Leave empty to cover every route inside the selected project scope.')}
                htmlFor="rate-limit-route-key"
                label={t('Route')}
              >
                <Input
                  id="rate-limit-route-key"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      route_key: event.target.value,
                    }))
                  }
                  placeholder={t('Example: /v1/chat/completions')}
                  value={draft.route_key}
                />
              </DialogField>
              <DialogField
                description={t('Leave empty to cover every model routed through this scope.')}
                htmlFor="rate-limit-model-name"
                label={t('Model')}
              >
                <Input
                  id="rate-limit-model-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      model_name: event.target.value,
                    }))
                  }
                  placeholder={t('Example: gpt-4.1')}
                  value={draft.model_name}
                />
              </DialogField>
            </FormGrid>
          </FormSection>

          <FormSection
            description={t('Set the identifier, sustained window, and burst allowance that the gateway should enforce.')}
            title={t('Policy envelope')}
          >
            <FormGrid columns={2}>
              <DialogField htmlFor="rate-limit-policy-id" label={t('Policy ID')}>
                <Input
                  id="rate-limit-policy-id"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      policy_id: event.target.value,
                    }))
                  }
                  required
                  value={draft.policy_id}
                />
              </DialogField>
              <DialogField htmlFor="rate-limit-requests" label={t('Requests per window')}>
                <Input
                  id="rate-limit-requests"
                  min={1}
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      requests_per_window: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={draft.requests_per_window}
                />
              </DialogField>
              <DialogField htmlFor="rate-limit-window-seconds" label={t('Window seconds')}>
                <Input
                  id="rate-limit-window-seconds"
                  min={1}
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      window_seconds: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={draft.window_seconds}
                />
              </DialogField>
              <DialogField htmlFor="rate-limit-burst" label={t('Burst requests')}>
                <Input
                  id="rate-limit-burst"
                  min={1}
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setDraft((current) => ({
                      ...current,
                      burst_requests: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={draft.burst_requests}
                />
              </DialogField>
            </FormGrid>
            <label className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3">
              <Checkbox
                checked={draft.enabled}
                onCheckedChange={(checked: boolean | 'indeterminate') =>
                  setDraft((current) => ({
                    ...current,
                    enabled: checked === true,
                  }))
                }
              />
              <div className="space-y-1">
                <div className="font-medium text-[var(--sdk-color-text-primary)]">
                  {t('Enabled')}
                </div>
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Disable this if you want to register the policy now but keep enforcement inactive.')}
                </div>
              </div>
            </label>
          </FormSection>

          <FormSection
            description={t('Capture operator intent so future reviews can explain why this policy exists.')}
            title={t('Notes')}
          >
            <DialogField htmlFor="rate-limit-notes" label={t('Notes')}>
              <Textarea
                id="rate-limit-notes"
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
          </FormSection>

          <FormActions>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {t('Create policy')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
