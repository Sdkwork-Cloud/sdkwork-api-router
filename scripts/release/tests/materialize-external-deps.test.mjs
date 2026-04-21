import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function createExternalReleaseSpec({
  id = 'sdkwork-core',
  repository = 'Sdkwork-Cloud/sdkwork-core',
  envRefKey = 'SDKWORK_CORE_GIT_REF',
  defaultRef = 'main',
  targetDir = path.join(repoRoot, '..', 'sdkwork-core'),
  expectedGitRoot = targetDir,
  cloneTargetDir = targetDir,
  requiredPaths = ['package.json'],
} = {}) {
  return {
    id,
    repository,
    envRefKey,
    defaultRef,
    targetDir,
    expectedGitRoot,
    cloneTargetDir,
    requiredPaths,
  };
}

function createExistingPathProbe(spec) {
  const readyPaths = new Set([
    path.resolve(spec.cloneTargetDir ?? spec.targetDir),
    path.resolve(spec.targetDir),
    ...spec.requiredPaths.map((relativePath) => path.resolve(spec.targetDir, relativePath)),
  ]);

  return function exists(candidatePath) {
    return readyPaths.has(path.resolve(candidatePath));
  };
}

function createGitAuditSpawn({
  cwd,
  topLevel,
  remoteUrl,
} = {}) {
  return function spawnSyncImpl(command, args, options = {}) {
    assert.match(String(command), /git(?:\.exe)?$/i);
    assert.equal(options.cwd, cwd);
    assert.equal(options.encoding, 'utf8');
    assert.equal(options.shell, false);

    const key = args.join('\u0000');
    if (key === 'rev-parse\u0000--show-toplevel') {
      return {
        status: 0,
        stdout: `${topLevel}\n`,
        stderr: '',
      };
    }

    if (key === 'remote\u0000get-url\u0000origin') {
      return {
        status: 0,
        stdout: `${remoteUrl}\n`,
        stderr: '',
      };
    }

    throw new Error(`unexpected git command: ${args.join(' ')}`);
  };
}

test('external release dependency catalogs expose strict governed lookup helpers', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );

  assert.equal(typeof module.findExternalReleaseDependencySpec, 'function');
  assert.equal(typeof module.listExternalReleaseDependencySpecsByIds, 'function');
  assert.equal(typeof module.findExternalReleaseDependencyScanRoot, 'function');
  assert.equal(typeof module.listExternalReleaseDependencyScanRootsByPaths, 'function');
  assert.equal(typeof module.listPackageJsonDependencyFields, 'function');
  assert.equal(typeof module.findPackageJsonDependencyField, 'function');
  assert.equal(typeof module.listPackageJsonDependencyFieldsByNames, 'function');
  assert.equal(typeof module.listExternalReleaseDependencyMaterializationScopes, 'function');
  assert.equal(typeof module.findExternalReleaseDependencyMaterializationScope, 'function');
  assert.equal(typeof module.listExternalReleaseDependencyMaterializationScopesByIds, 'function');

  assert.deepEqual(
    module.listExternalReleaseDependencySpecs().map(({ id }) => id),
    [
      'sdkwork-core',
      'sdkwork-ui',
      'sdkwork-appbase',
      'sdkwork-im-sdk',
    ],
  );
  assert.deepEqual(
    module.listExternalReleaseDependencySpecsByIds([
      'sdkwork-ui',
      'sdkwork-im-sdk',
    ]).map(({ id }) => id),
    [
      'sdkwork-ui',
      'sdkwork-im-sdk',
    ],
  );

  const sdkworkUi = module.findExternalReleaseDependencySpec('sdkwork-ui');
  sdkworkUi.requiredPaths.push('mutated-locally');
  assert.deepEqual(
    module.findExternalReleaseDependencySpec('sdkwork-ui').requiredPaths,
    ['sdkwork-ui-pc-react/package.json'],
  );

  const expectedScanRoots = [
    path.join(repoRoot, 'apps', 'sdkwork-router-admin'),
    path.join(repoRoot, 'apps', 'sdkwork-router-portal'),
  ];
  assert.deepEqual(
    module.listExternalReleaseDependencyScanRoots(),
    expectedScanRoots,
  );
  assert.equal(
    module.findExternalReleaseDependencyScanRoot(expectedScanRoots[1]),
    expectedScanRoots[1],
  );
  assert.deepEqual(
    module.listExternalReleaseDependencyScanRootsByPaths([
      expectedScanRoots[0],
    ]),
    [
      expectedScanRoots[0],
    ],
  );

  assert.deepEqual(
    module.listPackageJsonDependencyFields(),
    [
      'dependencies',
      'devDependencies',
      'optionalDependencies',
      'peerDependencies',
    ],
  );
  assert.equal(
    module.findPackageJsonDependencyField('peerDependencies'),
    'peerDependencies',
  );
  assert.deepEqual(
    module.listPackageJsonDependencyFieldsByNames([
      'dependencies',
      'peerDependencies',
    ]),
    [
      'dependencies',
      'peerDependencies',
    ],
  );

  assert.deepEqual(
    module.listExternalReleaseDependencyMaterializationScopes(),
    [
      'all',
      'referenced',
    ],
  );
  assert.equal(
    module.findExternalReleaseDependencyMaterializationScope('referenced'),
    'referenced',
  );
  assert.deepEqual(
    module.listExternalReleaseDependencyMaterializationScopesByIds([
      'all',
    ]),
    [
      'all',
    ],
  );

  assert.throws(
    () => module.findExternalReleaseDependencySpec('missing-external-release-dependency'),
    /missing external release dependency spec.*missing-external-release-dependency/i,
  );
  assert.throws(
    () => module.findExternalReleaseDependencyScanRoot(path.join(repoRoot, 'apps', 'missing-app')),
    /missing external release dependency scan root.*missing-app/i,
  );
  assert.throws(
    () => module.findPackageJsonDependencyField('bundledDependencies'),
    /missing package\.json dependency field.*bundledDependencies/i,
  );
  assert.throws(
    () => module.findExternalReleaseDependencyMaterializationScope('incremental'),
    /missing external release dependency materialization scope.*incremental/i,
  );
});

