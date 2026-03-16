import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

const requiredPackages = [
  'sdkwork-router-portal-types',
  'sdkwork-router-portal-commons',
  'sdkwork-router-portal-core',
  'sdkwork-router-portal-portal-api',
  'sdkwork-router-portal-auth',
  'sdkwork-router-portal-dashboard',
  'sdkwork-router-portal-api-keys',
  'sdkwork-router-portal-usage',
  'sdkwork-router-portal-credits',
  'sdkwork-router-portal-billing',
  'sdkwork-router-portal-account',
];

const requiredBusinessPackages = [
  'sdkwork-router-portal-auth',
  'sdkwork-router-portal-dashboard',
  'sdkwork-router-portal-api-keys',
  'sdkwork-router-portal-usage',
  'sdkwork-router-portal-credits',
  'sdkwork-router-portal-billing',
  'sdkwork-router-portal-account',
];

test('standalone sdkwork-router-portal app root exists', () => {
  assert.equal(existsSync(path.join(appRoot, 'package.json')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'App.tsx')), true);
  assert.equal(existsSync(path.join(appRoot, 'src', 'theme.css')), true);
});

test('app root exposes dev, build, typecheck, and preview scripts', () => {
  const packageJson = JSON.parse(read('package.json'));

  assert.equal(typeof packageJson.scripts?.dev, 'string');
  assert.equal(typeof packageJson.scripts?.build, 'string');
  assert.equal(typeof packageJson.scripts?.typecheck, 'string');
  assert.equal(typeof packageJson.scripts?.preview, 'string');
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

test('business packages follow the ARCHITECT directory convention', () => {
  for (const packageName of requiredBusinessPackages) {
    const srcRoot = path.join(appRoot, 'packages', packageName, 'src');

    for (const directory of ['types', 'components', 'repository', 'services', 'pages']) {
      assert.equal(
        existsSync(path.join(srcRoot, directory)),
        true,
        `${packageName} is missing src/${directory}`,
      );
    }

    const entryFile = read(path.join('packages', packageName, 'src', 'index.tsx'));
    assert.match(entryFile, /from '\.\/pages'/, `${packageName} entry must re-export from pages`);
  }
});

test('shell route manifest includes the portal product sections', () => {
  const routes = read('packages/sdkwork-router-portal-core/src/routes.ts');

  assert.match(routes, /dashboard/);
  assert.match(routes, /api-keys/);
  assert.match(routes, /usage/);
  assert.match(routes, /credits/);
  assert.match(routes, /billing/);
  assert.match(routes, /account/);
});

test('root app uses its own theme and does not depend on console/', () => {
  const app = read('src/App.tsx');

  assert.match(app, /import '\.\/theme\.css';/);
  assert.doesNotMatch(app, /console\//);
});

test('vite config serves static assets from the /portal/ base path', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /base:\s*'\/portal\/'/);
});
