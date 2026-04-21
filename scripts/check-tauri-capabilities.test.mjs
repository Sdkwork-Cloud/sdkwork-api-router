import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('tauri capability audit resolves command and window permission requirements for the desktop apps', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-tauri-capabilities.mjs')).href,
  );

  assert.equal(typeof module.runTauriCapabilityAudit, 'function');
  assert.equal(typeof module.permissionIdentifierForCommand, 'function');
  assert.equal(typeof module.permissionIdentifierForWindowMethod, 'function');
  assert.equal(typeof module.parseBuildCommandNames, 'function');
  assert.equal(typeof module.detectWindowControllerMethods, 'function');

  const result = module.runTauriCapabilityAudit({ workspaceRoot });
  assert.deepEqual(
    result.apps.map(({ appId }) => appId),
    ['portal', 'admin'],
  );

  const portal = result.apps.find(({ appId }) => appId === 'portal');
  const admin = result.apps.find(({ appId }) => appId === 'admin');

  assert.deepEqual(
    portal.requiredWindowPermissions,
    [
      'core:window:allow-close',
      'core:window:allow-maximize',
      'core:window:allow-minimize',
      'core:window:allow-toggle-maximize',
    ],
  );
  assert.deepEqual(
    portal.requiredCommandPermissions,
    [
      'allow-install-api-router-client-setup',
      'allow-list-api-key-instances',
      'allow-restart-product-runtime',
      'allow-runtime-base-url',
      'allow-runtime-desktop-snapshot',
      'allow-update-desktop-runtime-access-mode',
    ],
  );
  assert.deepEqual(portal.missingWindowPermissions, []);
  assert.deepEqual(portal.missingCommandPermissions, []);
  assert.deepEqual(
    portal.approvedTauriGlobalPaths,
    [
      'apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/desktopBridge.ts',
    ],
  );
  assert.deepEqual(
    portal.approvedWindowApiImportPaths,
    [
      'apps/sdkwork-router-portal/packages/sdkwork-router-portal-portal-api/src/desktopBridge.ts',
    ],
  );
  assert.deepEqual(portal.unapprovedTauriGlobalAccessPaths, []);
  assert.deepEqual(portal.unapprovedWindowApiImportPaths, []);

  assert.deepEqual(
    admin.requiredWindowPermissions,
    [
      'core:window:allow-close',
      'core:window:allow-is-maximized',
      'core:window:allow-maximize',
      'core:window:allow-minimize',
      'core:window:allow-toggle-maximize',
      'core:window:allow-unmaximize',
    ],
  );
  assert.deepEqual(
    admin.requiredCommandPermissions,
    [
      'allow-install-api-router-client-setup',
      'allow-list-api-key-instances',
      'allow-runtime-base-url',
    ],
  );
  assert.deepEqual(admin.missingWindowPermissions, []);
  assert.deepEqual(admin.missingCommandPermissions, []);
  assert.deepEqual(
    admin.approvedTauriGlobalPaths,
    [
      'apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/desktopBridge.ts',
    ],
  );
  assert.deepEqual(
    admin.approvedWindowApiImportPaths,
    [
      'apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/desktopBridge.ts',
    ],
  );
  assert.deepEqual(admin.unapprovedTauriGlobalAccessPaths, []);
  assert.deepEqual(admin.unapprovedWindowApiImportPaths, []);
});

test('tauri capability audit helpers translate generated command names and window methods into capability identifiers', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-tauri-capabilities.mjs')).href,
  );

  assert.equal(
    module.permissionIdentifierForCommand('update_desktop_runtime_access_mode'),
    'allow-update-desktop-runtime-access-mode',
  );
  assert.equal(
    module.permissionIdentifierForWindowMethod('toggleMaximize'),
    'core:window:allow-toggle-maximize',
  );
  assert.equal(
    module.permissionIdentifierForWindowMethod('isMaximized'),
    'core:window:allow-is-maximized',
  );
});
