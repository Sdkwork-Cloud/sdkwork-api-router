#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SOURCE_FILE_PATTERN = /\.(?:[cm]?js|tsx?)$/u;
const SKIPPED_SOURCE_DIR_NAMES = new Set([
  'dist',
  'node_modules',
  'target',
]);
const STORAGE_ACCESS_PATTERNS = Object.freeze([
  /\bwindow\.(?:localStorage|sessionStorage)\b/u,
  /\bglobalThis\.(?:localStorage|sessionStorage)\b/u,
  /\b(?:window|globalThis)\[\s*storageName\s*\]/u,
  /\b(?:window|globalThis)\[\s*['"](?:localStorage|sessionStorage)['"]\s*\]/u,
  /\b(?:localStorage|sessionStorage)\.(?:getItem|setItem|removeItem)\b/u,
]);

export const BROWSER_STORAGE_GOVERNANCE_SPECS = Object.freeze([
  {
    scopeId: 'admin',
    sourceRoots: [
      path.join('apps', 'sdkwork-router-admin', 'packages'),
    ],
    approvedStoragePaths: [
      path.join(
        'apps',
        'sdkwork-router-admin',
        'packages',
        'sdkwork-router-admin-admin-api',
        'src',
        'sessionStore.ts',
      ),
      path.join(
        'apps',
        'sdkwork-router-admin',
        'packages',
        'sdkwork-router-admin-apirouter',
        'src',
        'services',
        'gatewayWorkspaceStore.ts',
      ),
      path.join(
        'apps',
        'sdkwork-router-admin',
        'packages',
        'sdkwork-router-admin-apirouter',
        'src',
        'services',
        'sensitiveSessionStore.ts',
      ),
      path.join(
        'apps',
        'sdkwork-router-admin',
        'packages',
        'sdkwork-router-admin-core',
        'src',
        'localePreferenceStore.ts',
      ),
    ],
  },
  {
    scopeId: 'portal',
    sourceRoots: [
      path.join('apps', 'sdkwork-router-portal', 'packages'),
    ],
    approvedStoragePaths: [
      path.join(
        'apps',
        'sdkwork-router-portal',
        'packages',
        'sdkwork-router-portal-api-keys',
        'src',
        'services',
        'plaintextRevealSessionStore.ts',
      ),
      path.join(
        'apps',
        'sdkwork-router-portal',
        'packages',
        'sdkwork-router-portal-commons',
        'src',
        'localePreferenceStore.ts',
      ),
      path.join(
        'apps',
        'sdkwork-router-portal',
        'packages',
        'sdkwork-router-portal-user',
        'src',
        'services',
        'preferenceSessionStore.ts',
      ),
    ],
  },
]);

function uniqueSorted(values = []) {
  return [...new Set(values)].sort();
}

function relativeUnixPath(workspaceRoot, absolutePath) {
  return path.relative(workspaceRoot, absolutePath).replaceAll('\\', '/');
}

function walkSourceFiles(rootDir, output = []) {
  for (const entry of readdirSync(rootDir, { withFileTypes: true })) {
    const fullPath = path.join(rootDir, entry.name);
    if (entry.isDirectory()) {
      if (SKIPPED_SOURCE_DIR_NAMES.has(entry.name)) {
        continue;
      }

      walkSourceFiles(fullPath, output);
      continue;
    }

    if (SOURCE_FILE_PATTERN.test(entry.name)) {
      output.push(fullPath);
    }
  }

  return output;
}

function sourceFilePaths(workspaceRoot, sourceRoots = []) {
  return sourceRoots.flatMap((sourceRoot) =>
    walkSourceFiles(path.join(workspaceRoot, sourceRoot)),
  );
}

function hasStorageAccess(sourceText = '') {
  return STORAGE_ACCESS_PATTERNS.some((pattern) => pattern.test(sourceText));
}

function buildFailureMessage(results = []) {
  const lines = [
    'Browser storage governance audit failed.',
    'Direct browser localStorage/sessionStorage access is only allowed inside approved governed store modules.',
    'All business entrypoints and feature modules must delegate browser persistence to those governed stores.',
  ];

  for (const result of results) {
    lines.push(`- ${result.scopeId}`);

    if (result.missingApprovedStoragePaths.length > 0) {
      lines.push(
        `  missing approved paths: ${result.missingApprovedStoragePaths.join(', ')}`,
      );
    }

    if (result.staleApprovedStoragePaths.length > 0) {
      lines.push(
        `  stale approved paths without storage access: ${result.staleApprovedStoragePaths.join(', ')}`,
      );
    }

    if (result.unapprovedStorageAccessPaths.length > 0) {
      lines.push(
        `  unapproved storage access: ${result.unapprovedStorageAccessPaths.join(', ')}`,
      );
    }
  }

  return lines.join('\n');
}

export function scanBrowserStorageGovernance({
  workspaceRoot = path.resolve(__dirname, '..'),
  scopeSpec,
} = {}) {
  if (!scopeSpec) {
    throw new Error('scopeSpec is required');
  }

  const approvedStoragePaths = uniqueSorted(
    (scopeSpec.approvedStoragePaths ?? []).map((relativePath) =>
      relativePath.replaceAll('\\', '/'),
    ),
  );
  const missingApprovedStoragePaths = [];
  const staleApprovedStoragePaths = [];

  for (const approvedPath of approvedStoragePaths) {
    const absolutePath = path.join(workspaceRoot, approvedPath);

    if (!existsSync(absolutePath)) {
      missingApprovedStoragePaths.push(approvedPath);
      continue;
    }

    if (!hasStorageAccess(readFileSync(absolutePath, 'utf8'))) {
      staleApprovedStoragePaths.push(approvedPath);
    }
  }

  const unapprovedStorageAccessPaths = [];

  for (const sourceFilePath of sourceFilePaths(workspaceRoot, scopeSpec.sourceRoots ?? [])) {
    const relativePath = relativeUnixPath(workspaceRoot, sourceFilePath);
    if (approvedStoragePaths.includes(relativePath)) {
      continue;
    }

    const sourceText = readFileSync(sourceFilePath, 'utf8');
    if (hasStorageAccess(sourceText)) {
      unapprovedStorageAccessPaths.push(relativePath);
    }
  }

  return {
    scopeId: scopeSpec.scopeId,
    approvedStoragePaths,
    missingApprovedStoragePaths: uniqueSorted(missingApprovedStoragePaths),
    staleApprovedStoragePaths: uniqueSorted(staleApprovedStoragePaths),
    unapprovedStorageAccessPaths: uniqueSorted(unapprovedStorageAccessPaths),
  };
}

export function runBrowserStorageGovernanceCheck({
  workspaceRoot = path.resolve(__dirname, '..'),
  scopeSpecs = BROWSER_STORAGE_GOVERNANCE_SPECS,
} = {}) {
  const scopes = scopeSpecs.map((scopeSpec) =>
    scanBrowserStorageGovernance({
      workspaceRoot,
      scopeSpec,
    }),
  );
  const failingScopes = scopes.filter(
    (scope) =>
      scope.missingApprovedStoragePaths.length > 0
      || scope.staleApprovedStoragePaths.length > 0
      || scope.unapprovedStorageAccessPaths.length > 0,
  );

  if (failingScopes.length > 0) {
    throw new Error(buildFailureMessage(scopes));
  }

  return {
    ok: true,
    scopeCount: scopes.length,
    scopes,
  };
}

async function main() {
  const result = runBrowserStorageGovernanceCheck();
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
