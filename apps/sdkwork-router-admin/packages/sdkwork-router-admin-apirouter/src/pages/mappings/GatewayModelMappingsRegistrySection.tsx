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
import { useEffect, useState } from 'react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  useAdminI18n,
} from 'sdkwork-router-admin-core';

import {
  type GatewayModelMappingRecord,
  type GatewayModelMappingStatus,
} from '../../services/gatewayOverlayStore';

type GatewayModelMappingsRegistrySectionProps = {
  activeCount: number;
  columns: DataTableColumn<GatewayModelMappingRecord>[];
  filteredMappings: GatewayModelMappingRecord[];
  mappings: GatewayModelMappingRecord[];
  onDeleteMapping: (mapping: GatewayModelMappingRecord) => void;
  onEditMapping: (mapping: GatewayModelMappingRecord) => void;
  onSelectMapping: (mapping: GatewayModelMappingRecord) => void;
  onToggleStatus: (
    mappingId: string,
    nextStatus: GatewayModelMappingStatus,
  ) => void;
  selectedMapping: GatewayModelMappingRecord | null;
  totalRuleCount: number;
};

export function GatewayModelMappingsRegistrySection({
  activeCount,
  columns,
  filteredMappings,
  mappings,
  onDeleteMapping,
  onEditMapping,
  onSelectMapping,
  onToggleStatus,
  selectedMapping,
  totalRuleCount,
}: GatewayModelMappingsRegistrySectionProps) {
  const { t } = useAdminI18n();
  const [page, setPage] = useState(1);
  const pageSize = 10;
  const total = filteredMappings.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const paginatedMappings = filteredMappings.slice(startIndex, endIndex);

  useEffect(() => {
    setPage(1);
  }, [filteredMappings]);

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
          'Try a different search query or broaden the mapping status filter.',
        )}
        emptyTitle={t('No model mappings match the current filter')}
        getRowId={(mapping: GatewayModelMappingRecord) => mapping.id}
        getRowProps={buildEmbeddedAdminSingleSelectRowProps(
          selectedMapping?.id ?? null,
          (mapping: GatewayModelMappingRecord) => mapping.id,
        )}
        onRowClick={onSelectMapping}
        slotProps={embeddedAdminDataTableSlotProps}
        rowActions={(mapping: GatewayModelMappingRecord) => (
          <div className="flex items-center justify-end gap-2">
            <Button
              onClick={(event) => {
                event.stopPropagation();
                onEditMapping(mapping);
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
                onToggleStatus(
                  mapping.id,
                  mapping.status === 'active' ? 'disabled' : 'active',
                );
              }}
              size="sm"
              type="button"
              variant="outline"
            >
              {mapping.status === 'active' ? t('Disable') : t('Enable')}
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
                  onClick={() => onDeleteMapping(mapping)}
                >
                  <Trash2 className="w-3.5 h-3.5 mr-2" />
                  {t('Delete')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}
        rows={paginatedMappings}
        stickyHeader
      />
      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} mappings', { count: mappings.length.toLocaleString() })}</span>
            <span>{t('{count} active', { count: activeCount.toLocaleString() })}</span>
            <span>
              {t('{count} disabled', {
                count: (mappings.length - activeCount).toLocaleString(),
              })}
            </span>
            <span>{t('{count} rules', { count: totalRuleCount.toLocaleString() })}</span>
          </div>
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Page {page} of {totalPages}', { page, totalPages })}
          </div>
        </div>
        {total > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t('Showing {start} - {end} of {total}', {
                start: total === 0 ? 0 : startIndex + 1,
                end: Math.min(endIndex, total),
                total: total.toLocaleString(),
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
