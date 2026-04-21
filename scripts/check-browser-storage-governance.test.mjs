import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('browser storage governance audit centralizes approved storage access to governed store modules', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-browser-storage-governance.mjs')).href,
  );

  assert.equal(typeof module.runBrowserStorageGovernanceCheck, 'function');
  assert.equal(typeof module.scanBrowserStorageGovernance, 'function');

  const result = module.runBrowserStorageGovernanceCheck({ workspaceRoot });
  assert.equal(result.ok, true);
  assert.deepEqual(
    result.scopes.map(({ scopeId }) => scopeId),
    ['admin', 'portal'],
  );

  const admin = result.scopes.find(({ scopeId }) => scopeId === 'admin');
  const portal = result.scopes.find(({ scopeId }) => scopeId === 'portal');

  assert.deepEqual(
    admin.approvedStoragePaths,
    [
      'apps/sdkwork-router-admin/packages/sdkwork-router-admin-admin-api/src/sessionStore.ts',
      'apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/services/gatewayWorkspaceStore.ts',
      'apps/sdkwork-router-admin/packages/sdkwork-router-admin-apirouter/src/services/sensitiveSessionStore.ts',
      'apps/sdkwork-router-admin/packages/sdkwork-router-admin-core/src/localePreferenceStore.ts',
    ],
  );
  assert.deepEqual(admin.unapprovedStorageAccessPaths, []);

  assert.deepEqual(
    portal.approvedStoragePaths,
    [
      'apps/sdkwork-router-portal/packages/sdkwork-router-portal-api-keys/src/services/plaintextRevealSessionStore.ts',
      'apps/sdkwork-router-portal/packages/sdkwork-router-portal-commons/src/localePreferenceStore.ts',
      'apps/sdkwork-router-portal/packages/sdkwork-router-portal-user/src/services/preferenceSessionStore.ts',
    ],
  );
  assert.deepEqual(portal.unapprovedStorageAccessPaths, []);
});
