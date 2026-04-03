import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('docs center exposes a route-aware hero, implementation lanes, and operating loop', () => {
  const docsPage = read('packages/sdkwork-router-portal-docs/src/index.tsx');

  assert.match(docsPage, /PortalSiteHero/);
  assert.match(docsPage, /PortalSiteMetricCard/);
  assert.match(docsPage, /portal-docs-metrics/);
  assert.match(docsPage, /portal-docs-implementation-lanes/);
  assert.match(docsPage, /portal-docs-operating-loop/);
  assert.match(docsPage, /Documentation tracks/);
  assert.match(docsPage, /Implementation lanes/);
  assert.match(docsPage, /Operating loop/);
  assert.match(docsPage, /Move from evaluation to implementation with one route-aware documentation center\./);
  assert.match(docsPage, /Quickstart, integration, reference, and operations guidance stay connected to the same product flows used across models, downloads, and the console\./);
});
