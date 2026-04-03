import type {
  AdminWorkspaceSnapshot,
  RateLimitPolicyRecord,
  RateLimitWindowRecord,
} from 'sdkwork-router-admin-types';

export type RateLimitStatusFilter = 'all' | 'enabled' | 'disabled' | 'exceeded';

export type GatewayRateLimitScope = {
  apiKeyLabel: string | null;
  routeKey: string | null;
  modelName: string | null;
  projectWide: boolean;
};

export type GatewayRateLimitInventoryRow = {
  policy: RateLimitPolicyRecord;
  window: RateLimitWindowRecord | null;
  projectName: string | null;
  scope: GatewayRateLimitScope;
  searchText: string;
};

export type RateLimitDraft = {
  policy_id: string;
  project_id: string;
  api_key_hash: string;
  route_key: string;
  model_name: string;
  requests_per_window: string;
  window_seconds: string;
  burst_requests: string;
  enabled: boolean;
  notes: string;
};

export function buildRateLimitPolicyId(projectId: string) {
  const normalizedProjectId = projectId
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

  return `rl-${normalizedProjectId || 'policy'}-${Date.now().toString(36)}`;
}

export function createEmptyRateLimitDraft(
  snapshot: AdminWorkspaceSnapshot,
): RateLimitDraft {
  const defaultProjectId =
    snapshot.projects[0]?.id ?? snapshot.rateLimitPolicies[0]?.project_id ?? '';

  return {
    policy_id: buildRateLimitPolicyId(defaultProjectId),
    project_id: defaultProjectId,
    api_key_hash: '',
    route_key: '',
    model_name: '',
    requests_per_window: '120',
    window_seconds: '60',
    burst_requests: '20',
    enabled: true,
    notes: '',
  };
}

export function normalizeOptionalText(value: string): string | null {
  const trimmedValue = value.trim();
  return trimmedValue ? trimmedValue : null;
}
