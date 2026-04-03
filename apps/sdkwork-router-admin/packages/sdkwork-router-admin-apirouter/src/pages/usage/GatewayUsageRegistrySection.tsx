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
import type { UsageRecord } from 'sdkwork-router-admin-types';

import {
  buildUsageRecordKey,
  formatCurrency,
  formatNumber,
} from './shared';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  useAdminI18n,
} from 'sdkwork-router-admin-core';

type GatewayUsageRegistrySectionProps = {
  columns: DataTableColumn<UsageRecord>[];
  onNextPage: () => void;
  onPreviousPage: () => void;
  onSelectRecord: (record: UsageRecord, index: number) => void;
  page: number;
  pagedRecords: UsageRecord[];
  rowSelectionId: string | null;
  totalAmount: number;
  totalPages: number;
  totalTokens: number;
  totalUnits: number;
  totalVisibleRecords: number;
  uniqueProjects: number;
};

export function GatewayUsageRegistrySection({
  columns,
  onNextPage,
  onPreviousPage,
  onSelectRecord,
  page,
  pagedRecords,
  rowSelectionId,
  totalAmount,
  totalPages,
  totalTokens,
  totalUnits,
  totalVisibleRecords,
  uniqueProjects,
}: GatewayUsageRegistrySectionProps) {
  const { t } = useAdminI18n();

  return (
    <Card className="h-full flex flex-col overflow-hidden p-0">
      <DataTable
        className={embeddedAdminDataTableClassName}
        columns={columns}
        emptyDescription={t(
          'Broaden the selected API key or time range, or clear the text search.',
        )}
        emptyTitle={t('No usage records match the current filter')}
        getRowId={(record: UsageRecord, index: number) =>
          buildUsageRecordKey(record, index)
        }
        getRowProps={buildEmbeddedAdminSingleSelectRowProps(
          rowSelectionId,
          (record: UsageRecord, index: number) => buildUsageRecordKey(record, index),
        )}
        onRowClick={onSelectRecord}
        slotProps={embeddedAdminDataTableSlotProps}
        rowActions={(record: UsageRecord, index: number) => (
          <Button
            onClick={(event) => {
              event.stopPropagation();
              onSelectRecord(record, index);
            }}
            size="sm"
            type="button"
            variant="ghost"
          >
            {t('Inspect')}
          </Button>
        )}
        rows={pagedRecords}
        stickyHeader
      />
      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} records', { count: formatNumber(totalVisibleRecords) })}</span>
            <span>{t('{count} projects', { count: uniqueProjects.toLocaleString() })}</span>
            <span>{t('Tokens: {count}', { count: formatNumber(totalTokens) })}</span>
            <span>{t('Units: {count}', { count: formatNumber(totalUnits) })}</span>
            <span>{t('Amount: {amount}', { amount: formatCurrency(totalAmount) })}</span>
          </div>
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Page {page} of {totalPages}', { page, totalPages })}
          </div>
        </div>
        {totalVisibleRecords > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t('{count} visible', { count: formatNumber(totalVisibleRecords) })}
            </div>
            <Pagination>
              <PaginationContent>
                <PaginationItem>
                  <PaginationPrevious
                    className={page <= 1 ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                    onClick={onPreviousPage}
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
                        className="pointer-events-none"
                        isActive={page === pageNumber}
                      >
                        {pageNumber}
                      </PaginationLink>
                    </PaginationItem>
                  );
                })}
                <PaginationItem>
                  <PaginationNext
                    className={page >= totalPages ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                    onClick={onNextPage}
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
