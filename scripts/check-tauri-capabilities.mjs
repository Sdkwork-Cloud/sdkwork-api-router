#!/usr/bin/env node

import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const WINDOW_METHOD_PERMISSION_MAP = Object.freeze({
  close: 'core:window:allow-close',
  isMaximized: 'core:window:allow-is-maximized',
  maximize: 'core:window:allow-maximize',
  minimize: 'core:window:allow-minimize',
  toggleMaximize: 'core:window:allow-toggle-maximize',
  unmaximize: 'core:window:allow-unmaximize',
});

const SOURCE_FILE_PATTERN = /\.(?:[cm]?js|tsx?)$/u;
const SKIPPED_SOURCE_DIR_NAMES = new Set([
  'dist',
  'node_modules',
  'target',
]);
const TAURI_GLOBAL_PATTERNS = Object.freeze([
  /\b__TAURI_INTERNALS__\b/u,
  /\b__TAURI__\b/u,
  /\bisTauri\b/u,
]);
const WINDOW_API_IMPORT_PATTERN = /@tauri-apps\/api\/window/u;

const DESKTOP_APP_SPECS = Object.freeze([
  {
    appId: 'portal',
    buildRsPath: path.join(
      'apps',
      'sdkwork-router-portal',
      'src-tauri',
      'build.rs',
    ),
    capabilityPath: path.join(
      'apps',
      'sdkwork-router-portal',
      'src-tauri',
      'capabilities',
      'main.json',
    ),
    sourceRoots: [
      path.join(
        'apps',
        'sdkwork-router-portal',
        'packages',
      ),
    ],
    approvedTauriGlobalPaths: [
      path.join(
        'apps',
        'sdkwork-router-portal',
        'packages',
        'sdkwork-router-portal-portal-api',
        'src',
        'desktopBridge.ts',
      ),
    ],
    approvedWindowApiImportPaths: [
      path.join(
        'apps',
        'sdkwork-router-portal',
        'packages',
        'sdkwork-router-portal-portal-api',
        'src',
        'desktopBridge.ts',
      ),
    ],
    windowControllerPath: path.join(
      'apps',
      'sdkwork-router-portal',
      'packages',
      'sdkwork-router-portal-portal-api',
      'src',
      'desktopBridge.ts',
    ),
  },
  {
    appId: 'admin',
    buildRsPath: path.join(
      'apps',
      'sdkwork-router-admin',
      'src-tauri',
      'build.rs',
    ),
    capabilityPath: path.join(
      'apps',
      'sdkwork-router-admin',
      'src-tauri',
      'capabilities',
      'main.json',
    ),
    sourceRoots: [
      path.join(
        'apps',
        'sdkwork-router-admin',
        'packages',
      ),
    ],
    approvedTauriGlobalPaths: [
      path.join(
        'apps',
        'sdkwork-router-admin',
        'packages',
        'sdkwork-router-admin-admin-api',
        'src',
        'desktopBridge.ts',
      ),
    ],
    approvedWindowApiImportPaths: [
      path.join(
        'apps',
        'sdkwork-router-admin',
        'packages',
        'sdkwork-router-admin-admin-api',
        'src',
        'desktopBridge.ts',
      ),
    ],
    windowControllerPath: path.join(
      'apps',
      'sdkwork-router-admin',
      'packages',
      'sdkwork-router-admin-admin-api',
      'src',
      'desktopBridge.ts',
    ),
  },
]);

function readText(workspaceRoot, relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8');
}

function readJson(workspaceRoot, relativePath) {
  return JSON.parse(readText(workspaceRoot, relativePath));
}

export function permissionIdentifierForCommand(commandName) {
  return `allow-${String(commandName ?? '').trim().replaceAll('_', '-')}`;
}

export function permissionIdentifierForWindowMethod(methodName) {
  const permission = WINDOW_METHOD_PERMISSION_MAP[methodName];
  if (!permission) {
    throw new Error(`Unsupported Tauri window method: ${methodName}`);
  }

  return permission;
}

export function parseBuildCommandNames(buildRsSource = '') {
  const commandsBlockMatch = String(buildRsSource).match(
    /commands\(&\[(?<body>[\s\S]*?)\]\)/u,
  );
  if (!commandsBlockMatch?.groups?.body) {
    return [];
  }

  return [...commandsBlockMatch.groups.body.matchAll(/"([^"]+)"/g)]
    .map((match) => match[1])
    .filter(Boolean);
}

export function detectWindowControllerMethods(sourceText = '') {
  const matches = new Set();

  for (const methodName of Object.keys(WINDOW_METHOD_PERMISSION_MAP)) {
    if (new RegExp(`\\.${methodName}\\b`, 'u').test(sourceText)) {
      matches.add(methodName);
    }
  }

  return [...matches].sort();
}

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

function containsAnyPattern(sourceText, patterns = []) {
  return patterns.some((pattern) => pattern.test(sourceText));
}

