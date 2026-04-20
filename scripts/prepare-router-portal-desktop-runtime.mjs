#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { createDesktopAssetBuildPlan } from './build-router-desktop-assets.mjs';
import { resolveDesktopReleaseTarget } from './release/desktop-targets.mjs';
import { findNativePortalDesktopEmbeddedRuntimeLayoutSpec } from './release/native-runtime-layout-catalog.mjs';
import {
  withManagedWorkspaceTargetDir,
  withManagedWorkspaceTempDir,
} from './workspace-target-dir.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function withExecutable(binaryName, platform = process.platform) {
  return platform === 'win32' || platform === 'windows'
    ? `${binaryName}.exe`
    : binaryName;
}

function resolveServiceReleaseRoot({
  workspaceRoot,
  targetTriple = '',
  env = process.env,
  platform = process.platform,
} = {}) {
  const targetRoot = withManagedWorkspaceTargetDir({
    workspaceRoot,
    env,
    platform,
  }).CARGO_TARGET_DIR ?? path.join(workspaceRoot, 'target');

  const targetSegments = String(targetTriple ?? '').trim()
    ? [String(targetTriple).trim()]
    : [];
  return path.join(targetRoot, ...targetSegments, 'release');
}

function writeJsonFile(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function splitRelativeRuntimePath(relativePath) {
  return String(relativePath ?? '')
    .split('/')
    .filter((segment) => segment.length > 0);
}

function resolveRelativeRuntimePath(rootPath, relativePath) {
  return path.join(rootPath, ...splitRelativeRuntimePath(relativePath));
}

function toPortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/');
}

function withTrailingSlash(value) {
  const normalized = toPortablePath(value).replace(/\/+$/u, '');
  return normalized.length > 0 ? `${normalized}/` : './';
}

function resolveRuntimeLayoutMappedPath(entryMap, entryId, entryKind) {
  const relativePath = entryMap[entryId];
  if (!relativePath) {
    throw new Error(`Missing runtime layout ${entryKind}: ${entryId}`);
  }

  return relativePath;
}

function resolvePortalDesktopRuntimeRootRelativePath(
  runtimeLayout = findNativePortalDesktopEmbeddedRuntimeLayoutSpec(),
) {
  return path.posix.dirname(runtimeLayout.readmeFile);
}

function relativizePortalDesktopRuntimePath(
  relativePath,
  runtimeLayout = findNativePortalDesktopEmbeddedRuntimeLayoutSpec(),
) {
  return path.posix.relative(
    resolvePortalDesktopRuntimeRootRelativePath(runtimeLayout),
    relativePath,
  );
}

function writePortalDesktopPayloadReadme({
  releasePayloadReadmePath,
  target,
  runtimeLayout = findNativePortalDesktopEmbeddedRuntimeLayoutSpec(),
} = {}) {
  const routerBinaryPath = path.posix.join(
    relativizePortalDesktopRuntimePath(runtimeLayout.serviceBinaryDir, runtimeLayout),
    withExecutable(runtimeLayout.serviceBinaryName, target.platform),
  );
  const adminSiteDir = relativizePortalDesktopRuntimePath(
    resolveRuntimeLayoutMappedPath(runtimeLayout.siteTargetDirs, 'admin', 'site target dir'),
    runtimeLayout,
  );
  const portalSiteDir = relativizePortalDesktopRuntimePath(
    resolveRuntimeLayoutMappedPath(runtimeLayout.siteTargetDirs, 'portal', 'site target dir'),
    runtimeLayout,
  );
  const bootstrapDataDir = relativizePortalDesktopRuntimePath(
    resolveRuntimeLayoutMappedPath(
      runtimeLayout.bootstrapDataRootDirs,
      'data',
      'bootstrap data root dir',
    ),
    runtimeLayout,
  );
  const releaseManifestFile = relativizePortalDesktopRuntimePath(
    runtimeLayout.releaseManifestFile,
    runtimeLayout,
  );
  const readmeFile = relativizePortalDesktopRuntimePath(
    runtimeLayout.readmeFile,
    runtimeLayout,
  );

  writeFileSync(
    releasePayloadReadmePath,
    [
      'SDKWork Router Portal Desktop Embedded Runtime Payload',
      '',
      `platform: ${target.platform}`,
      `arch: ${target.arch}`,
      `target: ${target.targetTriple}`,
      '',
      'Contents:',
      `- ${routerBinaryPath}: supervised router-product sidecar`,
      `- ${adminSiteDir}: bundled admin site assets`,
      `- ${portalSiteDir}: bundled portal site assets`,
      `- ${bootstrapDataDir}: bootstrap data packs for first-start initialization`,
      `- ${releaseManifestFile}: embedded runtime payload contract metadata`,
      `- ${readmeFile}: operator-facing payload notes`,
      '',
      'Desktop runtime contract:',
      '- fixed public port 3001',
      '- local-only access mode binds 127.0.0.1:3001',
      '- shared access mode binds 0.0.0.0:3001',
      '- mutable config, data, and logs live in OS-standard per-user directories',
      '',
    ].join('\n'),
    'utf8',
  );
}

