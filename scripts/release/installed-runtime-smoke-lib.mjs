import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

import {
  materializeInstalledRuntimeControlLayout,
  materializeInstalledRuntimeMutableLayout,
} from './installed-runtime-layout-catalog.mjs';

export function createInstalledRuntimeSmokeLayout({
  installPlan,
  runtimeHome,
} = {}) {
  const productRoot = installPlan?.installRoot ?? runtimeHome;
  const mode = installPlan?.mode ?? 'portable';
  const controlLayout = materializeInstalledRuntimeControlLayout({
    productRoot,
    platform: process.platform,
    pathApi: path,
  });
  const mutableLayout = materializeInstalledRuntimeMutableLayout({
    productRoot,
    mode,
    platform: process.platform,
    env: process.env,
    pathApi: path,
  });

  return {
    productRoot,
    controlHome: installPlan?.controlRoot ?? controlLayout.controlRoot,
    configDir: mutableLayout.configRoot,
    logDir: mutableLayout.logRoot,
    runDir: mutableLayout.runRoot,
  };
}

export function resolveInstalledReleaseManifestPath(runtimeHome) {
  return materializeInstalledRuntimeControlLayout({
    productRoot: runtimeHome,
    platform: process.platform,
    pathApi: path,
  }).releaseManifestFile;
}

export function readInstalledReleaseManifest(runtimeHome, {
  exists = existsSync,
  readFile = readFileSync,
} = {}) {
  const manifestPath = resolveInstalledReleaseManifestPath(runtimeHome);
  if (!exists(manifestPath)) {
    throw new Error(`installed runtime is missing release manifest: ${manifestPath}`);
  }

  let manifest;
  try {
    manifest = JSON.parse(String(readFile(manifestPath, 'utf8')));
  } catch (error) {
    throw new Error(
      `installed runtime release manifest is not valid JSON: ${manifestPath}\n${error instanceof Error ? error.message : String(error)}`,
    );
  }

  return {
    manifestPath,
    manifest,
  };
}

