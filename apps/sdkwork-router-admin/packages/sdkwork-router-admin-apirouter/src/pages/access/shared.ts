import type {
  ApiKeyGroupRecord,
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
} from 'sdkwork-router-admin-types';
import {
  formatAdminDateTime,
  translateAdminText,
} from 'sdkwork-router-admin-core';

import {
  applyApiKeyQuickSetup,
  type ApiKeySetupClientId,
} from '../../services/gatewayApiKeyAccessService';
import {
  readGatewayApiKeyOverlay,
  readGatewayApiKeyPlaintextReveal,
  type GatewayRouteMode,
} from '../../services/gatewayOverlayStore';

export type CreateDraft = {
  tenant_id: string;
  project_id: string;
  environment: string;
  api_key_group_id: string;
  label: string;
  notes: string;
  expires_at: string;
  plaintext_key: string;
  route_mode: GatewayRouteMode;
  route_provider_id: string;
  model_mapping_id: string;
};

export type EditDraft = {
  label: string;
  notes: string;
  expires_at: string;
  api_key_group_id: string;
};

export type RouteDraft = {
  source: 'system-generated' | 'custom';
  route_mode: GatewayRouteMode;
  route_provider_id: string;
  model_mapping_id: string;
};

export const QUICK_SETUP_CLIENT_ORDER: ApiKeySetupClientId[] = [
  'codex',
  'claude-code',
  'opencode',
  'gemini',
  'openclaw',
];

export const QUICK_SETUP_CLIENT_LABELS: Record<ApiKeySetupClientId, string> = {
  codex: 'Codex',
  'claude-code': 'Claude Code',
  opencode: 'OpenCode',
  gemini: 'Gemini',
  openclaw: 'OpenClaw',
};

type AccessTranslate = (text: string) => string;

export function formatTimestamp(value?: number | null): string {
  return formatAdminDateTime(value);
}

export function formatEnvironmentLabel(
  value: string,
  translate: AccessTranslate = translateAdminText,
): string {
  switch (value.trim().toLowerCase()) {
    case 'live':
      return translate('Live');
    case 'staging':
      return translate('Staging');
    case 'test':
      return translate('Test');
    case 'production':
      return translate('Production');
    case 'development':
      return translate('Development');
    default:
      return value;
  }
}

export function formatAccountingModeLabel(
  value: string | null | undefined,
  translate: AccessTranslate = translateAdminText,
): string {
  if (!value?.trim()) {
    return translate('No accounting override');
  }

  switch (value.trim().toLowerCase()) {
    case 'platform_credit':
      return translate('Platform credit');
    case 'byok':
      return translate('BYOK');
    case 'passthrough':
      return translate('Passthrough');
    default:
      return value;
  }
}

export function formatApiKeyReferenceLabel(
  key: GatewayApiKeyRecord,
  translate: AccessTranslate = translateAdminText,
): string {
  return `${key.label || key.project_id} (${formatEnvironmentLabel(key.environment, translate)})`;
}

export function formatExpiryInput(value?: number | null): string {
  if (!value) return '';
  const date = new Date(value);
  const pad = (item: number) => String(item).padStart(2, '0');

  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function parseExpiryInput(value: string): number | null | undefined {
  const normalized = value.trim();

  if (!normalized) return undefined;

  const parsed = Date.parse(normalized);
  return Number.isFinite(parsed) ? parsed : null;
}

export function maskKey(value: string): string {
  return value.length <= 16
    ? value
    : `${value.slice(0, 10)}*****${value.slice(-4)}`;
}

export function createDraft(
  tenantId: string,
  projectId: string,
  overrides: Partial<CreateDraft> = {},
): CreateDraft {
  return {
    tenant_id: tenantId,
    project_id: projectId,
    environment: 'live',
    api_key_group_id: '',
    label: '',
    notes: '',
    expires_at: '',
    plaintext_key: '',
    route_mode: 'sdkwork-remote',
    route_provider_id: '',
    model_mapping_id: '',
    ...overrides,
  };
}

export function editDraftFromKey(key: GatewayApiKeyRecord): EditDraft {
  return {
    label: key.label,
    notes: key.notes ?? '',
    expires_at: formatExpiryInput(key.expires_at_ms),
    api_key_group_id: key.api_key_group_id ?? '',
  };
}

export function routeDraftFromKey(key: GatewayApiKeyRecord): RouteDraft {
  const overlay = readGatewayApiKeyOverlay(key.hashed_key);

  return {
    source: overlay.source,
    route_mode: overlay.route_mode,
    route_provider_id: overlay.route_provider_id ?? '',
    model_mapping_id: overlay.model_mapping_id ?? '',
  };
}

export async function copyToClipboard(value: string): Promise<void> {
  if (navigator.clipboard) {
    await navigator.clipboard.writeText(value);
  }
}

export function resolvePlaintextForKey(
  key: GatewayApiKeyRecord,
): string | null {
  if (key.raw_key?.trim()) return key.raw_key;

  return readGatewayApiKeyPlaintextReveal(key.hashed_key)?.plaintext_key ?? null;
}

export function buildUsageKeyFromCreateResponse(
  created: CreatedGatewayApiKey,
): GatewayApiKeyRecord {
  return {
    tenant_id: created.tenant_id,
    project_id: created.project_id,
    environment: created.environment,
    hashed_key: created.hashed,
    api_key_group_id: created.api_key_group_id ?? null,
    label: created.label,
    notes: created.notes,
    created_at_ms: created.created_at_ms,
    expires_at_ms: created.expires_at_ms,
    last_used_at_ms: null,
    active: true,
    raw_key: null,
  };
}

export function buildQuickSetupSummary(
  result: Awaited<ReturnType<typeof applyApiKeyQuickSetup>>,
): string {
  if (result.updatedInstanceIds.length) {
    return translateAdminText('Applied setup to {count} OpenClaw instance(s).', {
      count: result.updatedInstanceIds.length,
    });
  }

  if (result.updatedEnvironments.length) {
    return translateAdminText('Applied setup and updated {count} environment target(s).', {
      count: result.updatedEnvironments.length,
    });
  }

  return translateAdminText('Applied setup and wrote {count} file(s).', {
    count: result.writtenFiles.length,
  });
}

export function isExpiringSoon(key: GatewayApiKeyRecord): boolean {
  if (!key.active || !key.expires_at_ms) return false;

  const remaining = key.expires_at_ms - Date.now();
  return remaining > 0 && remaining <= 7 * 24 * 60 * 60 * 1000;
}

export function filterApiKeyGroupsByScope(
  groups: ApiKeyGroupRecord[],
  scope: {
    tenant_id: string;
    project_id: string;
    environment: string;
  },
): ApiKeyGroupRecord[] {
  return groups.filter(
    (group) =>
      group.tenant_id === scope.tenant_id
      && group.project_id === scope.project_id
      && group.environment === scope.environment,
  );
}
