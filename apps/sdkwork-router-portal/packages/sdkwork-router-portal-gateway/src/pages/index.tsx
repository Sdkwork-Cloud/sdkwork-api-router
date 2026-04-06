import { useDeferredValue, useEffect, useState } from 'react';
import type { ReactNode } from 'react';

import {
  formatDateTime,
  formatUnits,
  translatePortalText,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  Badge,
  DataTable,
} from 'sdkwork-router-portal-commons/framework/display';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from 'sdkwork-router-portal-commons/framework/entry';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import {
  FilterBar,
  FilterBarActions,
  FilterBarSection,
  SearchInput,
  FilterField,
} from 'sdkwork-router-portal-commons/framework/form';
import { ManagementWorkbench } from 'sdkwork-router-portal-commons/framework/workbench';
import {
  WorkspacePanel,
} from 'sdkwork-router-portal-commons/framework/workspace';

import {
  GatewayLaunchReadinessPanel,
  GatewayModeGrid,
  GatewayPostureGrid,
  GatewayReadinessGrid,
  GatewayRuntimeControlsGrid,
  GatewayTopologyGrid,
} from '../components';
import {
  loadGatewayCommandCenterSnapshot,
  restartGatewayCommandCenterDesktopRuntime,
} from '../repository';
import type { GatewayCommandCenterSnapshot, PortalGatewayPageProps } from '../types';

type GatewayWorkbenchLane =
  | 'compatibility'
  | 'rate-limit-policies'
  | 'rate-limit-windows'
  | 'service-health'
  | 'verification';

type GatewayWorkbenchRow = {
  id: string;
  focus: string;
  subject: ReactNode;
  scope: ReactNode;
  meter: ReactNode;
  status: ReactNode;
  detail: ReactNode;
  searchText: string;
};

type GatewayWorkbenchConfig = {
  laneLabel: string;
  focusOptions: Array<{ value: string; label: string }>;
  subjectLabel: string;
  scopeLabel: string;
  meterLabel: string;
  detailLabel: string;
  detail: string;
  emptyTitle: string;
  emptyDetail: string;
};

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;

const WORKBENCH_LANE_OPTIONS: GatewayWorkbenchLane[] = [
  'service-health',
  'compatibility',
  'rate-limit-policies',
  'rate-limit-windows',
  'verification',
];

function includesQuery(query: string, values: Array<string | number | null | undefined>): boolean {
  if (!query) {
    return true;
  }

  return values
    .filter((value) => value !== null && value !== undefined)
    .join(' ')
    .toLowerCase()
    .includes(query);
}

function serviceHealthLabel(status: GatewayCommandCenterSnapshot['serviceHealthChecks'][number]['status']) {
  if (status === 'healthy') {
    return translatePortalText('Healthy');
  }

  if (status === 'degraded') {
    return translatePortalText('Degraded');
  }

  return translatePortalText('Unreachable');
}

function serviceHealthTone(status: GatewayCommandCenterSnapshot['serviceHealthChecks'][number]['status']) {
  return status === 'healthy' ? 'success' : 'warning';
}

function rateLimitTone(enabled: boolean, exceeded: boolean) {
  if (!enabled || exceeded) {
    return 'warning';
  }

  return 'success';
}

function buildGatewayRateLimitScopeLabel(input: {
  api_key_hash?: string | null;
  route_key?: string | null;
  model_name?: string | null;
}): string {
  const parts = [
    input.api_key_hash
      ? translatePortalText('API key {keyHash}', {
          keyHash: `${input.api_key_hash.slice(0, 12)}...`,
        })
      : null,
    input.route_key
      ? translatePortalText('Route {routeKey}', { routeKey: input.route_key })
      : null,
    input.model_name
      ? translatePortalText('Model {modelName}', { modelName: input.model_name })
      : null,
  ].filter(Boolean);

  return parts.length ? parts.join(' / ') : translatePortalText('project-wide');
}

function verificationFocus(routeFamily: string): string {
  if (routeFamily.includes('/v1/messages')) {
    return 'anthropic';
  }

  if (routeFamily.includes('generateContent')) {
    return 'gemini';
  }

  return 'openai';
}

