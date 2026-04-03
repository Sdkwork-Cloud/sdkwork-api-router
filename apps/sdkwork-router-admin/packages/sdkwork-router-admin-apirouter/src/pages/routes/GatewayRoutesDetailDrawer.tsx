import {
  Button,
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { Edit, Trash2 } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import type { GatewayRouteInventoryRow } from '../../services/gatewayViewService';
import type { ProviderRoutingImpact } from './routingSnapshotAnalytics';
import { GatewayRoutesDetailPanel } from './GatewayRoutesDetailPanel';
import { statusVariant } from './shared';

type GatewayRoutesDetailDrawerProps = {
  onDelete: () => void;
  onEdit: () => void;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  providerRoutingImpact: ProviderRoutingImpact | null;
  selectedRow: GatewayRouteInventoryRow | null;
};

export function GatewayRoutesDetailDrawer({
  onDelete,
  onEdit,
  onOpenChange,
  open,
  providerRoutingImpact,
  selectedRow,
}: GatewayRoutesDetailDrawerProps) {
  const { t } = useAdminI18n();

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {selectedRow ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{selectedRow.provider.display_name}</DrawerTitle>
                    <DrawerDescription>{selectedRow.provider.id}</DrawerDescription>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <StatusBadge
                      label={t(selectedRow.health_status)}
                      showIcon
                      status={selectedRow.healthy ? 'active' : 'failed'}
                      variant={statusVariant(selectedRow)}
                    />
                    <StatusBadge
                      showIcon
                      status={selectedRow.provider.adapter_kind}
                      variant="secondary"
                    />
                  </div>
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">
              <GatewayRoutesDetailPanel
                providerRoutingImpact={providerRoutingImpact}
                selectedRow={selectedRow}
              />
            </DrawerBody>

            <DrawerFooter className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                {t(
                  'Route providers keep channel bindings, credentials, and pricing posture attached to the selected registry row.',
                )}
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Button onClick={onEdit} size="sm" type="button" variant="outline">
                  <Edit className="h-4 w-4" />
                  {t('Edit')}
                </Button>
                <Button onClick={onDelete} size="sm" type="button" variant="danger">
                  <Trash2 className="h-4 w-4" />
                  {t('Delete')}
                </Button>
              </div>
            </DrawerFooter>
          </>
        ) : null}
      </DrawerContent>
    </Drawer>
  );
}
