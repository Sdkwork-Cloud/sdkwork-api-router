import path from 'node:path';

import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';

function normalizePortablePath(value = '') {
  return String(value ?? '').trim().replaceAll('\\', '/');
}

const governedReleaseArtifactRelativePathCatalog = createStrictKeyedCatalog({
  entries: [
  path.join('docs', 'release', 'release-window-snapshot-latest.json'),
  path.join('docs', 'release', 'release-sync-audit-latest.json'),
  path.join('docs', 'release', 'release-telemetry-export-latest.json'),
  path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
  path.join('docs', 'release', 'slo-governance-latest.json'),
  path.join('docs', 'release', 'third-party-sbom-latest.spdx.json'),
  path.join('docs', 'release', 'third-party-notices-latest.json'),
  ].map((relativePath) => normalizePortablePath(relativePath)),
  getKey: (relativePath) => relativePath,
  duplicateKeyMessagePrefix: 'duplicate governed release artifact relative path',
  missingKeyMessagePrefix: 'missing governed release artifact relative path',
});

export const GOVERNED_RELEASE_ARTIFACT_RELATIVE_PATHS = Object.freeze(
  governedReleaseArtifactRelativePathCatalog.list(),
);

const governedReleaseArtifactRelativePathSet = new Set(
  GOVERNED_RELEASE_ARTIFACT_RELATIVE_PATHS,
);

export function listGovernedReleaseArtifactRelativePaths() {
  return governedReleaseArtifactRelativePathCatalog.list();
}

export function findGovernedReleaseArtifactRelativePath(relativePath) {
  return governedReleaseArtifactRelativePathCatalog.find(relativePath);
}

export function listGovernedReleaseArtifactRelativePathsByPaths(relativePaths = []) {
  return governedReleaseArtifactRelativePathCatalog.listByKeys(relativePaths);
}

export function parseGitStatusEntryPath(line = '') {
  const trimmedLine = String(line ?? '').trimEnd();
  if (trimmedLine.length === 0 || trimmedLine.startsWith('## ')) {
    return '';
  }

  let entryPath = trimmedLine.length > 3
    ? trimmedLine.slice(3).trim()
    : trimmedLine.trim();
  if (entryPath.includes(' -> ')) {
    entryPath = entryPath.split(' -> ').at(-1) ?? '';
  }

  return normalizePortablePath(entryPath);
}

export function isGovernedReleaseArtifactStatusLine(line = '') {
  const entryPath = parseGitStatusEntryPath(line);
  return governedReleaseArtifactRelativePathSet.has(entryPath);
}

export function filterGovernedReleaseArtifactStatusLines(lines = []) {
  return lines.filter((line) => !isGovernedReleaseArtifactStatusLine(line));
}