function verificationTone(focus: string) {
  if (focus === 'anthropic') {
    return 'default';
  }

  if (focus === 'gemini') {
    return 'outline';
  }

  return 'success';
}

function formatLatency(latencyMs?: number | null): string {
  if (latencyMs === null || latencyMs === undefined) {
    return translatePortalText('No latency sample');
  }

  return `${latencyMs} ms`;
}

function buildGatewayServiceHealthMeter(
  check: GatewayCommandCenterSnapshot['serviceHealthChecks'][number],
): string {
  return translatePortalText('HTTP {httpStatus} · {latency}', {
    httpStatus:
      check.httpStatus === null || check.httpStatus === undefined
        ? translatePortalText('No HTTP status')
        : String(check.httpStatus),
    latency: formatLatency(check.responseTimeMs),
  });
}

function buildGatewayRateLimitPolicyMeter(
  policy: GatewayCommandCenterSnapshot['rateLimitSnapshot']['policies'][number],
): string {
  return translatePortalText('{limitRequests} req / {windowSeconds}s · burst {burstRequests}', {
    limitRequests: formatUnits(policy.limit_requests),
    windowSeconds: policy.window_seconds,
    burstRequests: formatUnits(policy.burst_requests || policy.requests_per_window),
  });
}

function buildGatewayRateLimitWindowMeter(
  window: GatewayCommandCenterSnapshot['rateLimitSnapshot']['windows'][number],
): string {
  return translatePortalText('{requestCount} / {limitRequests} requests · {remainingRequests} remaining', {
    requestCount: formatUnits(window.request_count),
    limitRequests: formatUnits(window.limit_requests),
    remainingRequests: formatUnits(window.remaining_requests),
  });
}

function buildGatewayVerificationProtocolLabel(focus: string): string {
  if (focus === 'anthropic') {
    return translatePortalText('Anthropic Messages');
  }

  if (focus === 'gemini') {
    return translatePortalText('Gemini');
  }

  return translatePortalText('OpenAI-compatible');
}

