import type {
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
  PortalAuthSession,
  PortalUserProfile,
  PortalWorkspaceSummary,
} from 'sdkwork-api-types';

const portalSessionTokenKey = 'sdkwork.portal.session-token';
const MAX_SAFE_INTEGER_TEXT = String(Number.MAX_SAFE_INTEGER);

export class PortalApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
  }
}

export function portalBaseUrl(): string {
  return '/api/portal';
}

export function readPortalSessionToken(): string | null {
  return globalThis.localStorage?.getItem(portalSessionTokenKey) ?? null;
}

export function persistPortalSessionToken(token: string): void {
  globalThis.localStorage?.setItem(portalSessionTokenKey, token);
}

export function clearPortalSessionToken(): void {
  globalThis.localStorage?.removeItem(portalSessionTokenKey);
}

async function readJson<T>(response: Response): Promise<T> {
  const payload = await readResponsePayload(response);

  if (!response.ok) {
    let message = `Portal request failed with status ${response.status}`;

    try {
      const errorPayload = payload as { error?: { message?: string } } | null;
      if (errorPayload) {
        message = errorPayload.error?.message?.trim() || message;
      }
    } catch {
      // Ignore non-JSON error bodies and fall back to the status-based message.
    }

    throw new PortalApiError(message, response.status);
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

async function getJson<T>(path: string, token?: string): Promise<T> {
  const response = await fetch(`${portalBaseUrl()}${path}`, {
    headers: token
      ? {
          authorization: `Bearer ${token}`,
        }
      : undefined,
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

  const response = await fetch(`${portalBaseUrl()}${path}`, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  });

  return readJson<TResponse>(response);
}

function requiredPortalToken(providedToken?: string): string {
  const token = providedToken ?? readPortalSessionToken();
  if (!token) {
    throw new PortalApiError('Portal session token not found', 401);
  }
  return token;
}

export function registerPortalUser(input: {
  email: string;
  password: string;
  display_name: string;
}): Promise<PortalAuthSession> {
  return postJson<typeof input, PortalAuthSession>('/auth/register', input);
}

export function loginPortalUser(input: {
  email: string;
  password: string;
}): Promise<PortalAuthSession> {
  return postJson<typeof input, PortalAuthSession>('/auth/login', input);
}

export function getPortalMe(token?: string): Promise<PortalUserProfile> {
  return getJson<PortalUserProfile>('/auth/me', requiredPortalToken(token));
}

export function changePortalPassword(
  input: { current_password: string; new_password: string },
  token?: string,
): Promise<PortalUserProfile> {
  return postJson<typeof input, PortalUserProfile>(
    '/auth/change-password',
    input,
    requiredPortalToken(token),
  );
}

export function getPortalWorkspace(token?: string): Promise<PortalWorkspaceSummary> {
  return getJson<PortalWorkspaceSummary>('/workspace', requiredPortalToken(token));
}

export function listPortalApiKeys(token?: string): Promise<GatewayApiKeyRecord[]> {
  return getJson<GatewayApiKeyRecord[]>('/api-keys', requiredPortalToken(token));
}

export function createPortalApiKey(
  environment: string,
  token?: string,
): Promise<CreatedGatewayApiKey> {
  return postJson<{ environment: string }, CreatedGatewayApiKey>(
    '/api-keys',
    { environment },
    requiredPortalToken(token),
  );
}
