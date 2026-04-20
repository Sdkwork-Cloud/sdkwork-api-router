import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'release-governed-artifact-status.mjs'),
    ).href,
  );
}

test('release governed artifact status exposes strict governed path lookup helpers', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listGovernedReleaseArtifactRelativePaths, 'function');
  assert.equal(typeof module.findGovernedReleaseArtifactRelativePath, 'function');
  assert.equal(typeof module.listGovernedReleaseArtifactRelativePathsByPaths, 'function');
  assert.equal(typeof module.parseGitStatusEntryPath, 'function');
  assert.equal(typeof module.isGovernedReleaseArtifactStatusLine, 'function');
  assert.equal(typeof module.filterGovernedReleaseArtifactStatusLines, 'function');

  assert.deepEqual(
    module.listGovernedReleaseArtifactRelativePaths(),
    [
      'docs/release/release-window-snapshot-latest.json',
      'docs/release/release-sync-audit-latest.json',
      'docs/release/release-telemetry-export-latest.json',
      'docs/release/release-telemetry-snapshot-latest.json',
      'docs/release/slo-governance-latest.json',
      'docs/release/third-party-sbom-latest.spdx.json',
      'docs/release/third-party-notices-latest.json',
    ],
  );

  assert.equal(
    module.findGovernedReleaseArtifactRelativePath('docs/release/release-sync-audit-latest.json'),
    'docs/release/release-sync-audit-latest.json',
  );
  assert.deepEqual(
    module.listGovernedReleaseArtifactRelativePathsByPaths([
      'docs/release/release-window-snapshot-latest.json',
      'docs/release/slo-governance-latest.json',
      'docs/release/third-party-sbom-latest.spdx.json',
    ]),
    [
      'docs/release/release-window-snapshot-latest.json',
      'docs/release/slo-governance-latest.json',
      'docs/release/third-party-sbom-latest.spdx.json',
    ],
  );

  assert.equal(
    module.parseGitStatusEntryPath(' M docs\\release\\release-sync-audit-latest.json'),
    'docs/release/release-sync-audit-latest.json',
  );
  assert.equal(
    module.isGovernedReleaseArtifactStatusLine(' M docs/release/release-window-snapshot-latest.json'),
    true,
  );
  assert.equal(
    module.isGovernedReleaseArtifactStatusLine(' M docs/release/third-party-notices-latest.json'),
    true,
  );
  assert.equal(
    module.isGovernedReleaseArtifactStatusLine(' M docs/release/not-governed.json'),
    false,
  );
  assert.deepEqual(
    module.filterGovernedReleaseArtifactStatusLines([
      '## main...origin/main',
      ' M docs/release/release-window-snapshot-latest.json',
      ' M docs/release/third-party-sbom-latest.spdx.json',
      ' M docs/release/not-governed.json',
      '?? scripts/release/release-governed-artifact-status.mjs',
    ]),
    [
      '## main...origin/main',
      ' M docs/release/not-governed.json',
      '?? scripts/release/release-governed-artifact-status.mjs',
    ],
  );

  assert.throws(
    () => module.findGovernedReleaseArtifactRelativePath('docs/release/missing-governed-artifact.json'),
    /missing governed release artifact relative path.*missing-governed-artifact\.json/i,
  );
});
