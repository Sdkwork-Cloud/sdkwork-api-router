import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadPortalLocalePreferenceStore() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-commons',
      'src',
      'localePreferenceStore.ts',
    ),
  );
}

function storageDouble() {
  const store = new Map();
  return {
    getItem(key) {
      return store.has(key) ? store.get(key) : null;
    },
    setItem(key, value) {
      store.set(String(key), String(value));
    },
    removeItem(key) {
      store.delete(String(key));
    },
  };
}

function throwingStorageDouble() {
  return {
    getItem() {
      throw new Error('storage unavailable');
    },
    setItem() {
      throw new Error('storage unavailable');
    },
    removeItem() {
      throw new Error('storage unavailable');
    },
  };
}

test('portal i18n locale preferences persist through a dedicated governed store module', () => {
  const localeStore = loadPortalLocalePreferenceStore();
  const previousLocalStorage = globalThis.localStorage;
  const localStorage = storageDouble();

  globalThis.localStorage = localStorage;

  try {
    assert.equal(localeStore.readPersistedPortalLocale(), null);

    localeStore.persistPortalLocale('zh-CN');
    assert.equal(localeStore.readPersistedPortalLocale(), 'zh-CN');

    localStorage.setItem('sdkwork-router-portal.locale.v1', 'zh-Hans-CN');
    assert.equal(localeStore.readPersistedPortalLocale(), 'zh-CN');

    localStorage.setItem('sdkwork-router-portal.locale.v1', 'en-GB');
    assert.equal(localeStore.readPersistedPortalLocale(), 'en-US');
  } finally {
    globalThis.localStorage = previousLocalStorage;
  }
});

test('portal i18n locale preference store fails closed when browser local storage is unavailable', () => {
  const localeStore = loadPortalLocalePreferenceStore();
  const previousLocalStorage = globalThis.localStorage;

  globalThis.localStorage = throwingStorageDouble();

  try {
    assert.equal(localeStore.readPersistedPortalLocale(), null);
    assert.doesNotThrow(() => {
      localeStore.persistPortalLocale('zh-CN');
    });
  } finally {
    globalThis.localStorage = previousLocalStorage;
  }
});

test('portal i18n locale persistence is governed by a dedicated store module', () => {
  const storeModulePath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-commons',
    'src',
    'localePreferenceStore.ts',
  );
  const i18nSource = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.equal(existsSync(storeModulePath), true);
  assert.match(i18nSource, /readPersistedPortalLocale/);
  assert.match(i18nSource, /persistPortalLocale/);
  assert.doesNotMatch(i18nSource, /window\.localStorage\.getItem/);
  assert.doesNotMatch(i18nSource, /window\.localStorage\.setItem/);
});
