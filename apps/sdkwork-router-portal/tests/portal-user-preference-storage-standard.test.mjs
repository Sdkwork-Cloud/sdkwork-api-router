import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from 'jiti';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadPortalUserServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-user',
      'src',
      'services',
      'index.ts',
    ),
  );
}

function loadPortalUserCenterBridge() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-types',
      'src',
      'userCenter.ts',
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

const workspace = {
  user: { id: 'user-demo' },
  tenant: { id: 'tenant-demo' },
  project: { id: 'project-demo' },
};

test('portal user preferences migrate legacy local storage drafts into session storage', () => {
  const userServices = loadPortalUserServices();
  const bridge = loadPortalUserCenterBridge();
  const previousSessionStorage = globalThis.sessionStorage;
  const previousLocalStorage = globalThis.localStorage;
  const sessionStorage = storageDouble();
  const localStorage = storageDouble();

  localStorage.setItem(
    bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.preferencesKey,
    JSON.stringify({
      'user-demo:tenant-demo:project-demo': {
        phone_number: '13800138000',
        wechat_id: 'wechat-demo',
        privacy_preferences: {
          'workspace-profile': true,
          'invite-attribution': false,
          'usage-insights': true,
        },
      },
    }),
  );

  globalThis.sessionStorage = sessionStorage;
  globalThis.localStorage = localStorage;

  try {
    assert.deepEqual(
      userServices.readPortalUserPreferenceState(workspace),
      {
        phone_number: '13800138000',
        wechat_id: 'wechat-demo',
        privacy_preferences: {
          'workspace-profile': true,
          'invite-attribution': false,
          'usage-insights': true,
        },
      },
    );

    assert.match(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.preferencesKey) ?? '',
      /wechat-demo/,
    );
    assert.equal(
      localStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.preferencesKey),
      null,
    );

    userServices.writePortalUserPreferenceState(workspace, {
      phone_number: '13900139000',
      wechat_id: 'wechat-next',
      privacy_preferences: {
        'workspace-profile': false,
        'invite-attribution': true,
        'usage-insights': false,
      },
    });

    assert.match(
      sessionStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.preferencesKey) ?? '',
      /wechat-next/,
    );
    assert.equal(
      localStorage.getItem(bridge.ROUTER_PORTAL_USER_CENTER_STORAGE_PLAN.preferencesKey),
      null,
    );
  } finally {
    globalThis.sessionStorage = previousSessionStorage;
    globalThis.localStorage = previousLocalStorage;
  }
});

test('portal user preferences are governed by a dedicated preference session store module', () => {
  const storeModulePath = path.join(
    appRoot,
    'packages',
    'sdkwork-router-portal-user',
    'src',
    'services',
    'preferenceSessionStore.ts',
  );
  const servicesSource = read(
    'packages/sdkwork-router-portal-user/src/services/index.ts',
  );

  assert.equal(existsSync(storeModulePath), true);
  assert.match(servicesSource, /createPortalUserPreferenceSessionStore/);
  assert.doesNotMatch(servicesSource, /function storage\(/);
});
