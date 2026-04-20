#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  normalizeDesktopPlatform,
  normalizeDesktopArch,
  resolveDesktopOfficialInstallerRule,
  resolveDesktopReleaseTarget,
} from './desktop-targets.mjs';
import { resolveProductReleaseVersion } from '../../bin/lib/router-runtime-tooling.mjs';
import { materializeReleaseCatalog } from './materialize-release-catalog.mjs';
import {
  findNativeDesktopReleaseProductSpecByAppId,
  findNativeReleaseProductSpec,
} from './native-release-product-catalog.mjs';
import {
  materializeNativeDesktopBuildRootCandidates,
  materializeNativeServiceReleaseRootCandidates,
} from './native-build-root-catalog.mjs';
import {
  findNativePortalDesktopEmbeddedRuntimeLayoutSpec,
  findNativeProductServerBundleRuntimeLayoutSpec,
} from './native-runtime-layout-catalog.mjs';
import {
  renderProductServerBundleInstallPowerShellScript,
  renderProductServerBundleInstallShellScript,
} from './product-server-bundle-installer-generation.mjs';
import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const NATIVE_DESKTOP_APP_SPECS = [
  Object.freeze({
    id: 'admin',
    appDir: path.join(rootDir, 'apps', 'sdkwork-router-admin'),
    targetDirName: 'sdkwork-router-admin-tauri',
    releaseEnabled: false,
  }),
  Object.freeze({
    id: 'portal',
    appDir: path.join(rootDir, 'apps', 'sdkwork-router-portal'),
    targetDirName: 'sdkwork-router-portal-tauri',
    releaseEnabled: true,
  }),
];

const NATIVE_SERVICE_BINARY_NAMES = [
  'admin-api-service',
  'gateway-service',
  'portal-api-service',
  'router-web-service',
  'router-product-service',
];

const NATIVE_PRODUCT_SERVER_SITE_ASSET_ROOT_SPECS = [
  Object.freeze({
    id: 'admin',
    rootPath: path.join(rootDir, 'apps', 'sdkwork-router-admin', 'dist'),
  }),
  Object.freeze({
    id: 'portal',
    rootPath: path.join(rootDir, 'apps', 'sdkwork-router-portal', 'dist'),
  }),
];

const NATIVE_PRODUCT_SERVER_BOOTSTRAP_DATA_ROOT_SPECS = [
  Object.freeze({
    id: 'data',
    rootPath: path.join(rootDir, 'data'),
  }),
];

const NATIVE_PRODUCT_SERVER_DEPLOYMENT_ASSET_ROOT_SPECS = [
  Object.freeze({
    id: 'deploy',
    rootPath: path.join(rootDir, 'deploy'),
  }),
];

const NATIVE_PRODUCT_SERVER_CONTROL_SCRIPT_NAMES = [
  'start.sh',
  'stop.sh',
  'backup.sh',
  'restore.sh',
  'support-bundle.sh',
  'validate-config.sh',
  'start.ps1',
  'stop.ps1',
  'backup.ps1',
  'restore.ps1',
  'support-bundle.ps1',
  'validate-config.ps1',
];

const NATIVE_PRODUCT_SERVER_CONTROL_LIB_NAMES = [
  'runtime-common.sh',
  'runtime-common.ps1',
];

function cloneNativeProductServerAssetRootSpec(spec) {
  return {
    ...spec,
  };
}

function cloneNativeDesktopAppSpec(spec) {
  return {
    ...spec,
  };
}

const nativeDesktopAppSpecCatalog = createStrictKeyedCatalog({
  entries: NATIVE_DESKTOP_APP_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneNativeDesktopAppSpec,
  duplicateKeyMessagePrefix: 'duplicate native desktop app spec',
  missingKeyMessagePrefix: 'missing native desktop app spec',
});

const nativeServiceBinaryNameCatalog = createStrictKeyedCatalog({
  entries: NATIVE_SERVICE_BINARY_NAMES,
  getKey: (binaryName) => binaryName,
  duplicateKeyMessagePrefix: 'duplicate native service binary name',
  missingKeyMessagePrefix: 'missing native service binary name',
});

const nativeProductServerSiteAssetRootCatalog = createStrictKeyedCatalog({
  entries: NATIVE_PRODUCT_SERVER_SITE_ASSET_ROOT_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneNativeProductServerAssetRootSpec,
  duplicateKeyMessagePrefix: 'duplicate native product server site asset root',
  missingKeyMessagePrefix: 'missing native product server site asset root',
});

const nativeProductServerBootstrapDataRootCatalog = createStrictKeyedCatalog({
  entries: NATIVE_PRODUCT_SERVER_BOOTSTRAP_DATA_ROOT_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneNativeProductServerAssetRootSpec,
  duplicateKeyMessagePrefix: 'duplicate native product server bootstrap data root',
  missingKeyMessagePrefix: 'missing native product server bootstrap data root',
});

const nativeProductServerDeploymentAssetRootCatalog = createStrictKeyedCatalog({
  entries: NATIVE_PRODUCT_SERVER_DEPLOYMENT_ASSET_ROOT_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneNativeProductServerAssetRootSpec,
  duplicateKeyMessagePrefix: 'duplicate native product server deployment asset root',
  missingKeyMessagePrefix: 'missing native product server deployment asset root',
});