test('external release dependency materializer reuses a governed standalone checkout', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );

  const spec = createExternalReleaseSpec();
  const result = module.materializeExternalReleaseDependency({
    spec,
    env: {},
    exists: createExistingPathProbe(spec),
    spawnSyncImpl: createGitAuditSpawn({
      cwd: spec.targetDir,
      topLevel: spec.targetDir,
      remoteUrl: 'git@github.com:Sdkwork-Cloud/sdkwork-core.git',
    }),
  });

  assert.deepEqual(result, {
    id: 'sdkwork-core',
    repository: 'Sdkwork-Cloud/sdkwork-core',
    ref: 'main',
    status: 'ready',
    skipped: true,
  });
});

test('external release dependency materializer rejects an occupied target that is not the governed standalone repository', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );

  const spec = createExternalReleaseSpec();

  assert.throws(
    () => module.materializeExternalReleaseDependency({
      spec,
      env: {},
      exists: createExistingPathProbe(spec),
      spawnSyncImpl: createGitAuditSpawn({
        cwd: spec.targetDir,
        topLevel: path.resolve(spec.targetDir, '..', '..'),
        remoteUrl: 'git@github.com:Sdkwork-Cloud/spring-ai-plus2.git',
      }),
    }),
    /not-standalone-root[\s\S]*remote-url-mismatch/i,
  );
});

test('external release dependency materializer accepts the governed sdkwork-im-sdk repository nested inside the craw-chat workspace', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );

  const spec = createExternalReleaseSpec({
    id: 'sdkwork-im-sdk',
    repository: 'Sdkwork-Cloud/sdkwork-im-sdk',
    envRefKey: 'SDKWORK_IM_SDK_GIT_REF',
    targetDir: path.join(
      repoRoot,
      '..',
      'craw-chat',
      'sdks',
      'sdkwork-im-sdk',
    ),
    expectedGitRoot: path.join(repoRoot, '..', 'craw-chat', 'sdks', 'sdkwork-im-sdk'),
    cloneTargetDir: path.join(repoRoot, '..', 'craw-chat', 'sdks', 'sdkwork-im-sdk'),
    requiredPaths: ['sdkwork-im-sdk-typescript/package.json'],
  });

  const result = module.materializeExternalReleaseDependency({
    spec,
    env: {},
    exists: createExistingPathProbe(spec),
    spawnSyncImpl: createGitAuditSpawn({
      cwd: spec.targetDir,
      topLevel: spec.expectedGitRoot,
      remoteUrl: 'git@github.com:Sdkwork-Cloud/sdkwork-im-sdk.git',
    }),
  });

  assert.deepEqual(result, {
    id: 'sdkwork-im-sdk',
    repository: 'Sdkwork-Cloud/sdkwork-im-sdk',
    ref: 'main',
    status: 'ready',
    skipped: true,
  });
});

test('external release dependency specs can be remapped into a dedicated governed release root', async () => {
  const module = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'materialize-external-deps.mjs'),
    ).href,
  );

  assert.equal(typeof module.resolveExternalReleaseDependencySpecs, 'function');

  const governedRoot = path.join(repoRoot, 'artifacts', 'release-governance', 'external-deps');
  const specs = module.resolveExternalReleaseDependencySpecs({
    env: {
      SDKWORK_RELEASE_EXTERNAL_DEPENDENCY_ROOT: governedRoot,
    },
  });

  const sdkworkCore = specs.find((spec) => spec.id === 'sdkwork-core');
  assert.ok(sdkworkCore);
  assert.equal(
    sdkworkCore.targetDir,
    path.join(governedRoot, 'sdkwork-core'),
  );

  const sdkworkUi = specs.find((spec) => spec.id === 'sdkwork-ui');
  assert.ok(sdkworkUi);
  assert.equal(
    sdkworkUi.targetDir,
    path.join(governedRoot, 'sdkwork-ui'),
  );

  const sdkworkImSdk = specs.find((spec) => spec.id === 'sdkwork-im-sdk');
  assert.ok(sdkworkImSdk);
  assert.equal(
    sdkworkImSdk.cloneTargetDir,
    path.join(governedRoot, 'craw-chat', 'sdks', 'sdkwork-im-sdk'),
  );
  assert.equal(
    sdkworkImSdk.expectedGitRoot,
    path.join(governedRoot, 'craw-chat', 'sdks', 'sdkwork-im-sdk'),
  );
  assert.equal(
    sdkworkImSdk.targetDir,
    path.join(
      governedRoot,
      'craw-chat',
      'sdks',
      'sdkwork-im-sdk',
    ),
  );
});
