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
import { Edit, KeyRound, Power, Trash2, Waypoints } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type {
  ApiKeyGroupRecord,
  GatewayApiKeyRecord,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

import type { GatewayModelMappingRecord } from '../../services/gatewayOverlayStore';
import { readGatewayApiKeyOverlay } from '../../services/gatewayOverlayStore';
import { GatewayAccessDetailPanel } from './GatewayAccessDetailPanel';

type GatewayAccessDetailDrawerProps = {
  groupById: Map<string, ApiKeyGroupRecord>;
  mappingById: Map<string, GatewayModelMappingRecord>;
  onDelete: () => void;
  onEdit: () => void;
  onOpenChange: (open: boolean) => void;
  onOpenRouteDialog: () => void;
  onOpenUsageDialog: () => void;
  onToggleStatus: () => void;
  open: boolean;
  providerById: Map<string, ProxyProviderRecord>;
  selectedKey: GatewayApiKeyRecord | null;
};

export function GatewayAccessDetailDrawer({
  groupById,
  mappingById,
  onDelete,
  onEdit,
  onOpenChange,
  onOpenRouteDialog,
  onOpenUsageDialog,
  onToggleStatus,
  open,
  providerById,
  selectedKey,
}: GatewayAccessDetailDrawerProps) {
  const { t } = useAdminI18n();
  const overlay = selectedKey ? readGatewayApiKeyOverlay(selectedKey.hashed_key) : null;
  const provider = overlay?.route_provider_id
    ? providerById.get(overlay.route_provider_id)
    : null;

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {selectedKey ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{selectedKey.label || selectedKey.project_id}</DrawerTitle>
                    <DrawerDescription>{selectedKey.hashed_key}</DrawerDescription>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <StatusBadge
                      label={selectedKey.active ? t('Active') : t('Revoked')}
                      showIcon
                      status={selectedKey.active ? 'active' : 'revoked'}
                      variant={selectedKey.active ? 'success' : 'danger'}
                    />
                    <StatusBadge
                      label={
                        overlay?.route_mode === 'custom'
                          ? provider?.display_name ?? provider?.id ?? t('Custom route')
                          : t('Gateway default')
                      }
                      showIcon
                      status={
                        overlay?.route_mode === 'custom'
                          ? provider?.display_name ?? provider?.id ?? 'custom-route'
                          : 'gateway-default'
                      }
                      variant={overlay?.route_mode === 'custom' ? 'warning' : 'secondary'}
                    />
                  </div>
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">
              <GatewayAccessDetailPanel
                groupById={groupById}
                groupPolicyTitle={t('Group policy')}
                mappingById={mappingById}
                providerById={providerById}
                selectedKey={selectedKey}
              />
            </DrawerBody>

            <DrawerFooter className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                {t(
                  'API key lifecycle, route posture, and bootstrap workflows stay attached to the selected registry row.',
                )}
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Button onClick={onEdit} size="sm" type="button" variant="outline">
                  <Edit className="h-4 w-4" />
                  {t('Edit')}
                </Button>
                <Button onClick={onOpenRouteDialog} size="sm" type="button" variant="outline">
                  <Waypoints className="h-4 w-4" />
                  {t('Route config')}
                </Button>
                <Button onClick={onOpenUsageDialog} size="sm" type="button" variant="outline">
                  <KeyRound className="h-4 w-4" />
                  {t('Usage method')}
                </Button>
                <Button
                  onClick={onToggleStatus}
                  size="sm"
                  type="button"
                  variant={selectedKey.active ? 'outline' : 'primary'}
                >
                  <Power className="h-4 w-4" />
                  {selectedKey.active ? t('Revoke') : t('Restore')}
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
