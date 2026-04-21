#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { createStrictKeyedCatalog } from './strict-contract-catalog.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const GOVERNED_PRODUCT_SOURCE_FILE_PATTERN =
  /\.(?:[cm]?[jt]s|[jt]sx|json|ya?ml|sh|ps1|rs|toml)$/u;
const GOVERNED_PRODUCT_SOURCE_ROOT_SPECS = Object.freeze([
  {
    id: 'scripts',
    relativePath: 'scripts',
  },
  {
    id: 'github-workflows',
    relativePath: '.github/workflows',
  },
  {
    id: 'bin',
    relativePath: 'bin',
  },
  {
    id: 'router-admin',
    relativePath: 'apps/sdkwork-router-admin',
  },
  {
    id: 'router-portal',
    relativePath: 'apps/sdkwork-router-portal',
  },
  {
    id: 'router-product-service',
    relativePath: 'services/router-product-service',
  },
]);
const EXCLUDED_GOVERNED_SUBTREE_PREFIXES = Object.freeze([
  'scripts/release/',
]);

const governedProductSourceRootCatalog = createStrictKeyedCatalog({
  entries: GOVERNED_PRODUCT_SOURCE_ROOT_SPECS,
  getKey: ({ id }) => id,
  duplicateKeyMessagePrefix: 'duplicate governed product source root',
  missingKeyMessagePrefix: 'missing governed product source root',
});

function uniqueSorted(values = []) {
  return [...new Set(values)].sort();
}

function normalizeRelativePath(relativePath = '') {
  return String(relativePath)
    .trim()
    .replaceAll('\\', '/')
    .replace(/^\.\//u, '');
}

function isWithinGovernedRoot(relativePath, rootPaths = []) {
  return rootPaths.some((rootPath) =>
    relativePath === rootPath || relativePath.startsWith(`${rootPath}/`));
}

function isExcludedGovernedSubtree(relativePath) {
  return EXCLUDED_GOVERNED_SUBTREE_PREFIXES.some((prefix) =>
    relativePath.startsWith(prefix));
}

function isGovernedProductSourceFile(relativePath) {
  return GOVERNED_PRODUCT_SOURCE_FILE_PATTERN.test(path.posix.basename(relativePath));
}

function createGitFailureMessage(result) {
  const fragments = [];
  if (result?.error) {
    fragments.push(`error=${result.error.message}`);
  }
  if (String(result?.stderr ?? '').trim()) {
    fragments.push(`stderr=${String(result.stderr).trim()}`);
  }

  return `failed to list untracked governed product sources via git ls-files${fragments.length > 0 ? ` (${fragments.join('; ')})` : ''}`;
}

function buildFailureMessage(untrackedSources = []) {
  return [
    'Product source tracking audit failed.',
    'Governed product source files under scripts/, .github/workflows/, bin/, apps/sdkwork-router-admin/, apps/sdkwork-router-portal/, and services/router-product-service/ must be tracked in git so clean checkouts reproduce product verification behavior.',
    'This governed product surface includes JavaScript/TypeScript, JSON/YAML, shell/PowerShell, and Rust/TOML source files.',
    'The scripts/release/ subtree is governed by release-specific tracking and is excluded from this product-surface audit.',
    '',
    'Untracked governed product sources:',
    ...untrackedSources.map((entry) => `- ${entry}`),
  ].join('\n');
}

export function listGovernedProductSourceRootSpecs() {
  return governedProductSourceRootCatalog.list();
}

export function findGovernedProductSourceRootSpec(rootId) {
  return governedProductSourceRootCatalog.find(rootId);
}

export function listGovernedProductSourceRootSpecsByIds(rootIds = []) {
  return governedProductSourceRootCatalog.listByKeys(rootIds);
}

export function listUntrackedGovernedProductSources({
  repoRoot = path.resolve(__dirname, '..'),
  rootSpecs = listGovernedProductSourceRootSpecs(),
  spawnSyncImpl = spawnSync,
  platform = process.platform,
} = {}) {
  const governedRootPaths = rootSpecs.map(({ relativePath }) =>
    normalizeRelativePath(relativePath));
  const gitResult = spawnSyncImpl(
    platform === 'win32' ? 'git.exe' : 'git',
    ['ls-files', '--others', '--exclude-standard', ...governedRootPaths],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    },
  );

  if (gitResult?.error || gitResult?.status !== 0) {
    throw new Error(createGitFailureMessage(gitResult));
  }

  return uniqueSorted(
    String(gitResult.stdout ?? '')
      .split(/\r?\n/u)
      .map(normalizeRelativePath)
      .filter(Boolean)
      .filter((relativePath) => isWithinGovernedRoot(relativePath, governedRootPaths))
      .filter((relativePath) => !isExcludedGovernedSubtree(relativePath))
      .filter((relativePath) => isGovernedProductSourceFile(relativePath)),
  );
}

export function runProductSourceTrackingAudit({
  repoRoot = path.resolve(__dirname, '..'),
  rootSpecs = listGovernedProductSourceRootSpecs(),
  spawnSyncImpl = spawnSync,
  platform = process.platform,
} = {}) {
  const untrackedSources = listUntrackedGovernedProductSources({
    repoRoot,
    rootSpecs,
    spawnSyncImpl,
    platform,
  });

  if (untrackedSources.length > 0) {
    throw new Error(buildFailureMessage(untrackedSources));
  }

  return {
    ok: true,
    governedRootSpecs: rootSpecs,
    untrackedSources,
  };
}

async function main() {
  const result = runProductSourceTrackingAudit();
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
