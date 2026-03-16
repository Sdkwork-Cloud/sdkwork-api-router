import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const consoleRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(consoleRoot, relativePath), 'utf8');
}

test('admin, portal, and landing apps use dedicated stylesheet entry points', () => {
  const adminApp = read('src/admin/App.tsx');
  const portalApp = read('src/portal/App.tsx');
  const landingApp = read('src/LandingApp.tsx');

  assert.match(adminApp, /import '\.\/admin\.css';/);
  assert.match(portalApp, /import '\.\/portal\.css';/);
  assert.match(landingApp, /import '\.\/landing\.css';/);

  assert.doesNotMatch(adminApp, /App\.css/);
  assert.doesNotMatch(portalApp, /App\.css/);
  assert.doesNotMatch(landingApp, /App\.css/);

  assert.equal(existsSync(path.join(consoleRoot, 'src/admin/admin.css')), true);
  assert.equal(existsSync(path.join(consoleRoot, 'src/portal/portal.css')), true);
  assert.equal(existsSync(path.join(consoleRoot, 'src/landing.css')), true);
});

test('admin and portal apps expose different product roots', () => {
  const adminApp = read('src/admin/App.tsx');
  const portalApp = read('src/portal/App.tsx');

  assert.match(adminApp, /className="admin-app"/);
  assert.match(portalApp, /className="portal-app"/);
});

test('admin and portal SDKs keep isolated storage keys and API prefixes', () => {
  const adminSdk = read('packages/sdkwork-api-admin-sdk/src/index.ts');
  const portalSdk = read('packages/sdkwork-api-portal-sdk/src/index.ts');

  assert.match(adminSdk, /const adminSessionTokenKey = 'sdkwork\.admin\.session-token';/);
  assert.match(portalSdk, /const portalSessionTokenKey = 'sdkwork\.portal\.session-token';/);
  assert.match(adminSdk, /return '\/api\/admin';/);
  assert.match(portalSdk, /return '\/api\/portal';/);
});

test('admin and portal HTML entrypoints stay separate', () => {
  const adminHtml = read('admin/index.html');
  const portalHtml = read('portal/index.html');

  assert.match(adminHtml, /src="\/src\/admin\/main\.tsx"/);
  assert.match(portalHtml, /src="\/src\/portal\/main\.tsx"/);
});
