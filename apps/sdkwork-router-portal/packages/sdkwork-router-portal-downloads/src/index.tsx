import { useNavigate } from 'react-router-dom';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  PortalSiteHero,
  PortalSiteMetricCard,
  PortalSitePanel,
} from 'sdkwork-router-portal-commons/framework/site';

const softwareTargets = [
  {
    platform: 'Windows',
    artifact: 'SDKWork Router Desktop',
    detail: 'Best for local gateway runtime, desktop orchestration, and quick QR-based sign in.',
    command: 'pnpm product:start',
    guideGroup: 'quickstart',
    actionLabel: 'Open desktop setup',
  },
  {
    platform: 'macOS',
    artifact: 'SDKWork Router Desktop',
    detail: 'Optimized for Apple Silicon and operator-facing desktop management.',
    command: 'pnpm product:start',
    guideGroup: 'quickstart',
    actionLabel: 'Open desktop setup',
  },
  {
    platform: 'Linux',
    artifact: 'Gateway Service Bundle',
    detail: 'Run the router as a service and pair it with the portal console and public docs.',
    command: 'pnpm server:start',
    guideGroup: 'operations',
    actionLabel: 'Open server setup',
  },
];

const runtimeModes = [
  {
    id: 'desktop',
    title: 'Desktop mode',
    detail: 'Start the local product with the embedded portal shell and runtime evidence.',
    command: 'pnpm product:start',
  },
  {
    id: 'service',
    title: 'Background service',
    detail: 'Keep the desktop runtime available in the background for local routing.',
    command: 'pnpm product:service',
  },
  {
    id: 'server',
    title: 'Server mode',
    detail: 'Host the full router product for remote teams and shared gateway traffic.',
    command: 'pnpm server:start',
  },
];

const systemRequirements = [
  'Node.js 20+ for local tooling and workspace automation tasks.',
  'Desktop runtime access for QR sign-in, local health checks, and product orchestration.',
  'Gateway service connectivity so the console and software downloads flow point at the same runtime boundary.',
];

const installationSteps = [
  'Download the desktop or service artifact that matches your environment.',
  'Launch the runtime, sign in, and verify gateway health before creating workspace credentials.',
  'Use the console and docs center to finish provider routing, API key setup, and usage governance.',
];

const deploymentTracks = [
  {
    id: 'desktop',
    title: 'Desktop mode',
    detail: 'Best for local gateway runtime, desktop orchestration, and quick QR-based sign in.',
    actionLabel: 'Open desktop setup',
    href: '/docs?group=quickstart',
    tags: ['Windows', 'macOS', 'Desktop mode'],
  },
  {
    id: 'service',
    title: 'Background service',
    detail: 'Keep the desktop runtime available in the background for local routing.',
    actionLabel: 'Open install guide',
    href: '/docs?group=operations',
    tags: ['Linux', 'Background service', 'Runtime modes'],
  },
  {
    id: 'server',
    title: 'Server mode',
    detail: 'Host the full router product for remote teams and shared gateway traffic.',
    actionLabel: 'Open console',
    href: '/console/dashboard',
    tags: ['Linux', 'Server mode', 'Launch posture'],
  },
];

