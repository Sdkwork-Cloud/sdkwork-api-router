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
import { Edit, Power, Trash2 } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import type { GatewayModelMappingRecord } from '../../services/gatewayOverlayStore';
import { GatewayModelMappingsDetailPanel } from './GatewayModelMappingsDetailPanel';

type GatewayModelMappingsDetailDrawerProps = {
  onDelete: () => void;
  onEdit: () => void;
  onOpenChange: (open: boolean) => void;
  onToggleStatus: () => void;
  open: boolean;
  selectedMapping: GatewayModelMappingRecord | null;
};

export function GatewayModelMappingsDetailDrawer({
  onDelete,
  onEdit,
  onOpenChange,
  onToggleStatus,
  open,
  selectedMapping,
}: GatewayModelMappingsDetailDrawerProps) {
  const { t } = useAdminI18n();

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {selectedMapping ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{selectedMapping.name}</DrawerTitle>
                    <DrawerDescription>
                      {selectedMapping.description || t('Gateway overlay mapping')}
                    </DrawerDescription>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <StatusBadge
                      label={
                        selectedMapping.status === 'active'
                          ? t('Active')
                          : t('Disabled')
                      }
                      showIcon
                      status={selectedMapping.status}
                      variant={selectedMapping.status === 'active' ? 'success' : 'secondary'}
                    />
                    <StatusBadge
                      label={t('{count} rules', { count: selectedMapping.rules.length })}
                      showIcon
                      status={`${selectedMapping.rules.length}-rules`}
                      variant="secondary"
                    />
                  </div>
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">
              <GatewayModelMappingsDetailPanel selectedMapping={selectedMapping} />
            </DrawerBody>

            <DrawerFooter className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                {t(
                  'Mapping policy and rule inventory stay linked to the selected overlay so operators can edit or disable it without leaving the registry.',
                )}
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Button onClick={onEdit} size="sm" type="button" variant="outline">
                  <Edit className="h-4 w-4" />
                  {t('Edit')}
                </Button>
                <Button
                  onClick={onToggleStatus}
                  size="sm"
                  type="button"
                  variant={selectedMapping.status === 'active' ? 'outline' : 'primary'}
                >
                  <Power className="h-4 w-4" />
                  {selectedMapping.status === 'active' ? t('Disable') : t('Enable')}
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
