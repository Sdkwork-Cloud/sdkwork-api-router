import {
  Laptop,
  Moon,
  Palette,
  PanelLeft,
  RotateCcw,
  Sun,
  UserRound,
  type LucideIcon,
} from 'lucide-react';
import { motion } from 'motion/react';
import { useEffect, useMemo, useState } from 'react';

import { PORTAL_LOCALE_OPTIONS, usePortalI18n } from 'sdkwork-router-portal-commons';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from 'sdkwork-router-portal-commons/framework/entry';
import {
  SearchInput,
  SettingsField,
} from 'sdkwork-router-portal-commons/framework/form';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from 'sdkwork-router-portal-commons/framework/overlays';
import type { PortalThemeMode } from 'sdkwork-router-portal-types';

import {
  PORTAL_THEME_COLOR_OPTIONS,
  PORTAL_THEME_MODE_OPTIONS,
} from '../lib/portalPreferences';
import { portalSidebarRoutes } from '../routes';
import { usePortalAuthStore } from '../store/usePortalAuthStore';
import { usePortalShellStore } from '../store/usePortalShellStore';
import {
  PortalSettingsActionButton,
  PortalSettingsIdentityCard,
  PortalSettingsNavButton,
  PortalSettingsPanelCard,
  PortalSettingsToggleRow,
  PortalThemeColorSwatch,
  PortalThemeModeChoiceCard,
} from './settings/PortalSettingsPrimitives';

type ConfigCenterSectionId = 'appearance' | 'navigation' | 'workspace';

const THEME_MODE_ICONS: Record<PortalThemeMode, LucideIcon> = {
  light: Sun,
  dark: Moon,
  system: Laptop,
};

const CONFIG_CENTER_SECTIONS: Array<{
  id: ConfigCenterSectionId;
  icon: LucideIcon;
}> = [
  {
    id: 'appearance',
    icon: Palette,
  },
  {
    id: 'navigation',
    icon: PanelLeft,
  },
  {
    id: 'workspace',
    icon: UserRound,
  },
];

