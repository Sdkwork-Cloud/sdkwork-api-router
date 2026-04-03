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
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import { DialogField, type ChannelDraft } from './shared';

type CatalogChannelDialogProps = {
  channelDraft: ChannelDraft;
  editingChannelId: string | null;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setChannelDraft: Dispatch<SetStateAction<ChannelDraft>>;
};

export function CatalogChannelDialog({
  channelDraft,
  editingChannelId,
  onOpenChange,
  onSubmit,
  open,
  setChannelDraft,
}: CatalogChannelDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,36rem)]">
        <DialogHeader>
          <DialogTitle>{editingChannelId ? t('Edit channel') : t('Create channel')}</DialogTitle>
          <DialogDescription>
            {t('Keep channel metadata focused and manage publications separately from the detail rail.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection title={t('Channel profile')}>
            <FormGrid columns={2}>
              <DialogField htmlFor="channel-id" label={t('Channel id')}>
                <Input
                  disabled={Boolean(editingChannelId)}
                  id="channel-id"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setChannelDraft((current) => ({
                      ...current,
                      id: event.target.value,
                    }))
                  }
                  required
                  value={channelDraft.id}
                />
              </DialogField>
              <DialogField htmlFor="channel-name" label={t('Channel name')}>
                <Input
                  id="channel-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setChannelDraft((current) => ({
                      ...current,
                      name: event.target.value,
                    }))
                  }
                  required
                  value={channelDraft.name}
                />
              </DialogField>
            </FormGrid>
          </FormSection>
          <FormActions>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {editingChannelId ? t('Save channel') : t('Create channel')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
