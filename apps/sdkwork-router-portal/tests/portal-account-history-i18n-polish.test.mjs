import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('account history productizes provider and channel detail labels instead of exposing raw separators', () => {
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(accountPage, /function resolveHistoryProviderLabel\(/);
  assert.match(accountPage, /function resolveHistoryChannelLabel\(/);
  assert.match(accountPage, /t\('Provider'\)/);
  assert.match(accountPage, /t\('Channel'\)/);
  assert.match(accountPage, /join\(' \/ '\)/);
  assert.doesNotMatch(accountPage, /join\(' 路 '\)/);
  assert.doesNotMatch(accountPage, /\[row\.provider,\s*row\.channel_id,\s*row\.project_id\]/);

  assert.match(commons, /'OpenAI'/);
  assert.match(commons, /'Anthropic'/);
});