export function PortalSettingsCenter({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { locale, setLocale, t } = usePortalI18n();
  const workspace = usePortalAuthStore((state) => state.workspace);
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    resetShellPreferences,
    setSidebarCollapsed,
    themeColor,
    themeMode,
    toggleSidebarItem,
    setThemeColor,
    setThemeMode,
  } = usePortalShellStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [activeSection, setActiveSection] = useState<ConfigCenterSectionId>('appearance');
  const configSections = useMemo(
    () =>
      CONFIG_CENTER_SECTIONS.map((section) => ({
        ...section,
        label:
          section.id === 'appearance'
            ? t('Appearance')
            : section.id === 'navigation'
              ? t('Navigation')
              : t('Workspace'),
        description:
          section.id === 'appearance'
            ? t('Theme mode and Theme color')
            : section.id === 'navigation'
              ? t('Sidebar behavior and Sidebar navigation')
              : t('Language and workspace preferences'),
      })),
    [t],
  );

  const filteredSections = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();

    if (!normalizedQuery) {
      return configSections;
    }

    return configSections.filter((section) =>
      `${section.id} ${section.label} ${section.description}`
        .toLowerCase()
        .includes(normalizedQuery),
    );
  }, [configSections, searchQuery]);

  useEffect(() => {
    if (!filteredSections.some((section) => section.id === activeSection)) {
      setActiveSection(filteredSections[0]?.id ?? 'appearance');
    }
  }, [activeSection, filteredSections]);

  const workspaceName = workspace?.project.name ?? t('Portal workspace');
  const workspaceEmail = workspace?.user.email ?? t('Awaiting workspace session');
  const tenantName = workspace?.tenant.name ?? t('Portal tenant');
  const operatorName = workspace?.user.display_name ?? t('Portal operator');

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent className="max-h-[calc(100dvh-2rem)] w-[min(1180px,calc(100%-2rem))] overflow-hidden border-zinc-200/80 bg-white/95 p-0 shadow-[0_32px_80px_rgba(15,23,42,0.18)] dark:border-zinc-800/80 dark:bg-zinc-950/92">
        <div className="sr-only">
          <DialogTitle>{t('Settings')}</DialogTitle>
          <DialogDescription>{t('Portal workspace settings')}</DialogDescription>
        </div>

        <div className="flex h-full min-h-[760px] bg-zinc-50/50 dark:bg-zinc-950/50">
          <div className="flex w-72 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50/80 backdrop-blur-xl dark:border-zinc-800 dark:bg-zinc-900/80">
            <div className="p-6 pb-4">
              <h1 className="mb-6 text-2xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
                {t('Settings')}
              </h1>
              <SearchInput
                placeholder={t('Search settings...')}
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                inputClassName="h-10 pr-4 text-[13px]"
              />
            </div>

            <nav className="scrollbar-hide flex-1 space-y-1.5 overflow-y-auto px-4 pb-6">
              {filteredSections.length ? (
                filteredSections.map((section) => (
                  <PortalSettingsNavButton
                    key={section.id}
                    active={activeSection === section.id}
                    icon={section.icon}
                    label={section.label}
                    onClick={() => setActiveSection(section.id)}
                  />
                ))
              ) : (
                <div className="px-3 py-4 text-center text-sm text-zinc-500 dark:text-zinc-400">
                  {t('No settings found.')}
                </div>
              )}
            </nav>
          </div>

          <div className="scrollbar-hide flex-1 overflow-x-hidden overflow-y-auto">
            <div className="mx-auto w-full max-w-5xl p-8 md:p-12">
              <motion.div
                animate={{ opacity: 1, y: 0 }}
                initial={{ opacity: 0, y: 8 }}
                key={activeSection}
                className="w-full"
                transition={{ duration: 0.18, ease: 'easeOut' }}
              >
                {activeSection === 'appearance' ? (
                  <motion.section
                    animate={{ opacity: 1, y: 0 }}
                    className="space-y-6"
                    initial={{ opacity: 0, y: 8 }}
                    transition={{ duration: 0.18, ease: 'easeOut' }}
                  >
                    <PortalSettingsPanelCard
                      title={t('Theme mode')}
                      description={t('Theme mode stays synchronized across header, sidebar, content surfaces, and dialogs.')}
                    >
                      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                        {PORTAL_THEME_MODE_OPTIONS.map((option) => {
                          const Icon = THEME_MODE_ICONS[option.id];
                          return (
                            <PortalThemeModeChoiceCard
                              key={option.id}
                              active={themeMode === option.id}
                              icon={Icon}
                              label={t(option.labelKey)}
                              onClick={() => setThemeMode(option.id)}
                            />
                          );
                        })}
                      </div>
                    </PortalSettingsPanelCard>

                    <PortalSettingsPanelCard
                      title={t('Theme color')}
                      description={t('Theme color updates the accent surfaces without changing the claw-style shell contract.')}
                    >
                      <div className="flex flex-wrap gap-4">
                        {PORTAL_THEME_COLOR_OPTIONS.map((option) => (
                          <PortalThemeColorSwatch
                            key={option.id}
                            active={themeColor === option.id}
                            color={option.id}
                            label={t(option.labelKey)}
                            onClick={() => setThemeColor(option.id)}
                            previewClassName={option.previewClassName}
                          />
                        ))}
                      </div>
                    </PortalSettingsPanelCard>
                  </motion.section>
                ) : null}

                {activeSection === 'navigation' ? (
                  <motion.section
                    animate={{ opacity: 1, y: 0 }}
                    className="space-y-6"
                    initial={{ opacity: 0, y: 8 }}
                    transition={{ duration: 0.18, ease: 'easeOut' }}
                  >
                    <PortalSettingsPanelCard
                      title={t('Sidebar behavior')}
                      description={t('Keep the left rail aligned with claw-studio while preserving the portal route set.')}
                    >
                      <div className="flex flex-wrap gap-3">
                        <PortalSettingsActionButton
                          onClick={() => setSidebarCollapsed(!isSidebarCollapsed)}
                          icon={PanelLeft}
                          label={
                            isSidebarCollapsed ? t('Expand sidebar') : t('Collapse sidebar')
                          }
                        />
                        <PortalSettingsActionButton
                          emphasis="subtle"
                          onClick={resetShellPreferences}
                          icon={RotateCcw}
                          label={t('Reset shell preferences')}
                        />
                      </div>
                    </PortalSettingsPanelCard>

                    <PortalSettingsPanelCard
                      title={t('Navigation')}
                      description={t('Show or hide workspace modules while keeping the left rail compact and stable.')}
                    >
                      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                        {portalSidebarRoutes.map((route) => {
                          const visible = !hiddenSidebarItems.includes(route.key);

                          return (
                            <PortalSettingsToggleRow
                              key={route.key}
                              checked={visible}
                              label={t(route.labelKey)}
                              description={t(route.detailKey)}
                              onCheckedChange={() => toggleSidebarItem(route.key)}
                            />
                          );
                        })}
                      </div>
                    </PortalSettingsPanelCard>
                  </motion.section>
                ) : null}

                {activeSection === 'workspace' ? (
                  <motion.section
                    animate={{ opacity: 1, y: 0 }}
                    className="space-y-6"
                    initial={{ opacity: 0, y: 8 }}
                    transition={{ duration: 0.18, ease: 'easeOut' }}
                  >
                    <PortalSettingsPanelCard
                      title={t('Language and locale')}
                      description={t('Choose the portal workspace language. Shared shell copy and locale-aware formatting update immediately.')}
                    >
                      <div className="grid gap-4 md:grid-cols-2">
                        <SettingsField label={t('Language')} layout="vertical">
                          <Select
                            value={locale}
                            onValueChange={(value) => setLocale(value as typeof locale)}
                          >
                            <SelectTrigger>
                              <SelectValue placeholder={t('Language')} />
                            </SelectTrigger>
                            <SelectContent>
                              {PORTAL_LOCALE_OPTIONS.map((option) => (
                                <SelectItem key={option.id} value={option.id}>
                                  {t(option.labelKey)}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </SettingsField>
                      </div>
                    </PortalSettingsPanelCard>

                    <PortalSettingsPanelCard
                      title={t('Workspace preferences')}
                      description={t('Keep workspace identity and shell reset controls in one place.')}
                    >
                      <div className="grid gap-4 md:grid-cols-2">
                        <PortalSettingsIdentityCard
                          eyebrow={t('Workspace')}
                          title={workspaceName}
                          description={tenantName}
                        />
                        <PortalSettingsIdentityCard
                          eyebrow={t('Operator')}
                          title={operatorName}
                          description={workspaceEmail}
                        />
                      </div>

                      <div className="mt-5 flex flex-wrap gap-3">
                        <PortalSettingsActionButton
                          emphasis="primary"
                          onClick={resetShellPreferences}
                          label={t('Reset shell preferences')}
                        />
                      </div>
                    </PortalSettingsPanelCard>
                  </motion.section>
                ) : null}
              </motion.div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
