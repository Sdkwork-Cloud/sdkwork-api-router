import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent, ReactNode } from 'react';

import {
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  Badge,
  DataTable,
} from 'sdkwork-router-portal-commons/framework/display';
import {
  Checkbox,
  Input,
  Label,
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
  FilterField,
  SearchInput,
  SettingsField,
} from 'sdkwork-router-portal-commons/framework/form';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from 'sdkwork-router-portal-commons/framework/overlays';
import { ManagementWorkbench } from 'sdkwork-router-portal-commons/framework/workbench';
import {
  SectionHeader,
  WorkspacePanel,
} from 'sdkwork-router-portal-commons/framework/workspace';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  PortalCompiledRoutingSnapshotRecord,
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  PortalRoutingProviderOption,
  PortalRoutingSummary,
  RoutingProfileRecord,
} from 'sdkwork-router-portal-types';

import {
  PortalRoutingProfilesDialog,
  PortalRoutingSnapshotsDialog,
  RoutingCardGrid,
} from '../components';
import type { RoutingCardItem } from '../components';
import {
  issuePortalRoutingProfile,
  loadPortalRoutingProfiles,
  loadPortalRoutingSnapshots,
  loadPortalRoutingDecisionLogs,
  loadPortalRoutingSummary,
  runPortalRoutingPreview,
  updatePortalRoutingPreferences,
} from '../repository';
import {
  buildRoutingAssessmentHealthLabel,
  buildPortalRoutingViewModel,
  buildRoutingCapabilityLabel,
  buildRoutingDecisionSourceLabel,
  buildRoutingStrategyLabel,
} from '../services';
import type { PortalRoutingPageProps } from '../types';

type RoutingFormState = {
  preset_id: string;
  strategy: PortalRoutingPreferences['strategy'];
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost: string;
  max_latency_ms: string;
  require_healthy: boolean;
  preferred_region: string;
};

type RoutingPreviewFormState = {
  capability: string;
  model: string;
  requested_region: string;
  selection_seed: string;
};

type RoutingWorkbenchLane = 'presets' | 'providers' | 'evidence';

type RoutingWorkbenchRow = {
  id: string;
  focus: string;
  subject: ReactNode;
  scope: ReactNode;
  status: ReactNode;
  detail: ReactNode;
  actions: ReactNode;
  searchText: string;
};

type RoutingWorkbenchConfig = {
  laneLabel: string;
  scopeLabel: string;
  detailLabel: string;
  actionsLabel: string;
  detail: string;
  emptyTitle: string;
  emptyDetail: string;
  focusOptions: Array<{ value: string; label: string }>;
};

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;

const WORKBENCH_OPTIONS: RoutingWorkbenchLane[] = ['providers', 'presets', 'evidence'];

const AUTO_REGION_SELECT_VALUE = '__portal-routing-auto-region__';
const AUTO_PROVIDER_SELECT_VALUE = '__portal-routing-auto-provider__';
const DEFAULT_ROUTING_CAPABILITY = 'chat_completion';
const RELIABILITY_LATENCY_TARGET_MS = '250';

function toFormState(summary: PortalRoutingSummary): RoutingFormState {
  return {
    preset_id:
      summary.preferences.preset_id === 'platform_default'
        ? ''
        : (summary.preferences.preset_id ?? ''),
    strategy: summary.preferences.strategy,
    ordered_provider_ids: summary.provider_options.map((provider) => provider.provider_id),
    default_provider_id: summary.preferences.default_provider_id ?? null,
    max_cost:
      summary.preferences.max_cost === null || summary.preferences.max_cost === undefined
        ? ''
        : String(summary.preferences.max_cost),
    max_latency_ms:
      summary.preferences.max_latency_ms === null || summary.preferences.max_latency_ms === undefined
        ? ''
        : String(summary.preferences.max_latency_ms),
    require_healthy: summary.preferences.require_healthy,
    preferred_region: summary.preferences.preferred_region ?? '',
  };
}

function toFormStateFromProfile(profile: RoutingProfileRecord): RoutingFormState {
  return {
    preset_id: profile.slug,
    strategy: profile.strategy as PortalRoutingPreferences['strategy'],
    ordered_provider_ids: [...profile.ordered_provider_ids],
    default_provider_id: profile.default_provider_id ?? null,
    max_cost:
      profile.max_cost === null || profile.max_cost === undefined
        ? ''
        : String(profile.max_cost),
    max_latency_ms:
      profile.max_latency_ms === null || profile.max_latency_ms === undefined
        ? ''
        : String(profile.max_latency_ms),
    require_healthy: profile.require_healthy,
    preferred_region: profile.preferred_region ?? '',
  };
}

function numericOrNull(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : null;
}

function integerOrNull(value: string): number | null {
  const parsed = numericOrNull(value);
  if (parsed === null || !Number.isInteger(parsed) || parsed < 0) {
    return null;
  }

  return parsed;
}

function toPreviewFormState(
  summary: PortalRoutingSummary,
  decisionLogs: PortalRoutingDecisionLog[],
): RoutingPreviewFormState {
  const latestLog = [...decisionLogs].sort(
    (left, right) => right.created_at_ms - left.created_at_ms,
  )[0];

  return {
    capability: '',
    model: summary.latest_model_hint,
    requested_region: summary.preferences.preferred_region ?? '',
    selection_seed:
      latestLog?.selection_seed === null || latestLog?.selection_seed === undefined
        ? ''
        : String(latestLog.selection_seed),
  };
}

function syncPreviewFormState(
  current: RoutingPreviewFormState | null,
  summary: PortalRoutingSummary,
  decisionLogs: PortalRoutingDecisionLog[],
): RoutingPreviewFormState {
  if (!current) {
    return toPreviewFormState(summary, decisionLogs);
  }

  return {
    capability: current.capability,
    model: current.model || summary.latest_model_hint,
    requested_region: current.requested_region,
    selection_seed: current.selection_seed,
  };
}

