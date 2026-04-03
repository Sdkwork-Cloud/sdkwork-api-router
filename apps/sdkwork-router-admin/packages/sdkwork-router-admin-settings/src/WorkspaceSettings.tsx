import {
  Badge,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  FormSection,
  InlineAlert,
} from '@sdkwork/ui-pc-react';
import { useAdminAppStore, useAdminI18n } from 'sdkwork-router-admin-core';

import { SettingsBadge, SettingsSummaryCard } from './Shared';

export function WorkspaceSettings() {
  const { hiddenSidebarItems, isSidebarCollapsed, sidebarWidth, themeColor, themeMode } =
    useAdminAppStore();
  const { t } = useAdminI18n();

  return (
    <div className="space-y-8">
      <InlineAlert
        description={t(
          'Every shell preference persists so the control plane reopens with the same workspace and operator posture.',
        )}
        showIcon
        title={t('shell continuity')}
        tone="info"
      />

      <FormSection
        actions={<SettingsBadge variant="secondary">{t('shell posture')}</SettingsBadge>}
        description={t('Keep the left navigation rail and the right canvas in a single consistent shell contract.')}
        title={t('Workspace')}
      >
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
          <SettingsSummaryCard label={t('Theme mode')} value={t(themeMode)} />
          <SettingsSummaryCard label={t('Theme color')} value={t(themeColor)} />
          <SettingsSummaryCard label={t('Sidebar width')} value={`${sidebarWidth}px`} />
          <SettingsSummaryCard
            label={t('Sidebar mode')}
            value={isSidebarCollapsed ? t('collapsed') : t('expanded')}
          />
          <SettingsSummaryCard
            label={t('Hidden nav items')}
            value={hiddenSidebarItems.length}
          />
          <SettingsSummaryCard
            badge={<SettingsBadge variant="outline">{t('right canvas')}</SettingsBadge>}
            label={t('Content region')}
            value={t('single workspace surface')}
          />
        </div>
      </FormSection>

      <Card>
        <CardHeader>
          <CardTitle>{t('workspace persistence')}</CardTitle>
          <CardDescription>
            {t(
              'The layout stays split into a claw-style left navigation rail and a single right content region, keeping product behavior and visual framing consistent.',
            )}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 text-sm text-[var(--sdk-color-text-secondary)]">
          <p>
            {t(
              'Theme preferences, sidebar width, hidden entries, and collapse state are persisted so the control-plane workspace reopens with the same shell posture.',
            )}
          </p>
          <p>
            {t(
              'Appearance, navigation, and workspace sections now live in a real settings center instead of a standalone preferences panel.',
            )}
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
