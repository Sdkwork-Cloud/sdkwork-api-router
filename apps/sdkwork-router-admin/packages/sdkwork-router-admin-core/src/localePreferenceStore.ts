type AdminLocale = 'en-US' | 'zh-CN';

interface BrowserStorageLike {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
}

interface AdminLocalePreferenceStoreOptions {
  localStorage?: BrowserStorageLike | null;
}

const ADMIN_I18N_STORAGE_KEY = 'sdkwork-router-admin.locale.v2';

function normalizeAdminLocale(value: string | null | undefined): AdminLocale {
  if (!value) {
    return 'en-US';
  }

  return value.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}

function resolveBrowserStorage(
  explicitStorage?: BrowserStorageLike | null,
): BrowserStorageLike | null {
  if (explicitStorage !== undefined) {
    return explicitStorage;
  }

  try {
    if (globalThis.localStorage) {
      return globalThis.localStorage;
    }
  } catch {
    return null;
  }

  try {
    if (typeof window === 'undefined') {
      return null;
    }

    return window.localStorage ?? null;
  } catch {
    return null;
  }
}

export function readPersistedAdminLocale(
  options: AdminLocalePreferenceStoreOptions = {},
): AdminLocale | null {
  const storage = resolveBrowserStorage(options.localStorage);
  if (!storage) {
    return null;
  }

  try {
    const storedValue = storage.getItem(ADMIN_I18N_STORAGE_KEY);
    if (!storedValue) {
      return null;
    }

    return normalizeAdminLocale(storedValue);
  } catch {
    return null;
  }
}

export function persistAdminLocale(
  locale: AdminLocale,
  options: AdminLocalePreferenceStoreOptions = {},
): void {
  const storage = resolveBrowserStorage(options.localStorage);
  if (!storage) {
    return;
  }

  try {
    storage.setItem(ADMIN_I18N_STORAGE_KEY, locale);
  } catch {
    // Ignore storage write failures and keep the in-memory locale authoritative.
  }
}

export function createAdminLocalePreferenceStore(
  options: AdminLocalePreferenceStoreOptions = {},
) {
  return {
    readPersistedLocale() {
      return readPersistedAdminLocale(options);
    },
    persistLocale(locale: AdminLocale) {
      persistAdminLocale(locale, options);
    },
  };
}
