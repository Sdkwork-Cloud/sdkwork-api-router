import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  InlineAlert,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  ChannelModelRecord,
  CredentialRecord,
  ModelPriceRecord,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

import {
  credentialStorageLabel,
  priceUnitLabel,
  providerChannelIds,
  type CatalogLane,
  type ChannelRecord,
  type PendingDelete,
  type VariantRecord,
} from './shared';

type CatalogDetailPanelProps = {
  catalogLane: CatalogLane;
  channelNameById: Map<string, string>;
  defaultChannelId: string;
  onDeleteItem: (deleteTarget: NonNullable<PendingDelete>) => void;
  onEditChannelModel: (record: ChannelModelRecord) => void;
  onEditModelPrice: (record: ModelPriceRecord) => void;
  onPublishVariant: (channelId: string, variant: VariantRecord) => void;
  onStartPricing: (record: ChannelModelRecord) => void;
  providerNameById: Map<string, string>;
  selectedChannel: ChannelRecord | null;
  selectedChannelProviderCount: number;
  selectedChannelModels: ChannelModelRecord[];
  selectedCredential: CredentialRecord | null;
  selectedModelPrices: ModelPriceRecord[];
  selectedProvider: ProxyProviderRecord | null;
  selectedPublication: ChannelModelRecord | null;
  selectedVariant: VariantRecord | null;
};

