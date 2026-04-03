import { useMemo, useState, type ChangeEvent } from 'react';
import {
  Card,
  CardContent,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  StatCard,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { Search } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import { SelectField } from '../shared';
import type { RoutingSnapshotAnalytics } from './routingSnapshotAnalytics';

type GatewayRoutingSnapshotsDialogProps = {
  analytics: RoutingSnapshotAnalytics;
  onOpenChange: (open: boolean) => void;
  open: boolean;
};

export function GatewayRoutingSnapshotsDialog({
  analytics,
  onOpenChange,
  open,
}: GatewayRoutingSnapshotsDialogProps) {
  const { formatDateTime, formatNumber, t } = useAdminI18n();
  const [search, setSearch] = useState('');
  const [selectedProfileId, setSelectedProfileId] = useState('all');

  const filteredEvidenceRows = useMemo(() => {
    const normalizedSearch = search.trim().toLowerCase();

    return analytics.evidenceRows.filter((evidenceRow) => {
      if (
        selectedProfileId !== 'all'
        && (evidenceRow.routingProfile?.profile_id ?? '__unprofiled__') !== selectedProfileId
      ) {
        return false;
      }

      if (!normalizedSearch) {
        return true;
      }

      return [
        evidenceRow.snapshot.snapshot_id,
        evidenceRow.snapshot.route_key,
        evidenceRow.snapshot.capability,
        evidenceRow.snapshot.preferred_region ?? '',
        evidenceRow.snapshot.default_provider_id ?? '',
        evidenceRow.routingProfile?.name ?? '',
        evidenceRow.routingProfile?.profile_id ?? '',
        evidenceRow.apiKeyGroup?.name ?? '',
        evidenceRow.apiKeyGroup?.group_id ?? '',
        ...evidenceRow.snapshot.ordered_provider_ids,
      ]
        .join(' ')
        .toLowerCase()
        .includes(normalizedSearch);
    });
  }, [analytics.evidenceRows, search, selectedProfileId]);

  const profileOptions = useMemo(
    () => [
      { label: t('All profiles'), value: 'all' },
      { label: t('No applied routing profile'), value: '__unprofiled__' },
      ...analytics.topProfiles.map((profileImpact) => ({
        label: `${profileImpact.routingProfile.name} (${profileImpact.routingProfile.slug})`,
        value: profileImpact.routingProfile.profile_id,
      })),
    ],
    [analytics.topProfiles, t],
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(96vw,88rem)]">
        <DialogHeader>
          <DialogTitle>{t('Compiled snapshots')}</DialogTitle>
          <DialogDescription>
            {t('Inspect the compiled routing evidence that the gateway produced after combining policy, project defaults, and API key group routing profile overlays.')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
            <StatCard
              description={t('All compiled route snapshots currently loaded into the admin workspace.')}
              label={t('Compiled snapshots')}
              value={formatNumber(analytics.totalCompiledSnapshots)}
            />
            <StatCard
              description={t('Snapshots that carry an applied routing profile id.')}
              label={t('Applied routing profile')}
              value={formatNumber(analytics.profileBackedSnapshotCount)}
            />
            <StatCard
              description={t('API key groups currently bound to a reusable routing profile.')}
              label={t('Bound groups')}
              value={formatNumber(analytics.boundGroupCount)}
            />
            <StatCard
              description={t('Distinct route keys represented across the compiled snapshot evidence set.')}
              label={t('Route keys')}
              value={formatNumber(analytics.uniqueRouteKeyCount)}
            />
          </div>

          <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr),14rem]">
            <div>
              <Label className="sr-only" htmlFor="routing-snapshots-search">
                {t('Search compiled snapshots')}
              </Label>
              <div className="relative">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                <Input
                  className="pl-9"
                  id="routing-snapshots-search"
                  onChange={(event: ChangeEvent<HTMLInputElement>) =>
                    setSearch(event.target.value)
                  }
                  placeholder={t('Search compiled snapshots')}
                  value={search}
                />
              </div>
            </div>
            <SelectField
              label={t('Applied routing profile')}
              labelVisibility="sr-only"
              onValueChange={setSelectedProfileId}
              options={profileOptions}
              placeholder={t('Applied routing profile')}
              value={selectedProfileId}
            />
          </div>

          <div className="space-y-4">
            {filteredEvidenceRows.length ? (
              filteredEvidenceRows.map((evidenceRow) => (
                <Card key={evidenceRow.snapshot.snapshot_id}>
                  <CardContent className="space-y-4 p-4">
                    <div className="flex flex-wrap items-start justify-between gap-3">
                      <div className="space-y-1">
                        <div className="font-semibold text-[var(--sdk-color-text-primary)]">
                          {evidenceRow.snapshot.route_key}
                        </div>
                        <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                          {evidenceRow.snapshot.snapshot_id}
                        </div>
                      </div>
                      <div className="flex flex-wrap gap-2">
                        <StatusBadge
                          label={evidenceRow.snapshot.capability}
                          showIcon
                          status={evidenceRow.snapshot.capability}
                          variant="secondary"
                        />
                        <StatusBadge
                          label={
                            evidenceRow.routingProfile?.name
                            ?? t('No applied routing profile')
                          }
                          showIcon
                          status={
                            evidenceRow.routingProfile?.active ? 'active' : 'paused'
                          }
                          variant={
                            evidenceRow.routingProfile?.active ? 'success' : 'secondary'
                          }
                        />
                      </div>
                    </div>

                    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                      <div className="space-y-1 text-sm">
                        <div className="font-medium text-[var(--sdk-color-text-primary)]">
                          {t('Applied routing profile')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {evidenceRow.routingProfile
                            ? `${evidenceRow.routingProfile.name} (${evidenceRow.routingProfile.profile_id})`
                            : t('No applied routing profile')}
                        </div>
                      </div>
                      <div className="space-y-1 text-sm">
                        <div className="font-medium text-[var(--sdk-color-text-primary)]">
                          {t('Bound groups')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {evidenceRow.apiKeyGroup
                            ? `${evidenceRow.apiKeyGroup.name} (${evidenceRow.apiKeyGroup.group_id})`
                            : t('No API key group scope')}
                        </div>
                      </div>
                      <div className="space-y-1 text-sm">
                        <div className="font-medium text-[var(--sdk-color-text-primary)]">
                          {t('Capability')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {evidenceRow.snapshot.capability}
                        </div>
                      </div>
                      <div className="space-y-1 text-sm">
                        <div className="font-medium text-[var(--sdk-color-text-primary)]">
                          {t('Route key')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {evidenceRow.snapshot.route_key}
                        </div>
                      </div>
                    </div>

                    <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                      <div className="space-y-1 text-sm">
                        <div className="font-medium text-[var(--sdk-color-text-primary)]">
                          {t('Provider order')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {evidenceRow.snapshot.ordered_provider_ids.length
                            ? evidenceRow.snapshot.ordered_provider_ids.join(', ')
                            : t('No ordered providers')}
                        </div>
                      </div>
                      <div className="space-y-1 text-sm">
                        <div className="font-medium text-[var(--sdk-color-text-primary)]">
                          {t('Default provider')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {evidenceRow.snapshot.default_provider_id ?? t('No default provider')}
                        </div>
                      </div>
                      <div className="space-y-1 text-sm">
                        <div className="font-medium text-[var(--sdk-color-text-primary)]">
                          {t('Preferred region')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {evidenceRow.snapshot.preferred_region || t('Auto')}
                        </div>
                      </div>
                      <div className="space-y-1 text-sm">
                        <div className="font-medium text-[var(--sdk-color-text-primary)]">
                          {t('Updated')}
                        </div>
                        <div className="text-[var(--sdk-color-text-secondary)]">
                          {formatDateTime(evidenceRow.snapshot.updated_at_ms)}
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))
            ) : (
              <Card>
                <CardContent className="space-y-1 p-4 text-sm text-[var(--sdk-color-text-secondary)]">
                  <div className="font-medium text-[var(--sdk-color-text-primary)]">
                    {t('No compiled snapshots match the current filter')}
                  </div>
                  <div>
                    {t('Broaden the query or refresh the workspace after new routing decisions land.')}
                  </div>
                </CardContent>
              </Card>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
