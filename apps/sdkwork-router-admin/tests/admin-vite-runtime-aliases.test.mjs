import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin vite config pins zustand imports to ESM browser aliases', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /const zustandPackageRoot =/);
  assert.match(viteConfig, /const zustandEsmEntry = .*esm.*index\.mjs/);
  assert.match(viteConfig, /const zustandEsmSubpathRoot =/);
  assert.match(viteConfig, /dedupe:\s*\[[\s\S]*'zustand'/);
  assert.match(viteConfig, /find:\s*\/\^zustand\$\//);
  assert.match(viteConfig, /replacement:\s*zustandEsmEntry/);
  assert.match(viteConfig, /find:\s*\/\^zustand\\\/\//);
  assert.match(viteConfig, /replacement:\s*zustandEsmSubpathRoot/);
});

test('admin vite config remaps use-sync-external-store shim to a React-native browser shim', () => {
  const viteConfig = read('vite.config.ts');
  const browserShim = read('src/vendor/use-sync-external-store-shim.ts');

  assert.match(viteConfig, /find:\s*\/\^use-sync-external-store\\\/shim\$\//);
  assert.match(viteConfig, /replacement:\s*path\.join\(configDir,\s*'src',\s*'vendor',\s*'use-sync-external-store-shim\.ts'\)/);
  assert.match(browserShim, /export\s*\{\s*useSyncExternalStore\s*\}\s*from\s*'react';/);
});

test('admin vite config resolves sdkwork ui entrypoints from workspace source instead of dist bundles', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /const sdkworkUiSourceRoot = path\.resolve\(/);
  assert.match(viteConfig, /\.\.\/\.\.\/\.\.\/sdkwork-ui\/sdkwork-ui-pc-react\/src/);
  assert.match(viteConfig, /function resolveSdkworkUiSourcePath\(relativePath: string\)/);
  assert.match(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\$\//);
  assert.match(viteConfig, /replacement:\s*resolveSdkworkUiSourcePath\('index\.ts'\)/);
  assert.match(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\\\/theme\$\//);
  assert.match(viteConfig, /replacement:\s*resolveSdkworkUiSourcePath\('theme\/index\.ts'\)/);
  assert.match(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\\\/components\\\/ui\$\//);
  assert.match(viteConfig, /replacement:\s*resolveSdkworkUiSourcePath\('components\/ui\/index\.ts'\)/);
  assert.match(viteConfig, /find:\s*\/\^@sdkwork\\\/ui-pc-react\\\/components\\\/ui\\\/feedback\$\//);
  assert.match(viteConfig, /replacement:\s*resolveSdkworkUiSourcePath\('components\/ui\/feedback\/index\.ts'\)/);
});

test('admin vite config pins source-linked style and icon runtime packages to governed readable roots', () => {
  const viteConfig = read('vite.config.ts');

  assert.match(viteConfig, /const clsxPackageRoot = normalizeAliasPath\(resolveReadablePackageRoot\(/);
  assert.match(viteConfig, /const tailwindMergePackageRoot = normalizeAliasPath\(resolveReadablePackageRoot\(/);
  assert.match(viteConfig, /const lucideReactPackageRoot = normalizeAliasPath\(resolveReadablePackageRoot\(/);
  assert.match(viteConfig, /const lucideReactIconsRoot = `\$\{normalizeAliasPath\(path\.join\(lucideReactPackageRoot, 'dist', 'esm', 'icons'\)\)\}\/`;/);
  assert.match(viteConfig, /dedupe:\s*\[[\s\S]*'lucide-react'[\s\S]*'zustand'[\s\S]*\]/);
  assert.match(viteConfig, /find:\s*\/\^clsx\$\//);
  assert.match(viteConfig, /replacement:\s*clsxPackageRoot/);
  assert.match(viteConfig, /find:\s*\/\^tailwind-merge\$\//);
  assert.match(viteConfig, /replacement:\s*tailwindMergePackageRoot/);
  assert.match(viteConfig, /find:\s*\/\^lucide-react\$\//);
  assert.match(viteConfig, /replacement:\s*lucideReactPackageRoot/);
  assert.match(viteConfig, /find:\s*\/\^lucide-react\\\/dist\\\/esm\\\/icons\\\/\//);
  assert.match(viteConfig, /replacement:\s*lucideReactIconsRoot/);
});

test('admin typecheck uses the real lucide-react package typings instead of a local shim', () => {
  const packageJson = JSON.parse(read('package.json'));

  assert.equal(
    packageJson.dependencies?.['lucide-react'],
    '0.554.0',
    'admin app must declare the governed lucide-react runtime dependency directly',
  );
  assert.equal(
    existsSync(path.join(appRoot, 'src', 'types', 'lucide-react-shim.d.ts')),
    false,
    'admin app must not shadow lucide-react with a hand-written ambient shim',
  );
});
