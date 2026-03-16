import test from 'node:test';
import assert from 'node:assert/strict';

import {
  parseWebArgs,
  publicEntryUrls,
  webAccessLines,
} from '../web-launch-lib.mjs';

test('parseWebArgs keeps public Pingora bind by default', () => {
  assert.deepEqual(parseWebArgs([]), {
    bind: '0.0.0.0:3001',
    dryRun: false,
    help: false,
    install: false,
    preview: false,
    tauri: false,
  });
});

test('parseWebArgs accepts bind override and flags', () => {
  const settings = parseWebArgs([
    '--bind',
    '0.0.0.0:3901',
    '--install',
    '--preview',
    '--tauri',
    '--dry-run',
  ]);

  assert.equal(settings.bind, '0.0.0.0:3901');
  assert.equal(settings.install, true);
  assert.equal(settings.preview, true);
  assert.equal(settings.tauri, true);
  assert.equal(settings.dryRun, true);
});

test('publicEntryUrls exposes localhost when Pingora binds all interfaces', () => {
  const urls = publicEntryUrls('0.0.0.0:3901');

  assert.ok(urls.includes('http://127.0.0.1:3901'));
});

test('webAccessLines include admin and portal entrypoints', () => {
  const lines = webAccessLines('0.0.0.0:3901').join('\n');

  assert.match(lines, /SDKWORK_WEB_BIND=0\.0\.0\.0:3901/);
  assert.match(lines, /\/admin\//);
  assert.match(lines, /\/portal\//);
});
