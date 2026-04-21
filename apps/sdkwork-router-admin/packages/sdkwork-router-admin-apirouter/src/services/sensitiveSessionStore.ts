export interface GatewayApiKeyPlaintextRevealRecord {
  plaintext_key: string;
  source: 'system-generated' | 'custom';
  updated_at_ms: number;
}

interface BrowserStorageLike {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
  removeItem: (key: string) => void;
}

interface GatewaySensitiveSessionStoreOptions {
  sessionStorage?: BrowserStorageLike | null;
  localStorage?: BrowserStorageLike | null;
}

interface GatewaySensitiveSessionStore {
  readPlaintextReveal: (hashedKey: string) => GatewayApiKeyPlaintextRevealRecord | null;
  rememberPlaintextReveal: (input: {
    hashed_key: string;
    plaintext_key: string;
    source?: 'system-generated' | 'custom' | null;
  }) => GatewayApiKeyPlaintextRevealRecord;
  clearPlaintextReveal: (hashedKey: string) => void;
}

export const gatewayApiKeyPlaintextRevealStorageKey =
  'sdkwork-router-admin.api-router.plaintext-reveals';

let fallbackPlaintextReveals: Record<string, GatewayApiKeyPlaintextRevealRecord> = {};

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function resolveBrowserStorage(
  storageName: 'localStorage' | 'sessionStorage',
  explicitStorage?: BrowserStorageLike | null,
): BrowserStorageLike | null {
  if (explicitStorage !== undefined) {
    return explicitStorage;
  }

  try {
    const storage = globalThis[storageName];
    if (storage) {
      return storage;
    }
  } catch {
    return null;
  }

  try {
    if (typeof window === 'undefined') {
      return null;
    }

    return window[storageName] ?? null;
  } catch {
    return null;
  }
}

function readRevealCache(
  storage: BrowserStorageLike | null,
): { exists: boolean; value: Record<string, GatewayApiKeyPlaintextRevealRecord> | null } {
  if (!storage) {
    return { exists: false, value: null };
  }

  try {
    const rawValue = storage.getItem(gatewayApiKeyPlaintextRevealStorageKey);
    if (!rawValue) {
      return { exists: false, value: {} };
    }

    return {
      exists: true,
      value: JSON.parse(rawValue) as Record<string, GatewayApiKeyPlaintextRevealRecord>,
    };
  } catch {
    return { exists: false, value: null };
  }
}

function writeRevealCache(
  storage: BrowserStorageLike | null,
  value: Record<string, GatewayApiKeyPlaintextRevealRecord>,
): boolean {
  if (!storage) {
    return false;
  }

  try {
    if (Object.keys(value).length === 0) {
      storage.removeItem(gatewayApiKeyPlaintextRevealStorageKey);
      return true;
    }

    storage.setItem(
      gatewayApiKeyPlaintextRevealStorageKey,
      JSON.stringify(value),
    );
    return true;
  } catch {
    return false;
  }
}

function clearLegacyRevealCache(storage: BrowserStorageLike | null): void {
  if (!storage) {
    return;
  }

  try {
    storage.removeItem(gatewayApiKeyPlaintextRevealStorageKey);
  } catch {
    // Ignore storage deletion failures and fail closed.
  }
}

function normalizePlaintextReveal(
  value: GatewayApiKeyPlaintextRevealRecord | null | undefined,
): GatewayApiKeyPlaintextRevealRecord | null {
  if (!value?.plaintext_key?.trim()) {
    return null;
  }

  return {
    plaintext_key: value.plaintext_key,
    source: value.source === 'custom' ? 'custom' : 'system-generated',
    updated_at_ms: value.updated_at_ms ?? 0,
  };
}

function readManagedRevealCache(
  options: GatewaySensitiveSessionStoreOptions = {},
): Record<string, GatewayApiKeyPlaintextRevealRecord> {
  const sessionStorage = resolveBrowserStorage('sessionStorage', options.sessionStorage);
  const localStorage = resolveBrowserStorage('localStorage', options.localStorage);
  const sessionCache = readRevealCache(sessionStorage);

  if (sessionCache.exists && sessionCache.value) {
    fallbackPlaintextReveals = clone(sessionCache.value);
    clearLegacyRevealCache(localStorage);
    return clone(fallbackPlaintextReveals);
  }

  const legacyCache = readRevealCache(localStorage);
  if (legacyCache.exists && legacyCache.value) {
    fallbackPlaintextReveals = clone(legacyCache.value);
    if (writeRevealCache(sessionStorage, fallbackPlaintextReveals)) {
      clearLegacyRevealCache(localStorage);
    }
    return clone(fallbackPlaintextReveals);
  }

  return clone(fallbackPlaintextReveals);
}

function persistManagedRevealCache(
  value: Record<string, GatewayApiKeyPlaintextRevealRecord>,
  options: GatewaySensitiveSessionStoreOptions = {},
): void {
  fallbackPlaintextReveals = clone(value);
  const sessionStorage = resolveBrowserStorage('sessionStorage', options.sessionStorage);
  const localStorage = resolveBrowserStorage('localStorage', options.localStorage);

  if (writeRevealCache(sessionStorage, fallbackPlaintextReveals)) {
    clearLegacyRevealCache(localStorage);
  }
}

export function createGatewaySensitiveSessionStore(
  options: GatewaySensitiveSessionStoreOptions = {},
): GatewaySensitiveSessionStore {
  return {
    readPlaintextReveal(hashedKey) {
      const reveal = normalizePlaintextReveal(
        readManagedRevealCache(options)[hashedKey],
      );
      return reveal ? clone(reveal) : null;
    },
    rememberPlaintextReveal(input) {
      if (!input.plaintext_key.trim()) {
        throw new TypeError('Gateway plaintext reveal must not be empty.');
      }

      const next: GatewayApiKeyPlaintextRevealRecord = {
        plaintext_key: input.plaintext_key,
        source: input.source === 'custom' ? 'custom' : 'system-generated',
        updated_at_ms: Date.now(),
      };
      const cache = readManagedRevealCache(options);
      cache[input.hashed_key] = next;
      persistManagedRevealCache(cache, options);
      return clone(next);
    },
    clearPlaintextReveal(hashedKey) {
      const cache = readManagedRevealCache(options);
      if (!cache[hashedKey]) {
        return;
      }

      delete cache[hashedKey];
      persistManagedRevealCache(cache, options);
    },
  };
}
