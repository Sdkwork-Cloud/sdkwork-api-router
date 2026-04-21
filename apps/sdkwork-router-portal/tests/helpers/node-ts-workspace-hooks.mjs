import fs from 'node:fs';
import path from 'node:path';
import { registerHooks, stripTypeScriptTypes } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';

import {
  resolvePortalAppRoot,
  resolvePortalAppbaseRoot,
} from './portal-paths.mjs';

const FILE_SUFFIXES = ['.ts', '.tsx', '.js', '.mjs', '.cjs'];
const INDEX_SUFFIXES = ['index.ts', 'index.tsx', 'index.js', 'index.mjs', 'index.cjs'];

function isResolvableLocalSpecifier(specifier) {
  return specifier.startsWith('./')
    || specifier.startsWith('../')
    || specifier.startsWith('/')
    || specifier.startsWith('file:')
    || /^[A-Za-z]:[\\/]/.test(specifier);
}

function resolveBasePath(specifier, parentURL) {
  if (specifier.startsWith('file:')) {
    return fileURLToPath(specifier);
  }

  if (path.isAbsolute(specifier)) {
    return specifier;
  }

  const parentPath = parentURL?.startsWith('file:')
    ? fileURLToPath(parentURL)
    : path.join(process.cwd(), 'index.js');

  return path.resolve(path.dirname(parentPath), specifier);
}

function createCandidatePaths(basePath) {
  if (path.extname(basePath)) {
    return [basePath];
  }

  return [
    ...FILE_SUFFIXES.map((suffix) => `${basePath}${suffix}`),
    ...INDEX_SUFFIXES.map((suffix) => path.join(basePath, suffix)),
  ];
}

function findFirstExistingPath(candidatePaths) {
  for (const candidatePath of candidatePaths) {
    if (fs.existsSync(candidatePath)) {
      return candidatePath;
    }
  }

  return null;
}

function resolveWorkspacePackageRootExportPath(packageRoot, packageJson) {
  const exportsField = packageJson?.exports;
  const rootExport = typeof exportsField === 'string'
    ? exportsField
    : exportsField && typeof exportsField === 'object' && !Array.isArray(exportsField)
      ? exportsField['.']
      : null;

  if (typeof rootExport === 'string') {
    return findFirstExistingPath(createCandidatePaths(path.resolve(packageRoot, rootExport)));
  }

  if (rootExport && typeof rootExport === 'object' && !Array.isArray(rootExport)) {
    for (const condition of ['node', 'import', 'default']) {
      if (typeof rootExport[condition] !== 'string') {
        continue;
      }

      const resolvedPath = findFirstExistingPath(
        createCandidatePaths(path.resolve(packageRoot, rootExport[condition])),
      );
      if (resolvedPath) {
        return resolvedPath;
      }
    }
  }

  return null;
}

function resolveWorkspacePackageAliasPath(specifier, appRoot) {
  if (specifier.includes('/') || specifier.startsWith('@')) {
    return null;
  }

  const packageRoot = path.join(appRoot, 'packages', specifier);
  const packageJsonPath = path.join(packageRoot, 'package.json');
  if (!fs.existsSync(packageJsonPath)) {
    return null;
  }

  try {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    return resolveWorkspacePackageRootExportPath(packageRoot, packageJson)
      ?? findFirstExistingPath(createCandidatePaths(path.join(packageRoot, 'src', 'index')));
  } catch {
    return findFirstExistingPath(createCandidatePaths(path.join(packageRoot, 'src', 'index')));
  }
}

function resolveAppbaseOverridePath(resolvedPath, appbaseRoot) {
  const normalizedPath = resolvedPath.replace(/\\/g, '/');
  const marker = '/sdkwork-appbase/';
  const markerIndex = normalizedPath.lastIndexOf(marker);
  if (markerIndex < 0) {
    return null;
  }

  const relativePath = normalizedPath.slice(markerIndex + marker.length);
  return path.join(appbaseRoot, ...relativePath.split('/'));
}

function shouldTransformTypeScriptModule(url) {
  if (!url.startsWith('file:')) {
    return false;
  }

  const resolvedPath = fileURLToPath(url);
  return resolvedPath.endsWith('.ts') || resolvedPath.endsWith('.tsx');
}

export function createPortalNodeTsWorkspaceHooks({
  appRoot = resolvePortalAppRoot(import.meta.url),
  appbaseRoot = resolvePortalAppbaseRoot(import.meta.url),
} = {}) {
  const resolvedAppRoot = path.resolve(appRoot);
  const resolvedAppbaseRoot = path.resolve(appbaseRoot);

  return {
    resolve(specifier, context, nextResolve) {
      const workspacePackageAliasPath = resolveWorkspacePackageAliasPath(
        specifier,
        resolvedAppRoot,
      );
      if (workspacePackageAliasPath) {
        return nextResolve(
          pathToFileURL(workspacePackageAliasPath).href,
          context,
          nextResolve,
        );
      }

      try {
        return nextResolve(specifier, context, nextResolve);
      } catch (error) {
        if (!isResolvableLocalSpecifier(specifier)) {
          throw error;
        }

        const basePath = resolveBasePath(specifier, context.parentURL);
        const resolvedLocalPath = findFirstExistingPath(createCandidatePaths(basePath));
        if (resolvedLocalPath) {
          return nextResolve(
            pathToFileURL(resolvedLocalPath).href,
            context,
            nextResolve,
          );
        }

        const appbaseOverridePath = resolveAppbaseOverridePath(basePath, resolvedAppbaseRoot);
        if (appbaseOverridePath) {
          const resolvedAppbasePath = findFirstExistingPath(
            createCandidatePaths(appbaseOverridePath),
          );
          if (resolvedAppbasePath) {
            return nextResolve(
              pathToFileURL(resolvedAppbasePath).href,
              context,
              nextResolve,
            );
          }
        }

        throw error;
      }
    },

    load(url, context, nextLoad) {
      if (!shouldTransformTypeScriptModule(url)) {
        return nextLoad(url, context, nextLoad);
      }

      const resolvedPath = fileURLToPath(url);
      const sourceText = fs.readFileSync(resolvedPath, 'utf8');

      return {
        format: 'module',
        shortCircuit: true,
        source: stripTypeScriptTypes(sourceText, {
          mode: 'transform',
        }),
      };
    },
  };
}

export function registerPortalNodeTsWorkspaceHooks(options = {}) {
  const hooks = createPortalNodeTsWorkspaceHooks(options);
  registerHooks(hooks);
  return hooks;
}
