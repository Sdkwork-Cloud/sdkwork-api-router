import { useEffect, useState } from 'react';
import {
  Button,
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
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { MoreHorizontal, Trash2 } from 'lucide-react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  useAdminI18n,
} from 'sdkwork-router-admin-core';

import type { TenantDirectoryRow } from './shared';

type TenantsRegistrySectionProps = {
  activeApiKeyCount: number;
  columns: DataTableColumn<TenantDirectoryRow>[];
  filteredTenants: TenantDirectoryRow[];
  onOpenApiKeyDialog: (tenant?: TenantDirectoryRow) => void;
  onOpenProjectDialog: (tenant?: TenantDirectoryRow) => void;
  onOpenTenantDialog: (tenant?: TenantDirectoryRow) => void;
  onRequestDelete: (tenant: TenantDirectoryRow) => void;
  onSelectTenant: (tenant: TenantDirectoryRow) => void;
  selectedTenantId: string | null;
  totalPortalUsers: number;
  totalProjects: number;
};

export function TenantsRegistrySection({
  activeApiKeyCount,
  columns,
  filteredTenants,
  onOpenApiKeyDialog,
  onOpenProjectDialog,
  onOpenTenantDialog,
  onRequestDelete,
  onSelectTenant,
  selectedTenantId,
  totalPortalUsers,
  totalProjects,
}: TenantsRegistrySectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [page, setPage] = useState(1);
  const pageSize = 10;

  const total = filteredTenants.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const paginatedTenants = filteredTenants.slice(startIndex, endIndex);

  useEffect(() => {
    setPage(1);
  }, [filteredTenants]);

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
          'Create a tenant to start assigning projects and issuing gateway keys.',
        )}
        emptyTitle={t('No tenants available')}
        getRowId={(tenant: TenantDirectoryRow) => tenant.id}
        getRowProps={buildEmbeddedAdminSingleSelectRowProps(
          selectedTenantId,
          (tenant: TenantDirectoryRow) => tenant.id,
        )}
        onRowClick={onSelectTenant}
        slotProps={embeddedAdminDataTableSlotProps}
        rowActions={(tenant: TenantDirectoryRow) => (
          <div className="flex items-center justify-end gap-2">
            <Button
              onClick={(event) => {
                event.stopPropagation();
                onOpenTenantDialog(tenant);
              }}
              size="sm"
              type="button"
              variant="ghost"
            >
              {t('Edit')}
            </Button>
            <Button
              onClick={(event) => {
                event.stopPropagation();
                onOpenProjectDialog(tenant);
              }}
              size="sm"
              type="button"
              variant="outline"
            >
              {t('New project')}
            </Button>
            <Button
              disabled={!tenant.canIssueApiKey}
              onClick={(event) => {
                event.stopPropagation();
                onOpenApiKeyDialog(tenant);
              }}
              size="sm"
              type="button"
              variant="outline"
            >
              {t('Issue key')}
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button size="sm" type="button" variant="ghost">
                  <MoreHorizontal className="w-4 h-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem
                  className="text-[var(--sdk-color-state-danger)]"
                  disabled={tenant.projectCount > 0 || tenant.portalUserCount > 0}
                  onClick={() => onRequestDelete(tenant)}
                >
                  <Trash2 className="w-3.5 h-3.5 mr-2" />
                  {t('Delete')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}
        rows={paginatedTenants}
        stickyHeader
      />

      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} tenants', { count: formatNumber(total) })}</span>
            <span>{t('{count} projects', { count: formatNumber(totalProjects) })}</span>
            <span>{t('{count} portal users', { count: formatNumber(totalPortalUsers) })}</span>
            <span>{t('{count} active keys', { count: formatNumber(activeApiKeyCount) })}</span>
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
                    className={page <= 1 ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                    onClick={() => setPage((current) => Math.max(1, current - 1))}
                  />
                </PaginationItem>
                {Array.from({ length: Math.min(5, totalPages) }, (_, index) => {
                  let pageNumber: number;
                  if (totalPages <= 5) {
                    pageNumber = index + 1;
                  } else if (page <= 3) {
                    pageNumber = index + 1;
                  } else if (page >= totalPages - 2) {
                    pageNumber = totalPages - 4 + index;
                  } else {
                    pageNumber = page - 2 + index;
                  }

                  return (
                    <PaginationItem key={pageNumber}>
                      <PaginationLink
                        className="cursor-pointer"
                        isActive={page === pageNumber}
                        onClick={() => setPage(pageNumber)}
                      >
                        {pageNumber}
                      </PaginationLink>
                    </PaginationItem>
                  );
                })}
                <PaginationItem>
                  <PaginationNext
                    className={page >= totalPages ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                    onClick={() => setPage((current) => Math.min(totalPages, current + 1))}
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
