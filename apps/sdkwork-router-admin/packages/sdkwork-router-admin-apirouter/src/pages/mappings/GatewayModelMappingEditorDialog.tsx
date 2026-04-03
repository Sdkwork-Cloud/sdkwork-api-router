import type {
  ChangeEvent,
  Dispatch,
  FormEvent,
  SetStateAction,
} from 'react';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
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

import type { GatewayModelCatalogOption } from '../../services/gatewayViewService';
import type { GatewayModelMappingStatus } from '../../services/gatewayOverlayStore';
import { DialogField, SelectField } from '../shared';
import type { MappingDraft } from './shared';

type GatewayModelMappingEditorDialogProps = {
  catalog: GatewayModelCatalogOption[];
  editingMappingId: string | null;
  mappingDraft: MappingDraft;
  onAddRule: () => void;
  onOpenChange: (open: boolean) => void;
  onRemoveRule: (ruleId: string) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onUpdateRule: (
    ruleId: string,
    field: 'source_value' | 'target_value',
    value: string,
  ) => void;
  open: boolean;
  setMappingDraft: Dispatch<SetStateAction<MappingDraft>>;
};

export function GatewayModelMappingEditorDialog({
  catalog,
  editingMappingId,
  mappingDraft,
  onAddRule,
  onOpenChange,
  onRemoveRule,
  onSubmit,
  onUpdateRule,
  open,
  setMappingDraft,
}: GatewayModelMappingEditorDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,64rem)]">
        <DialogHeader>
          <DialogTitle>
            {editingMappingId ? t('Edit model mapping') : t('Create model mapping')}
          </DialogTitle>
          <DialogDescription>
            {t(
              'Build one or more source-to-target translation rules without leaving the shared gateway workbench.',
            )}
          </DialogDescription>
        </DialogHeader>

        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection
            description={t(
              'Capture the high-level mapping metadata before composing individual translation rules.',
            )}
            title={t('Mapping profile')}
          >
            <FormGrid columns={2}>
              <DialogField htmlFor="gateway-mapping-name" label={t('Mapping name')}>
                <Input
                  id="gateway-mapping-name"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setMappingDraft((current) => ({
                      ...current,
                      name: event.target.value,
                    }))
                  }
                  required
                  value={mappingDraft.name}
                />
              </DialogField>
              <SelectField<GatewayModelMappingStatus>
                label={t('Status')}
                onValueChange={(value) =>
                  setMappingDraft((current) => ({ ...current, status: value }))
                }
                options={[
                  { label: t('Active'), value: 'active' },
                  { label: t('Disabled'), value: 'disabled' },
                ]}
                value={mappingDraft.status}
              />
              <DialogField
                htmlFor="gateway-mapping-description"
                label={t('Description')}
              >
                <Textarea
                  id="gateway-mapping-description"
                  onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                    setMappingDraft((current) => ({
                      ...current,
                      description: event.target.value,
                    }))
                  }
                  rows={4}
                  value={mappingDraft.description}
                />
              </DialogField>
              <DialogField
                htmlFor="gateway-mapping-effective-from"
                label={t('Effective from')}
              >
                <Input
                  id="gateway-mapping-effective-from"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setMappingDraft((current) => ({
                      ...current,
                      effective_from: event.target.value,
                    }))
                  }
                  required
                  type="date"
                  value={mappingDraft.effective_from}
                />
              </DialogField>
              <DialogField
                htmlFor="gateway-mapping-effective-to"
                label={t('Effective to')}
              >
                <Input
                  id="gateway-mapping-effective-to"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setMappingDraft((current) => ({
                      ...current,
                      effective_to: event.target.value,
                    }))
                  }
                  type="date"
                  value={mappingDraft.effective_to}
                />
              </DialogField>
            </FormGrid>
          </FormSection>

          <FormSection
            actions={
              <Button onClick={onAddRule} type="button" variant="outline">
                {t('Add rule')}
              </Button>
            }
            description={t('Choose a source model and a target model for each translation row.')}
            title={t('Rule builder')}
          >
            <div className="space-y-4">
              {mappingDraft.rules.map((rule, index) => (
                <Card key={rule.id}>
                  <CardHeader className="space-y-2">
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <CardTitle className="text-base">
                          {t('Rule {index}', { index: index + 1 })}
                        </CardTitle>
                        <CardDescription>
                          {t('Map one client-facing model shape onto a target channel model.')}
                        </CardDescription>
                      </div>
                      <Button
                        disabled={mappingDraft.rules.length === 1}
                        onClick={() => onRemoveRule(rule.id)}
                        size="sm"
                        type="button"
                        variant="danger"
                      >
                        {t('Remove')}
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <FormGrid columns={2}>
                      <SelectField
                        label={t('Source model')}
                        onValueChange={(value) =>
                          onUpdateRule(rule.id, 'source_value', value)
                        }
                        options={catalog.map((item) => ({
                          label: item.label,
                          value: item.value,
                        }))}
                        value={rule.source_value}
                      />
                      <SelectField
                        label={t('Target model')}
                        onValueChange={(value) =>
                          onUpdateRule(rule.id, 'target_value', value)
                        }
                        options={catalog.map((item) => ({
                          label: item.label,
                          value: item.value,
                        }))}
                        value={rule.target_value}
                      />
                    </FormGrid>
                  </CardContent>
                </Card>
              ))}
            </div>
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
              {editingMappingId ? t('Save mapping') : t('Create mapping')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
