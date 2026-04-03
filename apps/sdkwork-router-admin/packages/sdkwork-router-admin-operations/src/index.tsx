import { useDeferredValue, useEffect, useState } from 'react';
import type { ChangeEvent, FormEvent, ReactNode } from 'react';

import {
  Button,
  Card,
  CardContent,
  DataTable,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  InlineAlert,
  Input,
  Label,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  SegmentedControl,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Search } from 'lucide-react';
import {
  buildEmbeddedAdminSingleSelectRowProps,
  embeddedAdminDataTableClassName,
  embeddedAdminDataTableSlotProps,
  formatAdminDateTime,
  formatAdminNumber,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type { AdminPageProps, ProviderHealthSnapshot, RuntimeReloadReport, RuntimeStatusRecord } from 'sdkwork-router-admin-types';

type ViewMode = 'providers' | 'runtimes';
type DetailItem = { label: string; mono?: boolean; value: ReactNode };
type ProviderRow = ProviderHealthSnapshot & { kind: 'providers' };
type RuntimeRow = RuntimeStatusRecord & { kind: 'runtimes' };
type OperationsRow = ProviderRow | RuntimeRow;
type FeedbackState = null | { description: string; title: string; tone: 'danger' | 'success' };
type TranslateFn = (text: string, values?: Record<string, number | string>) => string;
const pageSize = 10;

const viewModeOptions: Array<{ label: string; value: ViewMode }> = [
  { label: 'Runtimes', value: 'runtimes' },
  { label: 'Providers', value: 'providers' },
];

function formatCount(value: number) {
  return formatAdminNumber(value);
}

function formatDateTime(value: number) {
  return formatAdminDateTime(value);
}

function rowId(row: OperationsRow) {
  return row.kind === 'providers' ? `${row.provider_id}:${row.observed_at_ms}` : `${row.runtime}:${row.extension_id}:${row.instance_id ?? 'global'}`;
}

function getErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function DetailGrid({ items }: { items: DetailItem[] }) {
  return (
    <DescriptionList columns={1}>
      {items.map((item) => (
        <DescriptionItem key={item.label}>
          <DescriptionTerm>{item.label}</DescriptionTerm>
          <DescriptionDetails mono={item.mono}>{item.value}</DescriptionDetails>
        </DescriptionItem>
      ))}
    </DescriptionList>
  );
}

function buildDetail(row: OperationsRow, t: TranslateFn) {
  if (row.kind === 'providers') {
    return {
      alert: [
        row.healthy ? t('Provider healthy') : t('Provider requires attention'),
        t('Provider health stays visible alongside status and operator notes.'),
        row.healthy ? 'success' : 'danger',
      ] as const,
      description: row.status,
      items: [
        { label: t('Provider'), mono: true, value: row.provider_id },
        { label: t('Status'), value: row.status },
        { label: t('Healthy'), value: row.healthy ? t('Yes') : t('No') },
        { label: t('Message'), value: row.message ?? t('No message recorded') },
        { label: t('Observed'), value: formatDateTime(row.observed_at_ms) },
      ],
      title: row.provider_id,
    };
  }

  return {
    alert: [
      row.healthy ? t('Runtime healthy') : t('Runtime requires attention'),
      t('Runtime state remains inspectable with family, extension, instance, and message context.'),
      row.healthy ? 'success' : 'danger',
    ] as const,
    description: row.display_name,
    items: [
      { label: t('Display name'), value: row.display_name },
      { label: t('Runtime family'), value: row.runtime },
      { label: t('Extension id'), mono: true, value: row.extension_id },
      { label: t('Instance id'), mono: true, value: row.instance_id ?? t('Global instance') },
      { label: t('Running'), value: row.running ? t('Yes') : t('No') },
      { label: t('Healthy'), value: row.healthy ? t('Yes') : t('No') },
      { label: t('Message'), value: row.message ?? t('No message recorded') },
    ],
    title: row.display_name,
  };
}

function feedbackFromReport(report: RuntimeReloadReport, t: TranslateFn) {
  const scope = report.scope === 'full'
    ? t('Full runtime reload completed.')
    : t('Targeted reload completed for {scope}.', { scope: report.scope });
  return {
    description: t(
      '{scope} {activeCount} active runtime(s) and {packageCount} loadable package(s) were discovered at {reloadedAt}.',
      {
        activeCount: formatCount(report.active_runtime_count),
        packageCount: formatCount(report.loadable_package_count),
        reloadedAt: formatDateTime(report.reloaded_at_ms),
        scope,
      },
    ),
    title: t('Runtime reload finished'),
    tone: 'success' as const,
  };
}

export function OperationsPage({
  snapshot,
  onReloadRuntimes,
}: AdminPageProps & {
  onReloadRuntimes: (input?: { extension_id?: string; instance_id?: string }) => Promise<RuntimeReloadReport>;
}) {
  const { t } = useAdminI18n();
  const localizedViewModeOptions = viewModeOptions.map((option) => ({
    ...option,
    label: t(option.label),
  }));
  const [search, setSearch] = useState('');
  const [viewMode, setViewMode] = useState<ViewMode>('runtimes');
  const [page, setPage] = useState(1);
  const [selectedRowId, setSelectedRowId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const [isReloadDialogOpen, setIsReloadDialogOpen] = useState(false);
  const [reloadDraft, setReloadDraft] = useState({ extension_id: '', instance_id: '' });
  const [feedback, setFeedback] = useState<FeedbackState>(null);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  const runtimeRows: RuntimeRow[] = snapshot.runtimeStatuses.filter((runtime) => !deferredQuery || [runtime.display_name, runtime.runtime, runtime.instance_id ?? '', runtime.extension_id, runtime.message ?? '', runtime.healthy ? 'healthy' : 'attention', runtime.running ? 'running' : 'stopped'].join(' ').toLowerCase().includes(deferredQuery)).map((runtime) => ({ ...runtime, kind: 'runtimes' as const }));
  const providerRows: ProviderRow[] = snapshot.providerHealth.filter((provider) => !deferredQuery || [provider.provider_id, provider.status, provider.message ?? '', provider.healthy ? 'healthy' : 'attention'].join(' ').toLowerCase().includes(deferredQuery)).map((provider) => ({ ...provider, kind: 'providers' as const }));

  let rows: OperationsRow[] = runtimeRows;
  let columns: Array<DataTableColumn<OperationsRow>> = [
    { id: 'runtime', header: t('Runtime'), cell: (row) => row.kind === 'runtimes' ? <div className="space-y-1"><div className="font-medium text-[var(--sdk-color-text-primary)]">{row.display_name}</div><div className="text-sm text-[var(--sdk-color-text-secondary)]">{row.runtime}</div></div> : null },
    { id: 'extension', header: t('Extension'), cell: (row) => row.kind === 'runtimes' ? <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]"><div>{row.extension_id}</div><div>{row.instance_id ?? t('Global instance')}</div></div> : null },
    { id: 'running', header: t('Running'), cell: (row) => row.kind === 'runtimes' ? <StatusBadge label={row.running ? t('Running') : t('Stopped')} showIcon status={row.running ? 'running' : 'stopped'} variant={row.running ? 'success' : 'secondary'} /> : null, width: 140 },
    { id: 'healthy', header: t('Health'), cell: (row) => row.kind === 'runtimes' ? <StatusBadge label={row.healthy ? t('Healthy') : t('Attention')} showIcon status={row.healthy ? 'healthy' : 'attention'} variant={row.healthy ? 'success' : 'danger'} /> : null, width: 140 },
    { id: 'message', header: t('Message'), cell: (row) => row.kind === 'runtimes' ? <div className="max-w-[18rem] text-sm text-[var(--sdk-color-text-secondary)]">{row.message ?? t('No message recorded')}</div> : null },
  ];
  let tableCopy = { description: t('Runtime family, extension, health, and current operator message in one dense table.'), emptyDescription: t('Try a broader query to inspect more runtime statuses.'), emptyTitle: t('No runtimes match the current filters'), title: t('Managed runtimes') };
  let metrics = [
    { description: t('Runtimes visible in the current slice.'), label: t('Visible runtimes'), value: formatCount(runtimeRows.length) },
    { description: t('Runtimes currently marked healthy.'), label: t('Healthy'), value: formatCount(runtimeRows.filter((row) => row.healthy).length) },
    { description: t('Runtimes currently running.'), label: t('Running'), value: formatCount(runtimeRows.filter((row) => row.running).length) },
    { description: t('Distinct extensions represented by the current slice.'), label: t('Extensions'), value: formatCount(new Set(runtimeRows.map((row) => row.extension_id)).size) },
  ];

  if (viewMode === 'providers') {
    rows = providerRows;
    columns = [
      { id: 'provider', header: t('Provider'), cell: (row) => row.kind === 'providers' ? <div className="space-y-1"><div className="font-medium text-[var(--sdk-color-text-primary)]">{row.provider_id}</div><div className="text-sm text-[var(--sdk-color-text-secondary)]">{formatDateTime(row.observed_at_ms)}</div></div> : null },
      { id: 'status', header: t('Status'), cell: (row) => row.kind === 'providers' ? row.status : null, width: 180 },
      { id: 'healthy', header: t('Health'), cell: (row) => row.kind === 'providers' ? <StatusBadge label={row.healthy ? t('Healthy') : t('Attention')} showIcon status={row.healthy ? 'healthy' : 'attention'} variant={row.healthy ? 'success' : 'danger'} /> : null, width: 140 },
      { id: 'message', header: t('Message'), cell: (row) => row.kind === 'providers' ? <div className="max-w-[18rem] text-sm text-[var(--sdk-color-text-secondary)]">{row.message ?? t('No message recorded')}</div> : null },
    ];
    tableCopy = { description: t('Provider health snapshots with operator-facing status and latest message context.'), emptyDescription: t('Try a broader query to inspect more provider health rows.'), emptyTitle: t('No provider health rows match the current filters'), title: t('Provider health') };
    metrics = [
      { description: t('Providers visible in the current slice.'), label: t('Visible providers'), value: formatCount(providerRows.length) },
      { description: t('Providers currently marked healthy.'), label: t('Healthy'), value: formatCount(providerRows.filter((row) => row.healthy).length) },
      { description: t('Providers requiring operator attention.'), label: t('Attention'), value: formatCount(providerRows.filter((row) => !row.healthy).length) },
      { description: t('Distinct status strings in the current slice.'), label: t('Status variants'), value: formatCount(new Set(providerRows.map((row) => row.status)).size) },
    ];
  }

  const totalPages = Math.max(1, Math.ceil(rows.length / pageSize));
  const safePage = Math.min(page, totalPages);
  const startIndex = (safePage - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const pagedRows = rows.slice(startIndex, endIndex);

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  useEffect(() => {
    if (!pagedRows.length) {
      if (selectedRowId !== null) {
        setSelectedRowId(null);
      }
      setIsDetailDrawerOpen(false);
      return;
    }

    if (selectedRowId && pagedRows.some((row) => rowId(row) === selectedRowId)) {
      return;
    }

    setSelectedRowId(rowId(pagedRows[0]));
    setIsDetailDrawerOpen(false);
  }, [pagedRows, selectedRowId]);

  const selectedRow = pagedRows.find((row) => rowId(row) === selectedRowId) ?? pagedRows[0] ?? null;
  const detail = selectedRow ? buildDetail(selectedRow, t) : null;
  const currentViewLabel =
    localizedViewModeOptions.find((option) => option.value === viewMode)?.label ?? t('Runtimes');

  async function runReload(input?: { extension_id?: string; instance_id?: string }) {
    try {
      const report = await onReloadRuntimes(input);
      setFeedback(feedbackFromReport(report, t));
      setIsReloadDialogOpen(false);
      setReloadDraft({ extension_id: '', instance_id: '' });
    } catch (error) {
      setFeedback({
        description: getErrorMessage(error),
        title: t('Runtime reload failed'),
        tone: 'danger',
      });
    }
  }

  async function handleTargetedReload(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await runReload({
      extension_id: reloadDraft.extension_id.trim() || undefined,
      instance_id: reloadDraft.instance_id.trim() || undefined,
    });
  }

  function openDetailDrawer(row: OperationsRow) {
    setSelectedRowId(rowId(row));
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
  }

  function clearSearch() {
    setSearch('');
    setPage(1);
  }

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="operations-search">
                  {t('Search operations')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="operations-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) => {
                      setSearch(event.target.value);
                      setPage(1);
                    }}
                    placeholder={t('provider, runtime, instance, message')}
                    value={search}
                  />
                </div>
              </div>

              <div className="min-w-[18rem] flex-[1.1]">
                <div className="space-y-0">
                  <Label className="sr-only">{t('Operational view')}</Label>
                  <SegmentedControl
                    onValueChange={(value: string) => {
                      setViewMode(value as ViewMode);
                      setPage(1);
                      setSelectedRowId(null);
                      setIsDetailDrawerOpen(false);
                    }}
                    options={localizedViewModeOptions}
                    size="sm"
                    value={viewMode}
                  />
                </div>
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatCount(rows.length) })}
                  {' | '}
                  {currentViewLabel}
                  {' | '}
                  {t('Operational posture')}
                </div>
                <Button onClick={clearSearch} type="button" variant="ghost">
                  {t('Reset search')}
                </Button>
                <Button onClick={() => setIsReloadDialogOpen(true)} type="button" variant="outline">
                  {t('Targeted reload')}
                </Button>
                <Button onClick={() => void runReload()} type="button" variant="primary">
                  {t('Reload all runtimes')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <Card className="min-h-0 flex-1 flex flex-col overflow-hidden p-0">
          <DataTable
            className={embeddedAdminDataTableClassName}
            columns={columns}
            emptyDescription={tableCopy.emptyDescription}
            emptyTitle={tableCopy.emptyTitle}
            getRowId={(row: OperationsRow) => rowId(row)}
            getRowProps={buildEmbeddedAdminSingleSelectRowProps(selectedRowId, rowId)}
            onRowClick={openDetailDrawer}
            slotProps={embeddedAdminDataTableSlotProps}
            rowActions={(row: OperationsRow) => (
              <Button
                onClick={(event) => {
                  event.stopPropagation();
                  openDetailDrawer(row);
                }}
                size="sm"
                type="button"
                variant="ghost"
              >
                {t('Inspect')}
              </Button>
            )}
            rows={pagedRows}
            stickyHeader
          />

          <div className="flex flex-col gap-3 border-t border-[var(--sdk-color-border-default)] p-4">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="min-w-0">
                <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                  {tableCopy.title}
                </div>
                <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
                  {tableCopy.description}
                </div>
              </div>
              <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                {t('Page {page} of {totalPages}', {
                  page: formatCount(safePage),
                  totalPages: formatCount(totalPages),
                })}
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
              {metrics.map((metric) => (
                <span key={metric.label}>
                  <span className="font-medium text-[var(--sdk-color-text-primary)]">
                    {metric.value}
                  </span>
                  {' '}
                  {metric.label}
                </span>
              ))}
            </div>
            {feedback ? (
              <InlineAlert
                description={feedback.description}
                title={feedback.title}
                tone={feedback.tone}
              />
            ) : null}
            {rows.length > 0 ? (
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                  {t('Showing {start} - {end} of {total}', {
                    start: formatCount(startIndex + 1),
                    end: formatCount(Math.min(endIndex, rows.length)),
                    total: formatCount(rows.length),
                  })}
                </div>
                <Pagination>
                  <PaginationContent>
                    <PaginationItem>
                      <PaginationPrevious
                        className={safePage <= 1 ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                        onClick={() => setPage((current) => Math.max(1, current - 1))}
                      />
                    </PaginationItem>
                    {Array.from({ length: Math.min(5, totalPages) }, (_, index) => {
                      let pageNumber: number;

                      if (totalPages <= 5) {
                        pageNumber = index + 1;
                      } else if (safePage <= 3) {
                        pageNumber = index + 1;
                      } else if (safePage >= totalPages - 2) {
                        pageNumber = totalPages - 4 + index;
                      } else {
                        pageNumber = safePage - 2 + index;
                      }

                      return (
                        <PaginationItem key={pageNumber}>
                          <PaginationLink
                            className="cursor-pointer"
                            isActive={safePage === pageNumber}
                            onClick={() => setPage(pageNumber)}
                          >
                            {pageNumber}
                          </PaginationLink>
                        </PaginationItem>
                      );
                    })}
                    <PaginationItem>
                      <PaginationNext
                        className={safePage >= totalPages ? 'pointer-events-none opacity-50' : 'cursor-pointer'}
                        onClick={() => setPage((current) => Math.min(totalPages, current + 1))}
                      />
                    </PaginationItem>
                  </PaginationContent>
                </Pagination>
              </div>
            ) : null}
          </div>
        </Card>
      </div>

      <Drawer open={isDetailDrawerOpen} onOpenChange={handleDetailDrawerOpenChange}>
        <DrawerContent side="right" size="lg">
          {detail ? (
            <>
              <DrawerHeader>
                <div className="space-y-3">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="space-y-1">
                      <DrawerTitle>{detail.title}</DrawerTitle>
                      <DrawerDescription>{detail.description}</DrawerDescription>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <StatusBadge
                        label={currentViewLabel}
                        showIcon
                        status={viewMode}
                        variant="secondary"
                      />
                    </div>
                  </div>
                </div>
              </DrawerHeader>

              <DrawerBody className="space-y-4">
                <InlineAlert
                  description={detail.alert[1]}
                  title={detail.alert[0]}
                  tone={detail.alert[2]}
                />
                <DetailGrid items={detail.items} />
              </DrawerBody>

              <DrawerFooter className="text-xs text-[var(--sdk-color-text-secondary)]">
                {t('Operational posture remains inspectable without leaving the active table context.')}
              </DrawerFooter>
            </>
          ) : null}
        </DrawerContent>
      </Drawer>

      <Dialog open={isReloadDialogOpen} onOpenChange={setIsReloadDialogOpen}>
          <DialogContent className="w-[min(92vw,32rem)]">
            <DialogHeader>
            <DialogTitle>{t('Targeted runtime reload')}</DialogTitle>
            <DialogDescription>{t('Scope the reload to a specific extension or runtime instance when the full control-plane refresh is unnecessary.')}</DialogDescription>
            </DialogHeader>
          <form className="space-y-4" onSubmit={(event) => void handleTargetedReload(event)}>
            <div className="space-y-2">
              <Label htmlFor="reload-extension-id">{t('Extension id')}</Label>
              <Input id="reload-extension-id" onChange={(event: ChangeEvent<HTMLInputElement>) => setReloadDraft((current) => ({ ...current, extension_id: event.target.value }))} placeholder={t('optional extension id')} value={reloadDraft.extension_id} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="reload-instance-id">{t('Instance id')}</Label>
              <Input id="reload-instance-id" onChange={(event: ChangeEvent<HTMLInputElement>) => setReloadDraft((current) => ({ ...current, instance_id: event.target.value }))} placeholder={t('optional instance id')} value={reloadDraft.instance_id} />
            </div>
            <DialogFooter>
              <Button onClick={() => setIsReloadDialogOpen(false)} type="button" variant="ghost">{t('Cancel')}</Button>
              <Button type="submit" variant="primary">{t('Run targeted reload')}</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  );
}