const nativeProductServerControlScriptNameCatalog = createStrictKeyedCatalog({
  entries: NATIVE_PRODUCT_SERVER_CONTROL_SCRIPT_NAMES,
  getKey: (fileName) => fileName,
  duplicateKeyMessagePrefix: 'duplicate native product server control script name',
  missingKeyMessagePrefix: 'missing native product server control script name',
});

const nativeProductServerControlLibNameCatalog = createStrictKeyedCatalog({
  entries: NATIVE_PRODUCT_SERVER_CONTROL_LIB_NAMES,
  getKey: (fileName) => fileName,
  duplicateKeyMessagePrefix: 'duplicate native product server control lib name',
  missingKeyMessagePrefix: 'missing native product server control lib name',
});

function buildNativeProductServerAssetRootMap(specs = []) {
  return Object.fromEntries(
    specs.map((spec) => [spec.id, spec.rootPath]),
  );
}

export function normalizePlatformId(platform = process.platform) {
  return normalizeDesktopPlatform(platform);
}

export function shouldIncludeDesktopBundleFile(platformId, relativePath) {
  const normalizedPlatform = normalizePlatformId(platformId);
  const artifactRule = resolveDesktopOfficialInstallerRule({
    platform: normalizedPlatform,
  });
  const normalizedPath = relativePath.replaceAll('\\', '/');
  const [topLevelDirectory] = normalizedPath.split('/');
  if (topLevelDirectory !== artifactRule.expectedBundleDirectory) {
    return false;
  }

  const lowerCasePath = normalizedPath.toLowerCase();
  return lowerCasePath.endsWith(artifactRule.expectedFileSuffix);
}

export function resolveNativeBuildRoot({ appId, targetTriple = '' } = {}) {
  const appDir = resolveNativeDesktopAppSpecOrThrow(appId).appDir;

  const normalizedTargetTriple = String(targetTriple ?? '').trim();
  const targetSegments = normalizedTargetTriple.length > 0
    ? [normalizedTargetTriple]
    : [];

  return path.join(
    appDir,
    'src-tauri',
    'target',
    ...targetSegments,
    'release',
    'bundle',
  );
}

export function resolveNativeBuildRootCandidates({
  appId,
  targetTriple = '',
  env = process.env,
  platform = process.platform,
} = {}) {
  const appSpec = resolveNativeDesktopAppSpecOrThrow(appId);
  return materializeNativeDesktopBuildRootCandidates({
    appDir: appSpec.appDir,
    workspaceTargetDirName: appSpec.targetDirName,
    targetTriple,
    env,
    platform,
    workspaceRoot: rootDir,
  });
}

export function listNativeServiceBinaryNames() {
  return nativeServiceBinaryNameCatalog.list();
}

export function findNativeServiceBinaryName(binaryName) {
  return nativeServiceBinaryNameCatalog.find(binaryName);
}

export function listNativeServiceBinaryNamesByIds(binaryNames = []) {
  return nativeServiceBinaryNameCatalog.listByKeys(binaryNames);
}

export function listNativeDesktopAppIds() {
  return listNativeDesktopAppSpecs()
    .filter((spec) => spec.releaseEnabled)
    .map((spec) => spec.id);
}

export function listNativeDesktopAppSpecs() {
  return nativeDesktopAppSpecCatalog.list();
}

export function findNativeDesktopAppSpec(appId) {
  return nativeDesktopAppSpecCatalog.find(appId);
}

export function listNativeDesktopAppSpecsByIds(appIds = []) {
  return nativeDesktopAppSpecCatalog.listByKeys(appIds);
}

export function listNativeProductServerSiteAssetRoots() {
  return buildNativeProductServerAssetRootMap(
    nativeProductServerSiteAssetRootCatalog.list(),
  );
}

export function findNativeProductServerSiteAssetRoot(rootId) {
  return nativeProductServerSiteAssetRootCatalog.find(rootId).rootPath;
}

export function listNativeProductServerSiteAssetRootsByIds(rootIds = []) {
  return buildNativeProductServerAssetRootMap(
    nativeProductServerSiteAssetRootCatalog.listByKeys(rootIds),
  );
}

export function listNativeProductServerBootstrapDataRoots() {
  return buildNativeProductServerAssetRootMap(
    nativeProductServerBootstrapDataRootCatalog.list(),
  );
}

export function findNativeProductServerBootstrapDataRoot(rootId) {
  return nativeProductServerBootstrapDataRootCatalog.find(rootId).rootPath;
}

export function listNativeProductServerBootstrapDataRootsByIds(rootIds = []) {
  return buildNativeProductServerAssetRootMap(
    nativeProductServerBootstrapDataRootCatalog.listByKeys(rootIds),
  );
}

export function listNativeProductServerDeploymentAssetRoots() {
  return buildNativeProductServerAssetRootMap(
    nativeProductServerDeploymentAssetRootCatalog.list(),
  );
}

export function findNativeProductServerDeploymentAssetRoot(rootId) {
  return nativeProductServerDeploymentAssetRootCatalog.find(rootId).rootPath;
}

export function listNativeProductServerDeploymentAssetRootsByIds(rootIds = []) {
  return buildNativeProductServerAssetRootMap(
    nativeProductServerDeploymentAssetRootCatalog.listByKeys(rootIds),
  );
}

export function listNativeProductServerControlScriptNames() {
  return nativeProductServerControlScriptNameCatalog.list();
}

