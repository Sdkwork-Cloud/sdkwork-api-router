import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');
const modelMappingsStorageKey = 'sdkwork-router-admin.api-router.model-mappings';
const keyOverlaysStorageKey = 'sdkwork-router-admin.api-router.key-overlays';

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

test('admin gateway model mappings and overlays persist through the governed workspace local store', () => {
  const overlayStore = loadGatewayOverlayStore();
  const previousLocalStorage = globalThis.localStorage;
  const localStorage = storageDouble();

  globalThis.localStorage = localStorage;

  try {
    const created = overlayStore.createGatewayModelMapping({
      name: 'OpenAI to Claude',
      description: 'Map OpenAI requests onto the Claude upstream shape.',
      effective_from: '2026-04-21',
      rules: [
        {
          source_channel_id: 'openai',
          source_channel_name: 'OpenAI',
          source_model_id: 'gpt-5.4',
          source_model_name: 'GPT-5.4',
          target_channel_id: 'anthropic',
          target_channel_name: 'Anthropic',
          target_model_id: 'claude-sonnet-4',
          target_model_name: 'Claude Sonnet 4',
        },
      ],
    });

    overlayStore.saveGatewayApiKeyOverlay('hashed-demo', {
      source: 'custom',
      route_mode: 'custom',
      route_provider_id: 'provider-anthropic',
      model_mapping_id: created.id,
    });

    assert.match(localStorage.getItem(modelMappingsStorageKey) ?? '', /OpenAI to Claude/);
    assert.match(localStorage.getItem(keyOverlaysStorageKey) ?? '', /provider-anthropic/);

    assert.equal(overlayStore.listGatewayModelMappings().length, 1);
    assert.deepEqual(
      overlayStore.readGatewayApiKeyOverlay('hashed-demo'),
      {
        source: 'custom',
        route_mode: 'custom',
        route_provider_id: 'provider-anthropic',
        model_mapping_id: created.id,
        updated_at_ms: overlayStore.readGatewayApiKeyOverlay('hashed-demo').updated_at_ms,
      },
    );

    overlayStore.deleteGatewayModelMapping(created.id);

    assert.equal(overlayStore.listGatewayModelMappings().length, 0);
    assert.equal(
      overlayStore.readGatewayApiKeyOverlay('hashed-demo').model_mapping_id,
      null,
    );
  } finally {
    globalThis.localStorage = previousLocalStorage;
  }
});

test('admin gateway workspace persistence fails closed when browser local storage is unavailable', () => {
  const overlayStore = loadGatewayOverlayStore();
  const previousLocalStorage = globalThis.localStorage;

  globalThis.localStorage = throwingStorageDouble();

  try {
    assert.doesNotThrow(() => {
      overlayStore.createGatewayModelMapping({
        name: 'Transient mapping',
        description: '',
        effective_from: '2026-04-21',
        rules: [
          {
            source_channel_id: 'openai',
            source_channel_name: 'OpenAI',
            source_model_id: 'gpt-5.4',
            source_model_name: 'GPT-5.4',
            target_channel_id: 'openai',
            target_channel_name: 'OpenAI',
            target_model_id: 'gpt-5.4-mini',
            target_model_name: 'GPT-5.4 mini',
          },
        ],
      });
      overlayStore.saveGatewayApiKeyOverlay('hashed-transient', {
        route_mode: 'sdkwork-remote',
      });
    });
  } finally {
    globalThis.localStorage = previousLocalStorage;
  }
});

test('admin gateway non-sensitive workspace persistence is governed by a dedicated workspace store module', () => {
  const workspaceStorePath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-admin-apirouter',
    'src',
    'services',
    'gatewayWorkspaceStore.ts',
  );
  const overlayStoreSource = read(
    'packages/sdkwork-router-admin-apirouter/src/services/gatewayOverlayStore.ts',
  );

  assert.equal(existsSync(workspaceStorePath), true);
  assert.match(overlayStoreSource, /createGatewayWorkspaceStore/);
  assert.doesNotMatch(overlayStoreSource, /function storage\(/);
  assert.doesNotMatch(overlayStoreSource, /function readJson\(/);
  assert.doesNotMatch(overlayStoreSource, /function writeJson\(/);
});
