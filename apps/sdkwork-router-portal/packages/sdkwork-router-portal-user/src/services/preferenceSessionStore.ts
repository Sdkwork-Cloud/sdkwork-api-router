import type { PortalUserPreferenceState } from '../types';

interface BrowserStorageLike {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
  removeItem: (key: string) => void;
}

interface PortalUserPreferenceSessionStoreOptions {
  storageKey: string;
  sessionStorage?: BrowserStorageLike | null;
  localStorage?: BrowserStorageLike | null;
}

interface PortalUserPreferenceSessionStore {
  readPreferenceCache: () => Record<string, PortalUserPreferenceState>;
  writePreferenceCache: (value: Record<string, PortalUserPreferenceState>) => void;
}

const fallbackPreferenceCaches = new Map<string, Record<string, PortalUserPreferenceState>>();

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

function readJsonCache(
  storage: BrowserStorageLike | null,
  storageKey: string,
): { exists: boolean; value: Record<string, PortalUserPreferenceState> | null } {
  if (!storage) {
    return { exists: false, value: null };
  }

  try {
    const rawValue = storage.getItem(storageKey);
    if (!rawValue) {
      return { exists: false, value: {} };
    }

    return {
      exists: true,
      value: JSON.parse(rawValue) as Record<string, PortalUserPreferenceState>,
    };
  } catch {
    return { exists: false, value: null };
  }
}

function writeJsonCache(
  storage: BrowserStorageLike | null,
  storageKey: string,
  value: Record<string, PortalUserPreferenceState>,
): boolean {
  if (!storage) {
    return false;
  }

  try {
    if (Object.keys(value).length === 0) {
      storage.removeItem(storageKey);
      return true;
    }

    storage.setItem(storageKey, JSON.stringify(value));
    return true;
  } catch {
    return false;
  }
}

function clearLegacyCache(storage: BrowserStorageLike | null, storageKey: string): void {
  if (!storage) {
    return;
  }

  try {
    storage.removeItem(storageKey);
  } catch {
    // Ignore storage deletion failures and fail closed.
  }
}

function readManagedPreferenceCache(
  options: PortalUserPreferenceSessionStoreOptions,
): Record<string, PortalUserPreferenceState> {
  const sessionStorage = resolveBrowserStorage('sessionStorage', options.sessionStorage);
  const localStorage = resolveBrowserStorage('localStorage', options.localStorage);
  const sessionCache = readJsonCache(sessionStorage, options.storageKey);

  if (sessionCache.exists && sessionCache.value) {
    fallbackPreferenceCaches.set(options.storageKey, clone(sessionCache.value));
    clearLegacyCache(localStorage, options.storageKey);
    return clone(fallbackPreferenceCaches.get(options.storageKey) ?? {});
  }

  const legacyCache = readJsonCache(localStorage, options.storageKey);
  if (legacyCache.exists && legacyCache.value) {
    fallbackPreferenceCaches.set(options.storageKey, clone(legacyCache.value));
    const next = clone(fallbackPreferenceCaches.get(options.storageKey) ?? {});
    if (writeJsonCache(sessionStorage, options.storageKey, next)) {
      clearLegacyCache(localStorage, options.storageKey);
    }
    return next;
  }

  return clone(fallbackPreferenceCaches.get(options.storageKey) ?? {});
}

function persistManagedPreferenceCache(
  value: Record<string, PortalUserPreferenceState>,
  options: PortalUserPreferenceSessionStoreOptions,
): void {
  fallbackPreferenceCaches.set(options.storageKey, clone(value));
  const sessionStorage = resolveBrowserStorage('sessionStorage', options.sessionStorage);
  const localStorage = resolveBrowserStorage('localStorage', options.localStorage);
  const next = clone(fallbackPreferenceCaches.get(options.storageKey) ?? {});

  if (writeJsonCache(sessionStorage, options.storageKey, next)) {
    clearLegacyCache(localStorage, options.storageKey);
  }
}

export function createPortalUserPreferenceSessionStore(
  options: PortalUserPreferenceSessionStoreOptions,
): PortalUserPreferenceSessionStore {
  return {
    readPreferenceCache() {
      return readManagedPreferenceCache(options);
    },
    writePreferenceCache(value) {
      persistManagedPreferenceCache(value, options);
    },
  };
}