export function auditDesktopBridgeOwnership({
  workspaceRoot = path.resolve(__dirname, '..'),
  appSpec,
} = {}) {
  if (!appSpec) {
    throw new Error('appSpec is required');
  }

  const approvedTauriGlobalPaths = uniqueSorted(
    (appSpec.approvedTauriGlobalPaths ?? []).map((relativePath) =>
      relativePath.replaceAll('\\', '/'),
    ),
  );
  const approvedWindowApiImportPaths = uniqueSorted(
    (appSpec.approvedWindowApiImportPaths ?? []).map((relativePath) =>
      relativePath.replaceAll('\\', '/'),
    ),
  );
  const unapprovedTauriGlobalAccessPaths = [];
  const unapprovedWindowApiImportPaths = [];

  for (const sourceFilePath of sourceFilePaths(workspaceRoot, appSpec.sourceRoots ?? [])) {
    const relativePath = relativeUnixPath(workspaceRoot, sourceFilePath);
    const sourceText = readFileSync(sourceFilePath, 'utf8');

    if (
      containsAnyPattern(sourceText, TAURI_GLOBAL_PATTERNS)
      && !approvedTauriGlobalPaths.includes(relativePath)
    ) {
      unapprovedTauriGlobalAccessPaths.push(relativePath);
    }

    if (
      WINDOW_API_IMPORT_PATTERN.test(sourceText)
      && !approvedWindowApiImportPaths.includes(relativePath)
    ) {
      unapprovedWindowApiImportPaths.push(relativePath);
    }
  }

  return {
    approvedTauriGlobalPaths,
    approvedWindowApiImportPaths,
    unapprovedTauriGlobalAccessPaths: uniqueSorted(unapprovedTauriGlobalAccessPaths),
    unapprovedWindowApiImportPaths: uniqueSorted(unapprovedWindowApiImportPaths),
  };
}

export function auditDesktopAppCapabilities({
  workspaceRoot = path.resolve(__dirname, '..'),
  appSpec,
} = {}) {
  if (!appSpec) {
    throw new Error('appSpec is required');
  }

  const capability = readJson(workspaceRoot, appSpec.capabilityPath);
  const capabilityPermissions = uniqueSorted(capability.permissions ?? []);
  const buildCommands = parseBuildCommandNames(
    readText(workspaceRoot, appSpec.buildRsPath),
  );
  const requiredCommandPermissions = uniqueSorted(
    buildCommands.map(permissionIdentifierForCommand),
  );
  const requiredWindowPermissions = uniqueSorted(
    detectWindowControllerMethods(
      readText(workspaceRoot, appSpec.windowControllerPath),
    ).map(permissionIdentifierForWindowMethod),
  );
  const bridgeOwnership = auditDesktopBridgeOwnership({
    workspaceRoot,
    appSpec,
  });

  return {
    appId: appSpec.appId,
    capabilityPath: appSpec.capabilityPath.replaceAll('\\', '/'),
    buildRsPath: appSpec.buildRsPath.replaceAll('\\', '/'),
    windowControllerPath: appSpec.windowControllerPath.replaceAll('\\', '/'),
    capabilityPermissions,
    requiredCommandPermissions,
    requiredWindowPermissions,
    missingCommandPermissions: requiredCommandPermissions.filter(
      (permission) => !capabilityPermissions.includes(permission),
    ),
    missingWindowPermissions: requiredWindowPermissions.filter(
      (permission) => !capabilityPermissions.includes(permission),
    ),
    ...bridgeOwnership,
  };
}

function buildFailureMessage(results = []) {
  const failingApps = results.filter(
    (result) =>
      result.missingCommandPermissions.length > 0
      || result.missingWindowPermissions.length > 0
      || result.unapprovedTauriGlobalAccessPaths.length > 0
      || result.unapprovedWindowApiImportPaths.length > 0,
  );

  const lines = [
    'Tauri capability audit failed.',
    'Each desktop app capability must allow every Tauri command declared in build.rs and every window API used by the approved desktop bridge.',
    'Tauri runtime globals and @tauri-apps/api/window imports must stay centralized in the approved bridge files.',
  ];

  for (const result of failingApps) {
    lines.push(
      `- ${result.appId}: capability=${result.capabilityPath}; buildRs=${result.buildRsPath}; windowController=${result.windowControllerPath}`,
    );

    if (result.missingCommandPermissions.length > 0) {
      lines.push(
        `  missing command permissions: ${result.missingCommandPermissions.join(', ')}`,
      );
    }

    if (result.missingWindowPermissions.length > 0) {
      lines.push(
        `  missing window permissions: ${result.missingWindowPermissions.join(', ')}`,
      );
    }

    if (result.unapprovedTauriGlobalAccessPaths.length > 0) {
      lines.push(
        `  unapproved tauri global access: ${result.unapprovedTauriGlobalAccessPaths.join(', ')}`,
      );
    }

    if (result.unapprovedWindowApiImportPaths.length > 0) {
      lines.push(
        `  unapproved window api imports: ${result.unapprovedWindowApiImportPaths.join(', ')}`,
      );
    }
  }

  return lines.join('\n');
}

export function runTauriCapabilityAudit({
  workspaceRoot = path.resolve(__dirname, '..'),
  appSpecs = DESKTOP_APP_SPECS,
} = {}) {
  const apps = appSpecs.map((appSpec) =>
    auditDesktopAppCapabilities({
      workspaceRoot,
      appSpec,
    }),
  );
  const failingApps = apps.filter(
    (result) =>
      result.missingCommandPermissions.length > 0
      || result.missingWindowPermissions.length > 0
      || result.unapprovedTauriGlobalAccessPaths.length > 0
      || result.unapprovedWindowApiImportPaths.length > 0,
  );

  if (failingApps.length > 0) {
    throw new Error(buildFailureMessage(apps));
  }

  return {
    ok: true,
    appCount: apps.length,
    apps,
  };
}

async function main() {
  const result = runTauriCapabilityAudit();
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
