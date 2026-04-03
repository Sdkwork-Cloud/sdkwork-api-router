import {
  startTransition,
  useDeferredValue,
  useEffect,
  useMemo,
  useState,
} from 'react';
import {
  LayoutPanelLeft,
  Monitor,
  PanelsTopLeft,
  ShieldCheck,
} from 'lucide-react';
import { useSearchParams } from 'react-router-dom';
import {
  Badge,
  Button,
  InlineAlert,
  SettingsCenter,
  type SettingsCenterSection,
} from '@sdkwork/ui-pc-react';

import { useAdminI18n } from 'sdkwork-router-admin-core';

import { AppearanceSettings } from './AppearanceSettings';
import { GeneralSettings } from './GeneralSettings';
import { NavigationSettings } from './NavigationSettings';
import { SettingsBadge } from './Shared';
import { WorkspaceSettings } from './WorkspaceSettings';

type SettingsTab = 'general' | 'appearance' | 'navigation' | 'workspace';

type SettingsItemDefinition = {
  description: string;
  group: string;
  icon: typeof ShieldCheck;
  id: SettingsTab;
  keywords: string[];
  label: string;
};

function resolveTab(requestedTab: string | null): SettingsTab {
  if (
    requestedTab === 'general'
    || requestedTab === 'appearance'
    || requestedTab === 'navigation'
    || requestedTab === 'workspace'
  ) {
    return requestedTab;
  }

  return 'general';
}

function itemMatchesQuery(item: SettingsItemDefinition, query: string) {
  if (!query) {
    return true;
  }

  return [item.label, item.description, item.group, ...item.keywords]
    .join(' ')
    .toLowerCase()
    .includes(query);
}

export function SettingsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [search, setSearch] = useState('');
  const { t } = useAdminI18n();
  const activeTab = resolveTab(searchParams.get('tab'));
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const items = useMemo<SettingsItemDefinition[]>(
    () => [
      {
        id: 'general',
        label: t('General'),
        description: t('Operator identity, locale, and shell posture summary'),
        icon: ShieldCheck,
        group: t('Workspace'),
        keywords: ['workspace', 'operator', 'language', 'summary'],
      },
      {
        id: 'appearance',
        label: t('Appearance'),
        description: t('Theme mode, accent preset, and shared shell look'),
        icon: Monitor,
        group: t('Shell'),
        keywords: ['theme', 'color', 'mode', 'appearance'],
      },
      {
        id: 'navigation',
        label: t('Navigation'),
        description: t('Sidebar visibility, rail behavior, and module exposure'),
        icon: LayoutPanelLeft,
        group: t('Shell'),
        keywords: ['sidebar', 'navigation', 'routes', 'rail'],
      },
      {
        id: 'workspace',
        label: t('Workspace'),
        description: t('Persistence, content region, and shell continuity'),
        icon: PanelsTopLeft,
        group: t('Workspace'),
        keywords: ['workspace', 'persistence', 'canvas', 'continuity'],
      },
    ],
    [t],
  );

  const visibleItemIds = useMemo(
    () =>
      items
        .filter((item) => itemMatchesQuery(item, deferredSearch))
        .map((item) => item.id),
    [deferredSearch, items],
  );

  useEffect(() => {
    if (!visibleItemIds.length || visibleItemIds.includes(activeTab)) {
      return;
    }

    const nextSearchParams = new URLSearchParams(searchParams);
    nextSearchParams.set('tab', visibleItemIds[0]);
    startTransition(() => {
      setSearchParams(nextSearchParams, { replace: true });
    });
  }, [activeTab, searchParams, setSearchParams, visibleItemIds]);

  const sections = useMemo<SettingsCenterSection[]>(
    () => [
      {
        title: t('Control plane'),
        items: items
          .filter((item) => item.group === t('Workspace'))
          .map((item) => ({
            id: item.id,
            label: item.label,
            description: item.description,
            keywords: item.keywords,
            icon: <item.icon className="h-4 w-4" />,
            badge:
              item.id === activeTab ? (
                <SettingsBadge variant="secondary">{t('Live')}</SettingsBadge>
              ) : undefined,
          })),
      },
      {
        title: t('Shell'),
        items: items
          .filter((item) => item.group === t('Shell'))
          .map((item) => ({
            id: item.id,
            label: item.label,
            description: item.description,
            keywords: item.keywords,
            icon: <item.icon className="h-4 w-4" />,
          })),
      },
    ],
    [activeTab, items, t],
  );

  const renderActivePanel = () => {
    switch (activeTab) {
      case 'appearance':
        return <AppearanceSettings />;
      case 'navigation':
        return <NavigationSettings />;
      case 'workspace':
        return <WorkspaceSettings />;
      case 'general':
      default:
        return <GeneralSettings />;
    }
  };

  return (
    <SettingsCenter
      actions={
        <div className="flex items-center gap-2">
          <SettingsBadge variant="secondary">{t('settings center')}</SettingsBadge>
          {search ? (
            <Button onClick={() => setSearch('')} type="button" variant="ghost">
              {t('Clear filters')}
            </Button>
          ) : null}
        </div>
      }
      activeItem={activeTab}
      description={t(
        'This workspace keeps operator preferences, shell posture, and control plane continuity aligned with claw-studio while preserving router-admin workflows.',
      )}
      emptyState={
        <InlineAlert
          description={t('Try a different keyword or browse the navigation without a search term.')}
          showIcon
          title={t('No settings match your search')}
          tone="warning"
        />
      }
      navFooter={
        <InlineAlert
          description={t(
            'The left rail remains the navigation source of truth and the right canvas remains the only content display region for every admin page.',
          )}
          showIcon
          title={t('Shell continuity')}
          tone="info"
        />
      }
      navHeader={
        <div className="space-y-3">
          <SettingsBadge variant="outline">{t('Control plane')}</SettingsBadge>
          <div className="space-y-1">
            <div className="text-base font-semibold text-[var(--sdk-color-text-primary)]">
              {t('control plane settings center')}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {t('Search and switch settings without leaving the shared desktop shell.')}
            </div>
          </div>
        </div>
      }
      onActiveItemChange={(itemId) => {
        const nextSearchParams = new URLSearchParams(searchParams);
        nextSearchParams.set('tab', itemId);
        startTransition(() => {
          setSearchParams(nextSearchParams, { replace: true });
        });
      }}
      onSearchChange={setSearch}
      searchPlaceholder={t('Search settings')}
      searchValue={search}
      sections={sections}
      title={t('Settings center')}
    >
      {renderActivePanel()}
    </SettingsCenter>
  );
}
