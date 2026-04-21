import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');
const gatewayPlaintextRevealStorageKey =
  'sdkwork-router-admin.api-router.plaintext-reveals';

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadGatewayOverlayStore() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-apirouter',
      'src',
      'services',
      'gatewayOverlayStore.ts',
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

test('admin gateway plaintext reveals migrate from legacy local storage into session storage', () => {
  const overlayStore = loadGatewayOverlayStore();
  const previousSessionStorage = globalThis.sessionStorage;
  const previousLocalStorage = globalThis.localStorage;
  const sessionStorage = storageDouble();
  const localStorage = storageDouble();

  localStorage.setItem(
    gatewayPlaintextRevealStorageKey,
    JSON.stringify({
      'hashed-demo': {
        plaintext_key: 'sk-admin-demo',
        source: 'custom',
        updated_at_ms: 1,
      },
    }),
  );

  globalThis.sessionStorage = sessionStorage;
  globalThis.localStorage = localStorage;

  try {
    assert.deepEqual(
      overlayStore.readGatewayApiKeyPlaintextReveal('hashed-demo'),
      {
        plaintext_key: 'sk-admin-demo',
        source: 'custom',
        updated_at_ms: 1,
      },
    );

    assert.match(
      sessionStorage.getItem(gatewayPlaintextRevealStorageKey) ?? '',
      /sk-admin-demo/,
    );
    assert.equal(
      localStorage.getItem(gatewayPlaintextRevealStorageKey),
      null,
    );

    overlayStore.rememberGatewayApiKeyPlaintextReveal({
      hashed_key: 'hashed-fresh',
      plaintext_key: 'sk-admin-fresh',
      source: 'system-generated',
    });

    assert.match(
      sessionStorage.getItem(gatewayPlaintextRevealStorageKey) ?? '',
      /sk-admin-fresh/,
    );
    assert.equal(
      localStorage.getItem(gatewayPlaintextRevealStorageKey),
      null,
    );
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }
});

test('admin gateway overlay secrets are governed by a dedicated sensitive session store module', () => {
  const storeModulePath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-admin-apirouter',
    'src',
    'services',
    'sensitiveSessionStore.ts',
  );
  const overlayStoreSource = read(
    'packages/sdkwork-router-admin-apirouter/src/services/gatewayOverlayStore.ts',
  );

  assert.equal(existsSync(storeModulePath), true);
  assert.match(overlayStoreSource, /createGatewaySensitiveSessionStore/);
  assert.doesNotMatch(
    overlayStoreSource,
    /sdkwork-router-admin\.api-router\.plaintext-reveals/,
  );
  assert.doesNotMatch(
    overlayStoreSource,
    /sdkwork-router-admin\.api-router\.provider-secrets/,
  );
});
