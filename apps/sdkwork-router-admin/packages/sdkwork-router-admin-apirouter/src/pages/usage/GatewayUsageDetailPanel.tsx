import {
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  GatewayApiKeyRecord,
  UsageRecord,
} from 'sdkwork-router-admin-types';

import { formatApiKeyReferenceLabel } from '../access/shared';
import {
  formatCurrency,
  formatDateTime,
  formatNumber,
  formatTimeRangeLabel,
  type TimeRangePreset,
} from './shared';

type GatewayUsageDetailPanelProps = {
  selectedKeyRecord: GatewayApiKeyRecord | null;
  selectedRecord: UsageRecord;
  timeRange: TimeRangePreset;
};

export function GatewayUsageDetailPanel({
  selectedKeyRecord,
  selectedRecord,
  timeRange,
}: GatewayUsageDetailPanelProps) {
  const { t } = useAdminI18n();

  return (
    <div className="space-y-4">
      <DescriptionList columns={2}>
        <DescriptionItem>
          <DescriptionTerm>{t('Project')}</DescriptionTerm>
          <DescriptionDetails>{selectedRecord.project_id}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Provider')}</DescriptionTerm>
          <DescriptionDetails>{selectedRecord.provider}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Model')}</DescriptionTerm>
          <DescriptionDetails>{selectedRecord.model}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Created')}</DescriptionTerm>
          <DescriptionDetails>
            {formatDateTime(selectedRecord.created_at_ms)}
          </DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Total tokens')}</DescriptionTerm>
          <DescriptionDetails>
            {formatNumber(selectedRecord.total_tokens)}
          </DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Amount')}</DescriptionTerm>
          <DescriptionDetails>
            {formatCurrency(selectedRecord.amount)}
          </DescriptionDetails>
        </DescriptionItem>
      </DescriptionList>

      <DescriptionList columns={1}>
        <DescriptionItem>
          <DescriptionTerm>{t('API key scope')}</DescriptionTerm>
          <DescriptionDetails>
            {selectedKeyRecord
              ? formatApiKeyReferenceLabel(selectedKeyRecord, t)
              : t('All API keys')}
          </DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Time range')}</DescriptionTerm>
          <DescriptionDetails>{formatTimeRangeLabel(timeRange)}</DescriptionDetails>
        </DescriptionItem>
      </DescriptionList>
    </div>
  );
}
