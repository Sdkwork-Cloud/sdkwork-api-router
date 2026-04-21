export const adminSessionTokenKey = 'sdkwork.router.admin.session-token';

type AdminBrowserStorage = Pick<Storage, 'getItem' | 'removeItem' | 'setItem'>;

export interface AdminSessionStoreOptions {
  localStorage?: AdminBrowserStorage | null;
  sessionStorage?: AdminBrowserStorage | null;
}

export interface AdminSessionStore {
  clearSessionToken: () => void;
  persistSessionToken: (token: string) => boolean;
  readSessionToken: () => string | null;
}

function normalizeAdminSessionToken(value: unknown): string | null {
  const normalized = typeof value === 'string' ? value.trim() : '';
  return normalized || null;
}

function resolveBrowserStorage(
  storageName: 'localStorage' | 'sessionStorage',
  explicitStorage: AdminBrowserStorage | null | undefined,
): AdminBrowserStorage | null {
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

  if (typeof window === 'undefined') {
    return null;
  }

  try {
    return window[storageName] ?? null;
  } catch {
    return null;
  }
}

function readStorageToken(
  storage: AdminBrowserStorage | null,
  key: string,
): string | null | undefined {
  if (!storage) {
    return undefined;
  }

  try {
    const value = storage.getItem(key);
    if (value === null) {
      return null;
    }

    const normalized = normalizeAdminSessionToken(value);
    if (!normalized) {
      storage.removeItem(key);
      return null;
    }

    return normalized;
  } catch {
    return undefined;
  }
}

function writeStorageToken(
  storage: AdminBrowserStorage | null,
  key: string,
  token: string,
): boolean {
  if (!storage) {
    return false;
  }

  try {
    storage.setItem(key, token);
    return true;
  } catch {
    return false;
  }
}

function removeStorageToken(storage: AdminBrowserStorage | null, key: string): void {
  if (!storage) {
    return;
  }

  try {
    storage.removeItem(key);
  } catch {
    // Fail closed when browser storage is unavailable.
  }
}

export function createAdminSessionStore(
  sessionTokenKey = adminSessionTokenKey,
  options: AdminSessionStoreOptions = {},
): AdminSessionStore {
  return {
    clearSessionToken() {
      removeStorageToken(
        resolveBrowserStorage('sessionStorage', options.sessionStorage),
        sessionTokenKey,
      );
      removeStorageToken(
        resolveBrowserStorage('localStorage', options.localStorage),
        sessionTokenKey,
      );
    },

    persistSessionToken(token) {
      const normalizedToken = normalizeAdminSessionToken(token);
      if (!normalizedToken) {
        throw new TypeError('Admin session token must be a non-empty string.');
      }

      const sessionStorage = resolveBrowserStorage('sessionStorage', options.sessionStorage);
      const localStorage = resolveBrowserStorage('localStorage', options.localStorage);
      const persisted = writeStorageToken(sessionStorage, sessionTokenKey, normalizedToken);

      if (persisted) {
        removeStorageToken(localStorage, sessionTokenKey);
      }

      return persisted;
    },

    readSessionToken() {
      const sessionStorage = resolveBrowserStorage('sessionStorage', options.sessionStorage);
      const localStorage = resolveBrowserStorage('localStorage', options.localStorage);
      const sessionToken = readStorageToken(sessionStorage, sessionTokenKey);

      if (sessionToken) {
        return sessionToken;
      }

      const legacyToken = readStorageToken(localStorage, sessionTokenKey);
      if (!legacyToken) {
        return null;
      }

      if (writeStorageToken(sessionStorage, sessionTokenKey, legacyToken)) {
        removeStorageToken(localStorage, sessionTokenKey);
      }

      return legacyToken;
    },
  };
}
