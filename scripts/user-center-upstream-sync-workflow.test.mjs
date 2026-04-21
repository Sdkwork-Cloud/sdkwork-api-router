import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const repoRoot = path.resolve(import.meta.dirname, '..');
const workflowPath = path.join(repoRoot, '.github', 'workflows', 'user-center-upstream-sync.yml');

function readWorkflow() {
  return readFileSync(workflowPath, 'utf8');
}

test('router repository exposes a user-center upstream sync workflow for sdkwork-appbase dispatches', () => {
  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/user-center-upstream-sync.yml');

  const workflow = readWorkflow();

  assert.match(workflow, /repository_dispatch:/);
  assert.match(workflow, /sdkwork-appbase-user-center-standard-updated/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /permissions:\s*contents:\s*read/);
  assert.match(workflow, /actions\/checkout@v5/);
  assert.match(workflow, /actions\/setup-node@v5/);
  assert.match(workflow, /repository:\s*Sdkwork-Cloud\/sdkwork-appbase/);
  assert.match(workflow, /path:\s*external\/sdkwork-appbase/);
  assert.match(workflow, /node scripts\/user-center-upstream-sync-payload\.mjs/);
  assert.match(workflow, /SDKWORK_APPBASE_ROOT:\s*\$\{\{\s*github\.workspace\s*\}\}\/external\/sdkwork-appbase/);
  assert.match(workflow, /git -C external\/sdkwork-appbase rev-parse HEAD/);
  assert.match(workflow, /github\.event\.client_payload\.source_sha/);
  assert.match(workflow, /node scripts\/run-user-center-standard\.mjs/);
});
