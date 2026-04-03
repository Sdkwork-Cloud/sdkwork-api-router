import { useEffect, useMemo, useState } from 'react';

import { formatDateTime } from 'sdkwork-router-portal-commons/format-core';
import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Badge } from 'sdkwork-router-portal-commons/framework/display';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import { SearchInput } from 'sdkwork-router-portal-commons/framework/form';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from 'sdkwork-router-portal-commons/framework/overlays';
import type { PortalCompiledRoutingSnapshotRecord } from 'sdkwork-router-portal-types';

import { buildRoutingStrategyLabel } from '../services';

type PortalRoutingSnapshotsDialogProps = {
  loadingSnapshots: boolean;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  snapshotStatus: string;
  snapshots: PortalCompiledRoutingSnapshotRecord[];
  suggestedSearchQuery?: string;
};

function sortSnapshots(
  snapshots: PortalCompiledRoutingSnapshotRecord[],
): PortalCompiledRoutingSnapshotRecord[] {
  return [...snapshots].sort((left, right) =>
    right.updated_at_ms - left.updated_at_ms
    || right.created_at_ms - left.created_at_ms
    || left.route_key.localeCompare(right.route_key)
    || left.snapshot_id.localeCompare(right.snapshot_id),
  );
}

function searchMatches(
  query: string,
  values: Array<string | null | undefined>,
): boolean {
  if (!query) {
    return true;
  }

  return values.filter(Boolean).join(' ').toLowerCase().includes(query);
}

export function PortalRoutingSnapshotsDialog({
  loadingSnapshots,
  onOpenChange,
  open,
  snapshotStatus,
  snapshots,
  suggestedSearchQuery = '',
}: PortalRoutingSnapshotsDialogProps) {
  const { t } = usePortalI18n();
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    if (!open) {
      return;
    }

    setSearchQuery(suggestedSearchQuery);
  }, [open, suggestedSearchQuery]);

  const filteredSnapshots = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();

    return sortSnapshots(snapshots).filter((snapshot) =>
      searchMatches(normalizedQuery, [
        snapshot.snapshot_id,
        snapshot.route_key,
        snapshot.capability,
        snapshot.matched_policy_id,
        snapshot.applied_routing_profile_id,
        snapshot.api_key_group_id,
        snapshot.default_provider_id,
        snapshot.preferred_region,
        snapshot.strategy,
        ...snapshot.ordered_provider_ids,
      ]));
  }, [searchQuery, snapshots]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(96vw,84rem)]">
        <DialogHeader>
          <DialogTitle>{t('Compiled snapshots')}</DialogTitle>
          <DialogDescription>
            {t(
              'Inspect the compiled routing evidence for this workspace after policy, project defaults, and API key group profile overlays are combined.',
            )}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <SearchInput
            onChange={(event) => setSearchQuery(event.target.value)}
            placeholder={t('Search compiled snapshots')}
            value={searchQuery}
          />

          {snapshotStatus ? (
            <div className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300">
              {snapshotStatus}
            </div>
          ) : null}

          {loadingSnapshots ? (
            <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 px-4 py-5 text-sm text-zinc-500 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300">
              {t('Loading compiled snapshots...')}
            </div>
          ) : filteredSnapshots.length ? (
            <div className="max-h-[60vh] space-y-3 overflow-y-auto pr-1">
              {filteredSnapshots.map((snapshot) => (
                <article
                  key={snapshot.snapshot_id}
                  className="rounded-[24px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70"
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="space-y-1">
                      <strong className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                        {snapshot.route_key}
                      </strong>
                      <p className="text-xs text-zinc-500 dark:text-zinc-400">
                        {snapshot.snapshot_id}
                      </p>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="outline">{snapshot.capability}</Badge>
                      <Badge
                        variant={snapshot.applied_routing_profile_id ? 'success' : 'secondary'}
                      >
                        {snapshot.applied_routing_profile_id ?? t('No applied routing profile')}
                      </Badge>
                    </div>
                  </div>

                  <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                    <div className="space-y-1 text-sm">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {t('Applied routing profile')}
                      </div>
                      <div className="text-zinc-500 dark:text-zinc-400">
                        {snapshot.applied_routing_profile_id ?? t('No applied routing profile')}
                      </div>
                    </div>
                    <div className="space-y-1 text-sm">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {t('Bound API key group')}
                      </div>
                      <div className="text-zinc-500 dark:text-zinc-400">
                        {snapshot.api_key_group_id ?? t('No API key group scope')}
                      </div>
                    </div>
                    <div className="space-y-1 text-sm">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {t('Matched policy')}
                      </div>
                      <div className="text-zinc-500 dark:text-zinc-400">
                        {snapshot.matched_policy_id ?? t('No matched policy')}
                      </div>
                    </div>
                    <div className="space-y-1 text-sm">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {t('Strategy')}
                      </div>
                      <div className="text-zinc-500 dark:text-zinc-400">
                        {buildRoutingStrategyLabel(snapshot.strategy)}
                      </div>
                    </div>
                  </div>

                  <div className="mt-3 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                    <div className="space-y-1 text-sm">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {t('Default provider')}
                      </div>
                      <div className="text-zinc-500 dark:text-zinc-400">
                        {snapshot.default_provider_id ?? t('Auto fallback')}
                      </div>
                    </div>
                    <div className="space-y-1 text-sm">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {t('Preferred region')}
                      </div>
                      <div className="text-zinc-500 dark:text-zinc-400">
                        {snapshot.preferred_region ?? t('Auto')}
                      </div>
                    </div>
                    <div className="space-y-1 text-sm">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {t('Require healthy providers')}
                      </div>
                      <div className="text-zinc-500 dark:text-zinc-400">
                        {snapshot.require_healthy ? t('Enabled') : t('Disabled')}
                      </div>
                    </div>
                    <div className="space-y-1 text-sm">
                      <div className="font-medium text-zinc-950 dark:text-zinc-50">
                        {t('Created at')}
                      </div>
                      <div className="text-zinc-500 dark:text-zinc-400">
                        {formatDateTime(snapshot.updated_at_ms || snapshot.created_at_ms)}
                      </div>
                    </div>
                  </div>

                  <div className="mt-3 space-y-1 text-sm">
                    <div className="font-medium text-zinc-950 dark:text-zinc-50">
                      {t('Provider roster')}
                    </div>
                    <div className="text-zinc-500 dark:text-zinc-400">
                      {snapshot.ordered_provider_ids.length
                        ? snapshot.ordered_provider_ids.join(', ')
                        : t('Auto fallback')}
                    </div>
                  </div>
                </article>
              ))}
            </div>
          ) : (
            <EmptyState
              description={t(
                'Broaden the query or refresh the workspace after new routing decisions land.',
              )}
              title={t('No compiled snapshots match the current filter')}
            />
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