export function resolvePortalDesktopRuntimeResourceRoot({
  workspaceRoot = path.resolve(__dirname, '..'),
} = {}) {
  return path.join(workspaceRoot, 'bin', 'portal-rt');
}

export function resolvePortalDesktopRuntimeTauriResourceMap({
  workspaceRoot = path.resolve(__dirname, '..'),
  appRoot = path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  runtimeLayout = findNativePortalDesktopEmbeddedRuntimeLayoutSpec(),
} = {}) {
  const resourceRoot = resolvePortalDesktopRuntimeResourceRoot({ workspaceRoot });
  const runtimeRootRelativePath = resolvePortalDesktopRuntimeRootRelativePath(runtimeLayout);
  const sourceAbsolutePath = resolveRelativeRuntimePath(resourceRoot, runtimeRootRelativePath);
  const tauriConfigRoot = path.join(appRoot, 'src-tauri');

  return {
    resourceRoot,
    tauriConfigRoot,
    sourceAbsolutePath,
    sourceRelativePath: withTrailingSlash(path.relative(tauriConfigRoot, sourceAbsolutePath)),
    targetRelativePath: withTrailingSlash(runtimeRootRelativePath),
  };
}

export function resolvePortalDesktopRuntimeResourceLayout({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  targetTriple = '',
  env = process.env,
} = {}) {
  const runtimeLayout = findNativePortalDesktopEmbeddedRuntimeLayoutSpec();
  const resourceMap = resolvePortalDesktopRuntimeTauriResourceMap({
    workspaceRoot,
    runtimeLayout,
  });
  const resourceRoot = resourceMap.resourceRoot;
  const routerProductRoot = resourceMap.sourceAbsolutePath;
  const serviceReleaseRoot = resolveServiceReleaseRoot({
    workspaceRoot,
    targetTriple,
    env,
    platform,
  });

  return {
    resourceRoot,
    routerProductRoot,
    routerBinaryPath: resolveRelativeRuntimePath(
      resourceRoot,
      path.posix.join(
        runtimeLayout.serviceBinaryDir,
        withExecutable(runtimeLayout.serviceBinaryName, platform),
      ),
    ),
    routerBinarySourcePath: path.join(
      serviceReleaseRoot,
      withExecutable(runtimeLayout.serviceBinaryName, platform),
    ),
    adminSiteDir: resolveRelativeRuntimePath(
      resourceRoot,
      resolveRuntimeLayoutMappedPath(runtimeLayout.siteTargetDirs, 'admin', 'site target dir'),
    ),
    adminSiteSourceDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-admin', 'dist'),
    portalSiteDir: resolveRelativeRuntimePath(
      resourceRoot,
      resolveRuntimeLayoutMappedPath(runtimeLayout.siteTargetDirs, 'portal', 'site target dir'),
    ),
    portalSiteSourceDir: path.join(workspaceRoot, 'apps', 'sdkwork-router-portal', 'dist'),
    bootstrapDataDir: resolveRelativeRuntimePath(
      resourceRoot,
      resolveRuntimeLayoutMappedPath(
        runtimeLayout.bootstrapDataRootDirs,
        'data',
        'bootstrap data root dir',
      ),
    ),
    bootstrapDataSourceDir: path.join(workspaceRoot, 'data'),
    releasePayloadManifestPath: resolveRelativeRuntimePath(
      resourceRoot,
      runtimeLayout.releaseManifestFile,
    ),
    releasePayloadReadmePath: resolveRelativeRuntimePath(
      resourceRoot,
      runtimeLayout.readmeFile,
    ),
  };
}

export function createPortalDesktopRuntimeBuildPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  targetTriple = process.env.SDKWORK_DESKTOP_TARGET ?? '',
  env = process.env,
} = {}) {
  const frontendSteps = createDesktopAssetBuildPlan({
    workspaceRoot,
    platform,
  });

  return [
    {
      label: 'build admin frontend',
      ...frontendSteps[0],
    },
    {
      label: 'build portal frontend',
      ...frontendSteps[1],
    },
    {
      label: 'build router-product-service',
      command: 'cargo',
      args: [
        'build',
        '--release',
        ...(String(targetTriple ?? '').trim() ? ['--target', String(targetTriple).trim()] : []),
        '-p',
        'router-product-service',
      ],
      cwd: workspaceRoot,
      env: withManagedWorkspaceTargetDir({
        workspaceRoot,
        env: withManagedWorkspaceTempDir({
          workspaceRoot,
          env,
          platform,
        }),
        platform,
      }),
      shell: platform === 'win32',
      windowsHide: platform === 'win32',
    },
    {
      label: 'stage portal desktop router-product resources',
      command: process.execPath,
      args: [
        __filename,
        '--stage-only',
        ...(String(targetTriple ?? '').trim() ? ['--target', String(targetTriple).trim()] : []),
      ],
      cwd: workspaceRoot,
      env: { ...env },
      shell: false,
      windowsHide: platform === 'win32',
    },
  ];
}

