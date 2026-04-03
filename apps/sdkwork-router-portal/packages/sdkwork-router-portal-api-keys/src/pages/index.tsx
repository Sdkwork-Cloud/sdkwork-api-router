import { startTransition, useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';

import {
  copyText,
  formatCurrency,
  formatUnits,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
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
  SearchInput,
} from 'sdkwork-router-portal-commons/framework/form';
import {
  Card,
  CardContent,
} from 'sdkwork-router-portal-commons/framework/layout';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  ApiKeyGroupRecord,
  BillingAccountingMode,
  BillingEventSummary,
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
  RoutingProfileRecord,
} from 'sdkwork-router-portal-types';

import {
  PortalApiKeyDrawers,
  PortalApiKeyGroupsDialog,
  PortalApiKeyTable,
} from '../components';
import {
  editPortalApiKeyGroup,
  issuePortalApiKey,
  issuePortalApiKeyGroup,
  loadPortalApiKeyWorkbenchData,
  loadPortalRoutingProfiles,
  removePortalApiKey,
  removePortalApiKeyGroup,
  setPortalApiKeyActive,
  setPortalApiKeyGroupActive,
} from '../repository';
import {
  buildPortalApiKeyGovernanceViewModel,
  buildPortalApiKeyGroupFilterOptions,
  buildPortalApiKeyGroupOptions,
  buildPortalApiKeyEnvironmentOptions,
  buildPortalApiKeyUsagePreview,
  clearPortalApiKeyPlaintextReveal,
  createEmptyPortalApiKeyFormState,
  filterPortalApiKeys,
  readPortalApiKeyPlaintextReveal,
  rememberPortalApiKeyPlaintextReveal,
  resolvePortalApiKeyEnvironment,
  resolvePortalApiKeyExpiresAt,
  resolvePortalApiKeyGroupId,
  resolvePortalApiKeyNotes,
  resolvePortalApiKeyPlaintext,
} from '../services';
import {
  applyApiKeyQuickSetup,
  buildApiKeyQuickSetupPlans,
  listApiKeyInstances,
  resolveGatewayBaseUrl,
  type ApiKeySetupClientId,
  type ApiKeySetupInstance,
} from '../services/quickSetup';
import type {
  PortalApiKeyCreateFormState,
  PortalApiKeyFilterState,
  PortalApiKeysPageProps,
} from '../types';

const API_KEY_PAGE_SIZE = 10;

function clampPage(page: number, totalPages: number) {
  return Math.min(Math.max(page, 1), Math.max(totalPages, 1));
}

function createRecordFromCreatedKey(createdKey: CreatedGatewayApiKey): GatewayApiKeyRecord {
  return {
    active: true,
    created_at_ms: createdKey.created_at_ms,
    environment: createdKey.environment,
    expires_at_ms: createdKey.expires_at_ms ?? null,
    hashed_key: createdKey.hashed,
    api_key_group_id: createdKey.api_key_group_id ?? null,
    label: createdKey.label,
    last_used_at_ms: null,
    notes: createdKey.notes ?? null,
    project_id: createdKey.project_id,
    tenant_id: createdKey.tenant_id,
  };
}

function accountingModeLabel(
  mode: BillingAccountingMode | string | null | undefined,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  switch (mode?.trim().toLowerCase()) {
    case 'platform_credit':
      return t('Platform credit');
    case 'byok':
      return t('BYOK');
    case 'passthrough':
      return t('Passthrough');
    default:
      return t('No accounting override');
  }
}

async function copyPlaintextWithStatus(
  plaintext: string | null,
  setStatus: (value: string) => void,
  t: (text: string, values?: Record<string, string | number>) => string,
) {
  if (!plaintext) {
    return;
  }

  const copied = await copyText(plaintext);
  setStatus(
    copied
      ? t('Plaintext key copied to clipboard.')
      : t('Clipboard copy is unavailable in this browser context.'),
  );
}

