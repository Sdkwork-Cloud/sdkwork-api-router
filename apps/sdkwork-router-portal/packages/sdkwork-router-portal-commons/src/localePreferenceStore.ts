type PortalLocale = 'en-US' | 'zh-CN';

interface BrowserStorageLike {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
}

interface PortalLocalePreferenceStoreOptions {
  localStorage?: BrowserStorageLike | null;
}

const PORTAL_I18N_STORAGE_KEY = 'sdkwork-router-portal.locale.v1';

function normalizePortalLocale(value: string | null | undefined): PortalLocale {
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

export function readPersistedPortalLocale(
  options: PortalLocalePreferenceStoreOptions = {},
): PortalLocale | null {
  const storage = resolveBrowserStorage(options.localStorage);
  if (!storage) {
    return null;
  }

  try {
    const storedValue = storage.getItem(PORTAL_I18N_STORAGE_KEY);
    if (!storedValue) {
      return null;
    }

    return normalizePortalLocale(storedValue);
  } catch {
    return null;
  }
}

export function persistPortalLocale(
  locale: PortalLocale,
  options: PortalLocalePreferenceStoreOptions = {},
): void {
  const storage = resolveBrowserStorage(options.localStorage);
  if (!storage) {
    return;
  }

  try {
    storage.setItem(PORTAL_I18N_STORAGE_KEY, locale);
  } catch {
    // Ignore storage write failures and keep the in-memory locale authoritative.
  }
}

export function createPortalLocalePreferenceStore(
  options: PortalLocalePreferenceStoreOptions = {},
) {
  return {
    readPersistedLocale() {
      return readPersistedPortalLocale(options);
    },
    persistLocale(locale: PortalLocale) {
      persistPortalLocale(locale, options);
    },
  };
}