function reorderProviders(
  providers: PortalRoutingProviderOption[],
  orderedProviderIds: string[],
): PortalRoutingProviderOption[] {
  return [...providers].sort((left, right) => {
    const leftIndex = orderedProviderIds.indexOf(left.provider_id);
    const rightIndex = orderedProviderIds.indexOf(right.provider_id);
    const resolvedLeft = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
    const resolvedRight = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
    return resolvedLeft - resolvedRight;
  });
}

function reorderProviderIds(
  orderedProviderIds: string[],
  providerId: string,
  direction: -1 | 1,
): string[] {
  const currentIndex = orderedProviderIds.indexOf(providerId);
  const nextIndex = currentIndex + direction;

  if (currentIndex === -1 || nextIndex < 0 || nextIndex >= orderedProviderIds.length) {
    return orderedProviderIds;
  }

  const nextOrder = [...orderedProviderIds];
  const [moved] = nextOrder.splice(currentIndex, 1);
  nextOrder.splice(nextIndex, 0, moved);
  return nextOrder;
}

function applyPresetState(current: RoutingFormState, presetId: string): RoutingFormState {
  switch (presetId) {
    case 'predictable':
      return {
        ...current,
        preset_id: 'predictable',
        strategy: 'deterministic_priority',
        require_healthy: false,
      };
    case 'distribution':
      return {
        ...current,
        preset_id: 'distribution',
        strategy: 'weighted_random',
        require_healthy: false,
      };
    case 'reliability':
      return {
        ...current,
        preset_id: 'reliability',
        strategy: 'slo_aware',
        require_healthy: true,
        max_latency_ms: current.max_latency_ms || RELIABILITY_LATENCY_TARGET_MS,
      };
    case 'regional':
      return {
        ...current,
        preset_id: 'regional',
        strategy: 'geo_affinity',
        preferred_region: current.preferred_region || 'us-east',
      };
    default:
      return current;
  }
}

function searchMatches(query: string, values: Array<string | null | undefined>): boolean {
  if (!query) {
    return true;
  }

  return values.filter(Boolean).join(' ').toLowerCase().includes(query);
}

function evidenceStatusTone(log: PortalRoutingDecisionLog) {
  if (log.slo_degraded) {
    return 'warning';
  }

  if (log.slo_applied) {
    return 'success';
  }

  if (log.decision_source.toLowerCase().includes('preview')) {
    return 'default';
  }

  return 'secondary';
}

function evidenceFocus(log: PortalRoutingDecisionLog): string {
  if (log.decision_source.toLowerCase().includes('preview')) {
    return 'preview';
  }

  if (log.slo_applied || log.slo_degraded) {
    return 'guardrailed';
  }

  return 'live';
}

function buildWorkbenchConfig(
  lane: RoutingWorkbenchLane,
  t: TranslateFn,
): RoutingWorkbenchConfig {
  switch (lane) {
    case 'presets':
      return {
        laneLabel: t('Preset catalog'),
        scopeLabel: t('Strategy'),
        detailLabel: t('Operational detail'),
        actionsLabel: t('Actions'),
        detail: t(
          'Preset catalog converts routing strategy values into product choices that an operator can apply without reading implementation details.',
        ),
        emptyTitle: t('No routing presets in this slice'),
        emptyDetail: t('Adjust the operational focus or search to reveal a different routing preset.'),
        focusOptions: [
          { value: 'all', label: t('All presets') },
          { value: 'active', label: t('Active preset') },
          { value: 'available', label: t('Available presets') },
        ],
      };
    case 'evidence':
      return {
        laneLabel: t('Evidence stream'),
        scopeLabel: t('Routing signal'),
        detailLabel: t('Selection detail'),
        actionsLabel: t('Trace'),
        detail: t(
          'Evidence stream keeps preview and live routing traces on one operational table instead of splitting them across tabs.',
        ),
        emptyTitle: t('No routing evidence in this slice'),
        emptyDetail: t('Run a preview or send live traffic and routing evidence will appear here.'),
        focusOptions: [
          { value: 'all', label: t('All evidence') },
          { value: 'preview', label: t('Preview traces') },
          { value: 'live', label: t('Live traces') },
          { value: 'guardrailed', label: t('Guardrailed') },
        ],
      };
    case 'providers':
    default:
      return {
        laneLabel: t('Provider roster'),
        scopeLabel: t('Channel and order'),
        detailLabel: t('Routing role'),
        actionsLabel: t('Actions'),
        detail: t(
          'Provider roster keeps ordered fallback, default provider, and channel coverage inside one workbench so operations can adjust posture without digging through forms.',
        ),
        emptyTitle: t('No providers in this slice'),
        emptyDetail: t('Routing provider options will appear once the project summary is available.'),
        focusOptions: [
          { value: 'all', label: t('All providers') },
          { value: 'default', label: t('Default provider') },
          { value: 'ordered', label: t('Ordered providers') },
        ],
      };
  }
}

function buildPreviewOutcomeCards(
  preview: PortalRoutingDecision,
  t: TranslateFn,
): RoutingCardItem[] {
  return [
    {
      id: 'preview-provider',
      label: t('Selected provider'),
      value: preview.selected_provider_id,
      detail: t('The provider chosen by the latest routing preview.'),
      tone: 'success',
    },
    {
      id: 'preview-reason',
      label: t('Selection reason'),
      value: preview.selection_reason ?? t('Top-ranked eligible provider'),
      detail: t('The current preview explains why the selected provider won the route.'),
    },
    {
      id: 'preview-candidates',
      label: t('Candidate path'),
      value: preview.candidate_ids.join(' -> ') || t('No candidates'),
      detail: t('Candidate order remains visible so fallback posture is explainable.'),
    },
    {
      id: 'preview-slo',
      label: t('SLO posture'),
      value: preview.slo_degraded
        ? t('Degraded fallback')
        : preview.slo_applied
          ? t('Guardrails applied')
          : t('No active guardrails'),
      detail: preview.matched_policy_id
        ? t('Matched policy {policyId}.', { policyId: preview.matched_policy_id })
        : t('No routing policy matched the current preview inputs.'),
      tone: preview.slo_degraded ? 'warning' : preview.slo_applied ? 'success' : 'secondary',
    },
    {
      id: 'preview-snapshot',
      label: t('Compiled snapshot'),
      value: preview.compiled_routing_snapshot_id ?? t('No snapshot captured'),
      detail: t(
        'Selection evidence is linked to the compiled route state when a snapshot id is available.',
      ),
      tone: preview.compiled_routing_snapshot_id ? 'default' : 'secondary',
    },
    {
      id: 'preview-fallback',
      label: t('Fallback posture'),
      value: preview.fallback_reason ?? t('No fallback used'),
      detail: t(
        'Fallback reasoning is preserved so operators can distinguish degraded routing from normal preference selection.',
      ),
      tone: preview.fallback_reason ? 'warning' : 'secondary',
    },
  ];
}

