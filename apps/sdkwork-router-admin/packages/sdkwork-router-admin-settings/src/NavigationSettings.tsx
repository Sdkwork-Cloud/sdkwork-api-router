import {
  Badge,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Checkbox,
  FormSection,
  SettingsField,
  Switch,
} from '@sdkwork/ui-pc-react';
import { adminRoutes, useAdminAppStore, useAdminI18n } from 'sdkwork-router-admin-core';

import { SettingsBadge, SettingsSummaryCard } from './Shared';

export function NavigationSettings() {
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    setSidebarCollapsed,
    sidebarWidth,
    toggleSidebarItem,
  } = useAdminAppStore();
  const { t } = useAdminI18n();

  const sidebarRoutes = adminRoutes.filter((route) => route.key !== 'settings');
  const visibleSidebarItems = sidebarRoutes.length - hiddenSidebarItems.length;

  return (
    <div className="space-y-8">
      <FormSection
        actions={
          <SettingsBadge variant="secondary">{t('sidebar and canvas posture')}</SettingsBadge>
        }
        description={t('Keep the left rail expanded or collapse it into icon-only navigation.')}
        title={t('Sidebar behavior')}
      >
        <SettingsField
          description={t('Reduce the rail to icon-only navigation without changing the canvas.')}
          label={t('Collapsed sidebar')}
        >
          <div className="flex items-center justify-between gap-4 rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3">
            <div className="space-y-1">
              <div className="text-sm font-medium text-[var(--sdk-color-text-primary)]">
                {t('Compact navigation')}
              </div>
              <div className="text-sm text-[var(--sdk-color-text-secondary)]">
                {t('The left rail remains the navigation source of truth while labels collapse into icons.')}
              </div>
            </div>
            <Switch
              checked={isSidebarCollapsed}
              onCheckedChange={(checked: boolean) => setSidebarCollapsed(checked === true)}
            />
          </div>
        </SettingsField>

        <div className="grid gap-4 md:grid-cols-3">
          <SettingsSummaryCard label={t('Visible routes')} value={visibleSidebarItems} />
          <SettingsSummaryCard
            label={t('Sidebar mode')}
            value={isSidebarCollapsed ? t('collapsed') : t('expanded')}
          />
          <SettingsSummaryCard label={t('Sidebar width')} value={`${sidebarWidth}px`} />
        </div>
      </FormSection>

      <FormSection
        actions={<SettingsBadge variant="outline">{t('sidebar visibility')}</SettingsBadge>}
        description={t('Show or hide modules while keeping the left navigation rail compact and stable.')}
        title={t('Navigation')}
      >
        <div className="grid gap-4 sm:grid-cols-2">
          {sidebarRoutes.map((route) => {
            const visible = !hiddenSidebarItems.includes(route.key);

            return (
              <Card key={route.key}>
                <CardHeader className="space-y-2 pb-2">
                  <div className="flex items-start justify-between gap-3">
                    <div className="space-y-1">
                      <CardTitle className="text-base">{t(route.label)}</CardTitle>
                      <CardDescription>{t(route.detail)}</CardDescription>
                    </div>
                    <Checkbox
                      checked={visible}
                      onCheckedChange={() => toggleSidebarItem(route.key)}
                    />
                  </div>
                </CardHeader>
                <CardContent className="pt-0">
                  <div className="text-xs uppercase tracking-[0.18em] text-[var(--sdk-color-text-muted)]">
                    {t(route.group ?? 'Workspace')}
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      </FormSection>
    </div>
  );
}
