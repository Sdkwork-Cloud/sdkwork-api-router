import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

export function createInstalledRuntimeSmokeLayout({
  installPlan,
  runtimeHome,
} = {}) {
  const productRoot = installPlan?.installRoot ?? runtimeHome;
  const controlHome = installPlan?.controlRoot ?? path.join(productRoot, 'current');

  return {
    productRoot,
    controlHome,
    configDir: path.join(productRoot, 'config'),
    logDir: path.join(productRoot, 'log'),
    runDir: path.join(productRoot, 'run'),
  };
}

export function resolveInstalledReleaseManifestPath(runtimeHome) {
  return path.join(runtimeHome, 'current', 'release-manifest.json');
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
  const bootstrapDataRoot = resolveInstalledBootstrapDataRoot(runtimeHome, {
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

export function assertInstalledRuntimeBackupBundle(backupRoot, {
  exists = existsSync,
  readFile = readFileSync,
} = {}) {
  const bundleRoot = path.resolve(backupRoot);
  const manifestPath = path.join(bundleRoot, 'backup-manifest.json');
  const controlManifestPath = path.join(bundleRoot, 'control', 'release-manifest.json');
  const configDir = path.join(bundleRoot, 'config');
  const dataDir = path.join(bundleRoot, 'data');

  for (const requiredPath of [manifestPath, controlManifestPath, configDir, dataDir]) {
    if (!exists(requiredPath)) {
      throw new Error(`installed runtime backup bundle is missing required path: ${requiredPath}`);
    }
  }

  let manifest;
  try {
    manifest = JSON.parse(String(readFile(manifestPath, 'utf8')));
  } catch (error) {
    throw new Error(
      `installed runtime backup manifest is not valid JSON: ${manifestPath}\n${error instanceof Error ? error.message : String(error)}`,
    );
  }

  if (String(manifest?.formatVersion ?? '') !== '1') {
    throw new Error(`installed runtime backup manifest has unsupported formatVersion at ${manifestPath}`);
  }

  const databaseDumpFile = String(manifest?.database?.dumpFile ?? '').trim();
  if (databaseDumpFile.length > 0) {
    const dumpPath = path.join(bundleRoot, databaseDumpFile);
    if (!exists(dumpPath)) {
      throw new Error(`installed runtime backup bundle is missing declared database dump: ${dumpPath}`);
    }
  }

  return {
    bundleRoot,
    manifestPath,
    controlManifestPath,
    manifest,
  };
}
