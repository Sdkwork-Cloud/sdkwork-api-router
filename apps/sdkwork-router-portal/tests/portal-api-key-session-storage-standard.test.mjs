import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');
const portalPlaintextRevealStorageKey =
  'sdkwork-router-portal.api-keys.plaintext-reveals';

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadPortalApiKeyServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-api-keys',
      'src',
      'services',
      'index.ts',
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

test('portal api-key plaintext reveals migrate from legacy local storage into session storage', () => {
  const apiKeyServices = loadPortalApiKeyServices();
  const previousSessionStorage = globalThis.sessionStorage;
  const previousLocalStorage = globalThis.localStorage;
  const sessionStorage = storageDouble();
  const localStorage = storageDouble();

  localStorage.setItem(
    portalPlaintextRevealStorageKey,
    JSON.stringify({
      'hashed-demo': {
        plaintext_key: 'sk-portal-demo',
        updated_at_ms: 1,
      },
    }),
  );

  globalThis.sessionStorage = sessionStorage;
  globalThis.localStorage = localStorage;

  try {
    assert.equal(
      apiKeyServices.readPortalApiKeyPlaintextReveal('hashed-demo'),
      'sk-portal-demo',
    );

    assert.match(
      sessionStorage.getItem(portalPlaintextRevealStorageKey) ?? '',
      /sk-portal-demo/,
    );
    assert.equal(
      localStorage.getItem(portalPlaintextRevealStorageKey),
      null,
    );

    apiKeyServices.rememberPortalApiKeyPlaintextReveal(
      'hashed-fresh',
      'sk-portal-fresh',
    );

    assert.match(
      sessionStorage.getItem(portalPlaintextRevealStorageKey) ?? '',
      /sk-portal-fresh/,
    );
    assert.equal(
      localStorage.getItem(portalPlaintextRevealStorageKey),
      null,
    );
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }
});

test('portal api-key plaintext reveals are governed by a dedicated session store module', () => {
  const storeModulePath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-api-keys',
    'src',
    'services',
    'plaintextRevealSessionStore.ts',
  );
  const servicesSource = read(
    'packages/sdkwork-router-portal-api-keys/src/services/index.ts',
  );

  assert.equal(existsSync(storeModulePath), true);
  assert.match(servicesSource, /createPortalApiKeyPlaintextRevealSessionStore/);
  assert.doesNotMatch(
    servicesSource,
    /sdkwork-router-portal\.api-keys\.plaintext-reveals/,
  );
  assert.doesNotMatch(servicesSource, /function storage\(/);
});
