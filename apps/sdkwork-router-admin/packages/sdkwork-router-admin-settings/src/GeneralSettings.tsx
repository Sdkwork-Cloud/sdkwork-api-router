import {
  Badge,
  FormGrid,
  FormSection,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SettingsField,
} from '@sdkwork/ui-pc-react';

import {
  ADMIN_LOCALE_OPTIONS,
  useAdminAppStore,
  useAdminI18n,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';

import { SettingsBadge, SettingsSummaryCard } from './Shared';

export function GeneralSettings() {
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    sidebarWidth,
    themeColor,
    themeMode,
  } = useAdminAppStore();
  const { sessionUser, status } = useAdminWorkbench();
  const { locale, setLocale, t } = useAdminI18n();

  return (
    <div className="space-y-8">
      <FormSection
        description={t(
          'Choose the operator workspace language. Dates, numbers, and shared shell copy follow this setting immediately.',
        )}
        title={t('Language and locale')}
      >
        <FormGrid columns={1}>
          <SettingsField
            controlId="admin-settings-language"
            description={t('Language updates every route label, shell notice, and workspace detail immediately.')}
            label={t('Language')}
            layout="vertical"
          >
            <Select
              onValueChange={(value: string) => setLocale(value as typeof locale)}
              value={locale}
            >
              <SelectTrigger id="admin-settings-language">
                <SelectValue placeholder={t('Language')} />
              </SelectTrigger>
              <SelectContent>
                {ADMIN_LOCALE_OPTIONS.map((option) => (
                  <SelectItem key={option.id} value={option.id}>
                    {t(option.label)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </SettingsField>
        </FormGrid>
      </FormSection>

      <FormSection
        actions={
          <SettingsBadge variant={sessionUser?.active ? 'success' : 'warning'}>
            {sessionUser?.active ? t('live shell summary') : t('Workspace')}
          </SettingsBadge>
        }
        description={t('Current shell posture for the control plane workspace.')}
        title={t('Workspace posture')}
      >
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          <SettingsSummaryCard
            badge={sessionUser?.active ? t('active') : t('Settings center')}
            detail={sessionUser?.email ?? t(status)}
            label={t('Operator')}
            value={sessionUser?.display_name ?? t('Control plane operator')}
          />
          <SettingsSummaryCard label={t('Theme mode')} value={t(themeMode)} />
          <SettingsSummaryCard label={t('Theme color')} value={t(themeColor)} />
          <SettingsSummaryCard
            label={t('Sidebar mode')}
            value={isSidebarCollapsed ? t('collapsed') : t('expanded')}
          />
          <SettingsSummaryCard label={t('Sidebar width')} value={`${sidebarWidth}px`} />
          <SettingsSummaryCard
            label={t('Hidden nav items')}
            value={hiddenSidebarItems.length}
          />
        </div>
      </FormSection>
    </div>
  );
}
