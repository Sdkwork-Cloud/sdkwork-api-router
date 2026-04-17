import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import ts from '../node_modules/typescript/lib/typescript.js';

const consoleRoot = path.resolve(import.meta.dirname, '..');

async function loadTsModule(relativePath) {
  const source = readFileSync(path.join(consoleRoot, relativePath), 'utf8');
  const transpiled = ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.ES2022,
      target: ts.ScriptTarget.ES2022,
    },
    fileName: relativePath,
  }).outputText;

  return import(`data:text/javascript;base64,${Buffer.from(transpiled).toString('base64')}`);
}

function jsonTextResponse(body, init) {
  return new Response(body, {
    status: 200,
    headers: {
      'content-type': 'application/json',
    },
    ...init,
  });
}

test('console admin sdk preserves unsafe integers from successful responses', async () => {
  const adminSdk = await loadTsModule('packages/sdkwork-api-admin-sdk/src/index.ts');
  const previousFetch = globalThis.fetch;
  const previousLocalStorage = globalThis.localStorage;

  globalThis.localStorage = {
    getItem(key) {
      return key === 'sdkwork.admin.session-token' ? 'admin-session-token' : null;
    },
    setItem() {},
    removeItem() {},
  };
  globalThis.fetch = async () =>
    jsonTextResponse(
      '{"total_requests":0,"project_count":0,"model_count":0,"provider_count":0,"projects":[],"providers":[],"models":[],"unsafe_marker":9007199254740993}',
    );

  try {
    const usageSummary = await adminSdk.getUsageSummary();
    assert.equal(usageSummary.unsafe_marker, '9007199254740993');
  } finally {
    globalThis.fetch = previousFetch;
    globalThis.localStorage = previousLocalStorage;
  }
});

test('console portal sdk preserves unsafe integers from successful responses', async () => {
  const portalSdk = await loadTsModule('packages/sdkwork-api-portal-sdk/src/index.ts');
  const previousFetch = globalThis.fetch;
  const previousLocalStorage = globalThis.localStorage;

  globalThis.localStorage = {
    getItem(key) {
      return key === 'sdkwork.portal.session-token' ? 'portal-session-token' : null;
    },
    setItem() {},
    removeItem() {},
  };
  globalThis.fetch = async () =>
    jsonTextResponse(
      '{"user":{"id":"user-1","email":"portal@example.com","display_name":"Portal User","workspace_tenant_id":"tenant-1","workspace_project_id":"project-1","active":true,"created_at_ms":1},"tenant":{"id":"tenant-1","name":"Tenant"},"project":{"tenant_id":"tenant-1","id":"project-1","name":"Project"},"unsafe_marker":9007199254740993}',
    );

  try {
    const workspace = await portalSdk.getPortalWorkspace();
    assert.equal(workspace.unsafe_marker, '9007199254740993');
  } finally {
    globalThis.fetch = previousFetch;
    globalThis.localStorage = previousLocalStorage;
  }
});