export function PortalApiKeysPage({ onNavigate }: PortalApiKeysPageProps) {
  const { t } = usePortalI18n();
  const syncedStatus = t('Credential inventory is synced with the latest project key state.');
  const [apiKeys, setApiKeys] = useState<GatewayApiKeyRecord[]>([]);
  const [apiKeyGroups, setApiKeyGroups] = useState<ApiKeyGroupRecord[]>([]);
  const [billingEventSummary, setBillingEventSummary] = useState<BillingEventSummary | null>(null);
  const [routingProfiles, setRoutingProfiles] = useState<RoutingProfileRecord[]>([]);
  const [createdKey, setCreatedKey] = useState<CreatedGatewayApiKey | null>(null);
  const [filters, setFilters] = useState<PortalApiKeyFilterState>({
    searchQuery: '',
    environment: 'all',
    groupId: 'all',
  });
  const [groupsDialogOpen, setGroupsDialogOpen] = useState(false);
  const [createDrawerOpen, setCreateDrawerOpen] = useState(false);
  const [usageKey, setUsageKey] = useState<GatewayApiKeyRecord | null>(null);
  const [formState, setFormState] = useState<PortalApiKeyCreateFormState>(
    createEmptyPortalApiKeyFormState,
  );
  const [gatewayBaseUrl, setGatewayBaseUrl] = useState('http://127.0.0.1:8080');
  const [openClawInstances, setOpenClawInstances] = useState<ApiKeySetupInstance[]>([]);
  const [loadingInstances, setLoadingInstances] = useState(false);
  const [selectedClientId, setSelectedClientId] = useState<ApiKeySetupClientId>('codex');
  const [selectedInstanceIds, setSelectedInstanceIds] = useState<string[]>([]);
  const [applyingClientId, setApplyingClientId] = useState<ApiKeySetupClientId | null>(null);
  const [usageStatus, setUsageStatus] = useState('');
  const [groupsDialogStatus, setGroupsDialogStatus] = useState('');
  const [loadingRoutingProfiles, setLoadingRoutingProfiles] = useState(false);
  const [status, setStatus] = useState(t('Loading issued keys...'));
  const [submitting, setSubmitting] = useState(false);
  const [mutatingKey, setMutatingKey] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const deferredSearchQuery = useDeferredValue(filters.searchQuery);

  async function refreshWorkspaceState() {
    const data = await loadPortalApiKeyWorkbenchData();
    setApiKeys(data.api_keys);
    setApiKeyGroups(data.api_key_groups);
    setBillingEventSummary(data.billing_event_summary);
    return {
      keys: data.api_keys,
      groups: data.api_key_groups,
      billingEventSummary: data.billing_event_summary,
    };
  }

  async function refreshRoutingProfiles() {
    const profiles = await loadPortalRoutingProfiles();
    setRoutingProfiles(profiles);
    return profiles;
  }

  useEffect(() => {
    let cancelled = false;

    void refreshWorkspaceState()
      .then(() => {
        if (!cancelled) {
          setStatus(syncedStatus);
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
  }, [syncedStatus]);

  useEffect(() => {
    let cancelled = false;

    void resolveGatewayBaseUrl().then((baseUrl) => {
      if (!cancelled) {
        setGatewayBaseUrl(baseUrl);
      }
    });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!groupsDialogOpen) {
      setGroupsDialogStatus('');
      setLoadingRoutingProfiles(false);
      return;
    }

    let cancelled = false;
    setLoadingRoutingProfiles(true);
    setGroupsDialogStatus('');

    void refreshRoutingProfiles()
      .catch((error) => {
        if (!cancelled) {
          setGroupsDialogStatus(portalErrorMessage(error));
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
  }, [groupsDialogOpen]);

  const resolvedFilters = useMemo(
    () => ({
      ...filters,
      searchQuery: deferredSearchQuery,
    }),
    [deferredSearchQuery, filters],
  );

  const filteredKeys = useMemo(
    () => filterPortalApiKeys(apiKeys, resolvedFilters),
    [apiKeys, resolvedFilters],
  );
  const groupFilterOptions = useMemo(
    () => buildPortalApiKeyGroupFilterOptions(apiKeyGroups),
    [apiKeyGroups],
  );
  const environmentOptions = useMemo(
    () => buildPortalApiKeyEnvironmentOptions(apiKeys),
    [apiKeys],
  );
  const resolvedCreateEnvironment = resolvePortalApiKeyEnvironment(formState);
  const groupOptions = useMemo(
    () => buildPortalApiKeyGroupOptions(apiKeyGroups, resolvedCreateEnvironment),
    [apiKeyGroups, resolvedCreateEnvironment],
  );
  const totalPages = Math.max(1, Math.ceil(filteredKeys.length / API_KEY_PAGE_SIZE));
  const currentPage = clampPage(page, totalPages);
  const paginatedKeys = useMemo(() => {
    const start = (currentPage - 1) * API_KEY_PAGE_SIZE;
    return filteredKeys.slice(start, start + API_KEY_PAGE_SIZE);
  }, [currentPage, filteredKeys]);

  useEffect(() => {
    setPage((current) => clampPage(current, totalPages));
  }, [totalPages]);

  const usagePlaintext = usageKey
    ? readPortalApiKeyPlaintextReveal(usageKey.hashed_key)
      ?? (createdKey?.hashed === usageKey.hashed_key ? createdKey.plaintext : null)
    : null;
  const usagePreview = useMemo(
    () =>
      usageKey
        ? buildPortalApiKeyUsagePreview(usageKey, usagePlaintext, gatewayBaseUrl)
        : null,
    [gatewayBaseUrl, usageKey, usagePlaintext],
  );
  const quickSetupPlans = useMemo(
    () =>
      usageKey
        ? buildApiKeyQuickSetupPlans({
            hashedKey: usageKey.hashed_key,
            label: usageKey.label,
            plaintextKey: usagePlaintext,
            gatewayBaseUrl,
          })
        : [],
    [gatewayBaseUrl, usageKey, usagePlaintext],
  );
  const selectedPlan =
    quickSetupPlans.find((plan) => plan.id === selectedClientId) ?? quickSetupPlans[0] ?? null;

  useEffect(() => {
    if (groupOptions.some((option) => option.value === formState.apiKeyGroupId)) {
      return;
    }

    setFormState((current) => ({ ...current, apiKeyGroupId: 'none' }));
  }, [formState.apiKeyGroupId, groupOptions]);

  useEffect(() => {
    if (!usageKey) {
      setSelectedClientId('codex');
      setSelectedInstanceIds([]);
      setUsageStatus('');
      return;
    }

    let cancelled = false;
    setLoadingInstances(true);

    void Promise.all([resolveGatewayBaseUrl(), listApiKeyInstances()])
      .then(([baseUrl, instances]) => {
        if (cancelled) {
          return;
        }

        setGatewayBaseUrl(baseUrl);
        setOpenClawInstances(instances);
        setSelectedInstanceIds(instances.slice(0, 1).map((item) => item.id));
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingInstances(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [usageKey]);

  const groupById = useMemo(
    () => new Map(apiKeyGroups.map((group) => [group.group_id, group])),
    [apiKeyGroups],
  );
  const governanceViewModel = useMemo(
    () =>
      buildPortalApiKeyGovernanceViewModel({
        apiKeys,
        groups: apiKeyGroups,
        billingEventSummary,
      }),
    [apiKeyGroups, apiKeys, billingEventSummary],
  );

  function resolveGroupLabel(groupId: string | null | undefined): string {
    if (!groupId) {
      return t('No group binding');
    }

    return groupById.get(groupId)?.name ?? groupId;
  }

  async function handleCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!formState.label.trim()) {
      setStatus(t('Key label is required so credentials remain auditable after creation.'));
      return;
    }

    const environment = resolvePortalApiKeyEnvironment(formState);
    if (!environment) {
      setStatus(t('Custom environment is required when the custom environment option is selected.'));
      return;
    }

    const expiresAtMs = resolvePortalApiKeyExpiresAt(formState);
    if (formState.expiresAt.trim() && !expiresAtMs) {
      setStatus(t('Expires at must be a valid date before the credential can be created.'));
      return;
    }

    const customKey = resolvePortalApiKeyPlaintext(formState);
    if (formState.keyMode === 'custom' && !customKey) {
      setStatus(t('Custom key mode requires a plaintext key before the credential can be created.'));
      return;
    }

    const notes = resolvePortalApiKeyNotes(formState);
    const apiKeyGroupId = resolvePortalApiKeyGroupId(formState);

    setSubmitting(true);
    setStatus(
      formState.keyMode === 'custom'
        ? t('Registering a custom {environment} key for this workspace...', { environment })
        : t('Issuing a Portal-managed {environment} key for this workspace...', { environment }),
    );

    try {
      const nextKey = await issuePortalApiKey({
        api_key: customKey,
        api_key_group_id: apiKeyGroupId,
        environment,
        expires_at_ms: expiresAtMs,
        label: formState.label,
        notes,
      });

      rememberPortalApiKeyPlaintextReveal(nextKey.hashed, nextKey.plaintext);
      const { keys: nextKeys } = await refreshWorkspaceState();
      const nextUsageKey =
        nextKeys.find((item) => item.hashed_key === nextKey.hashed)
        ?? createRecordFromCreatedKey(nextKey);

      setCreatedKey(nextKey);
      setUsageKey(nextUsageKey);
      setCreateDrawerOpen(false);
      setFormState(createEmptyPortalApiKeyFormState());
      setPage(1);
      setStatus(
        formState.keyMode === 'custom'
          ? t(
              'Custom key stored for {environment}. Verify the plaintext value before leaving this page.',
              { environment },
            )
          : t(
              'Portal-managed key issued for {environment}. Copy the plaintext secret before leaving this page.',
              { environment },
            ),
      );
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleKeyStatusChange(key: GatewayApiKeyRecord, active: boolean) {
    setMutatingKey(key.hashed_key);
    setStatus(
      active
        ? t('Restoring {label}...', { label: key.label })
        : t('Revoking {label}...', { label: key.label }),
    );

    try {
      await setPortalApiKeyActive(key.hashed_key, active);
      await refreshWorkspaceState();
      setStatus(
        active
          ? t('{label} is active again and can authenticate gateway traffic.', {
              label: key.label,
            })
          : t('{label} has been revoked and will no longer authenticate requests.', {
              label: key.label,
            }),
      );
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setMutatingKey(null);
    }
  }

  async function handleDeleteKey(key: GatewayApiKeyRecord) {
    setMutatingKey(key.hashed_key);
    setStatus(t('Deleting {label}...', { label: key.label }));

    try {
      await removePortalApiKey(key.hashed_key);
      clearPortalApiKeyPlaintextReveal(key.hashed_key);
      await refreshWorkspaceState();

      if (createdKey?.hashed === key.hashed_key) {
        setCreatedKey(null);
      }

      if (usageKey?.hashed_key === key.hashed_key) {
        setUsageKey(null);
        setUsageStatus('');
      }

      setStatus(t('{label} was deleted from this workspace.', { label: key.label }));
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setMutatingKey(null);
    }
  }

  async function handleSaveApiKeyGroup(input: {
    group_id?: string;
    environment: string;
    name: string;
    slug?: string | null;
    description?: string | null;
    color?: string | null;
    default_capability_scope?: string | null;
    default_accounting_mode?: string | null;
    default_routing_profile_id?: string | null;
  }) {
    if (input.group_id) {
      await editPortalApiKeyGroup(input.group_id, {
        color: input.color,
        default_accounting_mode: input.default_accounting_mode,
        default_capability_scope: input.default_capability_scope,
        default_routing_profile_id: input.default_routing_profile_id,
        description: input.description,
        environment: input.environment,
        name: input.name,
        slug: input.slug,
      });
    } else {
      await issuePortalApiKeyGroup({
        color: input.color,
        default_accounting_mode: input.default_accounting_mode,
        default_capability_scope: input.default_capability_scope,
        default_routing_profile_id: input.default_routing_profile_id,
        description: input.description,
        environment: input.environment,
        name: input.name,
        slug: input.slug,
      });
    }

    await refreshWorkspaceState();
  }

  async function handleToggleApiKeyGroup(groupId: string, active: boolean) {
    await setPortalApiKeyGroupActive(groupId, active);
    await refreshWorkspaceState();
  }

  async function handleDeleteApiKeyGroup(groupId: string) {
    await removePortalApiKeyGroup(groupId);
    await refreshWorkspaceState();
  }

  async function handleApplySetup() {
    if (!selectedPlan) {
      return;
    }

    if (!selectedPlan.available) {
      setUsageStatus(
        selectedPlan.availabilityDetail
          ?? t('Plaintext Api key is no longer visible on this device. Create a replacement first.'),
      );
      return;
    }

    if (!usagePlaintext) {
      setUsageStatus(
        t('Plaintext Api key is no longer visible on this device. Create a replacement first.'),
      );
      return;
    }

    if (selectedPlan.requiresInstances && !selectedInstanceIds.length) {
      setUsageStatus(t('Select at least one OpenClaw instance before applying setup.'));
      return;
    }

    setApplyingClientId(selectedPlan.id);
    setUsageStatus(t('Applying {label} setup...', { label: selectedPlan.label }));

    try {
      const result = await applyApiKeyQuickSetup({
        ...selectedPlan.request,
        openClaw: selectedPlan.requiresInstances
          ? { instanceIds: selectedInstanceIds }
          : undefined,
        provider: {
          ...selectedPlan.request.provider,
          apiKey: usagePlaintext,
        },
      });

      setUsageStatus(
        result.updatedInstanceIds.length
          ? t('Applied setup to {count} OpenClaw instance(s).', {
              count: result.updatedInstanceIds.length,
            })
          : t('Applied setup and wrote {count} file(s).', {
              count: result.writtenFiles.length,
            }),
      );
    } catch (error) {
      setUsageStatus(portalErrorMessage(error));
    } finally {
      setApplyingClientId(null);
    }
  }

  const resolveVisiblePlaintext = (key: GatewayApiKeyRecord) =>
    readPortalApiKeyPlaintextReveal(key.hashed_key)
    ?? (createdKey?.hashed === key.hashed_key ? createdKey.plaintext : null);
  const hasActiveFilters = Boolean(
    filters.searchQuery.trim() || filters.environment !== 'all' || filters.groupId !== 'all',
  );
  const pageStatus = status !== syncedStatus && !createDrawerOpen ? status : '';
  const createDrawerStatus = status !== syncedStatus && createDrawerOpen ? status : '';
  const showingStart = filteredKeys.length === 0 ? 0 : (currentPage - 1) * API_KEY_PAGE_SIZE + 1;
  const showingEnd = filteredKeys.length === 0
    ? 0
    : Math.min(currentPage * API_KEY_PAGE_SIZE, filteredKeys.length);

  return (
    <div className="space-y-4" data-slot="api-router-page">
      <FilterBar data-slot="portal-api-key-toolbar" wrap={false}>
        <FilterBarSection className="min-w-0 flex-[1_1_20rem]" grow={false} wrap={false}>
          <SearchInput
            onChange={(event) =>
              startTransition(() => {
                setPage(1);
                setFilters((current) => ({ ...current, searchQuery: event.target.value }));
              })
            }
            placeholder={t('Search API keys')}
            value={filters.searchQuery}
          />
        </FilterBarSection>

        <FilterBarSection className="min-w-[11rem] shrink-0" grow={false} wrap={false}>
          <Select
            onValueChange={(value) =>
              startTransition(() => {
                setPage(1);
                setFilters((current) => ({ ...current, environment: value }));
              })
            }
            value={filters.environment}
          >
            <SelectTrigger>
              <SelectValue placeholder={t('Environment')} />
            </SelectTrigger>
            <SelectContent>
              {environmentOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {t(option.label)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </FilterBarSection>

        <FilterBarSection className="min-w-[12rem] shrink-0" grow={false} wrap={false}>
          <Select
            onValueChange={(value) =>
              startTransition(() => {
                setPage(1);
                setFilters((current) => ({ ...current, groupId: value }));
              })
            }
            value={filters.groupId}
          >
            <SelectTrigger>
              <SelectValue placeholder={t('Key group')} />
            </SelectTrigger>
            <SelectContent>
              {groupFilterOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {t(option.label)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </FilterBarSection>

        <FilterBarActions className="gap-2.5 whitespace-nowrap shrink-0" wrap={false}>
          <Button
            disabled={!hasActiveFilters}
            onClick={() =>
              startTransition(() => {
                setPage(1);
                setFilters({
                  environment: 'all',
                  groupId: 'all',
                  searchQuery: '',
                });
              })
            }
            variant="secondary"
          >
            {t('Clear filters')}
          </Button>
          <Button
            onClick={() => {
              setCreateDrawerOpen(false);
              setGroupsDialogOpen(true);
            }}
            variant="secondary"
          >
            {t('Manage groups')}
          </Button>
          <Button
            onClick={() => {
              setGroupsDialogOpen(false);
              setUsageKey(null);
              setCreateDrawerOpen(true);
            }}
            variant="primary"
          >
            {t('Create API key')}
          </Button>
        </FilterBarActions>
      </FilterBar>

      {pageStatus ? (
        <div
          className="rounded-2xl border border-zinc-200 bg-zinc-50/85 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300"
          data-slot="portal-api-key-feedback"
          role="status"
        >
          {pageStatus}
        </div>
      ) : null}

      <Card
        className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
        data-slot="portal-api-key-governance"
      >
        <CardContent className="space-y-4 p-5">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
              {t('Group governance')}
            </h2>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">
              {t('API key groups tie credential issuance, routing profile defaults, and chargeback accountability into one workspace view.')}
            </p>
          </div>

          <div className="grid gap-4 xl:grid-cols-3">
            <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
              <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Leading chargeback group')}
              </span>
              <strong className="mt-2 block text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                {governanceViewModel.leading_chargeback_group?.group_name ?? t('Ungrouped')}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {governanceViewModel.leading_chargeback_group
                  ? t('{requests} requests / {events} events', {
                      requests: formatUnits(
                        governanceViewModel.leading_chargeback_group.request_count,
                      ),
                      events: formatUnits(
                        governanceViewModel.leading_chargeback_group.event_count,
                      ),
                    })
                  : t('Billing event evidence will appear here after routed traffic starts recording chargeback activity.')}
              </p>
              <div className="mt-4 rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {t('Customer charge')}
                </span>
                <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {governanceViewModel.leading_chargeback_group
                    ? formatCurrency(
                        governanceViewModel.leading_chargeback_group.total_customer_charge,
                      )
                    : t('n/a')}
                </strong>
              </div>
            </div>

            <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
              <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Default accounting mode')}
              </span>
              <strong className="mt-2 block text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                {accountingModeLabel(governanceViewModel.dominant_default_accounting_mode, t)}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {governanceViewModel.dominant_default_accounting_mode
                  ? t('This is the most common accounting override across active API key groups.')
                  : t('No active API key group currently overrides accounting mode.')}
              </p>
              <div className="mt-4 rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {t('Routing profile')}
                </span>
                <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {governanceViewModel.leading_chargeback_group?.default_routing_profile_id
                    ?? t('No routing profile override')}
                </strong>
              </div>
            </div>

            <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
              <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Grouping posture')}
              </span>
              <strong className="mt-2 block text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                {formatUnits(governanceViewModel.summary.grouped_key_count)}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {t('{grouped} grouped keys / {ungrouped} ungrouped keys keep chargeback scope visible before traffic scales.', {
                  grouped: formatUnits(governanceViewModel.summary.grouped_key_count),
                  ungrouped: formatUnits(governanceViewModel.summary.ungrouped_key_count),
                })}
              </p>
              <div className="mt-4 grid gap-3 sm:grid-cols-2">
                <div className="rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('API key groups')}
                  </span>
                  <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {formatUnits(governanceViewModel.summary.active_group_count)}
                  </strong>
                </div>
                <div className="rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('Routing profiles')}
                  </span>
                  <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {formatUnits(governanceViewModel.summary.routing_profile_bound_group_count)}
                  </strong>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <PortalApiKeyTable
        className="rounded-[28px] border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950"
        data-slot="portal-api-key-table"
        footer={(
          <div
            className="flex flex-col gap-3 rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/50 lg:flex-row lg:items-center lg:justify-between"
            data-slot="portal-api-key-pagination"
          >
            <div className="text-sm text-zinc-600 dark:text-zinc-300">
              {t('Showing {start}-{end} of {total} keys', {
                end: showingEnd,
                start: showingStart,
                total: filteredKeys.length,
              })}
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Button
                disabled={currentPage <= 1}
                onClick={() => setPage((current) => clampPage(current - 1, totalPages))}
                variant="secondary"
              >
                {t('Previous page')}
              </Button>
              <span className="min-w-[8rem] text-center text-sm font-medium text-zinc-600 dark:text-zinc-300">
                {t('Page {page} of {total}', { page: currentPage, total: totalPages })}
              </span>
              <Button
                disabled={currentPage >= totalPages}
                onClick={() => setPage((current) => clampPage(current + 1, totalPages))}
                variant="secondary"
              >
                {t('Next page')}
              </Button>
            </div>
          </div>
        )}
        items={paginatedKeys}
        latestCreatedKey={createdKey}
        mutatingKey={mutatingKey}
        onCopyLatestPlaintext={() => {
          void copyPlaintextWithStatus(createdKey?.plaintext ?? null, setStatus, t);
        }}
        onCopyPlaintext={(key) => {
          void copyPlaintextWithStatus(resolveVisiblePlaintext(key), setStatus, t);
        }}
        onDelete={(key) => {
          void handleDeleteKey(key);
        }}
        onOpenDetails={(key) => {
          setUsageStatus('');
          setUsageKey(key);
        }}
        onToggleStatus={(key) => {
          void handleKeyStatusChange(key, !key.active);
        }}
        resolveGroupLabel={resolveGroupLabel}
        resolvePlaintext={resolveVisiblePlaintext}
      />

      <PortalApiKeyDrawers
        apiKeyGroups={apiKeyGroups}
        applyingClientId={applyingClientId}
        createFormState={formState}
        createOpen={createDrawerOpen}
        createStatus={createDrawerStatus}
        createdKey={createdKey}
        gatewayBaseUrl={gatewayBaseUrl}
        groupOptions={groupOptions}
        loadingInstances={loadingInstances}
        onApplySetup={() => {
          void handleApplySetup();
        }}
        onChangeForm={(updater) => setFormState((current) => updater(current))}
        onChangeInstanceSelection={(nextValue) => setSelectedInstanceIds(nextValue)}
        onCloseCreate={() => setCreateDrawerOpen(false)}
            onCloseUsage={() => {
              setUsageKey(null);
              setUsageStatus('');
            }}
            onCopyPlaintext={() => {
              void copyPlaintextWithStatus(
                usagePlaintext ?? createdKey?.plaintext ?? null,
                setUsageStatus,
                t,
          );
        }}
        onCreate={(event) => {
          void handleCreate(event);
        }}
        onSelectClient={(clientId) => setSelectedClientId(clientId)}
        openClawInstances={openClawInstances}
        quickSetupPlans={quickSetupPlans}
        selectedClientId={selectedClientId}
        selectedInstanceIds={selectedInstanceIds}
        selectedPlan={selectedPlan}
        submitting={submitting}
        usageKey={usageKey}
        usagePlaintext={usagePlaintext}
        usagePreview={usagePreview}
        usageStatus={usageStatus}
      />

      <PortalApiKeyGroupsDialog
        groups={apiKeyGroups}
        loadingRoutingProfiles={loadingRoutingProfiles}
        onDeleteGroup={(groupId) => handleDeleteApiKeyGroup(groupId)}
        onOpenChange={setGroupsDialogOpen}
        onSaveGroup={(input) => handleSaveApiKeyGroup(input)}
        onToggleGroup={(groupId, active) => handleToggleApiKeyGroup(groupId, active)}
        open={groupsDialogOpen}
        profileStatus={groupsDialogStatus}
        routingProfiles={routingProfiles}
      />
    </div>
  );
}
