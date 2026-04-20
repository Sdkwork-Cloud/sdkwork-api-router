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
    /\[start-portal\].*\btauri:dev\b/s,
  );
  assert.doesNotMatch(result.stdout, /\s--dir\s/);
});

test('start-portal forwards an explicit dev port override to the underlying Vite command', () => {
  const result = spawnSync(
    process.execPath,
    ['scripts/dev/start-portal.mjs', '--port', '16174', '--dry-run'],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /\[start-portal\].*\bdev\b.*--port 16174\b.*--strictPort\b/s);
});
