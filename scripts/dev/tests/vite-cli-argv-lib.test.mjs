import assert from 'node:assert/strict';
import test from 'node:test';

import { normalizeForwardedCliArgs } from '../vite-cli-argv-lib.mjs';

test('normalizeForwardedCliArgs preserves plain Vite arguments', () => {
  assert.deepEqual(
    normalizeForwardedCliArgs(['--host', '0.0.0.0']),
    ['--host', '0.0.0.0'],
  );
});

test('normalizeForwardedCliArgs strips the pnpm forwarded "--" sentinel before Vite sees the args', () => {
  assert.deepEqual(
    normalizeForwardedCliArgs(['--host', '0.0.0.0', '--', '--port', '52762', '--strictPort']),
    ['--host', '0.0.0.0', '--port', '52762', '--strictPort'],
  );
});
