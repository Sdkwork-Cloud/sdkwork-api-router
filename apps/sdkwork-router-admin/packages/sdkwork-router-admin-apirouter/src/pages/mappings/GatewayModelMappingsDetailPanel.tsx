import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import type { GatewayModelMappingRecord } from '../../services/gatewayOverlayStore';
import { formatDateLabel } from './shared';

type GatewayModelMappingsDetailPanelProps = {
  selectedMapping: GatewayModelMappingRecord;
};

export function GatewayModelMappingsDetailPanel({
  selectedMapping,
}: GatewayModelMappingsDetailPanelProps) {
  const { t } = useAdminI18n();

  return (
    <div className="space-y-4">
      <DescriptionList columns={2}>
        <DescriptionItem>
          <DescriptionTerm>{t('Status')}</DescriptionTerm>
          <DescriptionDetails>
            {selectedMapping.status === 'active' ? t('Active') : t('Disabled')}
          </DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Created')}</DescriptionTerm>
          <DescriptionDetails>{selectedMapping.created_at}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Effective from')}</DescriptionTerm>
          <DescriptionDetails>
            {formatDateLabel(selectedMapping.effective_from)}
          </DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Effective to')}</DescriptionTerm>
          <DescriptionDetails>
            {formatDateLabel(selectedMapping.effective_to)}
          </DescriptionDetails>
        </DescriptionItem>
      </DescriptionList>

      <div className="space-y-3">
        {selectedMapping.rules.map((rule, index) => (
          <Card key={rule.id}>
            <CardHeader className="space-y-2">
              <CardTitle className="text-base">
                {t('Rule {index}', { index: index + 1 })}
              </CardTitle>
              <CardDescription>
                {t('Map one public-facing source model onto a target channel model.')}
              </CardDescription>
            </CardHeader>
            <CardContent className="grid gap-3">
              <DescriptionList columns={1}>
                <DescriptionItem>
                  <DescriptionTerm>{t('Source')}</DescriptionTerm>
                  <DescriptionDetails>
                    {rule.source_channel_name} / {rule.source_model_name} (
                    {rule.source_model_id})
                  </DescriptionDetails>
                </DescriptionItem>
                <DescriptionItem>
                  <DescriptionTerm>{t('Target')}</DescriptionTerm>
                  <DescriptionDetails>
                    {rule.target_channel_name} / {rule.target_model_name} (
                    {rule.target_model_id})
                  </DescriptionDetails>
                </DescriptionItem>
              </DescriptionList>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
