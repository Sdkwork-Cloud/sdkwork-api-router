import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import path from 'node:path';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('start-admin routes --tauri through the admin desktop shell entrypoint without pnpm --dir indirection', () => {
  const result = spawnSync(
    process.execPath,
    ['scripts/dev/start-admin.mjs', '--tauri', '--dry-run'],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /\[start-admin\].*\btauri:dev\b/s);
  assert.doesNotMatch(result.stdout, /\s--dir\s/);
});

test('start-admin forwards an explicit dev port override to the underlying Vite command', () => {
  const result = spawnSync(
    process.execPath,
    ['scripts/dev/start-admin.mjs', '--port', '16173', '--dry-run'],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /\[start-admin\].*\bdev\b.*--port 16173\b.*--strictPort\b/s);
});
