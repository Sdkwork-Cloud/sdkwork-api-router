import { useEffect, useState } from 'react';
import {
  Button,
  type DataTableColumn,
  Card,
  DataTable,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from '@sdkwork/ui-pc-react';
import { MoreHorizontal, Trash2 } from 'lucide-react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { ManagedUser } from 'sdkwork-router-admin-types';

import { isProtectedUser } from './shared';

type UsersRegistrySectionProps = {
  activeCount: number;
  columns: DataTableColumn<ManagedUser>[];
  filteredUsers: ManagedUser[];
  onOpenOperatorDialog: (user?: ManagedUser) => void;
  onOpenPortalDialog: (user?: ManagedUser) => void;
  onRequestDelete: (user: ManagedUser) => void;
  onSelectUser: (user: ManagedUser) => void;
  onToggleOperatorUser: (userId: string, active: boolean) => Promise<void> | void;
  onTogglePortalUser: (userId: string, active: boolean) => Promise<void> | void;
  operatorCount: number;
  portalCount: number;
  selectedUserId: string | null;
  sessionUserId: string | null;
};

export function UsersRegistrySection({
  activeCount,
  columns,
  filteredUsers,
  onOpenOperatorDialog,
  onOpenPortalDialog,
  onRequestDelete,
  onSelectUser,
  onToggleOperatorUser,
  onTogglePortalUser,
  operatorCount,
  portalCount,
  selectedUserId,
  sessionUserId,
}: UsersRegistrySectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [page, setPage] = useState(1);
  const pageSize = 10;

  const total = filteredUsers.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const paginatedUsers = filteredUsers.slice(startIndex, endIndex);

  useEffect(() => {
    setPage(1);
  }, [filteredUsers]);

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  return (
    <Card className="h-full flex flex-col overflow-hidden p-0">
      <DataTable
        className={embeddedAdminDataTableClassName}
        columns={columns}
        emptyDescription={t(
          'Try a different keyword or broaden the user type and status filters.',
        )}
        emptyTitle={t('No users match the current filter')}
        getRowId={(user: ManagedUser) => user.id}
        getRowProps={buildEmbeddedAdminSingleSelectRowProps(
          selectedUserId,
          (user: ManagedUser) => user.id,
        )}
        onRowClick={onSelectUser}
        slotProps={embeddedAdminDataTableSlotProps}
        rowActions={(user: ManagedUser) => (
          <div className="flex items-center justify-end gap-2">
            <Button
              onClick={(e) => {
                e.stopPropagation();
                if (user.role === 'operator') {
                  onOpenOperatorDialog(user);
                  return;
                }
                onOpenPortalDialog(user);
              }}
              size="sm"
              type="button"
              variant="ghost"
            >
              {t('Edit')}
            </Button>
            <Button
              onClick={(e) => {
                e.stopPropagation();
                if (user.role === 'operator') {
                  void onToggleOperatorUser(user.id, !user.active);
                  return;
                }
                void onTogglePortalUser(user.id, !user.active);
              }}
              size="sm"
              type="button"
              variant="outline"
            >
              {user.active ? t('Disable') : t('Restore')}
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button size="sm" variant="ghost">
                  <MoreHorizontal className="w-4 h-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem
                  className="text-[var(--sdk-color-state-danger)]"
                  disabled={isProtectedUser(user, sessionUserId)}
                  onClick={() => onRequestDelete(user)}
                >
                  <Trash2 className="w-3.5 h-3.5 mr-2" />
                  {t('Delete')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}
        rows={paginatedUsers}
        stickyHeader
      />
      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} users', { count: formatNumber(total) })}</span>
            <span>{t('{count} active', { count: formatNumber(activeCount) })}</span>
            <span>{t('{count} operators', { count: formatNumber(operatorCount) })}</span>
            <span>{t('{count} portal users', { count: formatNumber(portalCount) })}</span>
          </div>
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Page {page} of {total}', {
              page: formatNumber(page),
              total: formatNumber(totalPages),
            })}
          </div>
        </div>
        {total > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t('Showing {start} - {end} of {total}', {
                end: formatNumber(Math.min(endIndex, total)),
                start: formatNumber(total === 0 ? 0 : startIndex + 1),
                total: formatNumber(total),
              })}
            </div>
            <Pagination>
              <PaginationContent>
                <PaginationItem>
                  <PaginationPrevious
                    onClick={() => setPage((p) => Math.max(1, p - 1))}
                    className={page <= 1 ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                  />
                </PaginationItem>
                {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                  let pageNum: number;
                  if (totalPages <= 5) {
                    pageNum = i + 1;
                  } else if (page <= 3) {
                    pageNum = i + 1;
                  } else if (page >= totalPages - 2) {
                    pageNum = totalPages - 4 + i;
                  } else {
                    pageNum = page - 2 + i;
                  }
                  return (
                    <PaginationItem key={pageNum}>
                      <PaginationLink
                        isActive={page === pageNum}
                        onClick={() => setPage(pageNum)}
                        className="cursor-pointer"
                      >
                        {pageNum}
                      </PaginationLink>
                    </PaginationItem>
                  );
                })}
                <PaginationItem>
                  <PaginationNext
                    onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                    className={page >= totalPages ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                  />
                </PaginationItem>
              </PaginationContent>
            </Pagination>
          </div>
        ) : null}
      </div>
    </Card>
  );
}