export function stagePortalDesktopRuntimeResources({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  targetTriple = process.env.SDKWORK_DESKTOP_TARGET ?? '',
  env = process.env,
} = {}) {
  const runtimeLayout = findNativePortalDesktopEmbeddedRuntimeLayoutSpec();
  const target = resolveDesktopReleaseTarget({
    targetTriple,
    platform,
    env,
  });
  const layout = resolvePortalDesktopRuntimeResourceLayout({
    workspaceRoot,
    platform,
    targetTriple,
    env,
  });

  if (!existsSync(layout.routerBinarySourcePath)) {
    throw new Error(`Missing router-product-service binary: ${layout.routerBinarySourcePath}`);
  }
  if (!existsSync(layout.adminSiteSourceDir)) {
    throw new Error(`Missing admin desktop site assets: ${layout.adminSiteSourceDir}`);
  }
  if (!existsSync(layout.portalSiteSourceDir)) {
    throw new Error(`Missing portal desktop site assets: ${layout.portalSiteSourceDir}`);
  }
  if (!existsSync(layout.bootstrapDataSourceDir)) {
    throw new Error(`Missing portal desktop bootstrap data: ${layout.bootstrapDataSourceDir}`);
  }

  rmSync(layout.routerProductRoot, { recursive: true, force: true });
  mkdirSync(layout.routerProductRoot, { recursive: true });
  mkdirSync(path.dirname(layout.routerBinaryPath), { recursive: true });
  mkdirSync(path.dirname(layout.adminSiteDir), { recursive: true });
  mkdirSync(path.dirname(layout.portalSiteDir), { recursive: true });
  cpSync(layout.routerBinarySourcePath, layout.routerBinaryPath);
  cpSync(layout.adminSiteSourceDir, layout.adminSiteDir, { recursive: true });
  cpSync(layout.portalSiteSourceDir, layout.portalSiteDir, { recursive: true });
  cpSync(layout.bootstrapDataSourceDir, layout.bootstrapDataDir, { recursive: true });
  writeJsonFile(layout.releasePayloadManifestPath, {
    type: 'portal-desktop-router-product',
    productId: 'sdkwork-router-portal-desktop',
    platform: target.platform,
    arch: target.arch,
    target: target.targetTriple,
    routerBinary: path.posix.join(
      relativizePortalDesktopRuntimePath(runtimeLayout.serviceBinaryDir, runtimeLayout),
      withExecutable(runtimeLayout.serviceBinaryName, target.platform),
    ),
    sites: ['admin', 'portal'],
    bootstrapDataRoots: ['data'],
  });
  writePortalDesktopPayloadReadme({
    releasePayloadReadmePath: layout.releasePayloadReadmePath,
    target,
    runtimeLayout,
  });

  return layout;
}

async function runBuildStep(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      env: step.env,
      stdio: 'inherit',
      shell: step.shell,
      windowsHide: step.windowsHide ?? process.platform === 'win32',
    });

    child.on('error', reject);
    child.on('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`${step.label} exited with signal ${signal}`));
        return;
      }
      if ((code ?? 1) !== 0) {
        reject(new Error(`${step.label} exited with code ${code}`));
        return;
      }
      resolve();
    });
  });
}

function parseCliArgs(argv) {
  const options = {
    stageOnly: false,
    targetTriple: process.env.SDKWORK_DESKTOP_TARGET ?? '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--stage-only') {
      options.stageOnly = true;
      continue;
    }

    if (token === '--target') {
      options.targetTriple = String(next ?? '').trim();
      index += 1;
    }
  }

  return options;
}

async function main() {
  const options = parseCliArgs(process.argv.slice(2));

  if (options.stageOnly) {
    stagePortalDesktopRuntimeResources({
      targetTriple: options.targetTriple,
    });
    return;
  }

  const plan = createPortalDesktopRuntimeBuildPlan({
    targetTriple: options.targetTriple,
  });
  for (const step of plan) {
    // eslint-disable-next-line no-await-in-loop
    await runBuildStep(step);
  }
}

if (__filename === process.argv[1]) {
  main().catch((error) => {
    console.error(`[prepare-router-portal-desktop-runtime] ${error.message}`);
    process.exit(1);
  });
}
