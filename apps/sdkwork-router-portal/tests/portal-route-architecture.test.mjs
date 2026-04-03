import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal top-level information architecture separates public modules from the console workspace', () => {
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const routePaths = read('packages/sdkwork-router-portal-core/src/application/router/routePaths.ts');
  const appRoutes = read('packages/sdkwork-router-portal-core/src/application/router/AppRoutes.tsx');
  const desktopShell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');

  for (const packageName of [
    'sdkwork-router-portal-home',
    'sdkwork-router-portal-models',
    'sdkwork-router-portal-docs',
    'sdkwork-router-portal-downloads',
    'sdkwork-router-portal-console',
  ]) {
    assert.equal(
      existsSync(path.join(appRoot, 'packages', packageName, 'package.json')),
      true,
      `missing ${packageName}`,
    );
    assert.equal(
      existsSync(path.join(appRoot, 'packages', packageName, 'src', 'index.tsx')),
      true,
      `missing ${packageName}/src/index.tsx`,
    );
  }

  assert.match(portalTypes, /PortalTopLevelRouteKey/);
  assert.match(portalTypes, /'home'/);
  assert.match(portalTypes, /'console'/);
  assert.match(portalTypes, /'models'/);
  assert.match(portalTypes, /'docs'/);
  assert.match(portalTypes, /'downloads'/);

  assert.match(routePaths, /home:\s*'\/'/);
  assert.match(routePaths, /models:\s*'\/models'/);
  assert.match(routePaths, /docs:\s*'\/docs'/);
  assert.match(routePaths, /downloads:\s*'\/downloads'/);
  assert.match(routePaths, /dashboard:\s*'\/console\/dashboard'/);
  assert.match(routePaths, /gateway:\s*'\/console\/gateway'/);
  assert.match(routePaths, /routing:\s*'\/console\/routing'/);
  assert.match(routePaths, /'api-keys':\s*'\/console\/api-keys'/);

  assert.match(appRoutes, /sdkwork-router-portal-home/);
  assert.match(appRoutes, /sdkwork-router-portal-models/);
  assert.match(appRoutes, /sdkwork-router-portal-docs/);
  assert.match(appRoutes, /sdkwork-router-portal-downloads/);
  assert.match(appRoutes, /sdkwork-router-portal-console/);
  assert.match(appRoutes, /PortalSiteLayout/);
  assert.match(appRoutes, /MainLayout/);

  assert.match(desktopShell, /PortalTopNavigation/);
  assert.match(desktopShell, /center=\{<PortalTopNavigation/);
});
