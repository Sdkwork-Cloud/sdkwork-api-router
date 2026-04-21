import type {
  ApiKeyGroupRecord,
  BillingEventGroupSummary,
  BillingEventSummary,
  GatewayApiKeyRecord,
} from 'sdkwork-router-portal-types';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';

import type {
  PortalApiKeyCreateFormState,
  PortalApiKeyEnvironmentOption,
  PortalApiKeyFilterState,
  PortalApiKeyGovernanceViewModel,
  PortalApiKeyGroupOption,
  PortalApiKeyUsagePreview,
} from '../types';
import { createPortalApiKeyPlaintextRevealSessionStore } from './plaintextRevealSessionStore';

const environmentOrder = ['live', 'staging', 'test'];

type EnvironmentSummary = {
  environment: string;
  total: number;
  active: number;
};

function emptyBillingEventSummary(): BillingEventSummary {
  return {
    total_events: 0,
    project_count: 0,
    group_count: 0,
    capability_count: 0,
    total_request_count: 0,
    total_units: 0,
    total_input_tokens: 0,
    total_output_tokens: 0,
    total_tokens: 0,
    total_image_count: 0,
    total_audio_seconds: 0,
    total_video_seconds: 0,
    total_music_seconds: 0,
    total_upstream_cost: 0,
    total_customer_charge: 0,
    projects: [],
    groups: [],
    capabilities: [],
    accounting_modes: [],
  };
}

export function rememberPortalApiKeyPlaintextReveal(
  hashedKey: string,
  plaintextKey: string,
): void {
  createPortalApiKeyPlaintextRevealSessionStore().rememberPlaintextReveal(
    hashedKey,
    plaintextKey,
  );
}

function trimTrailingSlash(value: string): string {
  return value.replace(/\/+$/g, '');
}

