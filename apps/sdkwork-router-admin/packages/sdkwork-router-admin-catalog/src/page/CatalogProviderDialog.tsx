import type {
  ChangeEvent,
  Dispatch,
  FormEvent,
  SetStateAction,
} from 'react';
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
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import { DialogField, SelectField, type ProviderDraft } from './shared';

type CatalogProviderDialogProps = {
  editingProviderId: string | null;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  providerDraft: ProviderDraft;
  setProviderDraft: Dispatch<SetStateAction<ProviderDraft>>;
  snapshot: AdminPageProps['snapshot'];
};

export function CatalogProviderDialog({
  editingProviderId,
  onOpenChange,
  onSubmit,
  open,
  providerDraft,
  setProviderDraft,
  snapshot,
}: CatalogProviderDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,56rem)]">
        <DialogHeader>
          <DialogTitle>{editingProviderId ? t('Edit provider') : t('Create provider')}</DialogTitle>
          <DialogDescription>
            {t('Capture upstream connectivity and channel bindings with the shared form primitives.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection title={t('Provider profile')}>
            <FormGrid columns={2}>
              <DialogField htmlFor="provider-id" label={t('Provider id')}>
                <Input
                  disabled={Boolean(editingProviderId)}
                  id="provider-id"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setProviderDraft((current) => ({
                      ...current,
                      id: event.target.value,
                    }))
                  }
                  required
                  value={providerDraft.id}
                />
              </DialogField>
              <DialogField htmlFor="provider-name" label={t('Display name')}>
                <Input
                  id="provider-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setProviderDraft((current) => ({
                      ...current,
                      display_name: event.target.value,
                    }))
                  }
                  required
                  value={providerDraft.display_name}
                />
              </DialogField>
              <DialogField htmlFor="provider-adapter" label={t('Adapter kind')}>
                <Input
                  id="provider-adapter"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setProviderDraft((current) => ({
                      ...current,
                      adapter_kind: event.target.value,
                    }))
                  }
                  required
                  value={providerDraft.adapter_kind}
                />
              </DialogField>
              <DialogField htmlFor="provider-url" label={t('Base URL')}>
                <Input
                  id="provider-url"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setProviderDraft((current) => ({
                      ...current,
                      base_url: event.target.value,
                    }))
                  }
                  required
                  value={providerDraft.base_url}
                />
              </DialogField>
              <DialogField htmlFor="provider-extension" label={t('Extension id')}>
                <Input
                  id="provider-extension"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setProviderDraft((current) => ({
                      ...current,
                      extension_id: event.target.value,
                    }))
                  }
                  value={providerDraft.extension_id}
                />
              </DialogField>
              <SelectField
                label={t('Primary channel')}
                onValueChange={(value) =>
                  setProviderDraft((current) => ({
                    ...current,
                    primary_channel_id: value,
                    bound_channel_ids: current.bound_channel_ids.includes(value)
                      ? current.bound_channel_ids
                      : [...current.bound_channel_ids, value],
                  }))
                }
                options={snapshot.channels.map((channel) => ({
                  label: `${channel.name} (${channel.id})`,
                  value: channel.id,
                }))}
                value={providerDraft.primary_channel_id}
              />
            </FormGrid>
          </FormSection>

          <FormSection title={t('Bound channels')}>
            <div className="grid gap-3 md:grid-cols-2">
              {snapshot.channels.map((channel) => {
                const checked = providerDraft.bound_channel_ids.includes(channel.id);

                return (
                  <label
                    className="flex items-start gap-3 rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] px-4 py-3"
                    key={channel.id}
                  >
                    <Checkbox
                      checked={checked}
                      onCheckedChange={(nextChecked: boolean | 'indeterminate') =>
                        setProviderDraft((current) => ({
                          ...current,
                          bound_channel_ids:
                            nextChecked === true
                              ? Array.from(
                                  new Set([...current.bound_channel_ids, channel.id]),
                                )
                              : current.bound_channel_ids.filter(
                                  (id) => id !== channel.id,
                                ),
                        }))
                      }
                    />
                    <div>
                      <div className="font-medium">{channel.name}</div>
                      <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                        {channel.id}
                      </div>
                    </div>
                  </label>
                );
              })}
            </div>
          </FormSection>

          <FormActions>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {editingProviderId ? t('Save provider') : t('Create provider')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