export function assertInstalledReleasePayloadContract(runtimeHome, {
  exists = existsSync,
  readFile = readFileSync,
} = {}) {
  const { manifestPath, manifest } = readInstalledReleaseManifest(runtimeHome, {
    exists,
    readFile,
  });
  const missingGeneratedMetadata = [];
  if (String(manifest?.runtime ?? '').trim().length === 0) {
    missingGeneratedMetadata.push('runtime');
  }
  if (String(manifest?.layoutVersion ?? '').trim().length === 0) {
    missingGeneratedMetadata.push('layoutVersion');
  }
  if (missingGeneratedMetadata.length > 0) {
    throw new Error(
      `installed runtime release manifest is missing required generated metadata fields ${missingGeneratedMetadata.join(', ')}: ${manifestPath}`,
    );
  }
  if (String(manifest?.runtime ?? '').trim() !== 'sdkwork-api-router') {
    throw new Error(
      `installed runtime release manifest runtime must equal sdkwork-api-router: ${manifestPath}`,
    );
  }
  if (String(manifest?.layoutVersion ?? '').trim() !== '2') {
    throw new Error(
      `installed runtime release manifest layoutVersion must equal 2: ${manifestPath}`,
    );
  }
  const releaseVersion = resolveRequiredManifestString({
    manifest,
    manifestPath,
    fieldName: 'releaseVersion',
  });
  resolveRequiredManifestString({
    manifest,
    manifestPath,
    fieldName: 'target',
  });
  resolveRequiredManifestString({
    manifest,
    manifestPath,
    fieldName: 'installedAt',
  });
  assertInstalledManifestArrayField({
    manifest,
    manifestPath,
    fieldName: 'installedBinaries',
  });

  const productRoot = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.productRoot,
    fieldName: 'productRoot',
  });
  const controlRoot = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.controlRoot,
    fieldName: 'controlRoot',
  });
  const releasesRoot = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.releasesRoot,
    fieldName: 'releasesRoot',
  });
  const releaseRoot = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.releaseRoot,
    fieldName: 'releaseRoot',
  });
  const bootstrapDataRoot = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.bootstrapDataRoot,
    fieldName: 'bootstrapDataRoot',
  });
  const deploymentAssetRoot = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.deploymentAssetRoot,
    fieldName: 'deploymentAssetRoot',
  });
  const releasePayloadManifest = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.releasePayloadManifest,
    fieldName: 'releasePayloadManifest',
  });
  const releasePayloadReadmeFile = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.releasePayloadReadmeFile,
    fieldName: 'releasePayloadReadmeFile',
  });
  const adminSiteDistDir = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.adminSiteDistDir,
    fieldName: 'adminSiteDistDir',
  });
  const portalSiteDistDir = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.portalSiteDistDir,
    fieldName: 'portalSiteDistDir',
  });
  const routerBinary = resolveManifestContractPath({
    manifest,
    manifestPath,
    relativePath: manifest?.routerBinary,
    fieldName: 'routerBinary',
  });

  const requiredExistingPaths = [
    productRoot,
    controlRoot,
    releasesRoot,
    releaseRoot,
    bootstrapDataRoot,
    deploymentAssetRoot,
    releasePayloadManifest,
    releasePayloadReadmeFile,
    adminSiteDistDir,
    portalSiteDistDir,
    routerBinary,
  ];
  for (const requiredPath of requiredExistingPaths) {
    if (!exists(requiredPath)) {
      throw new Error(`installed runtime release manifest declares a missing path: ${requiredPath}`);
    }
  }

  const manifestControlRoot = path.dirname(manifestPath);
  if (!samePath(controlRoot, manifestControlRoot)) {
    throw new Error(
      `installed runtime release manifest controlRoot does not match the manifest location: ${manifestPath}`,
    );
  }
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'controlRoot',
    fieldPath: controlRoot,
    rootPath: productRoot,
    expectedRelativePath: 'current',
  });
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'releasesRoot',
    fieldPath: releasesRoot,
    rootPath: productRoot,
    expectedRelativePath: 'releases',
  });
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'releaseRoot',
    fieldPath: releaseRoot,
    rootPath: releasesRoot,
    expectedRelativePath: releaseVersion,
  });
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'bootstrapDataRoot',
    fieldPath: bootstrapDataRoot,
    rootPath: releaseRoot,
    expectedRelativePath: 'data',
  });
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'deploymentAssetRoot',
    fieldPath: deploymentAssetRoot,
    rootPath: releaseRoot,
    expectedRelativePath: 'deploy',
  });
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'releasePayloadManifest',
    fieldPath: releasePayloadManifest,
    rootPath: releaseRoot,
    expectedRelativePath: 'release-manifest.json',
  });
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'releasePayloadReadmeFile',
    fieldPath: releasePayloadReadmeFile,
    rootPath: releaseRoot,
    expectedRelativePath: 'README.txt',
  });
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'adminSiteDistDir',
    fieldPath: adminSiteDistDir,
    rootPath: releaseRoot,
    expectedRelativePath: 'sites/admin/dist',
  });
  assertContractPathWithinRoot({
    manifestPath,
    fieldName: 'portalSiteDistDir',
    fieldPath: portalSiteDistDir,
    rootPath: releaseRoot,
    expectedRelativePath: 'sites/portal/dist',
  });
  const routerBinaryRelativePath = relativePathWithinRoot(routerBinary, releaseRoot);
  if (routerBinaryRelativePath == null || !/^bin\/router-product-service(\.[A-Za-z0-9_-]+)?$/u.test(routerBinaryRelativePath)) {
    throw new Error(
      `installed runtime release manifest routerBinary must resolve within the active release payload layout under bin/router-product-service*: ${manifestPath}`,
    );
  }

  return {
    manifestPath,
    manifest,
    productRoot,
    controlRoot,
    releasesRoot,
    releaseVersion,
    releaseRoot,
    bootstrapDataRoot,
    deploymentAssetRoot,
    releasePayloadManifest,
    releasePayloadReadmeFile,
    adminSiteDistDir,
    portalSiteDistDir,
    routerBinary,
  };
}

