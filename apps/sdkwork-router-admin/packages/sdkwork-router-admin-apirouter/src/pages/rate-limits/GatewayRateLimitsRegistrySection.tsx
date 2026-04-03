import { useEffect, useState } from 'react';
import {
  Button,
  Card,
  DataTable,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  useAdminI18n,
} from 'sdkwork-router-admin-core';

import type { GatewayRateLimitInventoryRow } from './shared';

type GatewayRateLimitsRegistrySectionProps = {
  columns: DataTableColumn<GatewayRateLimitInventoryRow>[];
  enabledCount: number;
  exceededCount: number;
  liveWindowCount: number;
  onSelectPolicy: (row: GatewayRateLimitInventoryRow) => void;
  rows: GatewayRateLimitInventoryRow[];
  selectedPolicyId: string | null;
};

export function GatewayRateLimitsRegistrySection({
  columns,
  enabledCount,
  exceededCount,
  liveWindowCount,
  onSelectPolicy,
  rows,
  selectedPolicyId,
}: GatewayRateLimitsRegistrySectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [page, setPage] = useState(1);
  const pageSize = 10;

  const total = rows.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const pagedRows = rows.slice(startIndex, endIndex);

  useEffect(() => {
    setPage(1);
  }, [rows]);

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  return (
    <Card className="flex h-full flex-col overflow-hidden p-0">
      <DataTable
        className={embeddedAdminDataTableClassName}
        columns={columns}
        emptyDescription={t(
          'Try a different keyword or broaden the project and state filters.',
        )}
        emptyTitle={t('No rate limit policies match the current filter')}
        getRowId={(row: GatewayRateLimitInventoryRow) => row.policy.policy_id}
        getRowProps={buildEmbeddedAdminSingleSelectRowProps(
          selectedPolicyId,
          (row: GatewayRateLimitInventoryRow) => row.policy.policy_id,
        )}
        onRowClick={onSelectPolicy}
        rowActions={(row: GatewayRateLimitInventoryRow) => (
          <div className="flex items-center justify-end gap-2">
            <Button
              onClick={(event) => {
                event.stopPropagation();
                onSelectPolicy(row);
              }}
              size="sm"
              type="button"
              variant="ghost"
            >
              {t('Inspect')}
            </Button>
          </div>
        )}
        rows={pagedRows}
        slotProps={embeddedAdminDataTableSlotProps}
        stickyHeader
      />
      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} policies', { count: formatNumber(total) })}</span>
            <span>{t('{count} enabled', { count: formatNumber(enabledCount) })}</span>
            <span>{t('{count} live windows', { count: formatNumber(liveWindowCount) })}</span>
            <span>{t('{count} exceeded', { count: formatNumber(exceededCount) })}</span>
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
                    onClick={() => setPage((currentPage) => Math.max(1, currentPage - 1))}
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
                    onClick={() => setPage((currentPage) => Math.min(totalPages, currentPage + 1))}
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
