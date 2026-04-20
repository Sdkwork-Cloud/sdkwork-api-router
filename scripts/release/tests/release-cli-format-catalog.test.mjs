import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-cli-format-catalog.mjs'),
    ).href,
  );
}

test('release CLI format catalog exposes strict format lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listReleaseCliFormats, 'function');
  assert.equal(typeof module.findReleaseCliFormat, 'function');
  assert.equal(typeof module.listReleaseCliFormatsByIds, 'function');
  assert.equal(typeof module.assertSupportedReleaseCliFormat, 'function');

  assert.deepEqual(
    module.listReleaseCliFormats(),
    ['text', 'json'],
  );
  assert.equal(
    module.findReleaseCliFormat('json'),
    'json',
  );
  assert.deepEqual(
    module.listReleaseCliFormatsByIds([
      'text',
      'json',
    ]),
    ['text', 'json'],
  );
  assert.equal(
    module.assertSupportedReleaseCliFormat('text'),
    'text',
  );

  assert.throws(
    () => module.findReleaseCliFormat('yaml'),
    /missing release cli format.*yaml/i,
  );
  assert.throws(
    () => module.assertSupportedReleaseCliFormat('yaml'),
    /unsupported format: yaml/i,
  );
});