function workbenchConfig(
  snapshot: GatewayCommandCenterSnapshot,
  lane: GatewayWorkbenchLane,
  t: TranslateFn,
): GatewayWorkbenchConfig {
  switch (lane) {
    case 'compatibility':
      return {
        laneLabel: t('Compatibility routes'),
        focusOptions: [
          { value: 'all', label: t('All routes') },
          { value: 'direct', label: t('Direct gateway') },
          { value: 'translated', label: t('Translated routes') },
          { value: 'desktop', label: t('Desktop setup') },
        ],
        subjectLabel: t('Tool'),
        scopeLabel: t('Route family'),
        meterLabel: t('Execution truth'),
        detailLabel: t('Operator outcome'),
        detail: t(
          'Compatibility routes keep Codex, Claude Code, Gemini, and OpenClaw onboarding on one shared gateway posture.',
        ),
        emptyTitle: t('No compatibility routes in this slice'),
        emptyDetail: t('Adjust the workbench lane or search to reveal a different protocol family.'),
      };
    case 'rate-limit-policies':
      return {
        laneLabel: t('Rate-limit policies'),
        focusOptions: [
          { value: 'all', label: t('All policies') },
          { value: 'enabled', label: t('Enabled') },
          { value: 'disabled', label: t('Disabled') },
        ],
        subjectLabel: t('Policy'),
        scopeLabel: t('Scope'),
        meterLabel: t('Limit'),
        detailLabel: t('Operator notes'),
        detail: translatePortalText('Project rate-limit policy posture was last checked {checkedAt}.', {
          checkedAt: formatDateTime(snapshot.rateLimitSnapshot.generated_at_ms),
        }),
        emptyTitle: t('No rate-limit policies in this slice'),
        emptyDetail: t('The workspace does not currently expose a matching project-scoped rate-limit policy.'),
      };
    case 'rate-limit-windows':
      return {
        laneLabel: t('Rate-limit windows'),
        focusOptions: [
          { value: 'all', label: t('All windows') },
          { value: 'within-limit', label: t('Within limit') },
          { value: 'over-limit', label: t('Over limit') },
          { value: 'disabled', label: t('Disabled') },
        ],
        subjectLabel: t('Window'),
        scopeLabel: t('Scope'),
        meterLabel: t('Usage'),
        detailLabel: t('Window detail'),
        detail: translatePortalText('Live rate-limit windows were last checked {checkedAt}.', {
          checkedAt: formatDateTime(snapshot.rateLimitSnapshot.generated_at_ms),
        }),
        emptyTitle: t('No live windows in this slice'),
        emptyDetail: t('Live rate-limit pressure will appear here once gateway activity is present.'),
      };
    case 'verification':
      return {
        laneLabel: t('Verification commands'),
        focusOptions: [
          { value: 'all', label: t('All commands') },
          { value: 'openai', label: t('OpenAI-compatible') },
          { value: 'anthropic', label: t('Anthropic Messages') },
          { value: 'gemini', label: t('Gemini') },
        ],
        subjectLabel: t('Check'),
        scopeLabel: t('Route family'),
        meterLabel: t('Protocol'),
        detailLabel: t('Verification command'),
        detail: t(
          'Verification commands turn gateway activation into an executable launch checklist instead of static documentation.',
        ),
        emptyTitle: t('No verification commands in this slice'),
        emptyDetail: t('Change the focus or search to reveal another verification route family.'),
      };
    case 'service-health':
    default:
      return {
        laneLabel: t('Service health'),
        focusOptions: [
          { value: 'all', label: t('All services') },
          { value: 'healthy', label: t('Healthy') },
          { value: 'degraded', label: t('Degraded') },
          { value: 'unreachable', label: t('Unreachable') },
        ],
        subjectLabel: t('Service'),
        scopeLabel: t('Health route'),
        meterLabel: t('Runtime signal'),
        detailLabel: t('Operator detail'),
        detail: translatePortalText('Live service health was last checked {checkedAt}.', {
          checkedAt: formatDateTime(snapshot.runtimeHealth.checkedAtMs),
        }),
        emptyTitle: t('No service health checks in this slice'),
        emptyDetail: t('Refresh service health to pull the latest runtime evidence into the command workbench.'),
      };
  }
}