function joinUrl(baseUrl: string, path: string): string {
  const normalizedBase = trimTrailingSlash(baseUrl);
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${normalizedBase}${normalizedPath}`;
}

export function readPortalApiKeyPlaintextReveal(hashedKey: string): string | null {
  return createPortalApiKeyPlaintextRevealSessionStore().readPlaintextReveal(hashedKey);
}

export function clearPortalApiKeyPlaintextReveal(hashedKey: string): void {
  createPortalApiKeyPlaintextRevealSessionStore().clearPlaintextReveal(hashedKey);
}

function sortKeys(keys: GatewayApiKeyRecord[]): GatewayApiKeyRecord[] {
  return [...keys].sort((left, right) => right.created_at_ms - left.created_at_ms);
}

function sortGroups(groups: ApiKeyGroupRecord[]): ApiKeyGroupRecord[] {
  return [...groups].sort((left, right) =>
    Number(right.active) - Number(left.active)
    || left.environment.localeCompare(right.environment)
    || left.name.localeCompare(right.name)
    || left.group_id.localeCompare(right.group_id)
  );
}

function sortGroupChargeback(
  items: BillingEventGroupSummary[],
): BillingEventGroupSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || (left.api_key_group_id ?? '').localeCompare(right.api_key_group_id ?? ''),
    );
}

function resolveDominantDefaultAccountingMode(
  groups: ApiKeyGroupRecord[],
): string | null {
  const counts = new Map<string, number>();

  for (const group of groups) {
    if (!group.active || !group.default_accounting_mode?.trim()) {
      continue;
    }

    const mode = group.default_accounting_mode.trim().toLowerCase();
    counts.set(mode, (counts.get(mode) ?? 0) + 1);
  }

  return [...counts.entries()]
    .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))[0]?.[0]
    ?? null;
}

export function buildPortalApiKeyGovernanceViewModel(input: {
  apiKeys: GatewayApiKeyRecord[];
  groups: ApiKeyGroupRecord[];
  billingEventSummary: BillingEventSummary | null | undefined;
}): PortalApiKeyGovernanceViewModel {
  const billingSummary = input.billingEventSummary ?? emptyBillingEventSummary();
  const activeGroups = input.groups.filter((group) => group.active);
  const groupedKeyCount = input.apiKeys.filter((key) => Boolean(key.api_key_group_id)).length;
  const groupById = new Map(input.groups.map((group) => [group.group_id, group]));
  const leadingGroup = sortGroupChargeback(billingSummary.groups)[0] ?? null;

  return {
    summary: {
      active_group_count: activeGroups.length,
      grouped_key_count: groupedKeyCount,
      ungrouped_key_count: Math.max(0, input.apiKeys.length - groupedKeyCount),
      routing_profile_bound_group_count: activeGroups.filter(
        (group) => Boolean(group.default_routing_profile_id),
      ).length,
    },
    leading_chargeback_group: leadingGroup
      ? {
          api_key_group_id: leadingGroup.api_key_group_id ?? null,
          group_name: leadingGroup.api_key_group_id
            ? (groupById.get(leadingGroup.api_key_group_id)?.name ?? leadingGroup.api_key_group_id)
            : translatePortalText('Ungrouped'),
          total_customer_charge: leadingGroup.total_customer_charge,
          total_upstream_cost: leadingGroup.total_upstream_cost,
          request_count: leadingGroup.request_count,
          event_count: leadingGroup.event_count,
          default_accounting_mode: leadingGroup.api_key_group_id
            ? (groupById.get(leadingGroup.api_key_group_id)?.default_accounting_mode ?? null)
            : null,
          default_routing_profile_id: leadingGroup.api_key_group_id
            ? (groupById.get(leadingGroup.api_key_group_id)?.default_routing_profile_id ?? null)
            : null,
        }
      : null,
    dominant_default_accounting_mode: resolveDominantDefaultAccountingMode(input.groups),
  };
}

function summarizeKeysByEnvironment(keys: GatewayApiKeyRecord[]): EnvironmentSummary[] {
  const grouped = new Map<string, EnvironmentSummary>();

  for (const key of keys) {
    const current = grouped.get(key.environment) ?? {
      environment: key.environment,
      total: 0,
      active: 0,
    };

    current.total += 1;
    if (key.active) {
      current.active += 1;
    }
    grouped.set(key.environment, current);
  }

  return [...grouped.values()].sort((left, right) => left.environment.localeCompare(right.environment));
}

export function buildPortalApiKeyEnvironmentOptions(
  keys: GatewayApiKeyRecord[],
): PortalApiKeyEnvironmentOption[] {
  const dynamicOptions = summarizeKeysByEnvironment(keys)
    .map((summary) => summary.environment)
    .filter((environment) => !environmentOrder.includes(environment))
    .sort((left, right) => left.localeCompare(right));

  return [
    {
      value: 'all',
      label: translatePortalText('All environments'),
      detail: translatePortalText('View every environment boundary in the active workspace.'),
    },
    ...environmentOrder.map((environment) => ({
      value: environment,
      label: environment,
      detail: translatePortalText('Recommended {environment} environment boundary.', {
        environment,
      }),
    })),
    ...dynamicOptions.map((environment) => ({
      value: environment,
      label: environment,
      detail: translatePortalText('Custom environment discovered in this workspace.'),
    })),
  ];
}

export function filterPortalApiKeys(
  keys: GatewayApiKeyRecord[],
  filters: PortalApiKeyFilterState,
): GatewayApiKeyRecord[] {
  const normalizedQuery = filters.searchQuery.trim().toLowerCase();

  return sortKeys(keys).filter((key) => {
    const matchesEnvironment =
      filters.environment === 'all' || key.environment === filters.environment;
    const matchesGroup =
      filters.groupId === 'all'
      || (filters.groupId === 'none'
        ? !key.api_key_group_id
        : key.api_key_group_id === filters.groupId);
    const matchesQuery =
      normalizedQuery.length === 0 ||
      key.label.toLowerCase().includes(normalizedQuery) ||
      (key.notes ?? '').toLowerCase().includes(normalizedQuery) ||
      key.environment.toLowerCase().includes(normalizedQuery) ||
      (key.api_key_group_id ?? '').toLowerCase().includes(normalizedQuery) ||
      key.hashed_key.toLowerCase().includes(normalizedQuery);

    return matchesEnvironment && matchesGroup && matchesQuery;
  });
}

export function createEmptyPortalApiKeyFormState(): PortalApiKeyCreateFormState {
  return {
    label: '',
    keyMode: 'system-generated',
    customKey: '',
    environment: 'live',
    customEnvironment: '',
    apiKeyGroupId: 'none',
    expiresAt: '',
    notes: '',
  };
}

export function buildPortalApiKeyGroupFilterOptions(
  groups: ApiKeyGroupRecord[],
): PortalApiKeyGroupOption[] {
  return [
    {
      value: 'all',
      label: translatePortalText('All groups'),
      detail: translatePortalText('View every group-backed and standalone key in this workspace.'),
    },
    {
      value: 'none',
      label: translatePortalText('Ungrouped'),
      detail: translatePortalText('Show keys that do not inherit a reusable group posture.'),
    },
    ...sortGroups(groups).map((group) => ({
      value: group.group_id,
      label: translatePortalText('{groupName} ({environment})', {
        environment: group.environment,
        groupName: group.name,
      }),
      detail: group.default_routing_profile_id
        ? translatePortalText('Routing profile {profileId}', {
            profileId: group.default_routing_profile_id,
          })
        : translatePortalText('No routing profile override'),
    })),
  ];
}

export function buildPortalApiKeyGroupOptions(
  groups: ApiKeyGroupRecord[],
  environment: string | null,
): PortalApiKeyGroupOption[] {
  const normalizedEnvironment = environment?.trim().toLowerCase() || null;
  const eligibleGroups = sortGroups(groups).filter(
    (group) =>
      group.active
      && (!normalizedEnvironment || group.environment.toLowerCase() === normalizedEnvironment),
  );

  return [
    {
      value: 'none',
      label: translatePortalText('No group binding'),
      detail: translatePortalText(
        'Create a standalone key without group-level routing or accounting defaults.',
      ),
    },
    ...eligibleGroups.map((group) => ({
      value: group.group_id,
      label: translatePortalText('{groupName} ({environment})', {
        environment: group.environment,
        groupName: group.name,
      }),
      detail: group.default_routing_profile_id
        ? translatePortalText('Routing profile {profileId}', {
            profileId: group.default_routing_profile_id,
          })
        : translatePortalText('No routing profile override'),
    })),
  ];
}

export function resolvePortalApiKeyPlaintext(
  formState: PortalApiKeyCreateFormState,
): string | null {
  if (formState.keyMode !== 'custom') {
    return null;
  }

  const customKey = formState.customKey.trim();
  return customKey.length ? customKey : null;
}

export function resolvePortalApiKeyEnvironment(
  formState: PortalApiKeyCreateFormState,
): string | null {
  if (formState.environment === 'custom') {
    const customEnvironment = formState.customEnvironment.trim();
    return customEnvironment.length ? customEnvironment : null;
  }

  return formState.environment;
}

export function resolvePortalApiKeyGroupId(
  formState: PortalApiKeyCreateFormState,
): string | null {
  return formState.apiKeyGroupId === 'none' ? null : formState.apiKeyGroupId;
}

export function resolvePortalApiKeyExpiresAt(
  formState: PortalApiKeyCreateFormState,
): number | null {
  const trimmed = formState.expiresAt.trim();
  if (!trimmed) {
    return null;
  }

  const parsed = Date.parse(`${trimmed}T23:59:59.000Z`);
  return Number.isNaN(parsed) ? null : parsed;
}

export function resolvePortalApiKeyNotes(
  formState: PortalApiKeyCreateFormState,
): string | null {
  const notes = formState.notes.trim();
  return notes.length ? notes : null;
}

export function buildPortalApiKeyUsagePreview(
  key: GatewayApiKeyRecord,
  plaintext: string | null,
  gatewayBaseUrl: string,
): PortalApiKeyUsagePreview {

  return {
    title: plaintext
      ? translatePortalText('How to use this key')
      : translatePortalText('Usage method'),
    detail: plaintext
      ? translatePortalText(
          'The newest plaintext secret is still available in this session, so you can validate the request shape before closing the page.',
        )
      : translatePortalText(
          'This key is already stored in write-only mode. If you need the plaintext again, rotate it by creating a replacement credential.',
        ),
    authorizationHeader: plaintext ? `Authorization: Bearer ${plaintext}` : null,
    curlExample: plaintext
      ? `curl ${joinUrl(gatewayBaseUrl, '/v1/models')} \\\n  -H "Authorization: Bearer ${plaintext}"`
      : null,
  };
}
