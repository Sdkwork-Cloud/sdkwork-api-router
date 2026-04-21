#!/usr/bin/env node

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

export const USER_CENTER_UPSTREAM_SYNC_SOURCE_REPOSITORY = 'Sdkwork-Cloud/sdkwork-appbase';
export const USER_CENTER_UPSTREAM_SYNC_WORKFLOW = 'user-center-upstream-sync';

const __filename = fileURLToPath(import.meta.url);

export function assertUserCenterUpstreamSyncPayload(payload, {
  expectedSourceRepository = USER_CENTER_UPSTREAM_SYNC_SOURCE_REPOSITORY,
  expectedWorkflow = USER_CENTER_UPSTREAM_SYNC_WORKFLOW,
} = {}) {
  assert.equal(typeof payload, 'object', 'user-center upstream sync payload must be an object');
  assert.ok(payload, 'user-center upstream sync payload must be present');
  assert.equal(
    payload.source_repository,
    expectedSourceRepository,
    'user-center upstream sync payload must originate from the canonical sdkwork-appbase source repository',
  );
  assert.equal(
    payload.workflow,
    expectedWorkflow,
    'user-center upstream sync payload must target the canonical downstream sync workflow',
  );
  assert.match(String(payload.source_ref ?? ''), /\S/u, 'user-center upstream sync payload must include a non-empty source ref');
  assert.match(String(payload.source_sha ?? ''), /\S/u, 'user-center upstream sync payload must include a non-empty source sha');

  return {
    source_ref: payload.source_ref,
    source_repository: payload.source_repository,
    source_sha: payload.source_sha,
    workflow: payload.workflow,
  };
}

export function readUserCenterUpstreamSyncPayload({
  eventPath = process.env.GITHUB_EVENT_PATH,
} = {}) {
  assert.match(String(eventPath ?? ''), /\S/u, 'GITHUB_EVENT_PATH is required for user-center upstream sync payload validation');

  const event = JSON.parse(fs.readFileSync(eventPath, 'utf8'));
  return event.client_payload ?? {};
}

function isDirectExecution({
  argv1 = process.argv[1] ?? '',
  moduleFile = __filename,
  platform = process.platform,
} = {}) {
  if (!argv1) {
    return false;
  }

  const resolvedArgv1 = path.resolve(argv1);
  const resolvedModuleFile = path.resolve(moduleFile);
  if (platform === 'win32') {
    return resolvedArgv1.toLowerCase() === resolvedModuleFile.toLowerCase();
  }

  return resolvedArgv1 === resolvedModuleFile;
}

if (isDirectExecution()) {
  assertUserCenterUpstreamSyncPayload(readUserCenterUpstreamSyncPayload());
}
