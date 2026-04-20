import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..', '..');

function normalizeRelativeEntry(relativeEntry) {
  if (Array.isArray(relativeEntry)) {
    return relativeEntry;
  }

  return [relativeEntry];
}

function defaultFileExists(filePath) {
  return fs.existsSync(filePath);
}

function defaultReadDir(directoryPath, options) {
  return fs.readdirSync(directoryPath, options);
}

function listWorkspaceAppRoots(appsRoot) {
  if (!defaultFileExists(appsRoot)) {
    return [];
  }

  let entries;
  try {
    entries = fs.readdirSync(appsRoot, { withFileTypes: true });
  } catch {
    return [];
  }

  return entries
    .filter((entry) => entry.isDirectory() && !entry.name.startsWith('.'))
    .map((entry) => path.join(appsRoot, entry.name))
    .filter((candidateRoot) => (
      defaultFileExists(path.join(candidateRoot, 'package.json'))
      && defaultFileExists(path.join(candidateRoot, 'node_modules'))
    ));
}

function defaultOpenFile(filePath) {
  return fs.openSync(filePath, 'r');
}

function defaultCloseFile(fileDescriptor) {
  fs.closeSync(fileDescriptor);
}

function defaultResolveFromRoot(root, specifier) {
  return createRequire(path.join(root, 'package.json')).resolve(specifier);
}

function defaultReadPackageJson(packageRoot) {
  return JSON.parse(fs.readFileSync(path.join(packageRoot, 'package.json'), 'utf8'));
}

function parseBarePackageSpecifier(specifier) {
  const normalizedSpecifier = String(specifier ?? '').trim();
  if (!normalizedSpecifier || normalizedSpecifier.startsWith('.') || normalizedSpecifier.startsWith('/')) {
    return null;
  }

  const segments = normalizedSpecifier.split('/');
  if (normalizedSpecifier.startsWith('@')) {
    if (segments.length < 2) {
      return null;
    }

    return {
      packageName: `${segments[0]}/${segments[1]}`,
      packageSubpath: segments.slice(2),
    };
  }

  return {
    packageName: segments[0],
    packageSubpath: segments.slice(1),
  };
}

function formatPackageExportKey(packageSubpath) {
  if (!Array.isArray(packageSubpath) || packageSubpath.length === 0) {
    return '.';
  }

  return `./${packageSubpath.join('/')}`;
}

function resolvePackageExportPatternTarget(exportsField, exportKey) {
  if (!exportsField || typeof exportsField !== 'object') {
    return '';
  }

  const patternEntries = Object.entries(exportsField)
    .filter(([candidateKey]) => (
      candidateKey.startsWith('./')
      && candidateKey.includes('*')
    ))
    .sort((left, right) => right[0].length - left[0].length);

  for (const [patternKey, patternTarget] of patternEntries) {
    const [patternPrefix, patternSuffix] = patternKey.split('*');
    if (!exportKey.startsWith(patternPrefix) || !exportKey.endsWith(patternSuffix)) {
      continue;
    }

    const wildcardValue = exportKey.slice(
      patternPrefix.length,
      exportKey.length - patternSuffix.length,
    );
    const resolvedTarget = selectImportEntryTarget(patternTarget);
    if (!resolvedTarget) {
      continue;
    }

    return resolvedTarget.replaceAll('*', wildcardValue);
  }

  return '';
}

function selectImportEntryTarget(candidate) {
  if (typeof candidate === 'string') {
    return candidate;
  }
  if (!candidate || typeof candidate !== 'object') {
    return '';
  }

  const prioritizedKeys = [
    'import',
    'browser',
    'development',
    'default',
    'module',
  ];

  for (const key of prioritizedKeys) {
    if (!Object.prototype.hasOwnProperty.call(candidate, key)) {
      continue;
    }

    const resolvedTarget = selectImportEntryTarget(candidate[key]);
    if (resolvedTarget) {
      return resolvedTarget;
    }
  }

  return '';
}

function resolvePackageImportEntry(packageJson, packageSubpath = []) {
  const exportsField = packageJson?.exports;
  const exportKey = formatPackageExportKey(packageSubpath);

  if (typeof exportsField === 'string') {
    if (exportKey !== '.') {
      return '';
    }

    return exportsField;
  }

  if (exportsField && typeof exportsField === 'object') {
    const hasExplicitSubpathKeys = Object.keys(exportsField).some((candidateKey) => (
      candidateKey === '.'
      || candidateKey.startsWith('./')
    ));
    const exportTarget = Object.prototype.hasOwnProperty.call(exportsField, exportKey)
      ? selectImportEntryTarget(exportsField[exportKey])
      : (
        exportKey === '.' && !hasExplicitSubpathKeys
          ? selectImportEntryTarget(exportsField)
          : resolvePackageExportPatternTarget(exportsField, exportKey)
      );
    if (exportTarget) {
      return exportTarget;
    }
  }

  if (packageSubpath.length > 0) {
    return '';
  }

  const moduleEntry = String(
    packageJson?.module
      ?? packageJson?.['jsnext:main']
      ?? packageJson?.main
      ?? '',
  ).trim();

  return moduleEntry;
}

export function probeReadableFile(
  filePath,
  {
    fileExists = defaultFileExists,
    openFile = defaultOpenFile,
    closeFile = defaultCloseFile,
  } = {},
) {
  if (!fileExists(filePath)) {
    return false;
  }

  try {
    const fileDescriptor = openFile(filePath);
    closeFile(fileDescriptor);
    return true;
  } catch {
    return false;
  }
}

function defaultIsReadable(filePath) {
  return probeReadableFile(filePath);
}

