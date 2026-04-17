import type {
  AdminAuthSession,
  AdminUserProfile,
  ChannelRecord,
  ExtensionRuntimeReloadRequest,
  ExtensionRuntimeReloadResult,
  ExtensionRuntimeStatusRecord,
  GatewayApiKeyRecord,
  LedgerEntry,
  ModelCatalogRecord,
  ProviderHealthSnapshot,
  ProjectRecord,
  ProxyProviderRecord,
  BillingSummary,
  UsageSummary,
  QuotaPolicyRecord,
  RoutingDecisionLog,
  RoutingSimulationResult,
  TenantRecord,
  UsageRecord,
} from 'sdkwork-api-types';

export class AdminApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }
}

const adminSessionTokenKey = 'sdkwork.admin.session-token';
const MAX_SAFE_INTEGER_TEXT = String(Number.MAX_SAFE_INTEGER);

export function adminBaseUrl(): string {
  return '/api/admin';
}

async function readJson<T>(response: Response): Promise<T> {
  const payload = await readResponsePayload(response);

  if (!response.ok) {
    throw new AdminApiError(`Admin request failed with status ${response.status}`, response.status);
  }

  return payload as T;
}

async function readResponsePayload(response: Response): Promise<unknown> {
  if (typeof response.text === 'function') {
    const body = await response.text();
    return body ? parseJsonBody(body) : null;
  }

  if (typeof response.json === 'function') {
    return response.json();
  }

  return null;
}

function parseJsonBody(body: string): unknown {
  return JSON.parse(quoteUnsafeIntegerTokens(body));
}

function quoteUnsafeIntegerTokens(body: string): string {
  let result = '';
  let index = 0;
  let inString = false;
  let escaped = false;

  while (index < body.length) {
    const character = body[index];

    if (inString) {
      result += character;
      if (escaped) {
        escaped = false;
      } else if (character === '\\') {
        escaped = true;
      } else if (character === '"') {
        inString = false;
      }
      index += 1;
      continue;
    }

    if (character === '"') {
      inString = true;
      result += character;
      index += 1;
      continue;
    }

    if (character === '-' || isDigit(character)) {
      const tokenEnd = findNumericTokenEnd(body, index);
      const token = body.slice(index, tokenEnd);

      if (shouldQuoteIntegerToken(token)) {
        result += `"${token}"`;
      } else {
        result += token;
      }

      index = tokenEnd;
      continue;
    }

    result += character;
    index += 1;
  }

  return result;
}

function findNumericTokenEnd(body: string, start: number): number {
  let index = start;

  if (body[index] === '-') {
    index += 1;
  }

  while (index < body.length && isDigit(body[index])) {
    index += 1;
  }

  if (body[index] === '.') {
    index += 1;
    while (index < body.length && isDigit(body[index])) {
      index += 1;
    }
  }

  if (body[index] === 'e' || body[index] === 'E') {
    index += 1;
    if (body[index] === '+' || body[index] === '-') {
      index += 1;
    }
    while (index < body.length && isDigit(body[index])) {
      index += 1;
    }
  }

  return index;
}

function shouldQuoteIntegerToken(token: string): boolean {
  if (!/^-?\d+$/.test(token)) {
    return false;
  }

  const normalized = token.startsWith('-') ? token.slice(1) : token;
  if (normalized.length < MAX_SAFE_INTEGER_TEXT.length) {
    return false;
  }
  if (normalized.length > MAX_SAFE_INTEGER_TEXT.length) {
    return true;
  }
  return normalized > MAX_SAFE_INTEGER_TEXT;
}

function isDigit(character: string | undefined): boolean {
  return character != null && character >= '0' && character <= '9';
}

function readAdminSessionToken(): string | null {
  return globalThis.localStorage?.getItem(adminSessionTokenKey) ?? null;
}

export function persistAdminSessionToken(token: string): void {
  globalThis.localStorage?.setItem(adminSessionTokenKey, token);
}

export function clearAdminSessionToken(): void {
  globalThis.localStorage?.removeItem(adminSessionTokenKey);
}

function requiredAdminToken(providedToken?: string): string {
  const token = providedToken ?? readAdminSessionToken();
  if (!token) {
    throw new AdminApiError('Admin session token not found', 401);
  }
  return token;
}

