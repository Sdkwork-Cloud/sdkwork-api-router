import { useMemo } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  PortalSiteHero,
  PortalSiteMetricCard,
  PortalSitePanel,
} from 'sdkwork-router-portal-commons/framework/site';

import { portalDocsRegistry } from './registry';

export function PortalDocsPage() {
  const { t } = usePortalI18n();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const defaultGroupId = portalDocsRegistry[0]?.id ?? 'quickstart';
  const requestedGroupId = searchParams.get('group');
  const activeGroupId = useMemo(() => {
    if (!requestedGroupId) {
      return defaultGroupId;
    }

    return portalDocsRegistry.some((group) => group.id === requestedGroupId)
      ? requestedGroupId
      : defaultGroupId;
  }, [defaultGroupId, requestedGroupId]);
  const activeGroup =
    portalDocsRegistry.find((group) => group.id === activeGroupId) ?? portalDocsRegistry[0];
  const totalEntryCount = useMemo(
    () => portalDocsRegistry.reduce((count, group) => count + group.entries.length, 0),
    [],
  );
  const totalActionCount = portalDocsRegistry.length * 2;
  const metricCards = useMemo(
    () => [
      {
        label: 'Guide groups',
        value: String(portalDocsRegistry.length),
        description:
          'Documentation modules remain grouped by implementation stage instead of splitting into disconnected help surfaces.',
      },
      {
        label: 'Launch steps',
        value: String(totalEntryCount),
        description:
          'Each registry entry keeps onboarding, integration, and operations work visible in one documentation center.',
      },
      {
        label: 'Route-aware actions',
        value: String(totalActionCount),
        description:
          'Every guide stays connected to the next real product destination, from downloads and models to the console.',
      },
    ],
    [totalActionCount, totalEntryCount],
  );

  function selectGroup(groupId: string) {
    const nextParams = new URLSearchParams(searchParams);
    if (groupId === defaultGroupId) {
      nextParams.delete('group');
    } else {
      nextParams.set('group', groupId);
    }
    setSearchParams(nextParams);
  }

  return (
    <div className="space-y-6" data-slot="portal-docs-page">
      <PortalSiteHero
        actions={(
          <>
            <Button type="button" onClick={() => selectGroup('quickstart')}>
              {t('Open quickstart')}
            </Button>
            <Button type="button" onClick={() => navigate('/models')} variant="secondary">
              {t('Explore models')}
            </Button>
            <Button type="button" onClick={() => navigate('/console')} variant="ghost">
              {t('Open console')}
            </Button>
          </>
        )}
        aside={(
          <PortalSitePanel
            className="rounded-[28px] border-zinc-200/80 bg-zinc-50/80 dark:border-zinc-800 dark:bg-zinc-900/60"
            description={t('Choose a guide family based on launch stage, integration depth, and operational ownership.')}
            title={t('Documentation tracks')}
          >
            {portalDocsRegistry.map((group, index) => (
              <button
                key={group.id}
                type="button"
                onClick={() => selectGroup(group.id)}
                className={`w-full rounded-2xl border px-4 py-3 text-left transition-colors ${
                  group.id === activeGroupId
                    ? 'border-zinc-950 bg-zinc-950 text-white dark:border-white dark:bg-white dark:text-zinc-950'
                    : 'border-zinc-200 bg-white text-zinc-950 hover:bg-zinc-100 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-50 dark:hover:bg-zinc-900'
                }`}
              >
                <div className="text-xs font-semibold uppercase tracking-[0.18em] opacity-70">
                  {t('Step {index}', { index: index + 1 })}
                </div>
                <div className="mt-1 text-sm font-semibold">{t(group.title)}</div>
                <div className="mt-1 text-sm leading-6 opacity-80">{t(group.description)}</div>
              </button>
            ))}
          </PortalSitePanel>
        )}
        description={t('Quickstart, integration, reference, and operations guidance stay connected to the same product flows used across models, downloads, and the console.')}
        eyebrow={t('Documentation center')}
        title={t('Move from evaluation to implementation with one route-aware documentation center.')}
      />

      <section className="grid gap-4 md:grid-cols-3" data-slot="portal-docs-metrics">
        {metricCards.map((item) => (
          <PortalSiteMetricCard
            key={item.label}
            description={t(item.description)}
            label={t(item.label)}
            value={item.value}
          />
        ))}
      </section>

      <div className="grid gap-4 xl:grid-cols-[minmax(18rem,0.74fr)_minmax(0,1.26fr)]">
        <PortalSitePanel
          description={t('A registry-driven docs module keeps quickstart, integration, reference, and operations guidance separate from the console workspace.')}
          title={t('Documentation center')}
        >
          {portalDocsRegistry.map((group) => (
            <button
              key={group.id}
              type="button"
              onClick={() => selectGroup(group.id)}
              className={`w-full rounded-[24px] border px-5 py-4 text-left transition-colors ${
                group.id === activeGroupId
                  ? 'border-zinc-950 bg-zinc-950 text-white dark:border-white dark:bg-white dark:text-zinc-950'
                  : 'border-zinc-200 bg-zinc-50/80 text-zinc-950 hover:bg-zinc-100 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-50 dark:hover:bg-zinc-900'
              }`}
            >
              <div className="text-sm font-semibold">{t(group.title)}</div>
              <div className="mt-1 text-sm leading-6 opacity-80">{t(group.description)}</div>
            </button>
          ))}
        </PortalSitePanel>

        <div className="grid gap-4">
          <PortalSitePanel
            description={t(activeGroup.description)}
            title={t(activeGroup.title)}
          >
            <div className="rounded-[24px] border border-dashed border-zinc-200 bg-zinc-50/80 px-5 py-4 text-sm leading-6 text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300">
              {t('The selected docs group stays in the URL so operators can reopen the same guide with one shared link.')}
            </div>

            {activeGroup.entries.map((entry, index) => (
              <div
                key={entry.id}
                className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 px-5 py-4 dark:border-zinc-800 dark:bg-zinc-900/60"
              >
                <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  {t('Step {index}', { index: index + 1 })}
                </div>
                <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t(entry.title)}
                </div>
                <div className="mt-1 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(entry.detail)}
                </div>
              </div>
            ))}

            <div className="flex flex-wrap gap-3">
              <Button type="button" onClick={() => navigate(activeGroup.primaryAction.href)}>
                {t(activeGroup.primaryAction.label)}
              </Button>
              <Button type="button" onClick={() => navigate(activeGroup.secondaryAction.href)} variant="secondary">
                {t(activeGroup.secondaryAction.label)}
              </Button>
            </div>
          </PortalSitePanel>

          <div data-slot="portal-docs-operating-loop">
            <PortalSitePanel
              description={t('The selected documentation group keeps its next actions and operational outcomes visible in one place.')}
              title={t('Operating loop')}
            >
              <div className="grid gap-4 md:grid-cols-2">
                <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
                  <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t(activeGroup.primaryAction.label)}
                  </div>
                  <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {activeGroup.primaryAction.href}
                  </div>
                  <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {t(activeGroup.description)}
                  </div>
                </div>

                <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
                  <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t(activeGroup.secondaryAction.label)}
                  </div>
                  <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {activeGroup.secondaryAction.href}
                  </div>
                  <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {t('The selected docs group stays in the URL so operators can reopen the same guide with one shared link.')}
                  </div>
                </div>
              </div>

              <div className="grid gap-3 md:grid-cols-2">
                {activeGroup.entries.map((entry, index) => (
                  <div
                    key={`operating-${entry.id}`}
                    className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950"
                  >
                    <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                      {t('Step {index}', { index: index + 1 })}
                    </div>
                    <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                      {t(entry.title)}
                    </div>
                    <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                      {t(entry.detail)}
                    </div>
                  </div>
                ))}
              </div>
            </PortalSitePanel>
          </div>
        </div>
      </div>

      <div data-slot="portal-docs-implementation-lanes">
        <PortalSitePanel
          description={t('Move through onboarding, integration, reference lookup, and operations review without leaving the documentation surface.')}
          title={t('Implementation lanes')}
        >
          <div className="grid gap-4 xl:grid-cols-4">
            {portalDocsRegistry.map((group) => (
              <div
                key={`lane-${group.id}`}
                className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
              >
                <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {t(group.title)}
                </div>
                <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(group.description)}
                </div>
                <div className="mt-4 grid gap-2">
                  {group.entries.map((entry) => (
                    <div
                      key={`lane-entry-${entry.id}`}
                      className="rounded-2xl border border-zinc-200 bg-white px-3 py-2 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300"
                    >
                      {t(entry.title)}
                    </div>
                  ))}
                </div>
                <div className="mt-4 flex flex-wrap gap-2">
                  <Button type="button" onClick={() => navigate(group.primaryAction.href)}>
                    {t(group.primaryAction.label)}
                  </Button>
                  <Button
                    type="button"
                    onClick={() => navigate(group.secondaryAction.href)}
                    variant="secondary"
                  >
                    {t(group.secondaryAction.label)}
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </PortalSitePanel>
      </div>
    </div>
  );
}
