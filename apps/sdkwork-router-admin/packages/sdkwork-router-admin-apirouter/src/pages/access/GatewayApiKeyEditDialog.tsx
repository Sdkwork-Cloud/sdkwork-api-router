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
  ApiKeyGroupRecord,
  GatewayApiKeyRecord,
} from 'sdkwork-router-admin-types';

import { DialogField, SelectField } from '../shared';
import type { EditDraft } from './shared';
import { formatEnvironmentLabel } from './shared';

type GatewayApiKeyEditDialogProps = {
  availableGroups: ApiKeyGroupRecord[];
  draft: EditDraft;
  editingKey: GatewayApiKeyRecord | null;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  setDraft: Dispatch<SetStateAction<EditDraft>>;
};

export function GatewayApiKeyEditDialog({
  availableGroups,
  draft,
  editingKey,
  onOpenChange,
  onSubmit,
  setDraft,
}: GatewayApiKeyEditDialogProps) {
  const { t } = useAdminI18n();
  const groupOptions = [
    { label: t('No group'), value: '' },
    ...availableGroups.map((group) => ({
      label: `${group.name} (${group.slug})${group.active ? '' : ` - ${t('Inactive')}`}`,
      value: group.group_id,
    })),
  ];

  return (
    <Dialog open={Boolean(editingKey)} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,42rem)]">
        <DialogHeader>
          <DialogTitle>{t('Edit API key')}</DialogTitle>
          <DialogDescription>
            {t('Update display metadata without changing the current workbench selection.')}
          </DialogDescription>
        </DialogHeader>
        {editingKey ? (
          <form className="space-y-6" onSubmit={onSubmit}>
            <FormSection title={t('Key metadata')}>
              <FormGrid columns={2}>
                <DialogField label={t('Workspace')}>
                  <Input
                    disabled
                    value={`${editingKey.tenant_id} / ${editingKey.project_id} / ${formatEnvironmentLabel(editingKey.environment, t)}`}
                  />
                </DialogField>
                <SelectField
                  label={t('API key group')}
                  onValueChange={(value) =>
                    setDraft((current) => ({
                      ...current,
                      api_key_group_id: value,
                    }))
                  }
                  options={groupOptions}
                  value={draft.api_key_group_id}
                />
                <DialogField label={t('Hashed key')}>
                  <Input disabled value={editingKey.hashed_key} />
                </DialogField>
                <DialogField htmlFor="edit-label" label={t('Label')}>
                  <Input
                    id="edit-label"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setDraft((current) => ({
                        ...current,
                        label: event.target.value,
                      }))
                    }
                    required
                    value={draft.label}
                  />
                </DialogField>
                <DialogField htmlFor="edit-expiry" label={t('Expires at')}>
                  <Input
                    id="edit-expiry"
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
                <DialogField htmlFor="edit-notes" label={t('Notes')}>
                  <Textarea
                    id="edit-notes"
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
            <FormActions>
              <Button
                onClick={() => onOpenChange(false)}
                type="button"
                variant="outline"
              >
                {t('Cancel')}
              </Button>
              <Button type="submit" variant="primary">
                {t('Save changes')}
              </Button>
            </FormActions>
          </form>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
