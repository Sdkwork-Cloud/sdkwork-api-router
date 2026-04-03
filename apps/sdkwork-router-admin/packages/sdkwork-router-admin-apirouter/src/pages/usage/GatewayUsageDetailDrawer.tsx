import {
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  GatewayApiKeyRecord,
  UsageRecord,
} from 'sdkwork-router-admin-types';

import { GatewayUsageDetailPanel } from './GatewayUsageDetailPanel';
import { type TimeRangePreset } from './shared';

type GatewayUsageDetailDrawerProps = {
  onOpenChange: (open: boolean) => void;
  open: boolean;
  selectedKeyRecord: GatewayApiKeyRecord | null;
  selectedRecord: UsageRecord | null;
  timeRange: TimeRangePreset;
};

export function GatewayUsageDetailDrawer({
  onOpenChange,
  open,
  selectedKeyRecord,
  selectedRecord,
  timeRange,
}: GatewayUsageDetailDrawerProps) {
  const { t } = useAdminI18n();

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {selectedRecord ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{selectedRecord.project_id}</DrawerTitle>
                    <DrawerDescription>{selectedRecord.model}</DrawerDescription>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <StatusBadge showIcon status={selectedRecord.provider} variant="secondary" />
                    <StatusBadge
                      label={selectedKeyRecord ? t('Key scoped') : t('All keys')}
                      showIcon
                      status={selectedKeyRecord ? 'key-scoped' : 'all-keys'}
                      variant={selectedKeyRecord ? 'success' : 'secondary'}
                    />
                  </div>
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">
              <GatewayUsageDetailPanel
                selectedKeyRecord={selectedKeyRecord}
                selectedRecord={selectedRecord}
                timeRange={timeRange}
              />
            </DrawerBody>

            <DrawerFooter className="text-xs text-[var(--sdk-color-text-secondary)]">
              {t(
                'Usage inspection keeps the active key filter and time window attached to the currently selected ledger row.',
              )}
            </DrawerFooter>
          </>
        ) : null}
      </DrawerContent>
    </Drawer>
  );
}
