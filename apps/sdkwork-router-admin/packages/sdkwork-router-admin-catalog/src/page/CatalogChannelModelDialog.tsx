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
  Input,
  Textarea,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import { DialogField, SelectField, type ChannelModelDraft } from './shared';

type CatalogChannelModelDialogProps = {
  channelModelDraft: ChannelModelDraft;
  editingChannelModelKey: string | null;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setChannelModelDraft: Dispatch<SetStateAction<ChannelModelDraft>>;
  snapshot: AdminPageProps['snapshot'];
};

export function CatalogChannelModelDialog({
  channelModelDraft,
  editingChannelModelKey,
  onOpenChange,
  onSubmit,
  open,
  setChannelModelDraft,
  snapshot,
}: CatalogChannelModelDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,56rem)]">
        <DialogHeader>
          <DialogTitle>
            {editingChannelModelKey ? t('Edit channel publication') : t('Publish model to channel')}
          </DialogTitle>
          <DialogDescription>
            {t('Publish a provider variant into a public channel using the shared form system.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection title={t('Publication')}>
            <FormGrid columns={2}>
              <SelectField
                label={t('Channel')}
                onValueChange={(value) =>
                  setChannelModelDraft((current) => ({
                    ...current,
                    channel_id: value,
                  }))
                }
                options={snapshot.channels.map((channel) => ({
                  label: `${channel.name} (${channel.id})`,
                  value: channel.id,
                }))}
                value={channelModelDraft.channel_id}
              />
              <DialogField htmlFor="publication-model-id" label={t('Model id')}>
                <Input
                  id="publication-model-id"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      model_id: event.target.value,
                    }))
                  }
                  required
                  value={channelModelDraft.model_id}
                />
              </DialogField>
              <DialogField htmlFor="publication-display-name" label={t('Display name')}>
                <Input
                  id="publication-display-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      model_display_name: event.target.value,
                    }))
                  }
                  required
                  value={channelModelDraft.model_display_name}
                />
              </DialogField>
              <DialogField htmlFor="publication-capabilities" label={t('Capabilities')}>
                <Input
                  id="publication-capabilities"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      capabilities: event.target.value,
                    }))
                  }
                  value={channelModelDraft.capabilities}
                />
              </DialogField>
              <DialogField htmlFor="publication-context-window" label={t('Context window')}>
                <Input
                  id="publication-context-window"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      context_window: event.target.value,
                    }))
                  }
                  value={channelModelDraft.context_window}
                />
              </DialogField>
              <SelectField<'active' | 'inactive'>
                label={t('Streaming')}
                onValueChange={(value) =>
                  setChannelModelDraft((current) => ({
                    ...current,
                    streaming: value === 'active',
                  }))
                }
                options={[
                  { label: t('Enabled'), value: 'active' },
                  { label: t('Disabled'), value: 'inactive' },
                ]}
                value={channelModelDraft.streaming ? 'active' : 'inactive'}
              />
              <DialogField htmlFor="publication-description" label={t('Description')}>
                <Textarea
                  id="publication-description"
                  onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                    setChannelModelDraft((current) => ({
                      ...current,
                      description: event.target.value,
                    }))
                  }
                  rows={4}
                  value={channelModelDraft.description}
                />
              </DialogField>
            </FormGrid>
          </FormSection>
          <FormActions>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {editingChannelModelKey ? t('Save publication') : t('Publish model')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
