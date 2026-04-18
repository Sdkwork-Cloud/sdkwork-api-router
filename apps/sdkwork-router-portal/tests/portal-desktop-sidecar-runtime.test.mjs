import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal desktop package stages a router-product sidecar payload instead of raw embedded-sites resources', () => {
  const tauriConfig = JSON.parse(read('src-tauri/tauri.conf.json'));

  assert.equal(
    tauriConfig.build?.beforeBuildCommand,
    'node ../../scripts/prepare-router-portal-desktop-runtime.mjs',
  );
  assert.deepEqual(
    tauriConfig.bundle?.resources,
    {
      '../../../bin/portal-rt/router-product/': 'router-product/',
    },
  );
});

test('portal tauri runtime delegates desktop startup to sidecar supervision and exposes access-mode commands', () => {
  const tauriMain = read('src-tauri/src/main.rs');

  assert.match(tauriMain, /router-product-service/);
  assert.match(tauriMain, /update_desktop_runtime_access_mode/);
  assert.match(tauriMain, /runtime_desktop_snapshot/);
  assert.doesNotMatch(tauriMain, /RouterProductRuntime::start/);
});

test('portal settings center exposes local-only and shared access controls for the fixed public desktop port', () => {
  const settingsCenter = read('packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx');
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');
  const types = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(settingsCenter, /Local-only access/);
  assert.match(settingsCenter, /Shared network access/);
  assert.match(settingsCenter, /127\.0\.0\.1:3001/);
  assert.match(settingsCenter, /0\.0\.0\.0:3001/);
  assert.match(settingsCenter, /updateDesktopRuntimeAccessMode/);
  assert.match(portalApi, /updateDesktopRuntimeAccessMode/);
  assert.match(portalApi, /update_desktop_runtime_access_mode/);
  assert.match(types, /accessMode/);
  assert.match(types, /'local' \| 'shared'/);
});
