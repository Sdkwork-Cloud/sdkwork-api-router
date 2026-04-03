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
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import {
  DialogField,
  PRICE_UNIT_OPTIONS,
  SelectField,
  type ModelPriceDraft,
} from './shared';

type CatalogModelPriceDialogProps = {
  editingModelPriceKey: string | null;
  modelPriceDraft: ModelPriceDraft;
  onOpenChange: (open: boolean) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  open: boolean;
  setModelPriceDraft: Dispatch<SetStateAction<ModelPriceDraft>>;
  snapshot: AdminPageProps['snapshot'];
};

export function CatalogModelPriceDialog({
  editingModelPriceKey,
  modelPriceDraft,
  onOpenChange,
  onSubmit,
  open,
  setModelPriceDraft,
  snapshot,
}: CatalogModelPriceDialogProps) {
  const { t } = useAdminI18n();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,64rem)]">
        <DialogHeader>
          <DialogTitle>
            {editingModelPriceKey ? t('Edit model pricing') : t('Add model pricing')}
          </DialogTitle>
          <DialogDescription>
            {t('Provider-specific pricing rows stay aligned with the selected publication.')}
          </DialogDescription>
        </DialogHeader>
        <form className="space-y-6" onSubmit={onSubmit}>
          <FormSection title={t('Pricing row')}>
            <FormGrid columns={2}>
              <DialogField label={t('Channel')}>
                <Input disabled value={modelPriceDraft.channel_id} />
              </DialogField>
              <DialogField label={t('Model')}>
                <Input disabled value={modelPriceDraft.model_id} />
              </DialogField>
              <SelectField
                label={t('Provider')}
                onValueChange={(value) =>
                  setModelPriceDraft((current) => ({
                    ...current,
                    proxy_provider_id: value,
                  }))
                }
                options={snapshot.providers.map((provider) => ({
                  label: `${provider.display_name} (${provider.id})`,
                  value: provider.id,
                }))}
                value={modelPriceDraft.proxy_provider_id}
              />
              <DialogField htmlFor="price-currency" label={t('Currency code')}>
                <Input
                  id="price-currency"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      currency_code: event.target.value,
                    }))
                  }
                  required
                  value={modelPriceDraft.currency_code}
                />
              </DialogField>
              <SelectField
                label={t('Price unit')}
                onValueChange={(value) =>
                  setModelPriceDraft((current) => ({
                    ...current,
                    price_unit: value,
                  }))
                }
                options={PRICE_UNIT_OPTIONS.map((option) => ({
                  label: t(option.label),
                  value: option.value,
                }))}
                value={modelPriceDraft.price_unit}
              />
              <DialogField htmlFor="price-input" label={t('Input price')}>
                <Input
                  id="price-input"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      input_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.input_price}
                />
              </DialogField>
              <DialogField htmlFor="price-output" label={t('Output price')}>
                <Input
                  id="price-output"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      output_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.output_price}
                />
              </DialogField>
              <DialogField htmlFor="price-cache-read" label={t('Cache read')}>
                <Input
                  id="price-cache-read"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      cache_read_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.cache_read_price}
                />
              </DialogField>
              <DialogField htmlFor="price-cache-write" label={t('Cache write')}>
                <Input
                  id="price-cache-write"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      cache_write_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.cache_write_price}
                />
              </DialogField>
              <DialogField htmlFor="price-request" label={t('Request price')}>
                <Input
                  id="price-request"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setModelPriceDraft((current) => ({
                      ...current,
                      request_price: event.target.value,
                    }))
                  }
                  required
                  type="number"
                  value={modelPriceDraft.request_price}
                />
              </DialogField>
              <SelectField<'active' | 'inactive'>
                label={t('Status')}
                onValueChange={(value) =>
                  setModelPriceDraft((current) => ({
                    ...current,
                    is_active: value === 'active',
                  }))
                }
                options={[
                  { label: t('Active'), value: 'active' },
                  { label: t('Inactive'), value: 'inactive' },
                ]}
                value={modelPriceDraft.is_active ? 'active' : 'inactive'}
              />
            </FormGrid>
          </FormSection>
          <FormActions>
            <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
              {t('Cancel')}
            </Button>
            <Button type="submit" variant="primary">
              {editingModelPriceKey ? t('Save pricing') : t('Add pricing')}
            </Button>
          </FormActions>
        </form>
      </DialogContent>
    </Dialog>
  );
}
