import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  Input,
  Label,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Plus, Search } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps } from 'sdkwork-router-admin-types';

import { SelectField } from './shared';
import { GatewayRateLimitPolicyDialog } from './rate-limits/GatewayRateLimitPolicyDialog';
import { GatewayRateLimitsDetailDrawer } from './rate-limits/GatewayRateLimitsDetailDrawer';
import { GatewayRateLimitsRegistrySection } from './rate-limits/GatewayRateLimitsRegistrySection';
import {
  buildRateLimitPolicyId,
  createEmptyRateLimitDraft,
  normalizeOptionalText,
  type GatewayRateLimitInventoryRow,
  type RateLimitStatusFilter,
} from './rate-limits/shared';

type GatewayRateLimitsPageProps = AdminPageProps & {
  onCreateRateLimitPolicy: (input: {
    policy_id: string;
    project_id: string;
    requests_per_window: number;
    window_seconds: number;
    burst_requests: number;
    enabled: boolean;
    route_key?: string | null;
    api_key_hash?: string | null;
    model_name?: string | null;
    notes?: string | null;
  }) => Promise<void>;
};

function parsePositiveInteger(value: string, fallback: number) {
  const parsedValue = Number.parseInt(value, 10);
  return Number.isFinite(parsedValue) && parsedValue > 0 ? parsedValue : fallback;
}

function buildScopeTitle(
  row: GatewayRateLimitInventoryRow,
  t: (text: string, values?: Record<string, string | number>) => string,
) {
  const parts: string[] = [];

  if (row.scope.apiKeyLabel) {
    parts.push(`${t('API key')}: ${row.scope.apiKeyLabel}`);
  }

  if (row.scope.routeKey) {
    parts.push(`${t('Route')}: ${row.scope.routeKey}`);
  }

  if (row.scope.modelName) {
    parts.push(`${t('Model')}: ${row.scope.modelName}`);
  }

  return parts.length ? parts.join(' | ') : t('Project-wide');
}

function buildScopeDetail(
  row: GatewayRateLimitInventoryRow,
  t: (text: string, values?: Record<string, string | number>) => string,
) {
  if (row.projectName) {
    return `${row.projectName} (${row.policy.project_id})`;
  }

  return row.policy.project_id || t('Project-wide');
}

