import { useNavigate } from 'react-router-dom';
import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  PortalSiteHero,
  PortalSiteMetricCard,
  PortalSitePanel,
} from 'sdkwork-router-portal-commons/framework/site';

const primaryLinks = [
  {
    href: '/console/dashboard',
    routeLabel: 'Console',
    title: 'Start with product posture',
    detail: 'Enter the operational workspace and manage runtime posture.',
    actionLabel: 'Enter console',
    variant: 'primary' as const,
  },
  {
    href: '/models',
    routeLabel: 'Models',
    title: 'Map the model layer',
    detail: 'Browse multimodal providers, capabilities, and deployment options.',
    actionLabel: 'Explore models',
    variant: 'secondary' as const,
  },
  {
    href: '/docs',
    routeLabel: 'Documentation center',
    title: 'Follow implementation guides',
    detail: 'Read the integration guides, quickstarts, and API references.',
    actionLabel: 'Read docs',
    variant: 'secondary' as const,
  },
  {
    href: '/downloads',
    routeLabel: 'Software Downloads',
    title: 'Install and launch runtime',
    detail: 'Install the desktop runtime and tooling packages.',
    actionLabel: 'Download software',
    variant: 'secondary' as const,
  },
];

const valuePillars = [
  {
    title: 'Business-ready surfaces',
    detail: 'Home, models, docs, and software downloads share one navigation contract and one visual language.',
    tags: ['Home', 'Models', 'Docs', 'Software Downloads'],
  },
  {
    title: 'Operator-first onboarding',
    detail: 'Every CTA moves toward a real console action, install flow, or implementation guide.',
    tags: ['Enter console', 'Read docs', 'Download software'],
  },
  {
    title: 'Launch without context loss',
    detail: 'Teams can evaluate the platform, compare models, and start the runtime without jumping between products.',
    tags: ['Console', 'Models', 'Software Downloads'],
  },
];

const launchTracks = [
  {
    title: 'Platform teams',
    detail: 'Compare platform capabilities, public navigation, and rollout paths before entering the console.',
    actionLabel: 'Enter console',
    href: '/console/dashboard',
    tags: ['Gateway', 'Launch posture', 'Console'],
    variant: 'primary' as const,
  },
  {
    title: 'Application teams',
    detail: 'Review models, SDK guidance, and API workflows before issuing credentials or integrating clients.',
    actionLabel: 'Read docs',
    href: '/docs',
    tags: ['Models', 'Docs', 'API Reference'],
    variant: 'secondary' as const,
  },
  {
    title: 'Operations teams',
    detail: 'Choose the correct desktop or service mode, verify requirements, and start the product with guided onboarding.',
    actionLabel: 'Download software',
    href: '/downloads',
    tags: ['Software Downloads', 'System requirements', 'Open quickstart'],
    variant: 'secondary' as const,
  },
];

const metricCards = [
  {
    label: 'Public modules',
    value: String(primaryLinks.length),
    description:
      'Four public surfaces stay aligned with one console contract and one visual language.',
  },
  {
    label: 'Connected pathways',
    value: String(primaryLinks.length),
    description:
      'Four route-level entry points carry evaluation, onboarding, and execution without fragmenting the product.',
  },
  {
    label: 'Team launch tracks',
    value: String(launchTracks.length),
    description:
      'Three role-based tracks move platform owners, application teams, and operators toward launch.',
  },
];

export function PortalHomePage() {
  const { t } = usePortalI18n();
  const navigate = useNavigate();

  return (
    <div className="space-y-6" data-slot="portal-home-page">
      <PortalSiteHero
        actions={(
          <>
            <Button onClick={() => navigate('/console/dashboard')}>{t('Enter console')}</Button>
            <Button onClick={() => navigate('/models')} variant="secondary">
              {t('Explore models')}
            </Button>
            <Button onClick={() => navigate('/downloads')} variant="ghost">
              {t('Download software')}
            </Button>
          </>
        )}
        aside={(
          <PortalSitePanel
            className="rounded-[28px] border-zinc-200/80 bg-zinc-50/80 dark:border-zinc-800 dark:bg-zinc-900/60"
            description={t('A single information architecture for evaluation, onboarding, and console operations.')}
            title={t('Launch sequence')}
          >
            {primaryLinks.map((item, index) => (
              <div
                key={item.href}
                className="rounded-2xl border border-zinc-200/80 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950"
              >
                <div className="mb-1 text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  {t('Step {index}', { index: index + 1 })}
                </div>
                <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {t(item.title)}
                </div>
                <div className="mt-1 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(item.detail)}
                </div>
              </div>
            ))}
          </PortalSitePanel>
        )}
        description={t('SDKWork Router Portal now separates the public product experience from the authenticated console so teams can evaluate the platform, compare models, read docs, and install software without losing a cohesive design language.')}
        eyebrow={t('Unified AI gateway workspace')}
        title={t('Operate routing, credentials, usage, and downloads from one product surface.')}
      />

      <section className="grid gap-4 md:grid-cols-3" data-slot="portal-home-metrics">
        {metricCards.map((item) => (
          <PortalSiteMetricCard
            key={item.label}
            description={t(item.description)}
            label={t(item.label)}
            value={item.value}
          />
        ))}
      </section>

      <PortalSitePanel
        description={t('Each pathway keeps discovery, onboarding, and execution connected instead of fragmenting into separate microsites.')}
        title={t('Product pathways')}
      >
        <div className="grid gap-4 xl:grid-cols-4" data-slot="portal-home-pathways">
          {primaryLinks.map((item) => (
            <div
              key={item.href}
              className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
            >
              <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t(item.routeLabel)}
              </div>
              <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                {t(item.title)}
              </div>
              <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                {t(item.detail)}
              </div>
              <div className="mt-4">
                <Button onClick={() => navigate(item.href)} variant={item.variant}>
                  {t(item.actionLabel)}
                </Button>
              </div>
            </div>
          ))}
        </div>
      </PortalSitePanel>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,0.92fr)_minmax(0,1.08fr)]">
        <div className="grid gap-4 md:grid-cols-3 xl:grid-cols-1" data-slot="portal-home-value-grid">
          {valuePillars.map((item) => (
            <PortalSitePanel
              key={item.title}
              className="rounded-[28px]"
              description={t(item.detail)}
              title={t(item.title)}
            >
              <div className="flex flex-wrap gap-2">
                {item.tags.map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300"
                  >
                    {t(tag)}
                  </span>
                ))}
              </div>
            </PortalSitePanel>
          ))}
        </div>

        <PortalSitePanel
          description={t('Choose the fastest path for platform owners, application teams, and operators without changing product context.')}
          title={t('Launch tracks')}
        >
          <div className="grid gap-4" data-slot="portal-home-launch-tracks">
            {launchTracks.map((track) => (
              <div
                key={track.title}
                className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
              >
                <div className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                  {t(track.title)}
                </div>
                <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(track.detail)}
                </div>
                <div className="mt-4 flex flex-wrap gap-2">
                  {track.tags.map((tag) => (
                    <span
                      key={tag}
                      className="inline-flex items-center rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300"
                    >
                      {t(tag)}
                    </span>
                  ))}
                </div>
                <div className="mt-4">
                  <Button onClick={() => navigate(track.href)} variant={track.variant}>
                    {t(track.actionLabel)}
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
