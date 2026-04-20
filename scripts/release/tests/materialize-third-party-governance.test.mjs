import assert from 'node:assert/strict';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function writeFile(root, relativePath, contents) {
  const targetPath = path.join(root, relativePath);
  mkdirSync(path.dirname(targetPath), { recursive: true });
  writeFileSync(targetPath, contents, 'utf8');
  return targetPath;
}

function writeJson(root, relativePath, payload) {
  return writeFile(root, relativePath, `${JSON.stringify(payload, null, 2)}\n`);
}

function createFixtureWorkspace(root) {
  writeFile(
    root,
    'Cargo.lock',
    [
      'version = 4',
      '',
      '[[package]]',
      'name = "router-product-service"',
      'version = "0.1.0"',
      '',
      '[[package]]',
      'name = "serde"',
      'version = "1.0.218"',
      'source = "registry+https://github.com/rust-lang/crates.io-index"',
      '',
      '[[package]]',
      'name = "tokio"',
      'version = "1.44.0"',
      'source = "registry+https://github.com/rust-lang/crates.io-index"',
      '',
    ].join('\n'),
  );

  writeFile(
    root,
    'vendor/serde-1.0.218/Cargo.toml',
    [
      '[package]',
      'name = "serde"',
      'version = "1.0.218"',
      'license = "MIT OR Apache-2.0"',
      'repository = "https://github.com/serde-rs/serde"',
      '',
    ].join('\n'),
  );
  writeFile(root, 'vendor/serde-1.0.218/LICENSE-MIT', 'MIT license text\n');
  writeFile(
    root,
    'vendor/tokio-1.44.0/Cargo.toml',
    [
      '[package]',
      'name = "tokio"',
      'version = "1.44.0"',
      'license = "MIT"',
      'repository = "https://github.com/tokio-rs/tokio"',
      '',
    ].join('\n'),
  );
  writeFile(root, 'vendor/tokio-1.44.0/LICENSE', 'Tokio license text\n');

  writeJson(root, 'apps/sdkwork-router-portal/package.json', {
    name: 'sdkwork-router-portal',
    private: true,
  });
  writeJson(root, 'apps/sdkwork-router-portal/node_modules/react/package.json', {
    name: 'react',
    version: '19.2.4',
    license: 'MIT',
    homepage: 'https://react.dev/',
  });
  writeJson(root, 'apps/sdkwork-router-portal/node_modules/lucide-react/package.json', {
    name: 'lucide-react',
    version: '0.554.0',
    license: 'ISC',
    repository: {
      type: 'git',
      url: 'https://github.com/lucide-icons/lucide.git',
    },
  });
  writeJson(root, 'apps/sdkwork-router-portal/node_modules/@sdkwork/ui-pc-react/package.json', {
    name: '@sdkwork/ui-pc-react',
    version: '0.1.0',
    license: 'MIT',
  });
  writeFile(root, 'apps/sdkwork-router-portal/node_modules/react/LICENSE', 'React MIT\n');
}

test('third-party governance materializer emits SPDX SBOM and notice inventory documents', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-third-party-governance.mjs'),
    ).href,
  );

  assert.equal(typeof module.materializeThirdPartyGovernance, 'function');
  assert.equal(typeof module.validateThirdPartySbomArtifact, 'function');
  assert.equal(typeof module.validateThirdPartyNoticesArtifact, 'function');

  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-third-party-governance-'));
  createFixtureWorkspace(fixtureRoot);

  try {
    const result = module.materializeThirdPartyGovernance({
      repoRoot: fixtureRoot,
      generatedAt: '2026-04-20T08:00:00.000Z',
    });

    assert.equal(
      existsSync(path.join(fixtureRoot, 'docs', 'release', 'third-party-sbom-latest.spdx.json')),
      true,
    );
    assert.equal(
      existsSync(path.join(fixtureRoot, 'docs', 'release', 'third-party-notices-latest.json')),
      true,
    );
    assert.equal(result.packageCount, 4);

    const sbom = JSON.parse(
      readFileSync(path.join(fixtureRoot, 'docs', 'release', 'third-party-sbom-latest.spdx.json'), 'utf8'),
    );
    const notices = JSON.parse(
      readFileSync(path.join(fixtureRoot, 'docs', 'release', 'third-party-notices-latest.json'), 'utf8'),
    );

    module.validateThirdPartySbomArtifact(sbom);
    module.validateThirdPartyNoticesArtifact(notices);

    assert.equal(sbom.spdxVersion, 'SPDX-2.3');
    assert.equal(sbom.creationInfo.created, '2026-04-20T08:00:00.000Z');
    assert.ok(
      sbom.packages.some((entry) => entry.name === 'serde' && entry.licenseDeclared === 'MIT OR Apache-2.0'),
    );
    assert.ok(
      sbom.packages.some((entry) => entry.name === 'react' && entry.licenseDeclared === 'MIT'),
    );
    assert.ok(
      notices.packages.some((entry) => entry.name === 'lucide-react' && entry.licenseDeclared === 'ISC'),
    );
    assert.equal(
      notices.packages.some((entry) => entry.name === '@sdkwork/ui-pc-react'),
      false,
    );
    assert.match(notices.noticeText, /react 19\.2\.4/i);
    assert.match(notices.noticeText, /serde 1\.0\.218/i);
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