export function GatewayRateLimitsPage({
  onCreateRateLimitPolicy,
  snapshot,
}: GatewayRateLimitsPageProps) {
  const { formatDateTime, formatNumber, t } = useAdminI18n();
  const [search, setSearch] = useState('');
  const [projectFilter, setProjectFilter] = useState('all');
  const [statusFilter, setStatusFilter] = useState<RateLimitStatusFilter>('all');
  const [selectedPolicyId, setSelectedPolicyId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [createDraft, setCreateDraft] = useState(() => createEmptyRateLimitDraft(snapshot));
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const projectById = useMemo(
    () => new Map(snapshot.projects.map((project) => [project.id, project])),
    [snapshot.projects],
  );
  const apiKeyByHash = useMemo(
    () => new Map(snapshot.apiKeys.map((apiKey) => [apiKey.hashed_key, apiKey])),
    [snapshot.apiKeys],
  );
  const latestWindowByPolicyId = useMemo(() => {
    const windowMap = new Map<string, AdminPageProps['snapshot']['rateLimitWindows'][number]>();

    for (const windowRecord of snapshot.rateLimitWindows) {
      const existingRecord = windowMap.get(windowRecord.policy_id);
      if (!existingRecord || existingRecord.updated_at_ms < windowRecord.updated_at_ms) {
        windowMap.set(windowRecord.policy_id, windowRecord);
      }
    }

    return windowMap;
  }, [snapshot.rateLimitWindows]);

  const rows = useMemo<GatewayRateLimitInventoryRow[]>(
    () =>
      snapshot.rateLimitPolicies
        .map((policy) => {
          const matchedApiKey =
            policy.api_key_hash ? apiKeyByHash.get(policy.api_key_hash) ?? null : null;
          const windowRecord = latestWindowByPolicyId.get(policy.policy_id) ?? null;
          const projectName = projectById.get(policy.project_id)?.name ?? null;
          const apiKeyLabel = matchedApiKey
            ? `${matchedApiKey.label || matchedApiKey.project_id} (${matchedApiKey.environment})`
            : policy.api_key_hash ?? null;

          return {
            policy,
            window: windowRecord,
            projectName,
            scope: {
              apiKeyLabel,
              routeKey: policy.route_key ?? null,
              modelName: policy.model_name ?? null,
              projectWide:
                !policy.api_key_hash && !policy.route_key && !policy.model_name,
            },
            searchText: [
              policy.policy_id,
              policy.project_id,
              projectName,
              policy.api_key_hash,
              apiKeyLabel,
              policy.route_key,
              policy.model_name,
              policy.notes,
            ]
              .filter(Boolean)
              .join(' ')
              .toLowerCase(),
          };
        })
        .sort(
          (left, right) =>
            Number(Boolean(right.window?.exceeded)) - Number(Boolean(left.window?.exceeded))
            || Number(right.policy.enabled) - Number(left.policy.enabled)
            || right.policy.updated_at_ms - left.policy.updated_at_ms,
        ),
    [apiKeyByHash, latestWindowByPolicyId, projectById, snapshot.rateLimitPolicies],
  );

  const projectOptions = useMemo(() => {
    const projectIds = new Set([
      ...snapshot.projects.map((project) => project.id),
      ...rows.map((row) => row.policy.project_id),
    ]);

    return Array.from(projectIds).map((projectId) => {
      const project = projectById.get(projectId);
      return {
        label: project ? `${project.name} (${projectId})` : projectId,
        value: projectId,
      };
    });
  }, [projectById, rows, snapshot.projects]);

  const filteredRows = useMemo(
    () =>
      rows.filter((row) => {
        if (projectFilter !== 'all' && row.policy.project_id !== projectFilter) {
          return false;
        }

        if (statusFilter === 'enabled' && !row.policy.enabled) {
          return false;
        }

        if (statusFilter === 'disabled' && row.policy.enabled) {
          return false;
        }

        if (statusFilter === 'exceeded' && !row.window?.exceeded) {
          return false;
        }

        if (!deferredSearch) {
          return true;
        }

        return row.searchText.includes(deferredSearch);
      }),
    [deferredSearch, projectFilter, rows, statusFilter],
  );

  useEffect(() => {
    if (projectFilter === 'all') {
      return;
    }

    if (projectOptions.some((option) => option.value === projectFilter)) {
      return;
    }

    setProjectFilter('all');
  }, [projectFilter, projectOptions]);

  useEffect(() => {
    setCreateDraft((current) => {
      if (current.project_id) {
        return current;
      }

      const nextProjectId = snapshot.projects[0]?.id ?? current.project_id;

      if (nextProjectId === current.project_id) {
        return current;
      }

      return {
        ...current,
        project_id: nextProjectId,
      };
    });
  }, [snapshot.projects]);

  useEffect(() => {
    setCreateDraft((current) => {
      if (!current.api_key_hash) {
        return current;
      }

      const hasMatchingApiKey = snapshot.apiKeys.some(
        (apiKey) =>
          apiKey.project_id === current.project_id
          && apiKey.hashed_key === current.api_key_hash,
      );

      if (hasMatchingApiKey) {
        return current;
      }

      return {
        ...current,
        api_key_hash: '',
      };
    });
  }, [snapshot.apiKeys]);

  useEffect(() => {
    if (selectedPolicyId && !filteredRows.some((row) => row.policy.policy_id === selectedPolicyId)) {
      setSelectedPolicyId(null);
      setIsDetailDrawerOpen(false);
    }
  }, [filteredRows, selectedPolicyId]);

  const selectedRow =
    filteredRows.find((row) => row.policy.policy_id === selectedPolicyId) ?? null;
  const totalPolicies = rows.length;
  const enabledPolicies = rows.filter((row) => row.policy.enabled).length;
  const liveWindows = rows.filter((row) => row.window).length;
  const exceededPolicies = rows.filter((row) => row.window?.exceeded).length;

  const columns = useMemo<DataTableColumn<GatewayRateLimitInventoryRow>[]>(
    () => [
      {
        id: 'policy',
        header: t('Policy'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {row.projectName ?? row.policy.project_id}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {row.policy.policy_id}
            </div>
          </div>
        ),
      },
      {
        id: 'scope',
        header: t('Scope'),
        cell: (row) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {buildScopeTitle(row, t)}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {buildScopeDetail(row, t)}
            </div>
          </div>
        ),
      },
      {
        id: 'window',
        header: t('Window'),
        cell: (row) => (
          <div className="space-y-1 text-sm">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {t('{count} req / {seconds}s', {
                count: formatNumber(row.window?.limit_requests ?? row.policy.limit_requests),
                seconds: formatNumber(row.window?.window_seconds ?? row.policy.window_seconds),
              })}
            </div>
            <div className="text-[var(--sdk-color-text-secondary)]">
              {t('Burst {count}', {
                count: formatNumber(row.window?.burst_requests ?? row.policy.burst_requests),
              })}
            </div>
          </div>
        ),
        width: 180,
      },
      {
        id: 'live-window',
        header: t('Live window'),
        cell: (row) =>
          row.window ? (
            <div className="space-y-1 text-sm">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">
                {t('{used} used | {remaining} left', {
                  remaining: formatNumber(row.window.remaining_requests),
                  used: formatNumber(row.window.request_count),
                })}
              </div>
              <div className="text-[var(--sdk-color-text-secondary)]">
                {formatDateTime(row.window.window_end_ms)}
              </div>
            </div>
          ) : (
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t('Waiting for traffic')}
            </div>
          ),
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (row) => (
          <div className="flex flex-wrap gap-2">
            <StatusBadge
              label={row.policy.enabled ? t('Enabled') : t('Disabled')}
              showIcon
              status={row.policy.enabled ? 'enabled' : 'disabled'}
              variant={row.policy.enabled ? 'success' : 'secondary'}
            />
            <StatusBadge
              label={
                row.window
                  ? row.window.exceeded
                    ? t('Exceeded')
                    : t('Observed')
                  : t('Idle')
              }
              showIcon
              status={
                row.window
                  ? row.window.exceeded
                    ? 'exceeded'
                    : 'observed'
                  : 'idle'
              }
              variant={row.window?.exceeded ? 'danger' : 'secondary'}
            />
          </div>
        ),
        width: 180,
      },
    ],
    [formatDateTime, formatNumber, t],
  );

  function openDetailDrawer(row: GatewayRateLimitInventoryRow) {
    setSelectedPolicyId(row.policy.policy_id);
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedPolicyId(null);
    }
  }

  function resetCreateDialog() {
    setIsCreateDialogOpen(false);
    setCreateDraft(createEmptyRateLimitDraft(snapshot));
  }

  function openCreateDialog() {
    setCreateDraft(createEmptyRateLimitDraft(snapshot));
    setIsCreateDialogOpen(true);
  }

  async function handleCreatePolicySubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const projectId = createDraft.project_id.trim();

    await onCreateRateLimitPolicy({
      policy_id: createDraft.policy_id.trim() || buildRateLimitPolicyId(projectId),
      project_id: projectId,
      requests_per_window: parsePositiveInteger(createDraft.requests_per_window, 120),
      window_seconds: parsePositiveInteger(createDraft.window_seconds, 60),
      burst_requests: parsePositiveInteger(createDraft.burst_requests, 20),
      enabled: createDraft.enabled,
      route_key: normalizeOptionalText(createDraft.route_key),
      api_key_hash: normalizeOptionalText(createDraft.api_key_hash),
      model_name: normalizeOptionalText(createDraft.model_name),
      notes: normalizeOptionalText(createDraft.notes),
    });

    resetCreateDialog();
  }

  function clearFilters() {
    setSearch('');
    setProjectFilter('all');
    setStatusFilter('all');
  }

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          <Card>
            <CardContent className="p-4">
              <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                {t('Policies')}
              </div>
              <div className="mt-2 text-2xl font-semibold text-[var(--sdk-color-text-primary)]">
                {formatNumber(totalPolicies)}
              </div>
              <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
                {t('Configured rate envelopes across the gateway surface.')}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4">
              <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                {t('Enabled')}
              </div>
              <div className="mt-2 text-2xl font-semibold text-[var(--sdk-color-text-primary)]">
                {formatNumber(enabledPolicies)}
              </div>
              <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
                {t('Policies currently participating in live enforcement.')}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4">
              <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                {t('Live windows')}
              </div>
              <div className="mt-2 text-2xl font-semibold text-[var(--sdk-color-text-primary)]">
                {formatNumber(liveWindows)}
              </div>
              <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
                {t('Policies that already have observed traffic in the current snapshot.')}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4">
              <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                {t('Exceeded')}
              </div>
              <div className="mt-2 text-2xl font-semibold text-[var(--sdk-color-text-primary)]">
                {formatNumber(exceededPolicies)}
              </div>
              <div className="mt-1 text-sm text-[var(--sdk-color-text-secondary)]">
                {t('Live windows that have crossed the configured request ceiling.')}
              </div>
            </CardContent>
          </Card>
        </div>

        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="gateway-rate-limit-search">
                  {t('Search rate limits')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="gateway-rate-limit-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setSearch(event.target.value)
                    }
                    placeholder={t('policy id, project, route, model, key')}
                    value={search}
                  />
                </div>
              </div>

              <div className="min-w-[14rem]">
                <SelectField
                  label={t('Project')}
                  labelVisibility="sr-only"
                  onValueChange={setProjectFilter}
                  options={[
                    { label: t('All projects'), value: 'all' },
                    ...projectOptions,
                  ]}
                  value={projectFilter}
                />
              </div>

              <div className="min-w-[12rem]">
                <SelectField<RateLimitStatusFilter>
                  label={t('Status')}
                  labelVisibility="sr-only"
                  onValueChange={setStatusFilter}
                  options={[
                    { label: t('All states'), value: 'all' },
                    { label: t('Enabled'), value: 'enabled' },
                    { label: t('Disabled'), value: 'disabled' },
                    { label: t('Exceeded'), value: 'exceeded' },
                  ]}
                  value={statusFilter}
                />
              </div>

              <div className="ml-auto flex flex-wrap items-center gap-2">
                <Button onClick={openCreateDialog} type="button" variant="primary">
                  <Plus className="h-4 w-4" />
                  {t('Create policy')}
                </Button>
                <Button onClick={clearFilters} type="button" variant="ghost">
                  {t('Clear filters')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <GatewayRateLimitsRegistrySection
            columns={columns}
            enabledCount={filteredRows.filter((row) => row.policy.enabled).length}
            exceededCount={filteredRows.filter((row) => row.window?.exceeded).length}
            liveWindowCount={filteredRows.filter((row) => row.window).length}
            onSelectPolicy={openDetailDrawer}
            rows={filteredRows}
            selectedPolicyId={selectedPolicyId}
          />
        </div>
      </div>

      <GatewayRateLimitsDetailDrawer
        onOpenChange={handleDetailDrawerOpenChange}
        open={isDetailDrawerOpen}
        row={selectedRow}
      />

      <GatewayRateLimitPolicyDialog
        draft={createDraft}
        onOpenChange={(open) => {
          if (!open) {
            resetCreateDialog();
            return;
          }

          setIsCreateDialogOpen(true);
        }}
        onSubmit={(event) => void handleCreatePolicySubmit(event)}
        open={isCreateDialogOpen}
        setDraft={setCreateDraft}
        snapshot={snapshot}
      />
    </>
  );
}
