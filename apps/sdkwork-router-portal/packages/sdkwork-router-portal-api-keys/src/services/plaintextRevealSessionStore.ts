interface BrowserStorageLike {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
  removeItem: (key: string) => void;
}

interface PortalApiKeyPlaintextRevealRecord {
  plaintext_key: string;
  updated_at_ms: number;
}

interface PortalApiKeyPlaintextRevealSessionStoreOptions {
  sessionStorage?: BrowserStorageLike | null;
  localStorage?: BrowserStorageLike | null;
}

interface PortalApiKeyPlaintextRevealSessionStore {
  readPlaintextReveal: (hashedKey: string) => string | null;
  rememberPlaintextReveal: (hashedKey: string, plaintextKey: string) => void;
  clearPlaintextReveal: (hashedKey: string) => void;
}

export const portalApiKeyPlaintextRevealStorageKey =
  'sdkwork-router-portal.api-keys.plaintext-reveals';

let fallbackPlaintextReveals: Record<string, PortalApiKeyPlaintextRevealRecord> = {};

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
): { exists: boolean; value: Record<string, PortalApiKeyPlaintextRevealRecord> | null } {
  if (!storage) {
    return { exists: false, value: null };
  }

  try {
    const rawValue = storage.getItem(portalApiKeyPlaintextRevealStorageKey);
    if (!rawValue) {
      return { exists: false, value: {} };
    }

    return {
      exists: true,
      value: JSON.parse(rawValue) as Record<string, PortalApiKeyPlaintextRevealRecord>,
    };
  } catch {
    return { exists: false, value: null };
  }
}

function writeRevealCache(
  storage: BrowserStorageLike | null,
  value: Record<string, PortalApiKeyPlaintextRevealRecord>,
): boolean {
  if (!storage) {
    return false;
  }

  try {
    if (Object.keys(value).length === 0) {
      storage.removeItem(portalApiKeyPlaintextRevealStorageKey);
      return true;
    }

    storage.setItem(portalApiKeyPlaintextRevealStorageKey, JSON.stringify(value));
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
    storage.removeItem(portalApiKeyPlaintextRevealStorageKey);
  } catch {
    // Ignore storage deletion failures and fail closed.
  }
}

function readManagedRevealCache(
  options: PortalApiKeyPlaintextRevealSessionStoreOptions = {},
): Record<string, PortalApiKeyPlaintextRevealRecord> {
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
  value: Record<string, PortalApiKeyPlaintextRevealRecord>,
  options: PortalApiKeyPlaintextRevealSessionStoreOptions = {},
): void {
  fallbackPlaintextReveals = clone(value);
  const sessionStorage = resolveBrowserStorage('sessionStorage', options.sessionStorage);
  const localStorage = resolveBrowserStorage('localStorage', options.localStorage);

  if (writeRevealCache(sessionStorage, fallbackPlaintextReveals)) {
    clearLegacyRevealCache(localStorage);
  }
}

export function createPortalApiKeyPlaintextRevealSessionStore(
  options: PortalApiKeyPlaintextRevealSessionStoreOptions = {},
): PortalApiKeyPlaintextRevealSessionStore {
  return {
    readPlaintextReveal(hashedKey) {
      const reveal = readManagedRevealCache(options)[hashedKey];
      return reveal?.plaintext_key?.trim() || null;
    },
    rememberPlaintextReveal(hashedKey, plaintextKey) {
      if (!plaintextKey.trim()) {
        throw new TypeError('Portal API key plaintext reveal must not be empty.');
      }

      const cache = readManagedRevealCache(options);
      cache[hashedKey] = {
        plaintext_key: plaintextKey,
        updated_at_ms: Date.now(),
      };
      persistManagedRevealCache(cache, options);
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
