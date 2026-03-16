import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

const requiredPackages = [
  'sdkwork-router-admin-types',
  'sdkwork-router-admin-commons',
  'sdkwork-router-admin-core',
  'sdkwork-router-admin-admin-api',
  'sdkwork-router-admin-auth',
  'sdkwork-router-admin-overview',
  'sdkwork-router-admin-users',
  'sdkwork-router-admin-tenants',
  'sdkwork-router-admin-coupons',
  'sdkwork-router-admin-catalog',
  'sdkwork-router-admin-traffic',
  'sdkwork-router-admin-operations',
];

test('standalone sdkwork-router-admin app root exists', () => {
  assert.equal(existsSync(path.join(appRoot, 'package.json')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'App.tsx')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'theme.css')), true);
  assert.equal(existsSync(path.join(appRoot, 'src-tauri', 'Cargo.toml')), true);
  assert.equal(existsSync(path.join(appRoot, 'src-tauri', 'src', 'main.rs')), true);
});

test('app root exposes standalone browser and tauri scripts', () => {
  const packageJson = JSON.parse(read('package.json'));

  assert.equal(typeof packageJson.scripts?.dev, 'string');
  assert.equal(typeof packageJson.scripts?.build, 'string');
  assert.equal(typeof packageJson.scripts?.typecheck, 'string');
  assert.equal(typeof packageJson.scripts?.preview, 'string');
  assert.equal(typeof packageJson.scripts?.['tauri:dev'], 'string');
  assert.equal(typeof packageJson.scripts?.['tauri:build'], 'string');
});

test('required packages exist under packages/', () => {
  for (const packageName of requiredPackages) {
    assert.equal(
      existsSync(path.join(appRoot, 'packages', packageName, 'package.json')),
      true,
      `missing ${packageName}`,
    );
  }
});

test('shell route manifest includes super-admin management sections', () => {
  const routes = read('packages/sdkwork-router-admin-core/src/routes.ts');

  assert.match(routes, /overview/);
  assert.match(routes, /users/);
  assert.match(routes, /tenants/);
  assert.match(routes, /coupons/);
  assert.match(routes, /catalog/);
  assert.match(routes, /traffic/);
  assert.match(routes, /operations/);
});

test('users module exposes delete capabilities for operator and portal identities', () => {
  const core = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const users = read('packages/sdkwork-router-admin-users/src/index.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /deleteOperatorUser/);
  assert.match(adminApi, /deletePortalUser/);
  assert.match(core, /onDeleteOperatorUser=/);
  assert.match(core, /onDeletePortalUser=/);
  assert.match(users, /Delete/);
});

test('tenants module exposes gateway key issuance workflow', () => {
  const core = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const tenants = read('packages/sdkwork-router-admin-tenants/src/index.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /createApiKey/);
  assert.match(adminApi, /updateApiKeyStatus/);
  assert.match(adminApi, /deleteApiKey/);
  assert.match(core, /onCreateApiKey=/);
  assert.match(core, /onUpdateApiKeyStatus=/);
  assert.match(core, /onDeleteApiKey=/);
  assert.match(tenants, /Issue gateway key/);
  assert.match(tenants, /Last issued key/);
  assert.match(tenants, /Revoke/);
  assert.match(tenants, /Delete/);
});

test('overview and traffic modules expose hotspot analytics', () => {
  const overview = read('packages/sdkwork-router-admin-overview/src/index.tsx');
  const traffic = read('packages/sdkwork-router-admin-traffic/src/index.tsx');

  assert.match(overview, /Top portal users/);
  assert.match(overview, /Hottest projects/);
  assert.match(traffic, /User traffic leaderboard/);
  assert.match(traffic, /Project hotspots/);
  assert.match(traffic, /Recent window/);
  assert.match(traffic, /Export usage CSV/);
  assert.match(traffic, /Portal user scope/);
});

test('operations module exposes runtime reload controls', () => {
  const core = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const operations = read('packages/sdkwork-router-admin-operations/src/index.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');

  assert.match(adminApi, /reloadExtensionRuntimes/);
  assert.match(core, /onReloadRuntimes=/);
  assert.match(operations, /Reload runtimes/);
  assert.match(operations, /Last reload report/);
});

test('catalog module exposes provider credential lifecycle management', () => {
  const core = read('packages/sdkwork-router-admin-core/src/index.tsx');
  const catalog = read('packages/sdkwork-router-admin-catalog/src/index.tsx');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');
  const types = read('packages/sdkwork-router-admin-types/src/index.ts');

  assert.match(types, /CredentialRecord/);
  assert.match(types, /credentials:/);
  assert.match(adminApi, /listCredentials/);
  assert.match(adminApi, /saveCredential/);
  assert.match(adminApi, /deleteCredential/);
  assert.match(core, /onSaveCredential=/);
  assert.match(core, /onDeleteCredential=/);
  assert.match(catalog, /Credential inventory/);
  assert.match(catalog, /Rotate secret/);
});

test('root app uses its own theme and does not depend on console/', () => {
  const app = read('src/App.tsx');

  assert.match(app, /import '\.\/theme\.css';/);
  assert.doesNotMatch(app, /console\//);
});

test('vite config serves static assets from the /admin/ base path', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /base:\s*'\/admin\/'/);
});
