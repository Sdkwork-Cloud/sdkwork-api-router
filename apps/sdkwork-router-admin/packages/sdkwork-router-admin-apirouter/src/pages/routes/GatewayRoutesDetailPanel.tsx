import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DescriptionDetails,
  DescriptionItem,
  DescriptionList,
  DescriptionTerm,
  InlineAlert,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import type { GatewayRouteInventoryRow } from '../../services/gatewayViewService';
import type { ProviderRoutingImpact } from './routingSnapshotAnalytics';
import { statusVariant } from './shared';

type GatewayRoutesDetailPanelProps = {
  providerRoutingImpact: ProviderRoutingImpact | null;
  selectedRow: GatewayRouteInventoryRow;
};

export function GatewayRoutesDetailPanel({
  providerRoutingImpact,
  selectedRow,
}: GatewayRoutesDetailPanelProps) {
  const { t } = useAdminI18n();

  return (
    <div className="space-y-4">
      <DescriptionList columns={2}>
        <DescriptionItem>
          <DescriptionTerm>{t('Provider id')}</DescriptionTerm>
          <DescriptionDetails mono>{selectedRow.provider.id}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Adapter')}</DescriptionTerm>
          <DescriptionDetails>{selectedRow.provider.adapter_kind}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Primary channel')}</DescriptionTerm>
          <DescriptionDetails>{selectedRow.primary_channel_name}</DescriptionDetails>
        </DescriptionItem>
        <DescriptionItem>
          <DescriptionTerm>{t('Base URL')}</DescriptionTerm>
          <DescriptionDetails mono>{selectedRow.provider.base_url}</DescriptionDetails>
        </DescriptionItem>
      </DescriptionList>

      <Card>
        <CardHeader className="space-y-2">
          <div className="flex items-start justify-between gap-3">
            <div>
              <CardTitle className="text-base">{t('Bound channels')}</CardTitle>
              <CardDescription>
                {t('Public API channels currently pointing at this provider.')}
              </CardDescription>
            </div>
            <StatusBadge
              label={t(selectedRow.health_status)}
              showIcon
              status={selectedRow.healthy ? 'active' : 'failed'}
              variant={statusVariant(selectedRow)}
            />
          </div>
        </CardHeader>
        <CardContent className="space-y-2 text-sm text-[var(--sdk-color-text-secondary)]">
          {selectedRow.channels.map((channel) => (
            <div key={channel.id}>
              <span className="font-medium text-[var(--sdk-color-text-primary)]">
                {channel.name}
              </span>
              <span className="ml-2 font-mono text-xs">{channel.id}</span>
            </div>
          ))}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('Credential inventory')}</CardTitle>
          <CardDescription>
            {t('Keys currently bound to the selected provider.')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm text-[var(--sdk-color-text-secondary)]">
          {selectedRow.credentials.length ? (
            selectedRow.credentials.map((credential) => (
              <div key={`${credential.tenant_id}:${credential.key_reference}`}>
                <span className="font-medium text-[var(--sdk-color-text-primary)]">
                  {credential.key_reference}
                </span>
                <span className="ml-2">
                  {credential.tenant_id} / {credential.secret_backend}
                </span>
              </div>
            ))
          ) : (
            <InlineAlert
              description={t('No encrypted credential records are currently assigned to this provider.')}
              title={t('Credential coverage is empty')}
              tone="warning"
            />
          )}
        </CardContent>
      </Card>

      {providerRoutingImpact ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t('Routing impact')}</CardTitle>
            <CardDescription>
              {t('Inspect how the selected provider participates in compiled snapshots, reusable routing profiles, and default-route posture.')}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <DescriptionList columns={2}>
              <DescriptionItem>
                <DescriptionTerm>{t('Compiled snapshots')}</DescriptionTerm>
                <DescriptionDetails>
                  {providerRoutingImpact.compiledSnapshotCount}
                </DescriptionDetails>
              </DescriptionItem>
              <DescriptionItem>
                <DescriptionTerm>{t('Bound groups')}</DescriptionTerm>
                <DescriptionDetails>{providerRoutingImpact.boundGroupCount}</DescriptionDetails>
              </DescriptionItem>
              <DescriptionItem>
                <DescriptionTerm>{t('Routing profiles')}</DescriptionTerm>
                <DescriptionDetails>{providerRoutingImpact.routingProfileCount}</DescriptionDetails>
              </DescriptionItem>
              <DescriptionItem>
                <DescriptionTerm>{t('Default provider')}</DescriptionTerm>
                <DescriptionDetails>{providerRoutingImpact.defaultSnapshotCount}</DescriptionDetails>
              </DescriptionItem>
            </DescriptionList>

            <div className="space-y-2 text-sm text-[var(--sdk-color-text-secondary)]">
              <div className="font-medium text-[var(--sdk-color-text-primary)]">
                {t('Top affected routing profiles')}
              </div>
              {providerRoutingImpact.topProfiles.length ? (
                providerRoutingImpact.topProfiles.slice(0, 3).map((profileImpact) => (
                  <div
                    className="flex items-center justify-between gap-3"
                    key={profileImpact.routingProfile.profile_id}
                  >
                    <div className="min-w-0">
                      <div className="truncate font-medium text-[var(--sdk-color-text-primary)]">
                        {profileImpact.routingProfile.name}
                      </div>
                      <div className="font-mono text-xs">
                        {profileImpact.routingProfile.profile_id}
                      </div>
                    </div>
                    <div className="text-right">
                      <div>
                        {t('{count} snapshots', {
                          count: profileImpact.compiledSnapshots.length,
                        })}
                      </div>
                      <div className="text-xs">
                        {t('{count} groups', { count: profileImpact.boundGroups.length })}
                      </div>
                    </div>
                  </div>
                ))
              ) : (
                <InlineAlert
                  description={t('No compiled routing evidence currently references this provider through a reusable routing profile.')}
                  title={t('Routing evidence is empty')}
                  tone="info"
                />
              )}
            </div>

            {providerRoutingImpact.recentSnapshots.length ? (
              <div className="space-y-2 text-sm text-[var(--sdk-color-text-secondary)]">
                <div className="font-medium text-[var(--sdk-color-text-primary)]">
                  {t('Recent compiled snapshots')}
                </div>
                {providerRoutingImpact.recentSnapshots.map((evidenceRow) => (
                  <div
                    className="flex items-center justify-between gap-3"
                    key={evidenceRow.snapshot.snapshot_id}
                  >
                    <div className="min-w-0">
                      <div className="truncate font-medium text-[var(--sdk-color-text-primary)]">
                        {evidenceRow.snapshot.route_key}
                      </div>
                      <div className="font-mono text-xs">
                        {evidenceRow.snapshot.snapshot_id}
                      </div>
                    </div>
                    <div className="text-right">
                      <div>{evidenceRow.snapshot.capability}</div>
                      <div className="text-xs">
                        {evidenceRow.snapshot.default_provider_id === selectedRow.provider.id
                          ? t('Default provider')
                          : t('Fallback path')}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            ) : null}
          </CardContent>
        </Card>
      ) : null}
    </div>
  );
}