export function findNativeProductServerControlScriptName(fileName) {
  return nativeProductServerControlScriptNameCatalog.find(fileName);
}

export function listNativeProductServerControlScriptNamesByIds(fileNames = []) {
  return nativeProductServerControlScriptNameCatalog.listByKeys(fileNames);
}

export function listNativeProductServerControlLibNames() {
  return nativeProductServerControlLibNameCatalog.list();
}

export function findNativeProductServerControlLibName(fileName) {
  return nativeProductServerControlLibNameCatalog.find(fileName);
}

export function listNativeProductServerControlLibNamesByIds(fileNames = []) {
  return nativeProductServerControlLibNameCatalog.listByKeys(fileNames);
}

export function buildNativeProductServerArchiveBaseName({ platformId, archId } = {}) {
  const productSpec = findNativeReleaseProductSpec('product-server');
  return `${productSpec.baseNamePrefix}-${platformId}-${archId}`;
}

export function createNativeProductServerReleaseAssetSpec({ platformId, archId } = {}) {
  const productSpec = findNativeReleaseProductSpec('product-server');
  const baseName = buildNativeProductServerArchiveBaseName({
    platformId,
    archId,
  });
  const fileName = `${baseName}${productSpec.archiveFileExtension}`;
  return {
    productId: productSpec.productId,
    fileName,
    checksumFileName: `${fileName}.sha256.txt`,
    manifestFileName: `${baseName}.manifest.json`,
  };
}

export function buildNativePortalDesktopArtifactBaseName({ platformId, archId } = {}) {
  const productSpec = findNativeDesktopReleaseProductSpecByAppId('portal');
  return `${productSpec.baseNamePrefix}-${platformId}-${archId}`;
}

export function createNativePortalDesktopReleaseAssetSpec({ platformId, archId } = {}) {
  const productSpec = findNativeDesktopReleaseProductSpecByAppId('portal');
  const normalizedPlatformId = normalizePlatformId(platformId);
  const normalizedArchId = normalizeDesktopArch(archId);
  const artifactRule = resolveDesktopOfficialInstallerRule({
    platform: normalizedPlatformId,
  });

  const baseName = buildNativePortalDesktopArtifactBaseName({
    platformId: normalizedPlatformId,
    archId: normalizedArchId,
  });
  const fileName = `${baseName}${artifactRule.expectedFileSuffix}`;

  return {
    appId: productSpec.appId,
    artifactKind: artifactRule.artifactKind,
    fileName,
    checksumFileName: `${fileName}.sha256.txt`,
    manifestFileName: `${baseName}.manifest.json`,
    expectedBundleDirectory: artifactRule.expectedBundleDirectory,
    expectedFileSuffix: artifactRule.expectedFileSuffix,
  };
}

export function resolveAvailableNativeBuildRoot({
  appId,
  targetTriple = '',
  buildRoots,
  exists = existsSync,
  listFiles = listFilesRecursively,
} = {}) {
  const candidates = Array.isArray(buildRoots) && buildRoots.length > 0
    ? buildRoots
    : resolveNativeBuildRootCandidates({
        appId,
        targetTriple,
      });

  let firstExistingRoot = '';
  for (const candidate of candidates) {
    if (!exists(candidate)) {
      continue;
    }

    if (firstExistingRoot.length === 0) {
      firstExistingRoot = candidate;
    }

    if (listFiles(candidate).length > 0) {
      return candidate;
    }
  }

  return firstExistingRoot;
}

export function resolveServiceReleaseRootCandidates({
  targetTriple = '',
  env = process.env,
  platform = process.platform,
} = {}) {
  return materializeNativeServiceReleaseRootCandidates({
    targetTriple,
    env,
    platform,
    workspaceRoot: rootDir,
  });
}

export function resolveAvailableServiceReleaseRoot({
  targetTriple = '',
  env = process.env,
  platform = process.platform,
  serviceBinaryNames = listNativeServiceBinaryNames(),
  serviceReleaseRoots,
  exists = existsSync,
} = {}) {
  const platformId = normalizePlatformId(platform);
  const candidates = Array.isArray(serviceReleaseRoots) && serviceReleaseRoots.length > 0
    ? [...new Set(serviceReleaseRoots)]
    : resolveServiceReleaseRootCandidates({
        targetTriple,
        env,
        platform,
      });

  let firstExistingRoot = '';
  for (const candidate of candidates) {
    if (!exists(candidate)) {
      continue;
    }

    if (firstExistingRoot.length === 0) {
      firstExistingRoot = candidate;
    }

    const hasAllServiceBinaries = serviceBinaryNames.every((binaryName) =>
      exists(path.join(candidate, withExecutable(binaryName, platformId))));
    if (hasAllServiceBinaries) {
      return candidate;
    }
  }

  return firstExistingRoot || candidates[0] || '';
}

function resolveServiceReleaseRoot(options = {}) {
  return resolveAvailableServiceReleaseRoot(options);
}

function resolveNativeDesktopAppSpecOrThrow(appId) {
  try {
    return findNativeDesktopAppSpec(appId);
  } catch {
    throw new Error(`Unsupported desktop application id: ${appId}`);
  }
}

