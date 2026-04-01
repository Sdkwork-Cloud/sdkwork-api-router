import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import {
  pnpmExecutable,
  pnpmSpawnOptions,
} from '../pnpm-launch-lib.mjs';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

test('pnpmExecutable selects the platform-specific launcher', () => {
  assert.equal(pnpmExecutable('win32'), 'pnpm.cmd');
  assert.equal(pnpmExecutable('linux'), 'pnpm');
  assert.equal(pnpmExecutable('darwin'), 'pnpm');
});

test('pnpmSpawnOptions enables Windows shell execution without opening another window', () => {
  const env = { PATH: 'C:/pnpm' };
  const options = pnpmSpawnOptions({
    platform: 'win32',
    env,
    cwd: 'D:/workspace/sdkwork-api-router',
  });

  assert.deepEqual(options, {
    cwd: 'D:/workspace/sdkwork-api-router',
    env: {
      ...env,
      NODE_OPTIONS: options.env.NODE_OPTIONS,
    },
    shell: true,
    stdio: 'inherit',
    windowsHide: true,
  });
  assert.match(options.env.NODE_OPTIONS, /vite-windows-realpath-preload\.mjs/);
  assert.match(options.env.NODE_OPTIONS, /--import=/);
});

test('pnpmSpawnOptions keeps non-Windows launches direct and foreground-safe', () => {
  const env = { PATH: '/usr/bin' };
  const options = pnpmSpawnOptions({
    platform: 'linux',
    env,
  });

  assert.deepEqual(options, {
    env,
    shell: false,
    stdio: 'inherit',
    windowsHide: false,
  });
});

test('pnpmSpawnOptions preserves existing NODE_OPTIONS while adding the Vite preload once on Windows', () => {
  const options = pnpmSpawnOptions({
    platform: 'win32',
    env: {
      NODE_OPTIONS: '--max-old-space-size=4096',
      PATH: 'C:/pnpm',
    },
  });

  assert.match(options.env.NODE_OPTIONS, /--max-old-space-size=4096/);
  assert.match(options.env.NODE_OPTIONS, /vite-windows-realpath-preload\.mjs/);
  assert.equal(
    options.env.NODE_OPTIONS.match(/vite-windows-realpath-preload\.mjs/g)?.length,
    1,
  );
});

test('dev launchers use the shared pnpm helper for Windows-safe process spawning', () => {
  const scriptPaths = [
    path.join(repoRoot, 'scripts', 'dev', 'start-admin.mjs'),
    path.join(repoRoot, 'scripts', 'dev', 'start-portal.mjs'),
    path.join(repoRoot, 'scripts', 'dev', 'start-console.mjs'),
    path.join(repoRoot, 'scripts', 'dev', 'start-web.mjs'),
  ];

  for (const scriptPath of scriptPaths) {
    const script = readFileSync(scriptPath, 'utf8');
    assert.match(script, /pnpm-launch-lib\.mjs/);
    assert.match(script, /pnpmSpawnOptions/);
  }
});