export function resolveInstalledBootstrapDataRoot(runtimeHome, {
  exists = existsSync,
  readFile = readFileSync,
} = {}) {
  const { manifestPath, manifest } = readInstalledReleaseManifest(runtimeHome, {
    exists,
    readFile,
  });
  const bootstrapDataRoot = String(manifest?.bootstrapDataRoot ?? '').trim();
  if (bootstrapDataRoot.length === 0) {
    throw new Error(`installed runtime release manifest is missing bootstrapDataRoot: ${manifestPath}`);
  }

  return path.isAbsolute(bootstrapDataRoot)
    ? bootstrapDataRoot
    : path.resolve(path.dirname(manifestPath), bootstrapDataRoot);
}

export function assertInstalledPackagedBootstrapData(runtimeHome, {
  exists = existsSync,
  readFile = readFileSync,
} = {}) {
  const { bootstrapDataRoot } = assertInstalledReleasePayloadContract(runtimeHome, {
    exists,
    readFile,
  });
  const requiredFiles = [
    path.join(bootstrapDataRoot, 'channels', 'default.json'),
    path.join(bootstrapDataRoot, 'providers', 'default.json'),
    path.join(bootstrapDataRoot, 'routing', 'default.json'),
  ];

  for (const filePath of requiredFiles) {
    if (!exists(filePath)) {
      throw new Error(`installed runtime is missing packaged bootstrap data: ${filePath}`);
    }
  }

  return bootstrapDataRoot;
}

function resolveRequiredManifestString({
  manifest,
  manifestPath,
  fieldName,
} = {}) {
  const value = String(manifest?.[fieldName] ?? '').trim();
  if (value.length === 0) {
    throw new Error(`installed runtime release manifest is missing ${fieldName}: ${manifestPath}`);
  }

  return value;
}

function assertInstalledManifestArrayField({
  manifest,
  manifestPath,
  fieldName,
} = {}) {
  const value = manifest?.[fieldName];
  if (!Array.isArray(value) || value.length === 0) {
    throw new Error(`installed runtime release manifest is missing ${fieldName}: ${manifestPath}`);
  }
}

function resolveManifestContractPath({
  manifest,
  manifestPath,
  relativePath,
  fieldName,
} = {}) {
  const normalizedRelativePath = String(relativePath ?? '').trim();
  if (normalizedRelativePath.length === 0) {
    throw new Error(`installed runtime release manifest is missing ${fieldName}: ${manifestPath}`);
  }

  if (path.isAbsolute(normalizedRelativePath)) {
    return path.normalize(normalizedRelativePath);
  }

  const productRoot = String(manifest?.productRoot ?? '').trim();
  const baseRoot = productRoot.length > 0
    ? productRoot
    : path.dirname(manifestPath);
  return path.resolve(baseRoot, normalizedRelativePath);
}

function assertContractPathWithinRoot({
  manifestPath,
  fieldName,
  fieldPath,
  rootPath,
  expectedRelativePath,
} = {}) {
  const actualRelativePath = relativePathWithinRoot(fieldPath, rootPath);
  if (actualRelativePath == null || actualRelativePath !== normalizePortablePath(expectedRelativePath)) {
    throw new Error(
      `installed runtime release manifest ${fieldName} must resolve within the active release payload layout at ${normalizePortablePath(expectedRelativePath)}: ${manifestPath}`,
    );
  }
}

function relativePathWithinRoot(targetPath, rootPath) {
  const relativePath = path.relative(rootPath, targetPath);
  if (relativePath.startsWith('..') || path.isAbsolute(relativePath)) {
    return null;
  }

  return normalizePortablePath(relativePath || '.');
}

function normalizePortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/').replace(/^\.\/+/u, '').replace(/\/+/gu, '/').replace(/\/$/u, '');
}

function samePath(leftPath, rightPath) {
  return normalizePortablePath(path.resolve(leftPath)) === normalizePortablePath(path.resolve(rightPath));
}

export function assertInstalledRuntimeBackupBundle(backupRoot, {
  exists = existsSync,
  readFile = readFileSync,
} = {}) {
  const bundleRoot = path.resolve(backupRoot);
  const manifestPath = path.join(bundleRoot, 'backup-manifest.json');
  if (!exists(manifestPath)) {
    throw new Error(`installed runtime backup bundle is missing required path: ${manifestPath}`);
  }

  let manifest;
  try {
    manifest = JSON.parse(String(readFile(manifestPath, 'utf8')));
  } catch (error) {
    throw new Error(
        `installed runtime backup manifest is not valid JSON: ${manifestPath}\n${error instanceof Error ? error.message : String(error)}`,
      );
  }

  if (String(manifest?.formatVersion ?? '') !== '2') {
    throw new Error(`installed runtime backup manifest has unsupported formatVersion at ${manifestPath}`);
  }

  const controlManifestPath = resolveBundleContractPath({
    bundleRoot,
    manifestPath,
    relativePath: manifest?.bundle?.controlManifestFile,
    fieldName: 'bundle.controlManifestFile',
  });
  const configSnapshotRoot = resolveBundleContractPath({
    bundleRoot,
    manifestPath,
    relativePath: manifest?.bundle?.configSnapshotRoot,
    fieldName: 'bundle.configSnapshotRoot',
  });
  const mutableDataSnapshotRoot = resolveBundleContractPath({
    bundleRoot,
    manifestPath,
    relativePath: manifest?.bundle?.mutableDataSnapshotRoot,
    fieldName: 'bundle.mutableDataSnapshotRoot',
  });

  for (const requiredPath of [controlManifestPath, configSnapshotRoot, mutableDataSnapshotRoot]) {
    if (!exists(requiredPath)) {
      throw new Error(`installed runtime backup bundle is missing required path: ${requiredPath}`);
    }
  }

  const databaseDumpFile = String(manifest?.database?.dumpFile ?? '').trim();
  if (databaseDumpFile.length > 0) {
    const dumpPath = resolveBundleContractPath({
      bundleRoot,
      manifestPath,
      relativePath: databaseDumpFile,
      fieldName: 'database.dumpFile',
    });
    if (!exists(dumpPath)) {
      throw new Error(`installed runtime backup bundle is missing declared database dump: ${dumpPath}`);
    }
  }

  return {
    bundleRoot,
    manifestPath,
    controlManifestPath,
    configSnapshotRoot,
    mutableDataSnapshotRoot,
    manifest,
  };
}

function resolveBundleContractPath({
  bundleRoot,
  manifestPath,
  relativePath,
  fieldName,
} = {}) {
  const normalizedRelativePath = String(relativePath ?? '').trim();
  if (normalizedRelativePath.length === 0) {
    throw new Error(`installed runtime backup manifest is missing ${fieldName}: ${manifestPath}`);
  }
  if (path.isAbsolute(normalizedRelativePath)) {
    throw new Error(`installed runtime backup manifest ${fieldName} must be relative to the bundle root: ${manifestPath}`);
  }

  const resolvedPath = path.resolve(bundleRoot, normalizedRelativePath);
  const relativeToBundleRoot = path.relative(bundleRoot, resolvedPath);
  if (relativeToBundleRoot.startsWith('..') || path.isAbsolute(relativeToBundleRoot)) {
    throw new Error(`installed runtime backup manifest ${fieldName} must stay within the bundle root: ${manifestPath}`);
  }

  return resolvedPath;
}