function buildWorkbenchRows(
  snapshot: GatewayCommandCenterSnapshot,
  lane: GatewayWorkbenchLane,
): GatewayWorkbenchRow[] {
  switch (lane) {
    case 'compatibility':
      return snapshot.compatibilityRows.map((row) => {
        const focus = row.truth.includes('translated')
          ? 'translated'
          : row.truth.includes('desktop-assisted')
            ? 'desktop'
            : 'direct';

        return {
          id: row.id,
          focus,
          subject: (
            <div className="space-y-1">
              <strong className="text-zinc-950 dark:text-zinc-50">{row.tool}</strong>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">{row.protocol}</p>
            </div>
          ),
          scope: row.routeFamily,
          meter: (
            <Badge
              variant={focus === 'translated' ? 'default' : focus === 'desktop' ? 'outline' : 'success'}
            >
              {row.truth}
            </Badge>
          ),
          status: (
            <Badge variant={focus === 'translated' ? 'default' : focus === 'desktop' ? 'outline' : 'success'}>
              {focus === 'translated'
                ? translatePortalText('Translated routes')
                : focus === 'desktop'
                  ? translatePortalText('Desktop setup')
                  : translatePortalText('Direct gateway')}
            </Badge>
          ),
          detail: <p className="max-w-[30rem] leading-6">{row.outcome}</p>,
          searchText: [
            row.tool,
            row.protocol,
            row.routeFamily,
            row.truth,
            row.outcome,
          ]
            .join(' ')
            .toLowerCase(),
        };
      });
    case 'rate-limit-policies':
      return snapshot.rateLimitSnapshot.policies.map((policy) => ({
        id: policy.policy_id,
        focus: policy.enabled ? 'enabled' : 'disabled',
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{policy.policy_id}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">
              {translatePortalText('Updated {checkedAt}', {
                checkedAt: formatDateTime(policy.updated_at_ms),
              })}
            </p>
          </div>
        ),
        scope: buildGatewayRateLimitScopeLabel(policy),
        meter: buildGatewayRateLimitPolicyMeter(policy),
        status: (
          <Badge variant={policy.enabled ? 'success' : 'warning'}>
            {policy.enabled ? translatePortalText('Enabled') : translatePortalText('Disabled')}
          </Badge>
        ),
        detail: policy.notes ?? translatePortalText('No operator notes were attached to this policy.'),
        searchText: [
          policy.policy_id,
          policy.project_id,
          policy.api_key_hash,
          policy.route_key,
          policy.model_name,
          policy.notes,
        ]
          .filter(Boolean)
          .join(' ')
          .toLowerCase(),
      }));
    case 'rate-limit-windows':
      return snapshot.rateLimitSnapshot.windows.map((window) => {
        const focus = !window.enabled ? 'disabled' : window.exceeded ? 'over-limit' : 'within-limit';

        return {
          id: `${window.policy_id}:${window.window_start_ms}`,
          focus,
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{window.policy_id}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">
              {translatePortalText('Started {checkedAt}', {
                checkedAt: formatDateTime(window.window_start_ms),
              })}
            </p>
          </div>
        ),
          scope: buildGatewayRateLimitScopeLabel(window),
          meter: buildGatewayRateLimitWindowMeter(window),
          status: (
            <Badge variant={rateLimitTone(window.enabled, window.exceeded)}>
              {!window.enabled
                ? translatePortalText('Disabled')
                : window.exceeded
                  ? translatePortalText('Over limit')
                  : translatePortalText('Within limit')}
            </Badge>
          ),
          detail: translatePortalText('{windowSeconds}s window ends {endsAt}', {
            windowSeconds: window.window_seconds,
            endsAt: formatDateTime(window.window_end_ms),
          }),
          searchText: [
            window.policy_id,
            window.project_id,
            window.api_key_hash,
            window.route_key,
            window.model_name,
            String(window.request_count),
            String(window.limit_requests),
          ]
            .filter(Boolean)
            .join(' ')
            .toLowerCase(),
        };
      });
    case 'verification':
      return snapshot.verificationSnippets.map((snippet) => {
        const focus = verificationFocus(snippet.routeFamily);

        return {
          id: snippet.id,
          focus,
          subject: (
            <div className="space-y-1">
              <strong className="text-zinc-950 dark:text-zinc-50">{snippet.title}</strong>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">{snapshot.gatewayBaseUrl}</p>
            </div>
          ),
          scope: snippet.routeFamily,
          meter: (
            <Badge variant={verificationTone(focus)}>
              {buildGatewayVerificationProtocolLabel(focus)}
            </Badge>
          ),
          status: <Badge variant="outline">{translatePortalText('Ready to run')}</Badge>,
          detail: (
            <pre className="max-w-[34rem] overflow-x-auto rounded-2xl bg-zinc-950 p-3 text-xs leading-6 text-zinc-300">
              <code>{snippet.command}</code>
            </pre>
          ),
          searchText: [snippet.title, snippet.routeFamily, snippet.command].join(' ').toLowerCase(),
        };
      });
    case 'service-health':
    default:
      return snapshot.serviceHealthChecks.map((check) => ({
        id: check.id,
        focus: check.status,
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{check.label}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">
              {snapshot.runtimeHealth.mode} {translatePortalText('mode')}
            </p>
          </div>
        ),
        scope: (
          <code className="break-all text-xs text-zinc-600 dark:text-zinc-300">
            {check.healthUrl}
          </code>
        ),
        meter: buildGatewayServiceHealthMeter(check),
        status: (
          <Badge variant={serviceHealthTone(check.status)}>
            {serviceHealthLabel(check.status)}
          </Badge>
        ),
        detail: check.detail,
        searchText: [check.label, check.status, check.healthUrl, check.detail].join(' ').toLowerCase(),
      }));
  }
}