function parseArgs(argv) {
  const [mode, ...rest] = argv;
  const options = {
    mode,
    platform: process.platform,
    arch: process.arch,
    target: '',
    outputDir: path.join(rootDir, 'artifacts', 'release'),
  };

  for (let index = 0; index < rest.length; index += 1) {
    const token = rest[index];
    const next = rest[index + 1];

    if (token === '--platform') {
      options.platform = next;
      index += 1;
      continue;
    }

    if (token === '--arch') {
      options.arch = next;
      index += 1;
      continue;
    }

    if (token === '--target') {
      options.target = next;
      index += 1;
      continue;
    }

    if (token === '--output-dir') {
      options.outputDir = path.resolve(next);
      index += 1;
      continue;
    }
  }

  return options;
}

function listFilesRecursively(sourceDir, relativePrefix = '') {
  const entries = readdirSync(sourceDir, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const relativePath = path.join(relativePrefix, entry.name);
    const absolutePath = path.join(sourceDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFilesRecursively(absolutePath, relativePath));
      continue;
    }

    if (entry.isFile()) {
      files.push({
        absolutePath,
        relativePath,
      });
    }
  }

  return files;
}

function ensureDirectory(directoryPath) {
  mkdirSync(directoryPath, { recursive: true });
}

function createManagedStagingRoot(stagingParent, prefix) {
  ensureDirectory(stagingParent);
  return mkdtempSync(path.join(stagingParent, prefix));
}

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function escapeGitHubActionsCommandValue(value, { property = false } = {}) {
  let escaped = String(value ?? '');
  escaped = escaped.replaceAll('%', '%25');
  escaped = escaped.replaceAll('\r', '%0D');
  escaped = escaped.replaceAll('\n', '%0A');
  if (property) {
    escaped = escaped.replaceAll(':', '%3A');
    escaped = escaped.replaceAll(',', '%2C');
  }

  return escaped;
}

export function buildGitHubActionsErrorAnnotation({
  title = 'package-release-assets',
  error,
} = {}) {
  const message = truncateText(
    error instanceof Error ? error.message : String(error),
    8000,
  );
  const escapedTitle = escapeGitHubActionsCommandValue(title, { property: true });
  const escapedMessage = escapeGitHubActionsCommandValue(message);
  return `::error title=${escapedTitle}::${escapedMessage}`;
}

function describeDirectoryState(sourceDir, maxEntries = 12) {
  if (!existsSync(sourceDir)) {
    return `${sourceDir} [missing]`;
  }

  const files = listFilesRecursively(sourceDir);
  if (files.length === 0) {
    return `${sourceDir} [exists, empty]`;
  }

  const sample = files
    .slice(0, maxEntries)
    .map((file) => file.relativePath.replaceAll('\\', '/'))
    .join(', ');
  const remainingCount = files.length - Math.min(files.length, maxEntries);
  const remainingSuffix = remainingCount > 0 ? ` (+${remainingCount} more)` : '';
  return `${sourceDir} [${files.length} files: ${sample}${remainingSuffix}]`;
}

function writeSha256File(filePath) {
  const checksum = createHash('sha256').update(readFileSync(filePath)).digest('hex');
  writeFileSync(
    `${filePath}.sha256.txt`,
    `${checksum}  ${path.basename(filePath)}\n`,
    'utf8',
  );
}

