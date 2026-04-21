import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');
const adminSessionTokenKey = 'sdkwork.router.admin.session-token';

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadAdminApi() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(appRoot, 'packages', 'sdkwork-router-admin-admin-api', 'src', 'index.ts'),
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
      throw new Error('storage read blocked');
    },
    setItem() {
      throw new Error('storage write blocked');
    },
    removeItem() {
      throw new Error('storage delete blocked');
    },
  };
}

test('admin session helpers migrate legacy local storage tokens into session storage', () => {
  const adminApi = loadAdminApi();
  const previousSessionStorage = globalThis.sessionStorage;
  const previousLocalStorage = globalThis.localStorage;
  const sessionStorage = storageDouble();
  const localStorage = storageDouble();

  localStorage.setItem(adminSessionTokenKey, 'legacy-admin-session-token');

  globalThis.sessionStorage = sessionStorage;
  globalThis.localStorage = localStorage;

  try {
    assert.equal(
      adminApi.readAdminSessionToken(),
      'legacy-admin-session-token',
    );
    assert.equal(
      sessionStorage.getItem(adminSessionTokenKey),
      'legacy-admin-session-token',
    );
    assert.equal(
      localStorage.getItem(adminSessionTokenKey),
      null,
    );

    adminApi.persistAdminSessionToken('fresh-admin-session-token');

    assert.equal(
      sessionStorage.getItem(adminSessionTokenKey),
      'fresh-admin-session-token',
    );

    adminApi.clearAdminSessionToken();

    assert.equal(
      sessionStorage.getItem(adminSessionTokenKey),
      null,
    );
    assert.equal(
      localStorage.getItem(adminSessionTokenKey),
      null,
    );
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }
});

test('admin api governs browser session persistence through a dedicated session store module', () => {
  const sessionStorePath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-admin-admin-api',
    'src',
    'sessionStore.ts',
  );
  const indexSource = read('packages/sdkwork-router-admin-admin-api/src/index.ts');
  const transportSource = read('packages/sdkwork-router-admin-admin-api/src/transport.ts');

  assert.equal(existsSync(sessionStorePath), true);
  assert.match(indexSource, /createAdminSessionStore/);
  assert.match(transportSource, /createAdminSessionStore/);
  assert.doesNotMatch(transportSource, /resolveBrowserStorage/);
  assert.doesNotMatch(transportSource, /readStorageToken/);
  assert.doesNotMatch(transportSource, /writeStorageToken/);
  assert.doesNotMatch(transportSource, /removeStorageToken/);
});

test('admin session helpers reject malformed tokens and fail closed when browser storage is unavailable', () => {
  const adminApi = loadAdminApi();
  const previousSessionStorage = globalThis.sessionStorage;
  const previousLocalStorage = globalThis.localStorage;

  globalThis.sessionStorage = throwingStorageDouble();
  globalThis.localStorage = throwingStorageDouble();

  try {
    assert.equal(adminApi.readAdminSessionToken(), null);
    assert.doesNotThrow(() => adminApi.persistAdminSessionToken('admin-demo-session'));
    assert.doesNotThrow(() => adminApi.clearAdminSessionToken());
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }

  const sessionStorage = storageDouble();
  const localStorage = storageDouble();
  globalThis.sessionStorage = sessionStorage;
  globalThis.localStorage = localStorage;

  try {
    assert.throws(
      () => adminApi.persistAdminSessionToken(''),
      {
        name: 'TypeError',
      },
    );

    assert.equal(
      sessionStorage.getItem(adminSessionTokenKey),
      null,
    );
    assert.equal(
      localStorage.getItem(adminSessionTokenKey),
      null,
    );
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }
});
