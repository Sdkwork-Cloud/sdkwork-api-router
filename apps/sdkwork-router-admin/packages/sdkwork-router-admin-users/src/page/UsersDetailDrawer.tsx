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
import type { AdminPageProps, ManagedUser } from 'sdkwork-router-admin-types';

import { UsersDetailPanel } from './UsersDetailPanel';

type UsersDetailDrawerProps = {
  isProtected: boolean;
  onDelete: () => void;
  onEdit: () => void;
  onOpenChange: (open: boolean) => void;
  onToggleStatus: () => void;
  open: boolean;
  user: ManagedUser | null;
  userBilling:
    | AdminPageProps['snapshot']['billingSummary']['projects'][number]
    | null
    | undefined;
  userProject: AdminPageProps['snapshot']['projects'][number] | null | undefined;
  userTraffic:
    | AdminPageProps['snapshot']['usageSummary']['projects'][number]
    | null
    | undefined;
};

export function UsersDetailDrawer({
  isProtected,
  onDelete,
  onEdit,
  onOpenChange,
  onToggleStatus,
  open,
  user,
  userBilling,
  userProject,
  userTraffic,
}: UsersDetailDrawerProps) {
  const { t } = useAdminI18n();

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {user ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{user.display_name}</DrawerTitle>
                    <DrawerDescription>{user.email}</DrawerDescription>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <StatusBadge
                      showIcon
                      status={user.role}
                      variant={user.role === 'operator' ? 'success' : 'secondary'}
                    />
                    <StatusBadge
                      showIcon
                      status={user.active ? 'active' : 'disabled'}
                      variant={user.active ? 'success' : 'danger'}
                    />
                  </div>
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">
              <UsersDetailPanel
                user={user}
                userBilling={userBilling}
                userProject={userProject}
                userTraffic={userTraffic}
              />
            </DrawerBody>

            <DrawerFooter className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                {user.role === 'operator'
                  ? t('Operator identities manage the control plane directly.')
                  : t('Portal identities inherit their tenant and project scope.')}
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
                  variant={user.active ? 'outline' : 'primary'}
                >
                  <Power className="h-4 w-4" />
                  {user.active ? t('Disable') : t('Enable')}
                </Button>
                <Button
                  disabled={isProtected}
                  onClick={onDelete}
                  size="sm"
                  type="button"
                  variant="danger"
                >
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
