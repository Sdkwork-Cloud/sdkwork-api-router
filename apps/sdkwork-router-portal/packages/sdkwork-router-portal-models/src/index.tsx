import { startTransition, useDeferredValue, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import { DataTable } from 'sdkwork-router-portal-commons/framework/display';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from 'sdkwork-router-portal-commons/framework/entry';
import {
  FilterBar,
  FilterBarActions,
  FilterBarSection,
  FilterField,
  SearchInput,
} from 'sdkwork-router-portal-commons/framework/form';
import {
  PortalSiteHero,
  PortalSiteMetricCard,
  PortalSitePanel,
} from 'sdkwork-router-portal-commons/framework/site';

import { portalModelCatalog } from './catalog';

const FEATURED_MODEL_ID = 'qwen-max';

const selectionTrackDefinitions = [
  { title: 'Best for global assistants', modelId: 'gemini-2-5-pro' },
  { title: 'Best for Chinese knowledge', modelId: 'ernie-4-5' },
  { title: 'Best for coding agents', modelId: 'claude-3-7-sonnet' },
];

export function PortalModelsPage() {
  const { t } = usePortalI18n();
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState('');
  const [providerFilter, setProviderFilter] = useState('all');
  const [modalityFilter, setModalityFilter] = useState('all');
  const deferredSearchQuery = useDeferredValue(searchQuery);

  const providerOptions = useMemo(
    () => ['all', ...new Set(portalModelCatalog.map((model) => model.provider))],
    [],
  );
  const modalityOptions = useMemo(
    () => ['all', ...new Set(portalModelCatalog.flatMap((model) => model.modalities))],
    [],
  );
  const filteredModels = useMemo(() => {
    const normalizedSearch = deferredSearchQuery.trim().toLowerCase();

    return portalModelCatalog.filter((model) => {
      const translatedCapability = t(model.capability);
      const translatedSummary = t(model.summary);
      const translatedModalities = model.modalities.map((modality) => t(modality));
      const matchesProvider = providerFilter === 'all' || model.provider === providerFilter;
      const matchesModality =
        modalityFilter === 'all' || model.modalities.includes(modalityFilter);
      const matchesSearch =
        normalizedSearch.length === 0
        || [
          model.name,
          model.provider,
          model.capability,
          model.summary,
          translatedCapability,
          translatedSummary,
          ...model.modalities,
          ...translatedModalities,
        ]
          .join(' ')
          .toLowerCase()
          .includes(normalizedSearch);

      return matchesProvider && matchesModality && matchesSearch;
    });
  }, [deferredSearchQuery, modalityFilter, providerFilter, t]);
  const visibleModels = filteredModels;
  const multimodalCount = visibleModels.filter((model) => model.modalities.length > 1).length;
  const providerCount = new Set(visibleModels.map((model) => model.provider)).size;
  const hasActiveFilters =
    searchQuery.trim().length > 0 || providerFilter !== 'all' || modalityFilter !== 'all';
  const featuredModel = useMemo(
    () =>
      portalModelCatalog.find((model) => model.id === FEATURED_MODEL_ID)
      ?? portalModelCatalog[0],
    [],
  );
  const productPanelModels = visibleModels.length > 0 ? visibleModels : portalModelCatalog;
  const providerLanes = useMemo(() => {
    const groups = new Map<
      string,
      {
        provider: string;
        models: typeof portalModelCatalog;
        modalities: Set<string>;
      }
    >();

    for (const model of productPanelModels) {
      const existing = groups.get(model.provider);

      if (existing) {
        existing.models.push(model);
        for (const modality of model.modalities) {
          existing.modalities.add(modality);
        }
        continue;
      }

      groups.set(model.provider, {
        provider: model.provider,
        models: [model],
        modalities: new Set(model.modalities),
      });
    }

    return [...groups.values()]
      .map((lane) => ({
        provider: lane.provider,
        count: lane.models.length,
        anchor:
          [...lane.models].sort((left, right) => right.modalities.length - left.modalities.length)[0]
          ?? lane.models[0],
        modalities: [...lane.modalities].sort(),
      }))
      .sort((left, right) => right.count - left.count || left.provider.localeCompare(right.provider));
  }, [productPanelModels]);
  const selectionTracks = useMemo(
    () =>
      selectionTrackDefinitions.map((track) => ({
        ...track,
        model:
          productPanelModels.find((model) => model.id === track.modelId)
          ?? portalModelCatalog.find((model) => model.id === track.modelId)
          ?? productPanelModels[0]
          ?? portalModelCatalog[0],
      })),
    [productPanelModels],
  );

  function clearFilters() {
    startTransition(() => {
      setSearchQuery('');
      setProviderFilter('all');
      setModalityFilter('all');
    });
  }

  return (
    <div className="space-y-6" data-slot="portal-models-page">
      <PortalSiteHero
        actions={(
          <>
            <Button type="button" onClick={() => navigate('/docs')}>
              {t('Read docs')}
            </Button>
            <Button type="button" onClick={() => navigate('/console')} variant="secondary">
              {t('Enter console')}
            </Button>
          </>
        )}
        aside={(
          <div data-slot="portal-models-featured">
            <PortalSitePanel
              className="rounded-[28px] border-zinc-200/80 bg-zinc-50/80 dark:border-zinc-800 dark:bg-zinc-900/60"
              description={t(featuredModel.capability)}
              title={t('Featured model')}
            >
              <div className="rounded-[24px] border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
                <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  {featuredModel.provider}
                </div>
                <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                  {featuredModel.name}
                </div>
                <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(featuredModel.summary)}
                </div>
              </div>

              <div className="grid gap-3 sm:grid-cols-2">
                <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t('Context window')}
                  </div>
                  <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {featuredModel.contextWindow}
                  </div>
                </div>

                <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t('Pricing')}
                  </div>
                  <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {featuredModel.price}
                  </div>
                </div>

                <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t('Modality')}
                  </div>
                  <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {featuredModel.modalities.map((modality) => t(modality)).join(' / ')}
                  </div>
                </div>

                <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t('Latency')}
                  </div>
                  <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {t(featuredModel.latencyClass)}
                  </div>
                </div>
              </div>
            </PortalSitePanel>
          </div>
        )}
        description={t('Search, compare, and shortlist models with the same product language used throughout the public site and console.')}
        eyebrow={t('Model center')}
        title={t('Compare multimodal providers, context posture, pricing bands, and launch fit before routing traffic.')}
      />

      <div className="grid gap-4 md:grid-cols-3">
        <PortalSiteMetricCard
          label={t('Providers')}
          value={String(providerCount)}
          description={t('Global and domestic providers visible in the current filtered catalog.')}
        />
        <PortalSiteMetricCard
          label={t('Multimodal models')}
          value={String(multimodalCount)}
          description={t('Models that simultaneously cover text, vision, audio, or other paired modalities.')}
        />
        <PortalSiteMetricCard
          label={t('Catalog size')}
          value={String(visibleModels.length)}
          description={t('Search and filters reduce the visible model set without leaving the page.')}
        />
      </div>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.08fr)_minmax(0,0.92fr)]">
        <div data-slot="portal-models-provider-lanes">
          <PortalSitePanel
            description={t('Provider coverage, modality range, and catalog density stay visible while filters narrow the shortlist.')}
            title={t('Provider lanes')}
          >
            <div className="grid gap-4 md:grid-cols-2">
              {providerLanes.map((lane) => (
                <div
                  key={lane.provider}
                  className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Provider')}
                      </div>
                      <div className="mt-1 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {lane.provider}
                      </div>
                    </div>

                    <div className="rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300">
                      {t('Catalog size')}: {lane.count}
                    </div>
                  </div>

                  <div className="mt-4 grid gap-3">
                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Featured model')}
                      </div>
                      <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                        {lane.anchor.name}
                      </div>
                      <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                        {t(lane.anchor.summary)}
                      </div>
                    </div>

                    <div className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                      <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Modality')}
                      </div>
                      <div className="mt-2 text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                        {lane.modalities.map((modality) => t(modality)).join(' / ')}
                      </div>
                      <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                        {t(lane.anchor.capability)}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </PortalSitePanel>
        </div>

        <div data-slot="portal-models-selection-tracks">
          <PortalSitePanel
            description={t('Recommended starting points for agentic workflows, enterprise assistants, search stacks, and audio pipelines.')}
            title={t('Selection tracks')}
          >
            <div className="grid gap-4">
              {selectionTracks.map((track) => (
                <div
                  key={track.title}
                  className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                        {t(track.title)}
                      </div>
                      <div className="mt-1 text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {track.model.provider}
                      </div>
                    </div>

                    <div className="rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300">
                      {track.model.contextWindow}
                    </div>
                  </div>

                  <div className="mt-3 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {track.model.name}
                  </div>
                  <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {t(track.model.summary)}
                  </div>

                  <div className="mt-4 flex flex-wrap gap-2">
                    {track.model.modalities.map((modality) => (
                      <span
                        key={`${track.title}-${modality}`}
                        className="inline-flex items-center rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300"
                      >
                        {t(modality)}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </PortalSitePanel>
        </div>
      </div>

      <PortalSitePanel
        description={t('Compare provider coverage, modality mix, context window, and operating posture in one searchable catalog.')}
        title={t('Model center')}
      >
        <FilterBar data-slot="portal-models-filter-bar" wrap={false}>
          <FilterBarSection className="min-w-[16rem] flex-[1_1_18rem]" grow={false} wrap={false}>
            <SearchInput
              inputClassName="h-11"
              placeholder={t('Search models, providers, capabilities, or modalities')}
              value={searchQuery}
              onChange={(event) =>
                startTransition(() => {
                  setSearchQuery(event.target.value);
                })
              }
            />
          </FilterBarSection>

          <FilterBarSection className="min-w-[12rem] shrink-0" grow={false} wrap={false}>
            <FilterField className="w-full" label={t('Provider')}>
              <Select value={providerFilter} onValueChange={setProviderFilter}>
                <SelectTrigger>
                  <SelectValue placeholder={t('Provider')} />
                </SelectTrigger>
                <SelectContent>
                  {providerOptions.map((option) => (
                    <SelectItem key={option} value={option}>
                      {option === 'all' ? t('All providers') : option}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FilterField>
          </FilterBarSection>

          <FilterBarSection className="min-w-[12rem] shrink-0" grow={false} wrap={false}>
            <FilterField className="w-full" label={t('Modality')}>
              <Select value={modalityFilter} onValueChange={setModalityFilter}>
                <SelectTrigger>
                  <SelectValue placeholder={t('Modality')} />
                </SelectTrigger>
                <SelectContent>
                  {modalityOptions.map((option) => (
                    <SelectItem key={option} value={option}>
                      {option === 'all' ? t('All modalities') : t(option)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FilterField>
          </FilterBarSection>

          <FilterBarActions wrap={false}>
            <Button disabled={!hasActiveFilters} onClick={clearFilters} variant="secondary">
              {t('Clear filters')}
            </Button>
          </FilterBarActions>
        </FilterBar>

        <DataTable
          className="rounded-[28px] border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950"
          columns={[
            {
              id: 'model',
              header: t('Model'),
              cell: (row) => (
                <div className="min-w-0">
                  <div className="font-semibold text-zinc-950 dark:text-zinc-50">{row.name}</div>
                  <div className="text-sm text-zinc-500 dark:text-zinc-400">{t(row.summary)}</div>
                </div>
              ),
            },
            { id: 'provider', header: t('Provider'), cell: (row) => row.provider },
            {
              id: 'modalities',
              header: t('Modality'),
              cell: (row) => row.modalities.map((modality: string) => t(modality)).join(' / '),
            },
            { id: 'capability', header: t('Capability'), cell: (row) => t(row.capability) },
            { id: 'context', header: t('Context window'), cell: (row) => row.contextWindow },
            { id: 'price', header: t('Pricing'), cell: (row) => row.price },
            { id: 'latency', header: t('Latency'), cell: (row) => t(row.latencyClass) },
          ]}
          emptyState={(
            <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
              <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                {t('No models match the current filter')}
              </strong>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">
                {t('Try broadening the provider, modality, or search criteria to compare more catalog entries.')}
              </p>
            </div>
          )}
          footer={(
            <div className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/50 dark:text-zinc-300">
              {t('{count} models visible in the current model center view.', {
                count: visibleModels.length,
              })}
            </div>
          )}
          rows={visibleModels}
        />
      </PortalSitePanel>
    </div>
  );
}
