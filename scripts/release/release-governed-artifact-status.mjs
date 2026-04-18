import path from 'node:path';

export const GOVERNED_RELEASE_ARTIFACT_RELATIVE_PATHS = Object.freeze([
  path.join('docs', 'release', 'release-window-snapshot-latest.json'),
  path.join('docs', 'release', 'release-sync-audit-latest.json'),
  path.join('docs', 'release', 'release-telemetry-export-latest.json'),
  path.join('docs', 'release', 'release-telemetry-snapshot-latest.json'),
  path.join('docs', 'release', 'slo-governance-latest.json'),
].map((relativePath) => relativePath.replaceAll('\\', '/')));

function normalizePortablePath(value = '') {
  return String(value ?? '').trim().replaceAll('\\', '/');
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
  return GOVERNED_RELEASE_ARTIFACT_RELATIVE_PATHS.includes(entryPath);
}

export function filterGovernedReleaseArtifactStatusLines(lines = []) {
  return lines.filter((line) => !isGovernedReleaseArtifactStatusLine(line));
}