async function getJson<T>(path: string, token?: string): Promise<T> {
  const response = await fetch(`${adminBaseUrl()}${path}`, {
    headers: {
      authorization: `Bearer ${requiredAdminToken(token)}`,
    },
  });
  return readJson<T>(response);
}

async function postJson<TRequest, TResponse>(
  path: string,
  body: TRequest,
  token?: string,
): Promise<TResponse> {
  const headers: Record<string, string> = {
    'content-type': 'application/json',
  };
  if (token) {
    headers.authorization = `Bearer ${token}`;
  }

  const response = await fetch(`${adminBaseUrl()}${path}`, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  });

  return readJson<TResponse>(response);
}

export function loginAdminUser(input: {
  email: string;
  password: string;
}): Promise<AdminAuthSession> {
  return postJson<typeof input, AdminAuthSession>('/auth/login', input);
}

export function getAdminMe(token?: string): Promise<AdminUserProfile> {
  return getJson<AdminUserProfile>('/auth/me', token);
}

export function changeAdminPassword(
  input: { current_password: string; new_password: string },
  token?: string,
): Promise<AdminUserProfile> {
  return postJson<typeof input, AdminUserProfile>(
    '/auth/change-password',
    input,
    requiredAdminToken(token),
  );
}

export function listTenants(token?: string): Promise<TenantRecord[]> {
  return getJson<TenantRecord[]>('/tenants', token);
}

export function listProjects(token?: string): Promise<ProjectRecord[]> {
  return getJson<ProjectRecord[]>('/projects', token);
}

export function listApiKeys(token?: string): Promise<GatewayApiKeyRecord[]> {
  return getJson<GatewayApiKeyRecord[]>('/api-keys', token);
}

export function listChannels(token?: string): Promise<ChannelRecord[]> {
  return getJson<ChannelRecord[]>('/channels', token);
}

export function listProviders(token?: string): Promise<ProxyProviderRecord[]> {
  return getJson<ProxyProviderRecord[]>('/providers', token);
}

export function listModels(token?: string): Promise<ModelCatalogRecord[]> {
  return getJson<ModelCatalogRecord[]>('/models', token);
}

export function listUsageRecords(token?: string): Promise<UsageRecord[]> {
  return getJson<UsageRecord[]>('/usage/records', token);
}

export function getUsageSummary(token?: string): Promise<UsageSummary> {
  return getJson<UsageSummary>('/usage/summary', token);
}

export function listLedgerEntries(token?: string): Promise<LedgerEntry[]> {
  return getJson<LedgerEntry[]>('/billing/ledger', token);
}

export function getBillingSummary(token?: string): Promise<BillingSummary> {
  return getJson<BillingSummary>('/billing/summary', token);
}

export function listQuotaPolicies(token?: string): Promise<QuotaPolicyRecord[]> {
  return getJson<QuotaPolicyRecord[]>('/billing/quota-policies', token);
}

export function listProviderHealthSnapshots(token?: string): Promise<ProviderHealthSnapshot[]> {
  return getJson<ProviderHealthSnapshot[]>('/routing/health-snapshots', token);
}

export function listExtensionRuntimeStatuses(token?: string): Promise<ExtensionRuntimeStatusRecord[]> {
  return getJson<ExtensionRuntimeStatusRecord[]>('/extensions/runtime-statuses', token);
}

export function reloadExtensionRuntimes(
  request?: ExtensionRuntimeReloadRequest,
  token?: string,
): Promise<ExtensionRuntimeReloadResult> {
  return postJson<ExtensionRuntimeReloadRequest | undefined, ExtensionRuntimeReloadResult>(
    '/extensions/runtime-reloads',
    request,
    requiredAdminToken(token),
  );
}

export function listRoutingDecisionLogs(token?: string): Promise<RoutingDecisionLog[]> {
  return getJson<RoutingDecisionLog[]>('/routing/decision-logs', token);
}

export function simulateRoute(
  capability: string,
  model: string,
  selectionSeed?: number,
  requestedRegion?: string,
  token?: string,
): Promise<RoutingSimulationResult> {
  return postJson<{ capability: string; model: string; selection_seed?: number; requested_region?: string }, RoutingSimulationResult>(
    '/routing/simulations',
    { capability, model, selection_seed: selectionSeed, requested_region: requestedRegion },
    requiredAdminToken(token),
  );
}
