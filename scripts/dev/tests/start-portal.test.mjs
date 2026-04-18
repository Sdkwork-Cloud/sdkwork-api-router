import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('start-portal routes --tauri through the portal desktop shell entrypoint', () => {
  const result = spawnSync(
    process.execPath,
    ['scripts/dev/start-portal.mjs', '--tauri', '--dry-run'],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(
    result.stdout,
    /\[start-portal\].*apps\/sdkwork-router-portal tauri:dev/s,
  );
});
