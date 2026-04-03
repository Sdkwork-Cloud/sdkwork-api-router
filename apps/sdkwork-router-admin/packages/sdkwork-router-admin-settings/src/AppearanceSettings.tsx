import { Check, Laptop, Moon, Sun } from 'lucide-react';
import {
  Badge,
  Button,
  FormGrid,
  FormSection,
  SettingsField,
} from '@sdkwork/ui-pc-react';

import { useAdminAppStore, useAdminI18n } from 'sdkwork-router-admin-core';

import { SettingsBadge, SettingsChoiceButton, SettingsSummaryCard } from './Shared';

const THEME_COLORS = [
  { id: 'tech-blue', label: 'tech-blue', colorClass: 'bg-sky-500' },
  { id: 'lobster', label: 'lobster', colorClass: 'bg-red-500' },
  { id: 'green-tech', label: 'green-tech', colorClass: 'bg-emerald-500' },
  { id: 'zinc', label: 'zinc', colorClass: 'bg-zinc-500' },
  { id: 'violet', label: 'violet', colorClass: 'bg-violet-500' },
  { id: 'rose', label: 'rose', colorClass: 'bg-rose-500' },
] as const;

export function AppearanceSettings() {
  const { setThemeColor, setThemeMode, themeColor, themeMode } = useAdminAppStore();
  const { t } = useAdminI18n();

  return (
    <div className="space-y-8">
      <FormSection
        actions={<SettingsBadge variant="secondary">{t('Theme posture')}</SettingsBadge>}
        description={t('Choose how the shell follows light, dark, or system appearance.')}
        title={t('Theme mode')}
      >
        <FormGrid columns={3}>
          <SettingsChoiceButton
            active={themeMode === 'light'}
            description={t('Bright shell with frosted content panes.')}
            icon={Sun}
            label={t('Light')}
            onClick={() => setThemeMode('light')}
          />
          <SettingsChoiceButton
            active={themeMode === 'dark'}
            description={t('Claw-style low-glare shell with higher contrast.')}
            icon={Moon}
            label={t('Dark')}
            onClick={() => setThemeMode('dark')}
          />
          <SettingsChoiceButton
            active={themeMode === 'system'}
            description={t('Follow the device preference automatically.')}
            icon={Laptop}
            label={t('System')}
            onClick={() => setThemeMode('system')}
          />
        </FormGrid>
      </FormSection>

      <FormSection
        actions={<SettingsBadge variant="outline">{t('Accent')}</SettingsBadge>}
        description={t('Theme color updates accent surfaces without changing the shell contract.')}
        title={t('Theme color')}
      >
        <SettingsField
          description={t('Accent preset')}
          label={t('Theme color')}
          layout="vertical"
        >
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {THEME_COLORS.map((color) => {
              const active = themeColor === color.id;

              return (
                <Button
                  className="h-auto justify-start px-4 py-4"
                  key={color.id}
                  onClick={() => setThemeColor(color.id)}
                  type="button"
                  variant={active ? 'primary' : 'outline'}
                >
                  <span className="flex w-full items-center gap-3">
                    <span
                      className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${color.colorClass}`}
                    >
                      {active ? <Check className="h-4 w-4 text-white" /> : null}
                    </span>
                    <span className="flex min-w-0 flex-1 flex-col items-start text-left">
                      <span className="text-sm font-semibold">{t(color.label)}</span>
                      <span className="text-xs font-normal opacity-80">
                        {t('Accent preset')}
                      </span>
                    </span>
                  </span>
                </Button>
              );
            })}
          </div>
        </SettingsField>

        <div className="grid gap-4 md:grid-cols-2">
          <SettingsSummaryCard label={t('Theme mode')} value={t(themeMode)} />
          <SettingsSummaryCard label={t('Theme color')} value={t(themeColor)} />
        </div>
      </FormSection>
    </div>
  );
}