export function PortalRoutingPage({ onNavigate }: PortalRoutingPageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<PortalRoutingSummary | null>(null);
  const [decisionLogs, setDecisionLogs] = useState<PortalRoutingDecisionLog[]>([]);
  const [preview, setPreview] = useState<PortalRoutingDecision | null>(null);
  const [form, setForm] = useState<RoutingFormState | null>(null);
  const [previewForm, setPreviewForm] = useState<RoutingPreviewFormState | null>(null);
  const [status, setStatus] = useState(t('Loading routing posture...'));
  const [saving, setSaving] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [previewDialogOpen, setPreviewDialogOpen] = useState(false);
  const [profilesDialogOpen, setProfilesDialogOpen] = useState(false);
  const [snapshotsDialogOpen, setSnapshotsDialogOpen] = useState(false);
  const [snapshotSearchQuery, setSnapshotSearchQuery] = useState('');
  const [routingProfiles, setRoutingProfiles] = useState<RoutingProfileRecord[]>([]);
  const [routingSnapshots, setRoutingSnapshots] = useState<PortalCompiledRoutingSnapshotRecord[]>(
    [],
  );
  const [loadingRoutingProfiles, setLoadingRoutingProfiles] = useState(false);
  const [loadingRoutingSnapshots, setLoadingRoutingSnapshots] = useState(false);
  const [profileStatus, setProfileStatus] = useState('');
  const [snapshotStatus, setSnapshotStatus] = useState('');
  const [workbenchLane, setWorkbenchLane] = useState<RoutingWorkbenchLane>('providers');
  const [focusFilter, setFocusFilter] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  async function refresh() {
    const [nextSummary, nextLogs] = await Promise.all([
      loadPortalRoutingSummary(),
      loadPortalRoutingDecisionLogs(),
    ]);

    setSummary(nextSummary);
    setPreview(nextSummary.preview);
    setForm(toFormState(nextSummary));
    setDecisionLogs(nextLogs);
    setPreviewForm((current) => syncPreviewFormState(current, nextSummary, nextLogs));
  }

  useEffect(() => {
    let cancelled = false;

    void refresh()
      .then(() => {
        if (!cancelled) {
          setStatus(
            t(
              'Routing workbench is synced with the latest project posture, provider order, and decision evidence.',
            ),
          );
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [t]);

  useEffect(() => {
    if (!profilesDialogOpen) {
      return;
    }

    let cancelled = false;
    setLoadingRoutingProfiles(true);
    setProfileStatus(t('Loading routing profiles...'));

    void loadPortalRoutingProfiles()
      .then((profiles) => {
        if (!cancelled) {
          setRoutingProfiles(profiles);
          setProfileStatus('');
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setProfileStatus(portalErrorMessage(error));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingRoutingProfiles(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [profilesDialogOpen, t]);

  useEffect(() => {
    if (!snapshotsDialogOpen) {
      return;
    }

    let cancelled = false;
    setLoadingRoutingSnapshots(true);
    setSnapshotStatus(t('Loading compiled snapshots...'));

    void loadPortalRoutingSnapshots()
      .then((snapshots) => {
        if (!cancelled) {
          setRoutingSnapshots(snapshots);
          setSnapshotStatus('');
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSnapshotStatus(portalErrorMessage(error));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingRoutingSnapshots(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [snapshotsDialogOpen, t]);

  const viewModel = useMemo(() => {
    if (!summary) {
      return null;
    }

    return buildPortalRoutingViewModel(summary, decisionLogs, preview);
  }, [decisionLogs, preview, summary, t]);

  const orderedProviders = useMemo(() => {
    if (!viewModel || !form) {
      return [];
    }

    return reorderProviders(viewModel.provider_options, form.ordered_provider_ids);
  }, [form, viewModel]);

  const summaryCards = useMemo(() => {
    if (!viewModel || !form || !previewForm) {
      return [];
    }

    return [
      {
        id: 'summary-posture',
        label: t('Active posture'),
        value: buildRoutingStrategyLabel(form.strategy),
        detail: t(
          'Routing strategy is translated into user-facing posture language instead of raw enum names.',
        ),
        tone: 'success' as const,
      },
      {
        id: 'summary-default',
        label: t('Default provider'),
        value: form.default_provider_id ?? t('Auto fallback'),
        detail: t(
          'Default provider acts as the stable fallback when multiple candidates remain eligible.',
        ),
      },
      {
        id: 'summary-model',
        label: t('Preview model'),
        value: previewForm.model || viewModel.summary.latest_model_hint,
        detail: t(
          'The current preview model stays visible so operators always know which workload is being tuned.',
        ),
      },
      {
        id: 'summary-evidence',
        label: t('Evidence entries'),
        value: String(decisionLogs.length),
        detail: t(
          'Preview and live routing traces remain close to the posture editor for faster diagnosis.',
        ),
      },
    ];
  }, [decisionLogs.length, form, previewForm, t, viewModel]);

  const workbenchConfig = useMemo(
    () => buildWorkbenchConfig(workbenchLane, t),
    [t, workbenchLane],
  );

  const workbenchRows = useMemo<RoutingWorkbenchRow[]>(() => {
    if (!viewModel || !form) {
      return [];
    }

    if (workbenchLane === 'presets') {
      return viewModel.preset_cards.map((preset) => ({
        id: preset.id,
        focus: preset.active ? 'active' : 'available',
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{preset.title}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">{preset.id}</p>
          </div>
        ),
        scope: buildRoutingStrategyLabel(preset.strategy),
        status: (
          <Badge variant={preset.active ? 'success' : 'secondary'}>
            {preset.active ? t('Active') : t('Available')}
          </Badge>
        ),
        detail: t(preset.detail),
        actions: (
          <Button
            onClick={() => {
              setForm((current) => (current ? applyPresetState(current, preset.id) : current));
              setStatus(
                t(
                  'Preset applied locally. Save posture when the updated routing shape looks right.',
                ),
              );
            }}
            variant={preset.active ? 'secondary' : 'primary'}
          >
            {preset.active ? t('Active preset') : t('Apply preset')}
          </Button>
        ),
        searchText: [preset.title, preset.detail, preset.id, preset.strategy]
          .join(' ')
          .toLowerCase(),
      }));
    }

    if (workbenchLane === 'evidence') {
      return [...viewModel.logs]
        .sort((left, right) => right.created_at_ms - left.created_at_ms)
        .map((log) => ({
          id: log.decision_id,
          focus: evidenceFocus(log),
          subject: (
            <div className="space-y-1">
              <strong className="text-zinc-950 dark:text-zinc-50">
                {log.route_key} -&gt; {log.selected_provider_id}
              </strong>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                {new Date(log.created_at_ms).toLocaleString()}
              </p>
            </div>
          ),
          scope: (
            <div className="space-y-1">
              <div>{buildRoutingStrategyLabel(log.strategy)}</div>
              <div className="text-xs text-zinc-500 dark:text-zinc-400">
                {buildRoutingCapabilityLabel(log.capability)}
                {log.requested_region ? ` / ${log.requested_region}` : ''}
              </div>
            </div>
          ),
        status: (
          <Badge variant={evidenceStatusTone(log)}>
            {log.slo_degraded
              ? t('Degraded')
              : log.slo_applied
                  ? t('Guardrailed')
                  : log.decision_source.toLowerCase().includes('preview')
                    ? t('Preview')
                    : t('Live')}
            </Badge>
          ),
          detail:
            log.fallback_reason
            ?? log.selection_reason
            ?? log.assessments[0]?.reasons[0]
            ?? t('Selection evidence is available from the current routing trace.'),
          actions: (
            <div className="space-y-2 text-xs text-zinc-500 dark:text-zinc-400">
              <div>{buildRoutingDecisionSourceLabel(log.decision_source)}</div>
              <div>{log.matched_policy_id ?? t('No matched policy')}</div>
              <div>{log.fallback_reason ?? t('No fallback used')}</div>
              {log.compiled_routing_snapshot_id ? (
                <Button
                  onClick={() => openSnapshotEvidence(log.compiled_routing_snapshot_id)}
                  variant="ghost"
                >
                  {t('Open snapshot evidence')}
                </Button>
              ) : null}
            </div>
          ),
          searchText: [
            log.route_key,
            log.selected_provider_id,
            log.strategy,
            log.decision_source,
            buildRoutingDecisionSourceLabel(log.decision_source),
            log.selection_reason,
            log.fallback_reason,
            log.compiled_routing_snapshot_id,
            log.requested_region,
            log.capability,
            buildRoutingCapabilityLabel(log.capability),
          ]
            .filter(Boolean)
            .join(' ')
            .toLowerCase(),
        }));
    }

    return orderedProviders.map((provider, index) => {
      const isDefault = form.default_provider_id === provider.provider_id;

      return {
        id: provider.provider_id,
        focus: isDefault ? 'default' : 'ordered',
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{provider.display_name}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">{provider.provider_id}</p>
          </div>
        ),
        scope: (
          <div className="space-y-1">
            <div>{provider.channel_id}</div>
            <div className="text-xs text-zinc-500 dark:text-zinc-400">
              {t('Priority #{priority}', { priority: index + 1 })}
            </div>
          </div>
        ),
        status: (
          <Badge variant={isDefault ? 'default' : 'success'}>
            {isDefault ? t('Default') : t('Ordered')}
          </Badge>
        ),
        detail: isDefault
          ? t(
              'Default provider stays available as the stable fallback when several providers remain eligible.',
            )
          : t(
              'Ordered providers keep deterministic failover readable for operators and support teams.',
            ),
        actions: (
          <div className="flex flex-wrap gap-2">
            <Button
              disabled={index === 0}
              onClick={() => {
                setForm((current) =>
                  current
                    ? {
                        ...current,
                        ordered_provider_ids: reorderProviderIds(
                          current.ordered_provider_ids,
                          provider.provider_id,
                          -1,
                        ),
                      }
                    : current,
                );
                setStatus(
                  t(
                    'Provider order changed locally. Save posture to publish the new fallback order.',
                  ),
                );
              }}
              variant="ghost"
            >
              {t('Move up')}
            </Button>
            <Button
              disabled={index === orderedProviders.length - 1}
              onClick={() => {
                setForm((current) =>
                  current
                    ? {
                        ...current,
                        ordered_provider_ids: reorderProviderIds(
                          current.ordered_provider_ids,
                          provider.provider_id,
                          1,
                        ),
                      }
                    : current,
                );
                setStatus(
                  t(
                    'Provider order changed locally. Save posture to publish the new fallback order.',
                  ),
                );
              }}
              variant="ghost"
            >
              {t('Move down')}
            </Button>
            <Button
              onClick={() => {
                setForm((current) =>
                  current
                    ? { ...current, default_provider_id: provider.provider_id }
                    : current,
                );
                setStatus(
                  t('Default provider updated locally. Save posture to publish the change.'),
                );
              }}
              variant={isDefault ? 'secondary' : 'primary'}
            >
              {isDefault ? t('Default provider') : t('Set default')}
            </Button>
          </div>
        ),
        searchText: [
          provider.display_name,
          provider.provider_id,
          provider.channel_id,
          String(index + 1),
          isDefault ? 'default' : 'ordered',
        ]
          .join(' ')
          .toLowerCase(),
      };
    });
  }, [form, openSnapshotEvidence, orderedProviders, t, viewModel, workbenchLane]);

  const visibleWorkbenchRows = useMemo(
    () =>
      workbenchRows.filter(
        (row) =>
          (focusFilter === 'all' || row.focus === focusFilter)
          && searchMatches(deferredSearch, [row.searchText]),
      ),
    [deferredSearch, focusFilter, workbenchRows],
  );

  const previewOutcomeCards = useMemo(
    () => (viewModel ? buildPreviewOutcomeCards(viewModel.preview, t) : []),
    [t, viewModel],
  );

  async function handleSave(event?: FormEvent<HTMLFormElement>): Promise<void> {
    event?.preventDefault();
    if (!form) {
      return;
    }

    setSaving(true);
    setStatus(t('Saving routing preferences for this project...'));

    try {
      await updatePortalRoutingPreferences({
        preset_id: form.preset_id,
        strategy: form.strategy,
        ordered_provider_ids: form.ordered_provider_ids,
        default_provider_id: form.default_provider_id,
        max_cost: numericOrNull(form.max_cost),
        max_latency_ms: integerOrNull(form.max_latency_ms),
        require_healthy: form.require_healthy,
        preferred_region: form.preferred_region || null,
      });
      await refresh();
      setEditDialogOpen(false);
      setStatus(
        t(
          'Routing posture saved. The workbench now reflects the updated provider order and guardrails.',
        ),
      );
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSaving(false);
    }
  }

  async function handlePreview(event?: FormEvent<HTMLFormElement>): Promise<void> {
    event?.preventDefault();
    if (!summary || !form || !previewForm) {
      return;
    }

    setPreviewing(true);
    setStatus(t('Previewing the active route...'));

    try {
      const capability = previewForm.capability.trim() || DEFAULT_ROUTING_CAPABILITY;
      const model = previewForm.model.trim() || summary.latest_model_hint;
      const requested_region = previewForm.requested_region.trim() || form.preferred_region || null;
      const selection_seed = integerOrNull(previewForm.selection_seed);

      const nextPreview = await runPortalRoutingPreview({
        capability,
        model,
        requested_region,
        selection_seed,
      });

      setPreview(nextPreview);
      const nextLogs = await loadPortalRoutingDecisionLogs();
      setDecisionLogs(nextLogs);
      setPreviewForm((current) => ({
        capability,
        model,
        requested_region: requested_region ?? '',
        selection_seed:
          selection_seed === null ? current?.selection_seed ?? '' : String(selection_seed),
      }));
      setPreviewDialogOpen(false);
      setWorkbenchLane('evidence');
      setFocusFilter('preview');
      setStatus(
        t('Preview updated with the current routing posture and added to the evidence stream.'),
      );
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setPreviewing(false);
    }
  }

  async function refreshRoutingProfilesAfterMutation(successMessage: string): Promise<void> {
    setLoadingRoutingProfiles(true);

    try {
      const profiles = await loadPortalRoutingProfiles();
      setRoutingProfiles(profiles);
      setProfileStatus(successMessage);
      setStatus(successMessage);
    } finally {
      setLoadingRoutingProfiles(false);
    }
  }

  async function handleCreateRoutingProfile(input: {
    name: string;
    slug?: string | null;
    description?: string | null;
    active?: boolean;
    strategy?: string;
    ordered_provider_ids?: string[];
    default_provider_id?: string | null;
    max_cost?: number | null;
    max_latency_ms?: number | null;
    require_healthy?: boolean;
    preferred_region?: string | null;
  }): Promise<void> {
    setProfileStatus(t('Saving...'));

    try {
      await issuePortalRoutingProfile(input);
      await refreshRoutingProfilesAfterMutation(
        t('Routing profile saved. API key groups can now bind to it.'),
      );
    } catch (error) {
      const message = portalErrorMessage(error);
      setStatus(message);
      throw error;
    }
  }

  function handleApplyRoutingProfile(profile: RoutingProfileRecord): void {
    setForm(toFormStateFromProfile(profile));
    setPreviewForm((current) =>
      current
        ? {
            ...current,
            requested_region: profile.preferred_region ?? '',
          }
        : current,
    );
    setProfilesDialogOpen(false);
    setStatus(
      t('Profile loaded into the current routing posture. Save posture when you are ready to publish it.'),
    );
  }

  function openSnapshotEvidence(searchValue?: string | null): void {
    setSnapshotSearchQuery(searchValue ?? '');
    setSnapshotsDialogOpen(true);
  }

  if (!viewModel || !form || !previewForm) {
    return (
      <WorkspacePanel
        description={status}
        title={t('Preparing routing workbench')}
      >
        <EmptyState
          description={t(
            'Routing posture will appear once the portal finishes loading project summary, provider options, and decision evidence.',
          )}
          title={t('Preparing routing workbench')}
        />
      </WorkspacePanel>
    );
  }

  const focusLabel =
    t(workbenchConfig.focusOptions.find((option) => option.value === focusFilter)?.label ?? 'All');
  const guardrailCards: RoutingCardItem[] = viewModel.guardrails.map((item) => {
    const tone: RoutingCardItem['tone'] =
      item.id === 'provider-default'
        ? 'default'
        : item.value === 'Open'
          ? 'secondary'
          : 'success';

    return {
      id: item.id,
      label: item.label,
      value: item.value === 'Open' ? t('Open') : item.value === 'Auto' ? t('Auto') : item.value,
      detail: item.detail,
      tone,
    };
  });
  const latestSignals = viewModel.evidence.slice(0, 3);
  const previewAssessments = viewModel.preview.assessments.slice(0, 4);
  const previewStatusTone =
    viewModel.preview.slo_degraded
      ? 'warning'
      : viewModel.preview.slo_applied
        ? 'success'
        : 'outline';
  const previewStatusLabel = viewModel.preview.slo_degraded
    ? t('Degraded fallback')
    : viewModel.preview.slo_applied
      ? t('Guardrails applied')
      : t('Preview only');

  const headerActions = (
    <div
      data-slot="portal-routing-toolbar"
      className="flex flex-wrap items-center gap-2"
    >
      <Button onClick={() => setEditDialogOpen(true)} variant="primary">
        {t('Edit posture')}
      </Button>
      <Button onClick={() => setPreviewDialogOpen(true)} variant="secondary">
        {t('Run preview')}
      </Button>
      <Button onClick={() => setProfilesDialogOpen(true)} variant="secondary">
        {t('Manage routing profiles')}
      </Button>
      <Button onClick={() => setSnapshotsDialogOpen(true)} variant="secondary">
        {t('View compiled snapshots')}
      </Button>
      <Button onClick={() => onNavigate('usage')} variant="secondary">
        {t('Open usage')}
      </Button>
      <Button onClick={() => onNavigate('api-keys')} variant="ghost">
        {t('Validate with a key')}
      </Button>
    </div>
  );

  return (
    <>
      <PortalRoutingProfilesDialog
        currentPosture={{
          strategy: form.strategy,
          ordered_provider_ids: form.ordered_provider_ids,
          default_provider_id: form.default_provider_id,
          max_cost: numericOrNull(form.max_cost),
          max_latency_ms: integerOrNull(form.max_latency_ms),
          require_healthy: form.require_healthy,
          preferred_region: form.preferred_region || null,
        }}
        loadingProfiles={loadingRoutingProfiles}
        onApplyProfile={handleApplyRoutingProfile}
        onCreateProfile={handleCreateRoutingProfile}
        onOpenChange={setProfilesDialogOpen}
        open={profilesDialogOpen}
        profileStatus={profileStatus}
        profiles={routingProfiles}
      />

      <PortalRoutingSnapshotsDialog
        loadingSnapshots={loadingRoutingSnapshots}
        onOpenChange={setSnapshotsDialogOpen}
        open={snapshotsDialogOpen}
        snapshotStatus={snapshotStatus}
        snapshots={routingSnapshots}
        suggestedSearchQuery={snapshotSearchQuery}
      />

      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('Edit routing posture')}</DialogTitle>
            <DialogDescription>
              {t(
                'Save posture after adjusting profile label, strategy, regional preference, and reliability guardrails.',
              )}
            </DialogDescription>
          </DialogHeader>

          <form className="grid gap-4 md:grid-cols-2" onSubmit={(event) => void handleSave(event)}>
            <SettingsField label={t('Routing profile label')} layout="vertical">
              <Input
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setForm({ ...form, preset_id: event.target.value })
                }
                placeholder={t('Example: Balanced production posture')}
                value={form.preset_id}
              />
            </SettingsField>
            <SettingsField label={t('Strategy')} layout="vertical">
              <Select
                onValueChange={(value) =>
                  setForm({
                    ...form,
                    strategy: value as PortalRoutingPreferences['strategy'],
                  })
                }
                value={form.strategy}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t('Strategy')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="deterministic_priority">{t('Predictable order')}</SelectItem>
                  <SelectItem value="weighted_random">{t('Traffic distribution')}</SelectItem>
                  <SelectItem value="slo_aware">{t('Reliability guardrails')}</SelectItem>
                  <SelectItem value="geo_affinity">{t('Regional preference')}</SelectItem>
                </SelectContent>
              </Select>
            </SettingsField>
            <SettingsField label={t('Max cost')} layout="vertical">
              <Input
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setForm({ ...form, max_cost: event.target.value })
                }
                placeholder={t('Example: 0.30 USD ceiling')}
                value={form.max_cost}
              />
            </SettingsField>
            <SettingsField label={t('Max latency ms')} layout="vertical">
              <Input
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setForm({ ...form, max_latency_ms: event.target.value })
                }
                placeholder={t('Example: 250 ms target')}
                value={form.max_latency_ms}
              />
            </SettingsField>
            <SettingsField label={t('Preferred region')} layout="vertical">
              <Select
                onValueChange={(value) =>
                  setForm({
                    ...form,
                    preferred_region:
                      value === AUTO_REGION_SELECT_VALUE ? '' : value,
                  })
                }
                value={form.preferred_region || AUTO_REGION_SELECT_VALUE}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t('Preferred region')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={AUTO_REGION_SELECT_VALUE}>{t('Auto')}</SelectItem>
                  <SelectItem value="us-east">us-east</SelectItem>
                  <SelectItem value="us-west">us-west</SelectItem>
                  <SelectItem value="eu-west">eu-west</SelectItem>
                  <SelectItem value="ap-southeast">ap-southeast</SelectItem>
                </SelectContent>
              </Select>
            </SettingsField>
            <SettingsField label={t('Default provider')} layout="vertical">
              <Select
                onValueChange={(value) =>
                  setForm({
                    ...form,
                    default_provider_id:
                      value === AUTO_PROVIDER_SELECT_VALUE ? null : value,
                  })
                }
                value={form.default_provider_id ?? AUTO_PROVIDER_SELECT_VALUE}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t('Default provider')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={AUTO_PROVIDER_SELECT_VALUE}>{t('Auto fallback')}</SelectItem>
                  {orderedProviders.map((provider) => (
                    <SelectItem key={provider.provider_id} value={provider.provider_id}>
                      {provider.display_name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </SettingsField>
            <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60 md:col-span-2">
              <Label className="flex items-center gap-3 text-sm font-medium text-zinc-700 dark:text-zinc-300">
                <Checkbox
                  checked={form.require_healthy}
                  onCheckedChange={(nextChecked) =>
                    setForm({
                      ...form,
                      require_healthy: nextChecked === true,
                    })
                  }
                />
                <span>{t('Require healthy providers')}</span>
              </Label>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {t(
                  'Reliability guardrails bias routing toward healthy, lower-risk providers before traffic leaves the workspace.',
                )}
              </p>
            </div>
            <DialogFooter className="md:col-span-2">
              <Button onClick={() => setEditDialogOpen(false)} variant="ghost">
                {t('Cancel')}
              </Button>
              <Button disabled={saving} variant="primary" type="submit">
                {saving ? t('Saving...') : t('Save posture')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={previewDialogOpen} onOpenChange={setPreviewDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('Preview route')}</DialogTitle>
            <DialogDescription>
              {t(
                'Preview route inputs are stored separately from the saved posture so operators can test scenarios before traffic shifts.',
              )}
            </DialogDescription>
          </DialogHeader>

          <form className="grid gap-4 md:grid-cols-2" onSubmit={(event) => void handlePreview(event)}>
            <SettingsField label={t('Capability')} layout="vertical">
              <Input
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setPreviewForm({ ...previewForm, capability: event.target.value })
                }
                placeholder={t('Example: Chat completions')}
                value={previewForm.capability}
              />
            </SettingsField>
            <SettingsField label={t('Requested model')} layout="vertical">
              <Input
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setPreviewForm({ ...previewForm, model: event.target.value })
                }
                placeholder={viewModel.summary.latest_model_hint}
                value={previewForm.model}
              />
            </SettingsField>
            <SettingsField label={t('Requested region')} layout="vertical">
              <Select
                onValueChange={(value) =>
                  setPreviewForm({
                    ...previewForm,
                    requested_region:
                      value === AUTO_REGION_SELECT_VALUE ? '' : value,
                  })
                }
                value={previewForm.requested_region || AUTO_REGION_SELECT_VALUE}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t('Requested region')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={AUTO_REGION_SELECT_VALUE}>{t('Auto')}</SelectItem>
                  <SelectItem value="us-east">us-east</SelectItem>
                  <SelectItem value="us-west">us-west</SelectItem>
                  <SelectItem value="eu-west">eu-west</SelectItem>
                  <SelectItem value="ap-southeast">ap-southeast</SelectItem>
                </SelectContent>
              </Select>
            </SettingsField>
            <SettingsField label={t('Selection seed')} layout="vertical">
              <Input
                inputMode="numeric"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setPreviewForm({ ...previewForm, selection_seed: event.target.value })
                }
                placeholder={t('Optional deterministic seed')}
                value={previewForm.selection_seed}
              />
            </SettingsField>
            <DialogFooter className="md:col-span-2">
              <Button onClick={() => setPreviewDialogOpen(false)} variant="ghost">
                {t('Close')}
              </Button>
              <Button disabled={previewing} variant="primary" type="submit">
                {previewing ? t('Running preview...') : t('Run preview')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="space-y-6">
        <SectionHeader
          actions={headerActions}
        description={status}
        eyebrow={t('Routing posture')}
        meta={(
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="default">{buildRoutingStrategyLabel(form.strategy)}</Badge>
            <Badge variant="outline">{`${t('Evidence entries')}: ${decisionLogs.length}`}</Badge>
          </div>
        )}
        title={t('Routing')}
        />

        <div className="grid gap-6 xl:grid-cols-[0.94fr_1.06fr]">
          <WorkspacePanel
            description={t(
              'Routing strategy, default provider, preview model, and evidence count stay visible before operators edit posture or run previews.',
            )}
            title={t('Routing posture')}
          >
            <RoutingCardGrid items={summaryCards} />
          </WorkspacePanel>

          <WorkspacePanel
            description={t(
              'Guardrail posture keeps cost, latency, regional preference, and the latest routing signals readable before you publish changes.',
            )}
            title={t('Guardrail posture')}
          >
            <div className="grid gap-4">
              <RoutingCardGrid columns="xl:grid-cols-2" items={guardrailCards} />

              <section className="grid gap-4">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="space-y-1">
                    <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {t('Latest routing signals')}
                    </strong>
                    <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                      {t(
                        'Preview and live traces stay adjacent to guardrails so posture changes remain explainable without secondary tabs.',
                      )}
                    </p>
                  </div>
                  <Badge variant="default">{t('{count} signals', { count: latestSignals.length })}</Badge>
                </div>

                {latestSignals.length ? (
                  <div className="grid gap-3">
                    {latestSignals.map((item) => (
                      <article
                        key={item.id}
                        className="rounded-[20px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70"
                      >
                        <div className="space-y-1">
                          <strong className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {item.title}
                          </strong>
                          <p className="text-xs uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                            {item.timestamp_label}
                          </p>
                        </div>
                        <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                          {item.detail}
                        </p>
                        {item.fallback_reason ? (
                          <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                            {item.fallback_reason}
                          </p>
                        ) : null}
                        {item.snapshot_id ? (
                          <div className="mt-3">
                            <Button
                              onClick={() => openSnapshotEvidence(item.snapshot_id)}
                              variant="ghost"
                            >
                              {t('Open snapshot evidence')}
                            </Button>
                          </div>
                        ) : null}
                      </article>
                    ))}
                  </div>
                ) : (
                  <EmptyState
                    description={t('Run a preview or wait for live traffic to collect routing signals.')}
                    title={t('No routing signals yet')}
                  />
                )}
              </section>
            </div>
          </WorkspacePanel>
        </div>

        <ManagementWorkbench
          description={t(workbenchConfig.detail)}
          detail={{
            children: (
              <div className="grid gap-4">
                <RoutingCardGrid columns="xl:grid-cols-2" items={previewOutcomeCards} />

                {viewModel.preview.compiled_routing_snapshot_id ? (
                  <div className="flex flex-wrap gap-2">
                    <Button
                      onClick={() =>
                        openSnapshotEvidence(viewModel.preview.compiled_routing_snapshot_id)
                      }
                      variant="secondary"
                    >
                      {t('Open snapshot evidence')}
                    </Button>
                  </div>
                ) : null}

                <section className="grid gap-4">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="space-y-1">
                      <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {t('Candidate assessments')}
                      </strong>
                      <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                      {t(
                        'Selection evidence stays operationally readable so support teams can validate health, latency, and policy posture before rollout.',
                      )}
                    </p>
                  </div>
                    <Badge variant={previewStatusTone}>
                      {previewStatusLabel}
                    </Badge>
                  </div>

                  {previewAssessments.length ? (
                    <div className="grid gap-3">
                      {previewAssessments.map((assessment) => (
                        <article
                          key={assessment.provider_id}
                          className="rounded-[20px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70"
                        >
                          <div className="flex flex-wrap items-start justify-between gap-3">
                            <div className="space-y-1">
                              <strong className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                                {assessment.provider_id}
                              </strong>
                              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                                {assessment.region ? `${assessment.region} / ` : ''}
                                {assessment.available ? t('Available') : t('Not available')}
                              </p>
                            </div>
                            <Badge variant={assessment.available ? 'success' : 'warning'}>
                              {buildRoutingAssessmentHealthLabel(assessment.health)}
                            </Badge>
                          </div>

                          <div className="mt-3 flex flex-wrap gap-2 text-xs text-zinc-500 dark:text-zinc-400">
                            <span>{t('Rank {rank}', { rank: assessment.policy_rank })}</span>
                            <span>
                              {t('Latency {latency}', {
                                latency:
                                  assessment.latency_ms === null || assessment.latency_ms === undefined
                                    ? t('No sample')
                                    : `${assessment.latency_ms}ms`,
                              })}
                            </span>
                            <span>
                              {t('Cost {cost}', {
                                cost:
                                  assessment.cost === null || assessment.cost === undefined
                                    ? t('No sample')
                                    : `$${assessment.cost.toFixed(4)}`,
                              })}
                            </span>
                          </div>

                          <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                            {assessment.reasons[0]
                              ?? assessment.slo_violations[0]
                              ?? t('The preview did not expose additional assessment detail for this provider.')}
                          </p>
                        </article>
                      ))}
                    </div>
                  ) : (
                    <EmptyState
                      description={t('Run a preview to inspect provider-level candidate assessments.')}
                      title={t('No preview assessments yet')}
                    />
                  )}
                </section>
              </div>
            ),
            description: t(
              'Preview outcome keeps the selected provider, fallback path, and provider assessments visible before traffic posture is saved.',
            ),
            summary: (
              <Badge variant={previewStatusTone}>
                {previewStatusLabel}
              </Badge>
            ),
            title: t('Preview outcome'),
          }}
          detailWidth={430}
          eyebrow={t('Routing posture')}
        filters={(
          <div className="grid gap-4">
            <div className="flex flex-wrap items-center gap-3 text-sm text-zinc-500 dark:text-zinc-400">
                <Badge variant="outline">{t(workbenchConfig.laneLabel)}</Badge>
                <span>{t('{visible} of {total} rows visible', {
                  visible: visibleWorkbenchRows.length,
                  total: workbenchRows.length,
                })}</span>
                <span>{t('Focus: {focus}', { focus: focusLabel })}</span>
              </div>

              <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                {t(
                  'Routing workbench keeps Provider roster, Preset catalog, and Evidence stream inside one operator table while edit and preview actions stay inside focused dialogs.',
                )}
              </p>

              <FilterBar data-slot="portal-routing-filter-bar">
                <FilterBarSection className="min-w-[15rem] flex-[0_1_20rem]" grow={false}>
                  <FilterField
                    className="w-full"
                    controlClassName="min-w-0"
                    label={t('Search routing evidence')}
                  >
                    <SearchInput
                      onChange={(event) => setSearchQuery(event.target.value)}
                      placeholder={t('Search routing evidence')}
                      value={searchQuery}
                    />
                  </FilterField>
                </FilterBarSection>
                <FilterBarSection className="min-w-[12rem] shrink-0" grow={false}>
                  <FilterField className="w-full" label={t('Workbench lane')}>
                    <Select
                      onValueChange={(value) => {
                        setWorkbenchLane(value as RoutingWorkbenchLane);
                        setFocusFilter('all');
                      }}
                      value={workbenchLane}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('Workbench lane')} />
                      </SelectTrigger>
                      <SelectContent>
                        {WORKBENCH_OPTIONS.map((option) => (
                          <SelectItem key={option} value={option}>
                            {option === 'providers'
                              ? t('Provider roster')
                              : option === 'presets'
                                ? t('Preset catalog')
                                : t('Evidence stream')}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </FilterField>
                </FilterBarSection>

                <FilterBarSection className="min-w-[12rem] shrink-0" grow={false}>
                  <FilterField className="w-full" label={t('Operational focus')}>
                    <Select
                      onValueChange={setFocusFilter}
                      value={focusFilter}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('Operational focus')} />
                      </SelectTrigger>
                      <SelectContent>
                        {workbenchConfig.focusOptions.map((option) => (
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
                  { id: 'subject', header: t('Subject'), cell: (row) => row.subject },
                  { id: 'scope', header: t(workbenchConfig.scopeLabel), cell: (row) => row.scope },
                  { id: 'status', header: t('Status'), cell: (row) => row.status },
                  { id: 'detail', header: t(workbenchConfig.detailLabel), cell: (row) => row.detail },
                  {
                    id: 'actions',
                    header: t(workbenchConfig.actionsLabel),
                    cell: (row) => row.actions,
                  },
                ]}
                emptyState={(
                  <div className="mx-auto flex max-w-[34rem] flex-col items-center gap-2 text-center">
                    <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                      {t(workbenchConfig.emptyTitle)}
                    </strong>
                    <p className="text-sm text-zinc-500 dark:text-zinc-400">
                      {t(workbenchConfig.emptyDetail)}
                    </p>
                  </div>
                )}
                getRowId={(row) => row.id}
                rows={visibleWorkbenchRows}
              />
            ),
            description: t(workbenchConfig.detail),
            title: t('Routing workbench'),
          }}
          title={t('Routing workbench')}
        />
      </div>
    </>
  );
}