export function resolveWorkspaceDonorRoots(appRoot) {
  const normalizedAppRoot = path.resolve(appRoot);
  const knownWorkspaceApps = [
    ...listWorkspaceAppRoots(path.join(repoRoot, 'apps')),
    ...listWorkspaceAppRoots(path.resolve(repoRoot, '..')),
  ];

  return knownWorkspaceApps
    .map((candidateRoot) => path.resolve(candidateRoot))
    .filter((candidateRoot) => candidateRoot !== normalizedAppRoot);
}

export function resolveReadablePackageEntry({
  appRoot,
  donorRoots = [],
  packageName,
  relativeEntry,
  fileExists = defaultFileExists,
  readDir = defaultReadDir,
  isReadable = defaultIsReadable,
}) {
  const entrySegments = normalizeRelativeEntry(relativeEntry);
  const candidateRoots = [appRoot, ...donorRoots]
    .map((candidateRoot) => path.resolve(candidateRoot))
    .filter((candidateRoot, index, roots) => roots.indexOf(candidateRoot) === index);

  for (const candidateRoot of candidateRoots) {
    const directEntry = path.join(
      candidateRoot,
      'node_modules',
      packageName,
      ...entrySegments,
    );

    const candidateEntries = [directEntry];
    const pnpmRoot = path.join(candidateRoot, 'node_modules', '.pnpm');
    if (fileExists(pnpmRoot)) {
      const pnpmDirectoryPrefix = `${packageName.replace('/', '+')}@`;

      let pnpmEntries = [];
      try {
        pnpmEntries = readDir(pnpmRoot, { withFileTypes: true });
      } catch {
        pnpmEntries = [];
      }

      candidateEntries.push(...pnpmEntries
        .filter((entry) => entry.isDirectory() && entry.name.startsWith(pnpmDirectoryPrefix))
        .sort((left, right) => right.name.localeCompare(left.name))
        .map((entry) => path.join(
          pnpmRoot,
          entry.name,
          'node_modules',
          packageName,
          ...entrySegments,
        )));
    }

    for (const candidateEntry of candidateEntries) {
      if (fileExists(candidateEntry) && isReadable(candidateEntry)) {
        return candidateEntry;
      }
    }
  }

  throw new Error(
    `unable to resolve a readable ${packageName} entry (${entrySegments.join('/')}) from ${candidateRoots.join(', ')}`,
  );
}

export function resolveReadablePackageImportUrl(options) {
  return pathToFileURL(resolveReadablePackageEntry(options)).href;
}

export function findReadableModuleResolution({
  appRoot,
  donorRoots = [],
  specifier,
  resolveFromRoot = defaultResolveFromRoot,
  isReadable = defaultIsReadable,
}) {
  const candidateRoots = [appRoot, ...donorRoots]
    .map((candidateRoot) => path.resolve(candidateRoot))
    .filter((candidateRoot, index, roots) => roots.indexOf(candidateRoot) === index);

  for (const candidateRoot of candidateRoots) {
    let resolvedPath;
    try {
      resolvedPath = resolveFromRoot(candidateRoot, specifier);
    } catch {
      continue;
    }

    if (isReadable(resolvedPath)) {
      return {
        candidateRoot,
        resolvedPath,
      };
    }
  }

  throw new Error(
    `unable to resolve readable module specifier "${specifier}" from ${candidateRoots.join(', ')}`,
  );
}

export function resolveReadableModuleSpecifier(options) {
  return findReadableModuleResolution(options).resolvedPath;
}

export function resolveReadableFallbackModulePath({
  specifier,
  resolution,
  resolveReadablePackageRootImpl = resolveReadablePackageRoot,
  readPackageJsonImpl = defaultReadPackageJson,
} = {}) {
  if (!resolution?.candidateRoot || !resolution?.resolvedPath) {
    throw new Error('resolution.candidateRoot and resolution.resolvedPath are required');
  }

  const parsedSpecifier = parseBarePackageSpecifier(specifier);
  if (!parsedSpecifier) {
    return resolution.resolvedPath;
  }

  if (parsedSpecifier.packageSubpath.length > 0) {
    try {
      const subpathPackageRoot = resolveReadablePackageRootImpl({
        appRoot: resolution.candidateRoot,
        donorRoots: [],
        packageName: `${parsedSpecifier.packageName}/${parsedSpecifier.packageSubpath.join('/')}`,
      });
      const subpathPackageJson = readPackageJsonImpl(subpathPackageRoot);
      const subpathImportEntry = resolvePackageImportEntry(subpathPackageJson);

      if (subpathImportEntry) {
        return path.resolve(subpathPackageRoot, subpathImportEntry);
      }
    } catch {
      // Not every package subpath is a published package root. Fall back to root package metadata.
    }
  }

  const packageRoot = resolveReadablePackageRootImpl({
    appRoot: resolution.candidateRoot,
    donorRoots: [],
    packageName: parsedSpecifier.packageName,
  });
  const packageJson = readPackageJsonImpl(packageRoot);
  const importEntry = resolvePackageImportEntry(packageJson, parsedSpecifier.packageSubpath);

  if (!importEntry) {
    return resolution.resolvedPath;
  }

  return path.resolve(packageRoot, importEntry);
}

export function resolveReadablePackageRoot({
  relativeEntry,
  ...options
}) {
  return path.dirname(resolveReadablePackageEntry({
    ...options,
    relativeEntry: 'package.json',
  }));
}

export async function importReadablePackageDefault(options) {
  const moduleUrl = resolveReadablePackageImportUrl(options);
  const loadedModule = await import(moduleUrl);
  return loadedModule.default ?? loadedModule;
}

export async function applyWindowsVitePreload({
  platform = process.platform,
} = {}) {
  if (platform !== 'win32') {
    return;
  }

  await import(pathToFileURL(path.join(scriptDir, 'vite-windows-realpath-preload.mjs')).href);
}