export function CatalogDetailPanel({
  catalogLane,
  channelNameById,
  defaultChannelId,
  onDeleteItem,
  onEditChannelModel,
  onEditModelPrice,
  onPublishVariant,
  onStartPricing,
  providerNameById,
  selectedChannel,
  selectedChannelProviderCount,
  selectedChannelModels,
  selectedCredential,
  selectedModelPrices,
  selectedProvider,
  selectedPublication,
  selectedVariant,
}: CatalogDetailPanelProps) {
  const { formatNumber, t } = useAdminI18n();

  if (catalogLane === 'channels' && selectedChannel) {
    return (
      <div className="space-y-4">
        <DescriptionList columns={2}>
          <DescriptionItem>
            <DescriptionTerm>{t('Channel id')}</DescriptionTerm>
            <DescriptionDetails mono>{selectedChannel.id}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Providers')}</DescriptionTerm>
            <DescriptionDetails>{formatNumber(selectedChannelProviderCount)}</DescriptionDetails>
          </DescriptionItem>
        </DescriptionList>
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t('Published models')}</CardTitle>
            <CardDescription>
              {t('Channel model publications and their pricing rows.')}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {selectedChannelModels.map((model) => (
              <div
                className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] p-3"
                key={`${model.channel_id}:${model.model_id}`}
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <div className="font-medium">{model.model_display_name}</div>
                    <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                      {model.model_id}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      onClick={() => onEditChannelModel(model)}
                      size="sm"
                      type="button"
                      variant="ghost"
                    >
                      {t('Edit')}
                    </Button>
                    <Button
                      onClick={() => onStartPricing(model)}
                      size="sm"
                      type="button"
                      variant="outline"
                    >
                      {t('Add pricing')}
                    </Button>
                    <Button
                      onClick={() =>
                        onDeleteItem({
                          kind: 'channel-model',
                          label: `${model.model_display_name} / ${model.model_id}`,
                          channelId: model.channel_id,
                          modelId: model.model_id,
                        })
                      }
                      size="sm"
                      type="button"
                      variant="danger"
                    >
                      {t('Delete')}
                    </Button>
                  </div>
                </div>
              </div>
            ))}
            {selectedChannelModels.length === 0 ? (
              <InlineAlert
                description={t('Publish a provider model into this channel to start exposing it to router consumers.')}
                title={t('No channel publications yet')}
                tone="info"
              />
            ) : null}
          </CardContent>
        </Card>
        {selectedPublication ? (
          <Card>
            <CardHeader>
              <CardTitle className="text-base">
                {t('Pricing for {name}', { name: selectedPublication.model_display_name })}
              </CardTitle>
              <CardDescription>
                {t('Provider-specific billing rows for the selected publication.')}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              {selectedModelPrices.map((record) => (
                <div
                  className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] p-3"
                  key={`${record.channel_id}:${record.model_id}:${record.proxy_provider_id}`}
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div>
                      <div className="font-medium">
                        {providerNameById.get(record.proxy_provider_id)
                          ?? record.proxy_provider_id}
                      </div>
                      <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                        {record.currency_code} / {priceUnitLabel(record.price_unit)}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Button
                        onClick={() => onEditModelPrice(record)}
                        size="sm"
                        type="button"
                        variant="ghost"
                      >
                        {t('Edit')}
                      </Button>
                      <Button
                        onClick={() =>
                          onDeleteItem({
                            kind: 'model-price',
                            label: `${record.model_id} / ${record.proxy_provider_id}`,
                            channelId: record.channel_id,
                            modelId: record.model_id,
                            proxyProviderId: record.proxy_provider_id,
                          })
                        }
                        size="sm"
                        type="button"
                        variant="danger"
                      >
                        {t('Delete')}
                      </Button>
                    </div>
                  </div>
                </div>
              ))}
              {selectedModelPrices.length === 0 ? (
                <InlineAlert
                  description={t('No provider pricing rows exist for the selected publication.')}
                  title={t('Pricing is empty')}
                  tone="warning"
                />
              ) : null}
            </CardContent>
          </Card>
        ) : null}
      </div>
    );
  }

  if (catalogLane === 'providers' && selectedProvider) {
    return (
      <div className="space-y-4">
        <DescriptionList columns={2}>
          <DescriptionItem>
            <DescriptionTerm>{t('Provider id')}</DescriptionTerm>
            <DescriptionDetails mono>{selectedProvider.id}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Adapter')}</DescriptionTerm>
            <DescriptionDetails>{selectedProvider.adapter_kind}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Primary channel')}</DescriptionTerm>
            <DescriptionDetails>
              {channelNameById.get(selectedProvider.channel_id) ?? selectedProvider.channel_id}
            </DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Base URL')}</DescriptionTerm>
            <DescriptionDetails mono>{selectedProvider.base_url}</DescriptionDetails>
          </DescriptionItem>
        </DescriptionList>
        <InlineAlert
          description={providerChannelIds(selectedProvider)
            .map((channelId) => channelNameById.get(channelId) ?? channelId)
            .join(', ')}
          title={t('Bound channels')}
          tone="info"
        />
      </div>
    );
  }

  if (catalogLane === 'credentials' && selectedCredential) {
    return (
      <DescriptionList columns={2}>
        <DescriptionItem>
          <DescriptionTerm>{t('Tenant')}</DescriptionTerm>
          <DescriptionDetails>{selectedCredential.tenant_id}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Provider')}</DescriptionTerm>
          <DescriptionDetails>
            {providerNameById.get(selectedCredential.provider_id)
              ?? selectedCredential.provider_id}
          </DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Backend')}</DescriptionTerm>
          <DescriptionDetails>{selectedCredential.secret_backend}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Storage')}</DescriptionTerm>
          <DescriptionDetails>
            {credentialStorageLabel(selectedCredential)}
          </DescriptionDetails>
        </DescriptionItem>
      </DescriptionList>
    );
  }

  if (catalogLane === 'variants' && selectedVariant) {
    return (
      <div className="space-y-4">
        <DescriptionList columns={2}>
          <DescriptionItem>
            <DescriptionTerm>{t('Provider')}</DescriptionTerm>
            <DescriptionDetails>
              {providerNameById.get(selectedVariant.provider_id) ?? selectedVariant.provider_id}
            </DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Capabilities')}</DescriptionTerm>
            <DescriptionDetails>{selectedVariant.capabilities.join(', ') || '-'}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Streaming')}</DescriptionTerm>
            <DescriptionDetails>{selectedVariant.streaming ? t('Enabled') : t('Disabled')}</DescriptionDetails>
          </DescriptionItem>
          <DescriptionItem>
            <DescriptionTerm>{t('Context window')}</DescriptionTerm>
            <DescriptionDetails>{selectedVariant.context_window ?? '-'}</DescriptionDetails>
          </DescriptionItem>
        </DescriptionList>
        <Button
          onClick={() => onPublishVariant(selectedChannel?.id ?? defaultChannelId, selectedVariant)}
          type="button"
          variant="primary"
        >
          {t('Publish to channel')}
        </Button>
      </div>
    );
  }

  return null;
}
