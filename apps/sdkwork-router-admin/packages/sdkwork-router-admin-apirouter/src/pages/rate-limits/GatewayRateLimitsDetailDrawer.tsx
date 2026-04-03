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
  Drawer,
  DrawerBody,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  InlineAlert,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';

import type { GatewayRateLimitInventoryRow } from './shared';

type GatewayRateLimitsDetailDrawerProps = {
  onOpenChange: (open: boolean) => void;
  open: boolean;
  row: GatewayRateLimitInventoryRow | null;
};

function buildScopeLines(
  row: GatewayRateLimitInventoryRow,
  t: (text: string, values?: Record<string, string | number>) => string,
) {
  const lines: string[] = [];

  if (row.scope.projectWide) {
    lines.push(t('Project-wide'));
  }

  if (row.scope.apiKeyLabel) {
    lines.push(`${t('API key')}: ${row.scope.apiKeyLabel}`);
  }

  if (row.scope.routeKey) {
    lines.push(`${t('Route')}: ${row.scope.routeKey}`);
  }

  if (row.scope.modelName) {
    lines.push(`${t('Model')}: ${row.scope.modelName}`);
  }

  return lines;
}

export function GatewayRateLimitsDetailDrawer({
  onOpenChange,
  open,
  row,
}: GatewayRateLimitsDetailDrawerProps) {
  const { formatDateTime, formatNumber, t } = useAdminI18n();

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent side="right" size="xl">
        {row ? (
          <>
            <DrawerHeader>
              <div className="space-y-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <DrawerTitle>{row.projectName ?? row.policy.project_id}</DrawerTitle>
                    <DrawerDescription>{row.policy.policy_id}</DrawerDescription>
                  </div>
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
                </div>
              </div>
            </DrawerHeader>

            <DrawerBody className="space-y-4">
              <div className="grid gap-3 text-sm text-[var(--sdk-color-text-secondary)] sm:grid-cols-3">
                <div className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] p-4">
                  <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                    {t('Limit')}
                  </div>
                  <div className="mt-2 text-xl font-semibold text-[var(--sdk-color-text-primary)]">
                    {formatNumber(row.window?.limit_requests ?? row.policy.limit_requests)}
                  </div>
                  <div className="mt-1 text-xs text-[var(--sdk-color-text-secondary)]">
                    {t('{count}s window', {
                      count: formatNumber(row.window?.window_seconds ?? row.policy.window_seconds),
                    })}
                  </div>
                </div>
                <div className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] p-4">
                  <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                    {t('Burst')}
                  </div>
                  <div className="mt-2 text-xl font-semibold text-[var(--sdk-color-text-primary)]">
                    {formatNumber(row.window?.burst_requests ?? row.policy.burst_requests)}
                  </div>
                  <div className="mt-1 text-xs text-[var(--sdk-color-text-secondary)]">
                    {t('Requests allowed before hard limiting.')}
                  </div>
                </div>
                <div className="rounded-[var(--sdk-radius-control)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] p-4">
                  <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                    {t('Remaining')}
                  </div>
                  <div className="mt-2 text-xl font-semibold text-[var(--sdk-color-text-primary)]">
                    {formatNumber(row.window?.remaining_requests ?? row.policy.limit_requests)}
                  </div>
                  <div className="mt-1 text-xs text-[var(--sdk-color-text-secondary)]">
                    {row.window
                      ? t('{count} requests observed', {
                          count: formatNumber(row.window.request_count),
                        })
                      : t('Waiting for the first live window')}
                  </div>
                </div>
              </div>

              <Card>
                <CardHeader>
                  <CardTitle className="text-base">{t('Applied scope')}</CardTitle>
                  <CardDescription>
                    {t('The router evaluates project scope first, then optional key, route, and model qualifiers.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <DescriptionList columns={2}>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Project')}</DescriptionTerm>
                      <DescriptionDetails>
                        {row.projectName
                          ? `${row.projectName} (${row.policy.project_id})`
                          : row.policy.project_id}
                      </DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Policy ID')}</DescriptionTerm>
                      <DescriptionDetails>{row.policy.policy_id}</DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('API key')}</DescriptionTerm>
                      <DescriptionDetails>
                        {row.scope.apiKeyLabel ?? t('All API keys')}
                      </DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Route')}</DescriptionTerm>
                      <DescriptionDetails>
                        {row.scope.routeKey ?? t('Any route')}
                      </DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Model')}</DescriptionTerm>
                      <DescriptionDetails>
                        {row.scope.modelName ?? t('Any model')}
                      </DescriptionDetails>
                    </DescriptionItem>
                    <DescriptionItem>
                      <DescriptionTerm>{t('Updated')}</DescriptionTerm>
                      <DescriptionDetails>
                        {formatDateTime(row.policy.updated_at_ms)}
                      </DescriptionDetails>
                    </DescriptionItem>
                  </DescriptionList>

                  <div className="flex flex-wrap gap-2">
                    {buildScopeLines(row, t).map((line) => (
                      <StatusBadge key={line} status={line} variant="secondary" />
                    ))}
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-base">{t('Live window')}</CardTitle>
                  <CardDescription>
                    {t('Live counters are refreshed from the gateway snapshot already loaded into the admin workspace.')}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {row.window ? (
                    <DescriptionList columns={2}>
                      <DescriptionItem>
                        <DescriptionTerm>{t('Requests')}</DescriptionTerm>
                        <DescriptionDetails>
                          {formatNumber(row.window.request_count)}
                        </DescriptionDetails>
                      </DescriptionItem>
                      <DescriptionItem>
                        <DescriptionTerm>{t('Remaining')}</DescriptionTerm>
                        <DescriptionDetails>
                          {formatNumber(row.window.remaining_requests)}
                        </DescriptionDetails>
                      </DescriptionItem>
                      <DescriptionItem>
                        <DescriptionTerm>{t('Window start')}</DescriptionTerm>
                        <DescriptionDetails>
                          {formatDateTime(row.window.window_start_ms)}
                        </DescriptionDetails>
                      </DescriptionItem>
                      <DescriptionItem>
                        <DescriptionTerm>{t('Window end')}</DescriptionTerm>
                        <DescriptionDetails>
                          {formatDateTime(row.window.window_end_ms)}
                        </DescriptionDetails>
                      </DescriptionItem>
                      <DescriptionItem>
                        <DescriptionTerm>{t('Observed update')}</DescriptionTerm>
                        <DescriptionDetails>
                          {formatDateTime(row.window.updated_at_ms)}
                        </DescriptionDetails>
                      </DescriptionItem>
                      <DescriptionItem>
                        <DescriptionTerm>{t('State')}</DescriptionTerm>
                        <DescriptionDetails>
                          {row.window.exceeded ? t('Exceeded') : t('Within threshold')}
                        </DescriptionDetails>
                      </DescriptionItem>
                    </DescriptionList>
                  ) : (
                    <InlineAlert
                      description={t(
                        'This policy has not emitted a live window yet. It will appear here after the first matching request reaches the router.',
                      )}
                      showIcon
                      title={t('Waiting for traffic')}
                      tone="info"
                    />
                  )}
                </CardContent>
              </Card>

              {row.policy.notes ? (
                <Card>
                  <CardHeader>
                    <CardTitle className="text-base">{t('Notes')}</CardTitle>
                  </CardHeader>
                  <CardContent className="text-sm text-[var(--sdk-color-text-secondary)]">
                    {row.policy.notes}
                  </CardContent>
                </Card>
              ) : null}
            </DrawerBody>

            <DrawerFooter className="text-xs text-[var(--sdk-color-text-secondary)]">
              {t(
                'The router applies the strongest matching policy. Use project-wide rules for a safe default and narrower key, route, or model scopes for exceptions.',
              )}
            </DrawerFooter>
          </>
        ) : null}
      </DrawerContent>
    </Drawer>
  );
}
