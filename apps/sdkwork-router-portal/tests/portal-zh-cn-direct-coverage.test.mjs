import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const dictionaryPath = path.join(
  appRoot,
  'packages',
  'sdkwork-router-portal-commons',
  'src',
  'portalMessages.zh-CN.ts',
);

const OWNER_PATHS = [
  'packages/sdkwork-router-portal-core/src/components/PortalSettingsCenter.tsx',
  'packages/sdkwork-router-portal-core/src/components/PortalNavigationRail.tsx',
  'packages/sdkwork-router-portal-core/src/routes.ts',
  'packages/sdkwork-router-portal-core/src/lib/portalPreferences.ts',
  'packages/sdkwork-router-portal-usage/src/pages/index.tsx',
  'packages/sdkwork-router-portal-api-keys/src/pages/index.tsx',
  'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyTable.tsx',
  'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyDrawers.tsx',
  'packages/sdkwork-router-portal-api-keys/src/components/PortalApiKeyCreateForm.tsx',
  'packages/sdkwork-router-portal-credits/src/pages/index.tsx',
  'packages/sdkwork-router-portal-account/src/pages/index.tsx',
  'packages/sdkwork-router-portal-user/src/pages/index.tsx',
  'packages/sdkwork-router-portal-dashboard/src/pages/index.tsx',
  'packages/sdkwork-router-portal-gateway/src/pages/index.tsx',
];

const KEY_PATTERNS = [
  /\bt\(\s*'((?:\\'|[^'])+)'/g,
  /\btranslatePortalText\(\s*'((?:\\'|[^'])+)'/g,
  /\b(?:labelKey|eyebrowKey|detailKey)\s*:\s*'((?:\\'|[^'])+)'/g,
];

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function readDirectDictionaryKeys() {
  const dictionarySource = readFileSync(dictionaryPath, 'utf8');
  return new Set(
    [
      ...dictionarySource.matchAll(/^\s*(?:'((?:\\'|[^'])+)'|([A-Za-z_$][A-Za-z0-9_$]*)):/gm),
    ].map((match) => (match[1] ?? match[2]).replace(/\\'/g, "'")),
  );
}

function collectKeys(source) {
  const keys = new Set();

  for (const pattern of KEY_PATTERNS) {
    for (const match of source.matchAll(pattern)) {
      keys.add(match[1].replace(/\\'/g, "'"));
    }
  }

  return [...keys].sort((left, right) => left.localeCompare(right));
}

test('portal shell and core workspaces provide direct zh-CN mappings for user-facing copy', () => {
  const directKeys = readDirectDictionaryKeys();
  const missingByOwner = [];

  for (const ownerPath of OWNER_PATHS) {
    const ownerSource = read(ownerPath);
    const missingKeys = collectKeys(ownerSource).filter((key) => !directKeys.has(key));

    if (missingKeys.length > 0) {
      missingByOwner.push({ ownerPath, missingKeys });
    }
  }

  assert.deepEqual(
    missingByOwner,
    [],
    missingByOwner
      .map(({ ownerPath, missingKeys }) => `${ownerPath}\n${missingKeys.map((key) => `  - ${key}`).join('\n')}`)
      .join('\n\n'),
  );
});
