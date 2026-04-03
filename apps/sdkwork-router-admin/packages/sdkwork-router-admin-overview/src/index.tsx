import type { ComponentProps, ReactNode } from 'react';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  ManagementWorkbench,
  StatCard,
  StatusBadge,
} from '@sdkwork/ui-pc-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps, AdminRouteKey } from 'sdkwork-router-admin-types';

import { buildAdminOverviewViewModel } from './view-model';

type OverviewWorkbenchMainProps = ComponentProps<typeof ManagementWorkbench>['main'] & {
  children: ReactNode;
};

type OverviewWorkbenchDetailProps = NonNullable<
  ComponentProps<typeof ManagementWorkbench>['detail']
> & {
  children: ReactNode;
};

type OverviewWorkbenchProps = Omit<
  ComponentProps<typeof ManagementWorkbench>,
  'main' | 'detail'
> & {
  main: OverviewWorkbenchMainProps;
  detail?: OverviewWorkbenchDetailProps;
};

function OverviewWorkbench(props: OverviewWorkbenchProps) {
  return (
    <ManagementWorkbench
      {...(props as unknown as ComponentProps<typeof ManagementWorkbench>)}
    />
  );
}

export function OverviewPage({
  snapshot,
  onNavigate: _onNavigate,
}: AdminPageProps & { onNavigate: (route: AdminRouteKey) => void }) {
  const { formatCurrency, formatNumber, t } = useAdminI18n();
  const viewModel = buildAdminOverviewViewModel(snapshot);
  const rankedUsers = viewModel.rankedUsers;
  const rankedProjects = viewModel.rankedProjects;

  return (
    <OverviewWorkbench
      description={t('Global health, operator alerts, and the most active workspace surfaces from the live control plane.')}
      eyebrow={t('Control plane')}
      main={{
        title: t('Live control-plane posture'),
        description: t(
          'Metrics and alerts refresh from the same shared shell contract used across the rest of the admin workspace.',
        ),
        children: (
          <div className="space-y-6">
            <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
              {viewModel.metrics.map((metric) => (
                <StatCard
                  description={t(metric.detail)}
                  key={metric.label}
                  label={t(metric.label)}
                  value={metric.value}
                />
              ))}
            </div>

            <Card>
              <CardHeader>
                <CardTitle>{t('Operator alerts')}</CardTitle>
                <CardDescription>
                  {t(
                    'Alerts are generated from live billing, runtime, catalog, and workspace health signals.',
                  )}
                </CardDescription>
              </CardHeader>
              <CardContent className="grid gap-4 xl:grid-cols-2">
                {viewModel.alerts.map((alert) => (
                  <Card key={alert.id}>
                    <CardHeader className="space-y-3">
                      <div className="flex items-start justify-between gap-3">
                        <CardTitle className="text-base">{t(alert.title)}</CardTitle>
                        <StatusBadge
                          showIcon
                          status={alert.severity}
                          variant={alert.severity === 'high' ? 'danger' : alert.severity === 'medium' ? 'warning' : 'secondary'}
                        />
                      </div>
                      <CardDescription>{t(alert.detail)}</CardDescription>
                    </CardHeader>
                  </Card>
                ))}
              </CardContent>
            </Card>
          </div>
        ),
      }}
      detail={{
        title: t('Traffic leaders'),
        description: t(
          'The right rail keeps the busiest users and projects visible without leaving the main overview.',
        ),
        children: (
          <div className="space-y-4">
            <div className="space-y-3">
              <div className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                {t('Top portal users')}
              </div>
              {rankedUsers.map((user) => (
                <Card key={user.id}>
                  <CardHeader className="space-y-2">
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <CardTitle className="text-base">{user.display_name}</CardTitle>
                        <CardDescription>{user.email}</CardDescription>
                      </div>
                      <StatusBadge
                        showIcon
                        status={user.active ? 'active' : 'disabled'}
                        variant={user.active ? 'success' : 'danger'}
                      />
                    </div>
                  </CardHeader>
                  <CardContent className="grid gap-1 text-sm text-[var(--sdk-color-text-secondary)]">
                    <div>{t('Requests: {count}', { count: formatNumber(user.request_count) })}</div>
                    <div>{t('Tokens: {count}', { count: formatNumber(user.total_tokens) })}</div>
                    <div>{t('Units: {count}', { count: formatNumber(user.usage_units) })}</div>
                  </CardContent>
                </Card>
              ))}
            </div>

            <div className="space-y-3">
              <div className="text-xs font-semibold uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                {t('Hottest projects')}
              </div>
              {rankedProjects.map((project) => (
                <Card key={project.id}>
                  <CardHeader className="space-y-2">
                    <CardTitle className="text-base">{project.name}</CardTitle>
                      <CardDescription>{project.tenant_id}</CardDescription>
                  </CardHeader>
                  <CardContent className="grid gap-1 text-sm text-[var(--sdk-color-text-secondary)]">
                    <div>{t('Requests: {count}', { count: formatNumber(project.request_count) })}</div>
                    <div>{t('Tokens: {count}', { count: formatNumber(project.total_tokens) })}</div>
                    <div>{t('Amount: {amount}', { amount: formatCurrency(project.booked_amount) })}</div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        ),
      }}
      title={t('Overview')}
    />
  );
}
