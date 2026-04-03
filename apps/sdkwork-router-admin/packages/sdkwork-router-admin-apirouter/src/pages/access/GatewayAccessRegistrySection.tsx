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
  StatusBadge,
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
import type {
  ApiKeyGroupRecord,
  GatewayApiKeyRecord,
  ProxyProviderRecord,
} from 'sdkwork-router-admin-types';

import type { GatewayModelMappingRecord } from '../../services/gatewayOverlayStore';
import { readGatewayApiKeyOverlay } from '../../services/gatewayOverlayStore';
import { maskKey, resolvePlaintextForKey } from './shared';

type GatewayAccessRegistrySectionProps = {
  activeKeys: number;
  customRouteCount: number;
  expiringSoonCount: number;
  filteredKeys: GatewayApiKeyRecord[];
  groupById: Map<string, ApiKeyGroupRecord>;
  mappingById: Map<string, GatewayModelMappingRecord>;
  onDeleteKey: (key: GatewayApiKeyRecord) => void;
  onOpenEditDialog: (key: GatewayApiKeyRecord) => void;
  onOpenRouteDialog: (key: GatewayApiKeyRecord) => void;
  onOpenUsageDialog: (key: GatewayApiKeyRecord) => void;
  onSelectKey: (key: GatewayApiKeyRecord) => void;
  providerById: Map<string, ProxyProviderRecord>;
  selectedKey: GatewayApiKeyRecord | null;
  totalKeys: number;
};

export function GatewayAccessRegistrySection({
  activeKeys,
  customRouteCount,
  expiringSoonCount,
  filteredKeys,
  groupById,
  mappingById,
  onDeleteKey,
  onOpenEditDialog,
  onOpenRouteDialog,
  onOpenUsageDialog,
  onSelectKey,
  providerById,
  selectedKey,
  totalKeys,
}: GatewayAccessRegistrySectionProps) {
  const { formatNumber, t } = useAdminI18n();
  const [page, setPage] = useState(1);
  const pageSize = 10;
  const total = filteredKeys.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const paginatedKeys = filteredKeys.slice(startIndex, endIndex);

  useEffect(() => {
    setPage(1);
  }, [filteredKeys]);

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  const columns: DataTableColumn<GatewayApiKeyRecord>[] = [
    {
      id: 'identity',
      header: t('API key'),
      cell: (key) => {
        const plaintext = resolvePlaintextForKey(key);

        return (
          <div className="space-y-1">
            <div className="font-semibold text-[var(--sdk-color-text-primary)]">
              {key.label || key.project_id}
            </div>
            <div className="font-mono text-xs text-[var(--sdk-color-text-secondary)]">
              {plaintext ? maskKey(plaintext) : key.hashed_key}
            </div>
          </div>
        );
      },
    },
    {
      id: 'workspace',
      header: t('Workspace'),
      cell: (key) => (
        <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
          <div>
            {key.tenant_id} / {key.project_id}
          </div>
          <div>{key.environment}</div>
        </div>
      ),
    },
    {
      id: 'group',
      header: t('API key group'),
      cell: (key) => {
        const group = key.api_key_group_id ? groupById.get(key.api_key_group_id) : null;

        return (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {group?.name ?? t('No group')}
            </div>
            <div>{group?.slug ?? key.api_key_group_id ?? t('Direct key policy')}</div>
          </div>
        );
      },
    },
    {
      id: 'route',
      header: t('Route config'),
      cell: (key) => {
        const overlay = readGatewayApiKeyOverlay(key.hashed_key);
        const provider = overlay.route_provider_id
          ? providerById.get(overlay.route_provider_id)
          : null;
        const mapping = overlay.model_mapping_id
          ? mappingById.get(overlay.model_mapping_id)
          : null;

        return (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {overlay.route_mode === 'custom'
                ? provider?.display_name ?? provider?.id ?? t('Custom provider')
                : t('SDKWork gateway default')}
            </div>
            <div>{mapping?.name ?? t('No model mapping')}</div>
          </div>
        );
      },
    },
    {
      id: 'status',
      header: t('Status'),
      cell: (key) => (
        <StatusBadge
          label={key.active ? t('Active') : t('Revoked')}
          showIcon
          status={key.active ? 'active' : 'failed'}
          variant={key.active ? 'success' : 'danger'}
        />
      ),
      width: 128,
    },
  ];

  return (
    <Card className="h-full flex flex-col overflow-hidden p-0">
      <DataTable
        className={embeddedAdminDataTableClassName}
        columns={columns}
        emptyDescription={t('Try a broader search query or create a new key.')}
        emptyTitle={t('No API keys match the current filter')}
        getRowId={(key: GatewayApiKeyRecord) => key.hashed_key}
        getRowProps={buildEmbeddedAdminSingleSelectRowProps(
          selectedKey?.hashed_key ?? null,
          (key: GatewayApiKeyRecord) => key.hashed_key,
        )}
        onRowClick={onSelectKey}
        slotProps={embeddedAdminDataTableSlotProps}
        rowActions={(key: GatewayApiKeyRecord) => (
          <div className="flex items-center justify-end gap-2">
            <Button
              onClick={(event) => {
                event.stopPropagation();
                onOpenUsageDialog(key);
              }}
              size="sm"
              type="button"
              variant="outline"
            >
              {t('Usage')}
            </Button>
            <Button
              onClick={(event) => {
                event.stopPropagation();
                onOpenEditDialog(key);
              }}
              size="sm"
              type="button"
              variant="ghost"
            >
              {t('Edit')}
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button size="sm" type="button" variant="ghost">
                  <MoreHorizontal className="w-4 h-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => onOpenRouteDialog(key)}>
                  {t('Route config')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="text-[var(--sdk-color-state-danger)]"
                  onClick={() => onDeleteKey(key)}
                >
                  <Trash2 className="w-3.5 h-3.5 mr-2" />
                  {t('Delete')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}
        rows={paginatedKeys}
        stickyHeader
      />
      <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <span>{t('{count} API keys', { count: formatNumber(totalKeys) })}</span>
            <span>{t('{count} active', { count: formatNumber(activeKeys) })}</span>
            <span>{t('{count} custom routes', { count: formatNumber(customRouteCount) })}</span>
            <span>{t('{count} expiring soon', { count: formatNumber(expiringSoonCount) })}</span>
          </div>
          <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
            {t('Page {page} of {totalPages}', {
              page: formatNumber(page),
              totalPages: formatNumber(totalPages),
            })}
          </div>
        </div>
        {total > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t('Showing {start} - {end} of {total}', {
                start: total === 0 ? 0 : formatNumber(startIndex + 1),
                end: formatNumber(Math.min(endIndex, total)),
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