function writeJsonFile(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function withExecutable(binaryName, platformId) {
  return platformId === 'windows' ? `${binaryName}.exe` : binaryName;
}

function splitRelativeRuntimePath(relativePath) {
  return String(relativePath ?? '')
    .split('/')
    .filter((segment) => segment.length > 0);
}

function resolveArchiveTargetPath(archiveRoot, relativePath) {
  return path.join(archiveRoot, ...splitRelativeRuntimePath(relativePath));
}

function resolveRuntimeLayoutMappedPath(entryMap, entryId, entryKind) {
  const relativePath = entryMap[entryId];
  if (!relativePath) {
    throw new Error(`Missing runtime layout ${entryKind}: ${entryId}`);
  }

  return relativePath;
}

function normalizeWindowsRelativePath(relativePath) {
  return relativePath.replaceAll('/', '\\');
}

function resolveBundleInstallerMetadata(runtimeLayout = {}) {
  return {
    shell: String(runtimeLayout?.bundleInstallers?.shell ?? 'install.sh').trim() || 'install.sh',
    powershell: String(runtimeLayout?.bundleInstallers?.powershell ?? 'install.ps1').trim() || 'install.ps1',
  };
}

function createPortalDesktopEmbeddedRuntimeManifest({ platformId } = {}) {
  const runtimeLayout = findNativePortalDesktopEmbeddedRuntimeLayoutSpec();
  const adminSiteDir = resolveRuntimeLayoutMappedPath(
    runtimeLayout.siteTargetDirs,
    'admin',
    'site target dir',
  );
  const portalSiteDir = resolveRuntimeLayoutMappedPath(
    runtimeLayout.siteTargetDirs,
    'portal',
    'site target dir',
  );
  const bootstrapDataDir = resolveRuntimeLayoutMappedPath(
    runtimeLayout.bootstrapDataRootDirs,
    'data',
    'bootstrap data root dir',
  );

  return {
    routerBinary: path.posix.join(
      runtimeLayout.serviceBinaryDir,
      withExecutable(runtimeLayout.serviceBinaryName, platformId),
    ),
    adminSiteDir,
    portalSiteDir,
    bootstrapDataDir,
    releaseManifestFile: runtimeLayout.releaseManifestFile,
    readmeFile: runtimeLayout.readmeFile,
  };
}

function copyServiceBinaries({
  platformId,
  targetTriple,
  targetDir,
  writeChecksums = false,
  resolveServiceRoot = resolveServiceReleaseRoot,
  serviceReleaseRoots,
  serviceBinaryNames = listNativeServiceBinaryNames(),
}) {
  const serviceReleaseRoot = resolveServiceRoot({
    targetTriple,
    platform: platformId,
    serviceReleaseRoots,
    serviceBinaryNames,
  });
  ensureDirectory(targetDir);

  for (const binaryName of serviceBinaryNames) {
    const fileName = withExecutable(binaryName, platformId);
    const sourcePath = path.join(serviceReleaseRoot, fileName);
    if (!existsSync(sourcePath)) {
      throw new Error(
        `Missing release service binary: ${sourcePath}\nservice release root: ${describeDirectoryState(serviceReleaseRoot)}`,
      );
    }

    const targetPath = path.join(targetDir, fileName);
    cpSync(sourcePath, targetPath);
    if (writeChecksums) {
      writeSha256File(targetPath);
    }
  }
}

export function packageDesktopBundles({
  platformId,
  archId,
  targetTriple,
  outputDir,
  resolveBuildRoots = resolveNativeBuildRootCandidates,
  resolveBuildRoot = resolveAvailableNativeBuildRoot,
} = {}) {
  const packagedAssets = [];

  for (const appId of listNativeDesktopAppIds()) {
    const releaseProductSpec = findNativeDesktopReleaseProductSpecByAppId(appId);
    const releaseAssetSpec = createNativePortalDesktopReleaseAssetSpec({
      platformId,
      archId,
    });
    const buildRoots = resolveBuildRoots({ appId, targetTriple });
    const buildRoot = resolveBuildRoot({
      appId,
      targetTriple,
      buildRoots,
    });
    if (!buildRoot) {
      throw new Error(
        `Missing desktop bundle output directory for ${appId}. candidates: ${buildRoots.map((root) => describeDirectoryState(root)).join(' | ')}`,
      );
    }

    const allBundleFiles = listFilesRecursively(buildRoot);
    const bundleFiles = allBundleFiles
      .filter((file) => shouldIncludeDesktopBundleFile(platformId, file.relativePath));

    if (bundleFiles.length === 0) {
      throw new Error(
        [
          `Missing official ${platformId} desktop installer for ${appId}.`,
          `Expected ${releaseAssetSpec.expectedBundleDirectory}/*${releaseAssetSpec.expectedFileSuffix} under ${buildRoot}`,
          `bundle root: ${describeDirectoryState(buildRoot)}`,
        ].join('\n'),
      );
    }
    if (bundleFiles.length !== 1) {
      throw new Error(
        [
          `Expected exactly one official ${platformId} desktop installer for ${appId}, found ${bundleFiles.length}.`,
          `Matched files: ${bundleFiles.map((file) => file.relativePath.replaceAll('\\', '/')).join(', ')}`,
          `Expected ${releaseAssetSpec.expectedBundleDirectory}/*${releaseAssetSpec.expectedFileSuffix} under ${buildRoot}`,
        ].join('\n'),
      );
    }

    const appOutputDir = path.join(
      outputDir,
      'native',
      platformId,
      archId,
      ...releaseProductSpec.outputPathSegments,
    );
    rmSync(appOutputDir, { recursive: true, force: true });
    ensureDirectory(appOutputDir);

    const [bundleFile] = bundleFiles;
    const installerTargetPath = path.join(appOutputDir, releaseAssetSpec.fileName);
    cpSync(bundleFile.absolutePath, installerTargetPath);
    writeSha256File(installerTargetPath);
    writeJsonFile(
      path.join(appOutputDir, releaseAssetSpec.manifestFileName),
      {
        type: releaseProductSpec.manifestType,
        productId: releaseProductSpec.productId,
        appId,
        platform: platformId,
        arch: archId,
        target: targetTriple,
        artifactKind: releaseAssetSpec.artifactKind,
        installerFile: releaseAssetSpec.fileName,
        checksumFile: releaseAssetSpec.checksumFileName,
        sourceBundlePath: bundleFile.relativePath.replaceAll('\\', '/'),
        embeddedRuntime: createPortalDesktopEmbeddedRuntimeManifest({ platformId }),
      },
    );

    packagedAssets.push({
      appId,
      platformId,
      archId,
      targetTriple,
      fileName: releaseAssetSpec.fileName,
      checksumFileName: releaseAssetSpec.checksumFileName,
      manifestFileName: releaseAssetSpec.manifestFileName,
      outputDir: appOutputDir,
    });
  }

  return packagedAssets;
}

function writeProductServerBundleReadme({
  archiveRoot,
  releaseVersion,
  platformId,
  archId,
  targetTriple,
  runtimeLayout = findNativeProductServerBundleRuntimeLayoutSpec(),
}) {
  const adminSiteDir = resolveRuntimeLayoutMappedPath(
    runtimeLayout.siteTargetDirs,
    'admin',
    'site target dir',
  );
  const portalSiteDir = resolveRuntimeLayoutMappedPath(
    runtimeLayout.siteTargetDirs,
    'portal',
    'site target dir',
  );
  const bootstrapDataDir = resolveRuntimeLayoutMappedPath(
    runtimeLayout.bootstrapDataRootDirs,
    'data',
    'bootstrap data root dir',
  );
  const deploymentAssetDir = resolveRuntimeLayoutMappedPath(
    runtimeLayout.deploymentAssetRootDirs,
    'deploy',
    'deployment asset root dir',
  );
  const routerBinary = path.posix.join(
    runtimeLayout.serviceBinaryDir,
    withExecutable(runtimeLayout.serviceBinaryName, platformId),
  );
  const installers = resolveBundleInstallerMetadata(runtimeLayout);

  writeFileSync(
    resolveArchiveTargetPath(archiveRoot, runtimeLayout.readmeFile),
    [
      'SDKWork API Router Product Server Bundle',
      '',
      `release-version: ${releaseVersion}`,
      `platform: ${platformId}`,
      `arch: ${archId}`,
      `target: ${targetTriple}`,
      '',
      'Contents:',
      `- ${runtimeLayout.serviceBinaryDir}/: standalone services plus ${runtimeLayout.serviceBinaryName}`,
      `- ${runtimeLayout.controlScriptDir}/: installed current/bin operator control scripts and helpers`,
      `- ${adminSiteDir}/: admin web assets`,
      `- ${portalSiteDir}/: portal web assets`,
      `- ${bootstrapDataDir}/: bootstrap data packs for first-start initialization`,
      `- ${deploymentAssetDir}/: docker, compose, and helm deployment assets`,
      '',
      'Install after extracting the archive:',
      `  ./${installers.shell} --mode system`,
      `  powershell -NoProfile -ExecutionPolicy Bypass -File .\\${normalizeWindowsRelativePath(installers.powershell)} -Mode system`,
      '',
      'Example startup:',
      platformId === 'windows'
        ? `  set SDKWORK_BOOTSTRAP_DATA_DIR=${normalizeWindowsRelativePath(bootstrapDataDir)} && set SDKWORK_ADMIN_SITE_DIR=${normalizeWindowsRelativePath(adminSiteDir)} && set SDKWORK_PORTAL_SITE_DIR=${normalizeWindowsRelativePath(portalSiteDir)} && ${normalizeWindowsRelativePath(routerBinary)}`
        : `  SDKWORK_BOOTSTRAP_DATA_DIR=${bootstrapDataDir} SDKWORK_ADMIN_SITE_DIR=${adminSiteDir} SDKWORK_PORTAL_SITE_DIR=${portalSiteDir} ./${routerBinary}`,
      '',
      'Container image builds reuse the Linux product-server bundle with:',
      `  docker build -f ${path.posix.join(deploymentAssetDir, 'docker', 'Dockerfile')} -t sdkwork-api-router:<tag> .`,
      '',
      'Override SDKWORK_CONFIG_DIR, SDKWORK_CONFIG_FILE, SDKWORK_DATABASE_URL, and role/upstream flags as needed.',
      '',
    ].join('\n'),
    'utf8',
  );
}

export function packageProductServerBundle({
  platformId,
  archId,
  targetTriple,
  outputDir,
  resolveServiceRoot = resolveServiceReleaseRoot,
  resolveServiceRootCandidates,
  serviceBinaryNames = listNativeServiceBinaryNames(),
  siteAssetRoots = listNativeProductServerSiteAssetRoots(),
  bootstrapDataRoots = listNativeProductServerBootstrapDataRoots(),
  deploymentAssetRoots = listNativeProductServerDeploymentAssetRoots(),
  runTar = runTarCommand,
} = {}) {
  const releaseProductSpec = findNativeReleaseProductSpec('product-server');
  const runtimeLayout = findNativeProductServerBundleRuntimeLayoutSpec();
  const installers = resolveBundleInstallerMetadata(runtimeLayout);
  const releaseVersion = resolveProductReleaseVersion(rootDir);
  for (const [label, sourceDir] of Object.entries(siteAssetRoots)) {
    if (!existsSync(sourceDir)) {
      throw new Error(
        `Missing product server site assets for ${label}: ${sourceDir}\nsite asset root: ${describeDirectoryState(sourceDir)}`,
      );
    }
  }
  for (const [label, sourceDir] of Object.entries(bootstrapDataRoots)) {
    if (!existsSync(sourceDir)) {
      throw new Error(
        `Missing product server bootstrap data for ${label}: ${sourceDir}\nbootstrap data root: ${describeDirectoryState(sourceDir)}`,
      );
    }
  }
  for (const [label, sourceDir] of Object.entries(deploymentAssetRoots)) {
    if (!existsSync(sourceDir)) {
      throw new Error(
        `Missing product server deployment assets for ${label}: ${sourceDir}\ndeployment asset root: ${describeDirectoryState(sourceDir)}`,
      );
    }
  }

  const archiveBaseName = buildNativeProductServerArchiveBaseName({
    platformId,
    archId,
  });
  const releaseAssetSpec = createNativeProductServerReleaseAssetSpec({
    platformId,
    archId,
  });
  const bundleOutputDir = path.join(
    outputDir,
    'native',
    platformId,
    archId,
    ...releaseProductSpec.outputPathSegments,
  );
  const serviceReleaseRoots = typeof resolveServiceRootCandidates === 'function'
    ? resolveServiceRootCandidates({
        targetTriple,
        platform: platformId,
        serviceBinaryNames,
      })
    : undefined;
  ensureDirectory(bundleOutputDir);

  const stagingRoot = createManagedStagingRoot(bundleOutputDir, '.sdkwork-api-router-native-server-');
  const archiveRoot = path.join(stagingRoot, archiveBaseName);

  try {
    copyServiceBinaries({
      platformId,
      targetTriple,
      targetDir: resolveArchiveTargetPath(archiveRoot, runtimeLayout.serviceBinaryDir),
      resolveServiceRoot,
      serviceReleaseRoots,
      serviceBinaryNames,
    });

    copyNamedFiles({
      sourceDir: path.join(rootDir, 'bin'),
      targetDir: resolveArchiveTargetPath(archiveRoot, runtimeLayout.controlScriptDir),
      fileNames: listNativeProductServerControlScriptNames(),
      executableExtensions: ['.sh'],
    });

    copyNamedFiles({
      sourceDir: path.join(rootDir, 'bin', 'lib'),
      targetDir: resolveArchiveTargetPath(
        archiveRoot,
        path.posix.join(runtimeLayout.controlScriptDir, 'lib'),
      ),
      fileNames: listNativeProductServerControlLibNames(),
      executableExtensions: ['.sh'],
    });

    for (const [label, sourceDir] of Object.entries(siteAssetRoots)) {
      const targetDir = resolveArchiveTargetPath(
        archiveRoot,
        resolveRuntimeLayoutMappedPath(runtimeLayout.siteTargetDirs, label, 'site target dir'),
      );
      ensureDirectory(path.dirname(targetDir));
      cpSync(sourceDir, targetDir, { recursive: true });
    }

    for (const [label, sourceDir] of Object.entries(bootstrapDataRoots)) {
      const targetDir = resolveArchiveTargetPath(
        archiveRoot,
        resolveRuntimeLayoutMappedPath(
          runtimeLayout.bootstrapDataRootDirs,
          label,
          'bootstrap data root dir',
        ),
      );
      ensureDirectory(path.dirname(targetDir));
      cpSync(sourceDir, targetDir, { recursive: true });
    }

    for (const [label, sourceDir] of Object.entries(deploymentAssetRoots)) {
      const targetDir = resolveArchiveTargetPath(
        archiveRoot,
        resolveRuntimeLayoutMappedPath(
          runtimeLayout.deploymentAssetRootDirs,
          label,
          'deployment asset root dir',
        ),
      );
      ensureDirectory(path.dirname(targetDir));
      cpSync(sourceDir, targetDir, { recursive: true });
    }

    writeFileSync(
      resolveArchiveTargetPath(archiveRoot, installers.shell),
      renderProductServerBundleInstallShellScript({
        releaseVersion,
        runtimeLayout,
        platformId,
        archId,
        targetTriple,
        serviceBinaryNames,
      }),
      'utf8',
    );
    chmodSync(resolveArchiveTargetPath(archiveRoot, installers.shell), 0o755);

    writeFileSync(
      resolveArchiveTargetPath(archiveRoot, installers.powershell),
      renderProductServerBundleInstallPowerShellScript({
        releaseVersion,
        runtimeLayout,
        platformId,
        archId,
        targetTriple,
        serviceBinaryNames,
      }),
      'utf8',
    );

    writeProductServerBundleReadme({
      archiveRoot,
      releaseVersion,
      platformId,
      archId,
      targetTriple,
      runtimeLayout,
    });

    writeFileSync(
      resolveArchiveTargetPath(archiveRoot, runtimeLayout.releaseManifestFile),
      JSON.stringify(
        {
          type: releaseProductSpec.embeddedManifestType,
          productId: releaseAssetSpec.productId,
          releaseVersion,
          platform: platformId,
          arch: archId,
          target: targetTriple,
          installers,
          services: [...serviceBinaryNames],
          sites: Object.keys(siteAssetRoots),
          bootstrapDataRoots: Object.keys(bootstrapDataRoots),
          deploymentAssetRoots: Object.keys(deploymentAssetRoots),
        },
        null,
        2,
      ),
      'utf8',
    );

    const archivePath = path.join(bundleOutputDir, releaseAssetSpec.fileName);
    rmSync(archivePath, { force: true });
    rmSync(path.join(bundleOutputDir, releaseAssetSpec.checksumFileName), { force: true });
    rmSync(path.join(bundleOutputDir, releaseAssetSpec.manifestFileName), { force: true });
    runTar(archivePath, stagingRoot, archiveBaseName);
    writeSha256File(archivePath);
    writeJsonFile(
      path.join(bundleOutputDir, releaseAssetSpec.manifestFileName),
      {
        type: releaseProductSpec.archiveManifestType,
        productId: releaseAssetSpec.productId,
        releaseVersion,
        platform: platformId,
        arch: archId,
        target: targetTriple,
        archiveFile: releaseAssetSpec.fileName,
        checksumFile: releaseAssetSpec.checksumFileName,
        embeddedManifestFile: runtimeLayout.releaseManifestFile,
        installers,
        services: [...serviceBinaryNames],
        sites: Object.keys(siteAssetRoots),
        bootstrapDataRoots: Object.keys(bootstrapDataRoots),
        deploymentAssetRoots: Object.keys(deploymentAssetRoots),
      },
    );
    return {
      productId: releaseAssetSpec.productId,
      platformId,
      archId,
      targetTriple,
      fileName: releaseAssetSpec.fileName,
      checksumFileName: releaseAssetSpec.checksumFileName,
      manifestFileName: releaseAssetSpec.manifestFileName,
      outputDir: bundleOutputDir,
    };
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
}

function copyNamedFiles({
  sourceDir,
  targetDir,
  fileNames = [],
  executableExtensions = [],
} = {}) {
  ensureDirectory(targetDir);

  for (const fileName of fileNames) {
    const sourcePath = path.join(sourceDir, fileName);
    if (!existsSync(sourcePath)) {
      throw new Error(`Missing packaged control asset source: ${sourcePath}`);
    }

    const targetPath = path.join(targetDir, fileName);
    cpSync(sourcePath, targetPath);

    if (executableExtensions.some((extension) => fileName.endsWith(extension))) {
      chmodSync(targetPath, 0o755);
    }
  }
}

export function packageNativeAssets({
  platform,
  arch,
  target,
  outputDir,
  releaseTag = '',
  generatedAt = '',
  packageDesktopBundlesImpl = packageDesktopBundles,
  packageProductServerBundleImpl = packageProductServerBundle,
  materializeReleaseCatalogImpl = materializeReleaseCatalog,
} = {}) {
  const targetSpec = resolveDesktopReleaseTarget({
    targetTriple: target,
    platform,
    arch,
  });
  const platformId = normalizePlatformId(targetSpec.platform);
  const archId = normalizeDesktopArch(targetSpec.arch);
  const nativeOutputDir = path.join(outputDir, 'native', platformId, archId);
  rmSync(nativeOutputDir, { recursive: true, force: true });
  ensureDirectory(nativeOutputDir);

  const desktopAssets = packageDesktopBundlesImpl({
    platformId,
    archId,
    targetTriple: targetSpec.targetTriple,
    outputDir,
  });
  const productServerBundle = packageProductServerBundleImpl({
    platformId,
    archId,
    targetTriple: targetSpec.targetTriple,
    outputDir,
  });
  const releaseCatalog = materializeReleaseCatalogImpl({
    assetsRoot: outputDir,
    releaseTag,
    generatedAt,
    outputPath: path.join(outputDir, 'release-catalog.json'),
  });

  return {
    target: targetSpec,
    desktopAssets,
    productServerBundle,
    releaseCatalog,
  };
}

export function detectTarFlavor({
  platform = process.platform,
  spawn = spawnSync,
} = {}) {
  if (platform !== 'win32') {
    return 'default';
  }

  const result = spawn('tar', ['--version'], {
    cwd: rootDir,
    shell: false,
    encoding: 'utf8',
  });
  if (result.error || result.status !== 0) {
    return 'unknown';
  }

  const versionOutput = `${result.stdout ?? ''}\n${result.stderr ?? ''}`.toLowerCase();
  if (versionOutput.includes('gnu tar')) {
    return 'gnu';
  }
  if (versionOutput.includes('bsdtar') || versionOutput.includes('libarchive')) {
    return 'bsd';
  }

  return 'unknown';
}

export function createTarCommandPlan({
  archivePath,
  workingDirectory,
  entryName,
  platform = process.platform,
  tarFlavor = platform === 'win32' ? 'unknown' : 'default',
} = {}) {
  const args = [];
  if (platform === 'win32' && tarFlavor === 'gnu') {
    args.push('--force-local');
  }
  args.push('-czf', archivePath, '-C', workingDirectory, entryName);

  return {
    command: 'tar',
    args,
    shell: platform === 'win32',
  };
}

function runTarCommand(archivePath, workingDirectory, entryName) {
  const tarFlavor = detectTarFlavor();
  const plan = createTarCommandPlan({
    archivePath,
    workingDirectory,
    entryName,
    tarFlavor,
  });
  const result = spawnSync(plan.command, plan.args, {
    cwd: rootDir,
    shell: plan.shell,
    encoding: 'utf8',
  });

  if (result.error) {
    throw new Error(`tar failed while packaging ${archivePath}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    const stdout = truncateText(result.stdout, 2000);
    const stderr = truncateText(result.stderr, 2000);
    const output = [stdout && `stdout: ${stdout}`, stderr && `stderr: ${stderr}`]
      .filter(Boolean)
      .join('\n');
    throw new Error(
      `tar failed while packaging ${archivePath} with exit code ${result.status ?? 'unknown'}${output ? `\n${output}` : ''}`,
    );
  }
}

function printUsage() {
  console.error(
    [
      'Usage:',
      '  node scripts/release/package-release-assets.mjs native --platform <windows|linux|macos> --arch <x64|arm64> --target <triple> --output-dir <dir>',
    ].join('\n'),
  );
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  if (!options.mode) {
    printUsage();
    process.exit(1);
  }

  ensureDirectory(options.outputDir);

  if (options.mode === 'native') {
    packageNativeAssets(options);
    return;
  }

  console.error(`Unsupported packaging mode: ${options.mode}`);
  printUsage();
  process.exit(1);
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  try {
    main();
  } catch (error) {
    if (process.env.GITHUB_ACTIONS === 'true') {
      console.error(buildGitHubActionsErrorAnnotation({ error }));
    }
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  }
}