export function PortalDownloadsPage() {
  const { t } = usePortalI18n();
  const navigate = useNavigate();
  const metricCards = [
    {
      label: 'Delivery targets',
      value: String(softwareTargets.length),
      description:
        'Install packages stay segmented by environment while keeping the same runtime contract and onboarding path.',
    },
    {
      label: 'Runtime postures',
      value: String(runtimeModes.length),
      description:
        'Runtime modes stay visible so operators can choose the correct service shape before launch.',
    },
    {
      label: 'Guided launch steps',
      value: String(installationSteps.length),
      description:
        'One guided sequence keeps install, sign-in, health checks, and console handoff aligned.',
    },
  ];

  return (
    <div className="space-y-6" data-slot="portal-downloads-page">
      <PortalSiteHero
        actions={(
          <>
            <Button type="button" onClick={() => navigate('/docs?group=quickstart')}>
              {t('Open install guide')}
            </Button>
            <Button type="button" onClick={() => navigate('/console/dashboard')} variant="secondary">
              {t('Open console')}
            </Button>
          </>
        )}
        aside={(
          <PortalSitePanel
            className="rounded-[28px] border-zinc-200/80 bg-zinc-50/80 dark:border-zinc-800 dark:bg-zinc-900/60"
            description={t('Choose the operating mode that matches how your team will run gateway traffic and operator workflows.')}
            title={t('Launch posture')}
          >
            {runtimeModes.map((mode) => (
              <div
                key={mode.id}
                className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950"
              >
                <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {t(mode.title)}
                </div>
                <div className="mt-1 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(mode.detail)}
                </div>
              </div>
            ))}
          </PortalSitePanel>
        )}
        description={t('Desktop, background service, and shared gateway distributions stay connected to docs, console, and onboarding actions from one software center.')}
        eyebrow={t('Software Downloads')}
        title={t('Launch the runtime your team will actually operate.')}
      />

      <section className="grid gap-4 md:grid-cols-3" data-slot="portal-downloads-metrics">
        {metricCards.map((item) => (
          <PortalSiteMetricCard
            key={item.label}
            description={t(item.description)}
            label={t(item.label)}
            value={item.value}
          />
        ))}
      </section>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.08fr)_minmax(0,0.92fr)]">
        <div data-slot="portal-downloads-deployment-tracks">
          <PortalSitePanel
            description={t('Choose the software path that matches local operators, background automation, or shared gateway delivery.')}
            title={t('Deployment tracks')}
          >
            <div className="grid gap-4 md:grid-cols-3 xl:grid-cols-1">
              {deploymentTracks.map((track) => (
                <div
                  key={track.id}
                  className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
                >
                  <div className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {t(track.title)}
                  </div>
                  <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {t(track.detail)}
                  </div>
                  <div className="mt-4 flex flex-wrap gap-2">
                    {track.tags.map((tag) => (
                      <span
                        key={`${track.id}-${tag}`}
                        className="inline-flex items-center rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300"
                      >
                        {t(tag)}
                      </span>
                    ))}
                  </div>
                  <div className="mt-4">
                    <Button type="button" onClick={() => navigate(track.href)}>
                      {t(track.actionLabel)}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </PortalSitePanel>
        </div>

        <div data-slot="portal-downloads-rollout-loop">
          <PortalSitePanel
            description={t('Installation, launch, verification, and console handoff stay connected in one software delivery surface.')}
            title={t('Rollout loop')}
          >
            <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
              <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Installation steps')}
              </div>
              <div className="mt-4 grid gap-3">
                {installationSteps.map((step, index) => (
                  <div
                    key={step}
                    className="rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950"
                  >
                    <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                      {t('Step {index}', { index: index + 1 })}
                    </div>
                    <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                      {t(step)}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <div
                className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
              >
                <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  {t('Open install guide')}
                </div>
                <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Install targets')}
                </div>
                <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t('Pick the desktop or service path that matches the environment you need to onboard.')}
                </div>
                <div className="mt-4">
                  <Button type="button" onClick={() => navigate('/docs?group=quickstart')}>
                    {t('Open install guide')}
                  </Button>
                </div>
              </div>

              <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
                <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  {t('Open console')}
                </div>
                <div className="mt-2 text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Runtime modes')}
                </div>
                <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t('Launch the product in the same posture your operators will use in production or local evaluation.')}
                </div>
                <div className="mt-4">
                  <Button type="button" onClick={() => navigate('/console/dashboard')} variant="secondary">
                    {t('Open console')}
                  </Button>
                </div>
              </div>
            </div>
          </PortalSitePanel>
        </div>
      </div>

      <PortalSitePanel
        description={t('Pick the desktop or service path that matches the environment you need to onboard.')}
        title={t('Install targets')}
      >
        <div className="grid gap-4 xl:grid-cols-3">
          {softwareTargets.map((target) => (
            <div
              key={target.platform}
              className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
            >
              <div className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {target.platform}
              </div>
              <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                {t(target.artifact)}
              </div>
              <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                {t(target.detail)}
              </div>
              <div className="mt-4 rounded-2xl border border-zinc-200 bg-white px-3 py-2 font-mono text-xs text-zinc-700 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-200">
                {target.command}
              </div>
              <div className="mt-4 flex flex-wrap gap-2">
                <Button
                  type="button"
                  onClick={() => navigate(`/docs?group=${target.guideGroup}`)}
                >
                  {t(target.actionLabel)}
                </Button>
              </div>
            </div>
          ))}
        </div>
      </PortalSitePanel>

      <div className="grid gap-4 xl:grid-cols-3">
        <PortalSitePanel
          className="xl:col-span-2"
          description={t('Launch the product in the same posture your operators will use in production or local evaluation.')}
          title={t('Runtime modes')}
        >
          <div className="grid gap-4 md:grid-cols-3">
            {runtimeModes.map((mode) => (
              <div
                key={mode.id}
                className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60"
              >
                <div className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t(mode.title)}
                </div>
                <div className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(mode.detail)}
                </div>
                <div className="mt-4 rounded-2xl border border-zinc-200 bg-white px-3 py-2 font-mono text-xs text-zinc-700 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-200">
                  {mode.command}
                </div>
              </div>
            ))}
          </div>
        </PortalSitePanel>

        <PortalSitePanel
          description={t('Review the minimum runtime expectations before distributing portal software to operators or teams.')}
          title={t('System requirements')}
        >
          <div className="grid gap-3">
            {systemRequirements.map((item) => (
              <div
                key={item}
                className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 text-sm leading-6 dark:border-zinc-800 dark:bg-zinc-900/60"
              >
                {t(item)}
              </div>
            ))}
          </div>
        </PortalSitePanel>
      </div>
    </div>
  );
}