export function PortalGatewayPage({ onNavigate }: PortalGatewayPageProps) {
  const { t } = usePortalI18n();
  const [snapshot, setSnapshot] = useState<GatewayCommandCenterSnapshot | null>(null);
  const [status, setStatus] = useState(
    t('Loading the router product command center and current launch posture...'),
  );
  const [refreshing, setRefreshing] = useState(false);
  const [restartingRuntime, setRestartingRuntime] = useState(false);
  const [workbenchLane, setWorkbenchLane] = useState<GatewayWorkbenchLane>('service-health');
  const [focusFilter, setFocusFilter] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  useEffect(() => {
    let cancelled = false;

    const loadSnapshot = async () => {
      try {
        const nextSnapshot = await loadGatewayCommandCenterSnapshot();
        if (cancelled) {
          return;
        }

        setSnapshot(nextSnapshot);
        setStatus(
          t(
            'The portal now exposes compatibility, deployment modes, runtime evidence, and commercial runway as one operator-facing product surface.',
          ),
        );
      } catch (error) {
        if (cancelled) {
          return;
        }

        setStatus(
          error instanceof Error
            ? error.message
            : t('The command center could not load the current gateway posture.'),
        );
      }
    };

    void loadSnapshot();

    return () => {
      cancelled = true;
    };
  }, [t]);

  const refreshCommandCenter = async (nextStatus: string) => {
    setRefreshing(true);
    setStatus(nextStatus);

    try {
      const nextSnapshot = await loadGatewayCommandCenterSnapshot();
      setSnapshot(nextSnapshot);
      setStatus(
        t('The command center is showing the latest compatibility, runtime, and commercial posture.'),
      );
    } catch (error) {
      setStatus(
        error instanceof Error
          ? error.message
          : t('The command center could not refresh the current gateway posture.'),
      );
    } finally {
      setRefreshing(false);
    }
  };

  const handleRuntimeControl = async (action: 'restart-desktop-runtime') => {
    if (action !== 'restart-desktop-runtime') {
      return;
    }

    setRestartingRuntime(true);
    setStatus(t('Restarting the embedded desktop runtime and refreshing live service posture...'));

    try {
      const nextSnapshot = await restartGatewayCommandCenterDesktopRuntime();
      setSnapshot(nextSnapshot);
      setStatus(
        t(
          'Desktop runtime restarted successfully and the command center has been refreshed with the latest service posture.',
        ),
      );
    } catch (error) {
      setStatus(
        error instanceof Error
          ? error.message
          : t('Desktop runtime restart failed before the command center could refresh.'),
      );
    } finally {
      setRestartingRuntime(false);
    }
  };

  if (!snapshot) {
    return (
      <WorkspacePanel
        description={status}
        title={t('Preparing gateway command center')}
      >
        <EmptyState
          description={t(
            'The command center will appear once the portal finishes assembling the product-facing router view.',
          )}
          title={t('Preparing gateway command center')}
        />
      </WorkspacePanel>
    );
  }

  const config = workbenchConfig(snapshot, workbenchLane, t);
  const allRows = buildWorkbenchRows(snapshot, workbenchLane);
  const visibleRows = allRows.filter(
    (row) =>
      (focusFilter === 'all' || row.focus === focusFilter)
      && includesQuery(deferredSearch, [row.searchText]),
  );
  const focusLabel =
    t(config.focusOptions.find((option) => option.value === focusFilter)?.label ?? 'All');
  const readinessTone =
    snapshot.launchReadiness.status === 'ready'
      ? 'success'
      : snapshot.launchReadiness.status === 'watch'
        ? 'default'
        : 'warning';

  return (
    <div className="space-y-6">
      <div
        data-slot="portal-gateway-toolbar"
        className="flex flex-wrap items-start justify-between gap-3 rounded-[24px] border border-zinc-200/80 bg-zinc-50/85 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/45"
      >
        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2 text-sm text-zinc-600 dark:text-zinc-300">
          <Badge variant={readinessTone}>
            {snapshot.launchReadiness.headline}
          </Badge>
          <p className="min-w-[16rem] flex-1 leading-6 text-zinc-500 dark:text-zinc-400">
            {status}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            disabled={refreshing || restartingRuntime}
            onClick={() => {
              void refreshCommandCenter(t('Refreshing the full command center posture...'));
            }}
            variant="secondary"
          >
            {refreshing ? t('Refreshing command center...') : t('Refresh command center')}
          </Button>
        </div>
      </div>

      <div className="grid gap-6 xl:grid-cols-[0.96fr_1.04fr]">
        <WorkspacePanel
          description={t('{detail} {suffix}', {
            detail: snapshot.launchReadiness.detail,
            suffix: t('Critical blockers and watchpoints stay visible before launch traffic expands.'),
          })}
          title={t('Launch readiness')}
        >
          <GatewayLaunchReadinessPanel readiness={snapshot.launchReadiness} />
        </WorkspacePanel>

        <WorkspacePanel
          description={t(
            'Desktop runtime cards keep the local bind story visible while Restart desktop runtime remains intentionally narrow.',
          )}
          title={t('Desktop runtime')}
        >
          <div className="grid gap-6 xl:grid-cols-[1.15fr_0.85fr]">
            <div className="grid gap-4">
              <GatewayPostureGrid cards={snapshot.runtimeCards} />
            </div>
            <div className="grid gap-4">
              <GatewayRuntimeControlsGrid
                busyAction={restartingRuntime ? 'restart-desktop-runtime' : null}
                controls={snapshot.runtimeControls}
                onAction={(action) => {
                  void handleRuntimeControl(action);
                }}
              />
            </div>
          </div>
        </WorkspacePanel>
      </div>

      <ManagementWorkbench
        actions={(
          <Button
            disabled={refreshing || restartingRuntime}
            onClick={() => {
              void refreshCommandCenter(t('Refreshing service health and gateway evidence...'));
            }}
            variant="secondary"
          >
            {refreshing ? t('Refreshing service health...') : t('Refresh service health')}
          </Button>
        )}
        description={t(config.detail)}
        detail={{
          children: <GatewayPostureGrid cards={snapshot.postureCards} />,
          description: status,
          summary: (
            <Badge variant={readinessTone}>
              {snapshot.launchReadiness.headline}
            </Badge>
          ),
          title: t('Gateway posture'),
        }}
        detailWidth={420}
        eyebrow={t('Gateway posture')}
        filters={(
          <div className="grid gap-4">
            <div className="flex flex-wrap items-center gap-3 text-sm text-zinc-500 dark:text-zinc-400">
              <Badge variant="outline">{t(config.laneLabel)}</Badge>
              <span>{t('{visible} of {total} rows visible', {
                visible: formatUnits(visibleRows.length),
                total: formatUnits(allRows.length),
              })}</span>
              <span>{t('Focus: {focus}', { focus: focusLabel })}</span>
            </div>

            <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
              {t('Verification commands cover')} <code>{'/api/v1/models'}</code>,{' '}
              <code>{'/v1/messages'}</code>, {t('and')} <code>{'generateContent'}</code>{' '}
              {t('so each compatibility family can be checked from one workbench.')}
            </p>

            <FilterBar data-slot="portal-gateway-filter-bar">
              <FilterBarSection className="min-w-[15rem] flex-[0_1_20rem]" grow={false}>
                <FilterField
                  className="w-full"
                  controlClassName="min-w-0"
                  label={t('Search gateway evidence')}
                >
                  <SearchInput
                    value={searchQuery}
                    onChange={(event) => setSearchQuery(event.target.value)}
                    placeholder={t('Search gateway evidence')}
                  />
                </FilterField>
              </FilterBarSection>
              <FilterBarSection className="min-w-[12rem] shrink-0" grow={false}>
                <FilterField className="w-full" label={t('Workbench lane')}>
                  <Select
                    value={workbenchLane}
                    onValueChange={(value) => {
                      setWorkbenchLane(value as GatewayWorkbenchLane);
                      setFocusFilter('all');
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('Workbench lane')} />
                    </SelectTrigger>
                    <SelectContent>
                      {WORKBENCH_LANE_OPTIONS.map((option) => (
                        <SelectItem key={option} value={option}>
                          {option === 'service-health'
                            ? t('Service health')
                            : option === 'compatibility'
                              ? t('Compatibility routes')
                              : option === 'rate-limit-policies'
                                ? t('Rate-limit policies')
                                : option === 'rate-limit-windows'
                                  ? t('Rate-limit windows')
                                  : t('Verification commands')}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FilterField>
              </FilterBarSection>

              <FilterBarSection className="min-w-[12rem] shrink-0" grow={false}>
                <FilterField className="w-full" label={t('Operational focus')}>
                  <Select
                    value={focusFilter}
                    onValueChange={setFocusFilter}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('Operational focus')} />
                    </SelectTrigger>
                    <SelectContent>
                      {config.focusOptions.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {t(option.label)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FilterField>
              </FilterBarSection>

              <FilterBarActions className="gap-2.5 whitespace-nowrap">
                <Button
                  onClick={() => {
                    setFocusFilter('all');
                    setSearchQuery('');
                  }}
                  variant="secondary"
                >
                  {t('Clear filters')}
                </Button>
              </FilterBarActions>
            </FilterBar>
          </div>
        )}
        main={{
          children: (
            <DataTable
              columns={[
                { id: 'subject', header: t(config.subjectLabel), cell: (row) => row.subject },
                { id: 'scope', header: t(config.scopeLabel), cell: (row) => row.scope },
                { id: 'meter', header: t(config.meterLabel), cell: (row) => row.meter },
                { id: 'status', header: t('Status'), cell: (row) => row.status },
                { id: 'detail', header: t(config.detailLabel), cell: (row) => row.detail },
              ]}
              emptyState={(
                <div className="mx-auto flex max-w-[34rem] flex-col items-center gap-2 text-center">
                  <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {t(config.emptyTitle)}
                  </strong>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">
                    {t(config.emptyDetail)}
                  </p>
                </div>
              )}
              getRowId={(row) => row.id}
              rows={visibleRows}
            />
          ),
          description: t(config.detail),
          title: t('Command workbench'),
        }}
        title={t('Command workbench')}
      />

      <div className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
        <WorkspacePanel
          description={t(
            'Mode switchboard and topology playbooks keep the path from desktop mode to hosted server mode explicit.',
          )}
          title={t('Deployment playbooks')}
        >
          <div className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
            <section className="grid gap-4">
              <div className="space-y-2">
                <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Mode switchboard')}
                </strong>
                <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(
                    'Keep the product launch path readable whether the router is running on one machine or transitioning into a hosted topology.',
                  )}
                </p>
              </div>
              <GatewayModeGrid cards={snapshot.modeCards} />
            </section>

            <section className="grid gap-4">
              <div className="space-y-2">
                <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Topology playbooks')}
                </strong>
                <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(
                    'Promote runtime documentation into executable rollout playbooks that operators can apply immediately.',
                  )}
                </p>
              </div>
              <GatewayTopologyGrid playbooks={snapshot.topologyPlaybooks} />
            </section>
          </div>
        </WorkspacePanel>

        <WorkspacePanel
          description={t(
            'Commerce catalog and launch actions keep access, routing, and billing runway on one commercial surface.',
          )}
          title={t('Commercial runway')}
        >
          <div className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
            <section className="grid gap-4">
              <div className="space-y-2">
                <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Commerce catalog')}
                </strong>
                <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(
                    'Active membership, recharge packs, and coupon campaigns remain visible as live product inventory instead of drifting into static launch copy.',
                  )}
                </p>
              </div>
              <GatewayPostureGrid cards={snapshot.commerceCatalogCards} />
            </section>

            <section className="grid gap-4">
              <div className="space-y-2">
                <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Launch actions')}
                </strong>
                <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(
                    'Open API Keys, Open Routing, and Open Billing are the three fastest actions for turning this command center into a real launch workflow.',
                  )}
                </p>
              </div>
              <GatewayReadinessGrid actions={snapshot.readinessActions} onNavigate={onNavigate} />
            </section>
          </div>
        </WorkspacePanel>
      </div>
    </div>
  );
}



