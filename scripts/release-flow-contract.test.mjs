import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const rootDir = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(rootDir, relativePath), 'utf8');
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function assertImporterLockEntry(lockfile, packageName, specifier, versionPattern) {
  const packageNamePattern = `(?:'${escapeRegExp(packageName)}'|${escapeRegExp(packageName)})`;
  assert.match(
    lockfile,
    new RegExp(
      `${packageNamePattern}:\\r?\\n\\s+specifier: ${escapeRegExp(specifier)}\\r?\\n\\s+version: ${versionPattern}`,
    ),
  );
}

function extractTarArchive({ archivePath, outputDir, packager }) {
  const tarFlavor = packager.detectTarFlavor({
    platform: process.platform,
    spawn: spawnSync,
  });
  const args = [];
  if (process.platform === 'win32' && tarFlavor === 'gnu') {
    args.push('--force-local');
  }
  args.push('-xzf', archivePath, '-C', outputDir);

  const result = spawnSync('tar', args, {
    cwd: rootDir,
    shell: process.platform === 'win32',
    encoding: 'utf8',
  });
  if (result.error) {
    assert.fail(`tar extract failed while reading ${archivePath}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    assert.fail(
      `tar extract exited ${result.status ?? 'unknown'} for ${archivePath}\nstdout: ${result.stdout ?? ''}\nstderr: ${result.stderr ?? ''}`,
    );
  }
}

test('repository exposes a native platform and architecture release workflow', () => {
  const workflowPath = path.join(rootDir, '.github', 'workflows', 'release.yml');
  assert.equal(existsSync(workflowPath), true, 'missing .github/workflows/release.yml');

  const workflow = read('.github/workflows/release.yml');

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /push:\s*[\s\S]*tags:\s*[\s\S]*release-\*/);
  assert.match(workflow, /windows-2022/);
  assert.match(workflow, /windows-11-arm/);
  assert.match(workflow, /ubuntu-22\.04/);
  assert.match(workflow, /macos-14/);
  assert.match(workflow, /arch:\s*x64/);
  assert.match(workflow, /arch:\s*arm64/);
  assert.match(workflow, /target:\s*x86_64-pc-windows-msvc/);
  assert.match(workflow, /target:\s*aarch64-pc-windows-msvc/);
  assert.match(workflow, /target:\s*x86_64-unknown-linux-gnu/);
  assert.match(workflow, /target:\s*aarch64-unknown-linux-gnu/);
  assert.match(workflow, /target:\s*x86_64-apple-darwin/);
  assert.match(workflow, /target:\s*aarch64-apple-darwin/);
  assert.match(workflow, /ubuntu-24\.04-arm/);
  assert.match(workflow, /cargo build --release --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /router-product-service/);
  assert.match(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app portal --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /node scripts\/release\/package-release-assets\.mjs native --platform \$\{\{ matrix\.platform \}\} --arch \$\{\{ matrix\.arch \}\} --target \$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /release-assets-native-\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}/);
  assert.doesNotMatch(workflow, /node scripts\/release\/run-desktop-release-build\.mjs --app admin --target \$\{\{ matrix\.target \}\}/);
  assert.doesNotMatch(workflow, /node scripts\/release\/package-release-assets\.mjs web/);
  assert.doesNotMatch(workflow, /release-assets-web/);
  assert.doesNotMatch(workflow, /web-release:/);
  assert.match(workflow, /softprops\/action-gh-release@/);
});

test('tauri package scripts stay portable across admin, portal, and console apps', () => {
  const adminPackage = JSON.parse(read('apps/sdkwork-router-admin/package.json'));
  const portalPackage = JSON.parse(read('apps/sdkwork-router-portal/package.json'));
  const consolePackage = JSON.parse(read('console/package.json'));
  const consoleTauriCargo = read('console/src-tauri/Cargo.toml');

  assert.match(adminPackage.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(adminPackage.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(portalPackage.scripts['tauri:dev'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(portalPackage.scripts['tauri:build'], /node \.\.\/\.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(consolePackage.scripts['tauri:dev'], /node \.\.\/scripts\/run-tauri-cli\.mjs dev/);
  assert.match(consolePackage.scripts['tauri:build'], /node \.\.\/scripts\/run-tauri-cli\.mjs build/);
  assert.match(consoleTauriCargo, /^\[workspace\]$/m);
  assert.doesNotMatch(portalPackage.scripts['tauri:dev'], /powershell/i);
  assert.doesNotMatch(portalPackage.scripts['tauri:build'], /powershell/i);
});

test('web workspace scripts stay portable across Windows, native Unix, and WSL-mounted worktrees', () => {
  const adminPackage = JSON.parse(read('apps/sdkwork-router-admin/package.json'));
  const portalPackage = JSON.parse(read('apps/sdkwork-router-portal/package.json'));
  const consolePackage = JSON.parse(read('console/package.json'));
  const pnpmLaunchLib = read('scripts/dev/pnpm-launch-lib.mjs');
  const runTscCli = read('scripts/dev/run-tsc-cli.mjs');

  assert.match(adminPackage.scripts.dev, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(adminPackage.scripts.build, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(adminPackage.scripts.typecheck, /^node \.\.\/\.\.\/scripts\/dev\/run-tsc-cli\.mjs(?:\s|$)/);
  assert.match(adminPackage.scripts.preview, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(portalPackage.scripts.dev, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(portalPackage.scripts.build, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(portalPackage.scripts.typecheck, /^tsc(?:\s|$)/);
  assert.match(portalPackage.scripts.preview, /^node \.\.\/\.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(consolePackage.scripts.dev, /^node \.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(consolePackage.scripts.build, /^node \.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.match(consolePackage.scripts.preview, /^node \.\.\/scripts\/dev\/run-vite-cli\.mjs(?:\s|$)/);
  assert.doesNotMatch(adminPackage.scripts.dev, /run-frontend-tool/);
  assert.doesNotMatch(adminPackage.scripts.build, /run-frontend-tool/);
  assert.doesNotMatch(adminPackage.scripts.typecheck, /run-frontend-tool/);
  assert.doesNotMatch(adminPackage.scripts.preview, /run-frontend-tool/);
  assert.doesNotMatch(portalPackage.scripts.dev, /run-frontend-tool/);
  assert.doesNotMatch(portalPackage.scripts.build, /run-frontend-tool/);
  assert.doesNotMatch(portalPackage.scripts.typecheck, /run-frontend-tool/);
  assert.doesNotMatch(portalPackage.scripts.preview, /run-frontend-tool/);
  assert.doesNotMatch(consolePackage.scripts.dev, /run-frontend-tool/);
  assert.doesNotMatch(consolePackage.scripts.build, /run-frontend-tool/);
  assert.doesNotMatch(consolePackage.scripts.preview, /run-frontend-tool/);

  assert.match(pnpmLaunchLib, /requiredBinCommands/);
  assert.match(pnpmLaunchLib, /node_modules', '\.bin'/);
  assert.match(pnpmLaunchLib, /platform === 'win32'/);
  assert.match(pnpmLaunchLib, /\.cmd/);
  assert.match(runTscCli, /resolveReadablePackageEntry/);
  assert.match(runTscCli, /typescript/);
  assert.match(runTscCli, /lib', 'tsc\.js'/);
});

test('console lockfile stays aligned with the committed frontend toolchain versions', () => {
  const consolePackage = JSON.parse(read('console/package.json'));
  const consoleLockfile = read('console/pnpm-lock.yaml');
  const reactVersion = consolePackage.dependencies.react;
  const reactDomVersion = consolePackage.dependencies['react-dom'];
  const viteVersion = consolePackage.devDependencies.vite;
  const pluginReactVersion = consolePackage.devDependencies['@vitejs/plugin-react'];
  const typescriptVersion = consolePackage.devDependencies.typescript;

  assertImporterLockEntry(
    consoleLockfile,
    'react',
    reactVersion,
    escapeRegExp(reactVersion),
  );
  assertImporterLockEntry(
    consoleLockfile,
    'react-dom',
    reactDomVersion,
    `${escapeRegExp(reactDomVersion)}\\(react@${escapeRegExp(reactVersion)}\\)`,
  );
  assertImporterLockEntry(
    consoleLockfile,
    '@vitejs/plugin-react',
    pluginReactVersion,
    `${escapeRegExp(pluginReactVersion)}\\(vite@${escapeRegExp(viteVersion)}\\(lightningcss@`,
  );
  assertImporterLockEntry(
    consoleLockfile,
    'typescript',
    typescriptVersion,
    escapeRegExp(typescriptVersion),
  );
  assertImporterLockEntry(
    consoleLockfile,
    'vite',
    viteVersion,
    `${escapeRegExp(viteVersion)}\\(lightningcss@`,
  );
});

test('desktop tauri configs enable bundling for native release packaging', () => {
  const adminTauriConfig = JSON.parse(read('apps/sdkwork-router-admin/src-tauri/tauri.conf.json'));
  const portalTauriConfig = JSON.parse(read('apps/sdkwork-router-portal/src-tauri/tauri.conf.json'));

  assert.equal(adminTauriConfig.bundle?.active, true);
  assert.equal(portalTauriConfig.bundle?.active, true);
  assert.deepEqual(
    adminTauriConfig.bundle?.icon,
    [
      'icons/32x32.png',
      'icons/128x128.png',
      'icons/128x128@2x.png',
      'icons/icon.icns',
      'icons/icon.ico',
    ],
  );
  assert.deepEqual(
    portalTauriConfig.bundle?.icon,
    [
      'icons/32x32.png',
      'icons/128x128.png',
      'icons/128x128@2x.png',
      'icons/icon.icns',
      'icons/icon.ico',
    ],
  );
  assert.deepEqual(
    adminTauriConfig.bundle?.resources,
    {
      '../dist/': 'embedded-sites/admin/',
      '../../sdkwork-router-portal/dist/': 'embedded-sites/portal/',
    },
  );
  assert.deepEqual(
    portalTauriConfig.bundle?.resources,
    {
      '../../../bin/portal-rt/router-product/': 'router-product/',
    },
  );
  assert.equal(
    portalTauriConfig.build?.beforeBuildCommand,
    'node ../../scripts/prepare-router-portal-desktop-runtime.mjs',
  );
  assert.doesNotMatch(
    read('scripts/release/package-release-assets.mjs'),
    /sdkwork-api-router-web-assets|package-release-assets\.mjs web|buildWebArchiveBaseName|packageWebAssets/,
  );
  assert.doesNotMatch(
    read('scripts/release/package-release-assets.mjs'),
    /sdkwork-api-console-tauri|console:\s*path\.join\(rootDir,\s*'console'\)/,
  );
  assert.doesNotMatch(
    read('scripts/release/package-release-assets.mjs'),
    /packageServiceBinaries\(|native', platformId, archId, 'services'/,
  );
});

test('release target helpers and desktop release runner resolve explicit target triples', async () => {
  const helperPath = path.join(rootDir, 'scripts', 'release', 'desktop-targets.mjs');
  const runnerPath = path.join(rootDir, 'scripts', 'release', 'run-desktop-release-build.mjs');
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const tauriRunnerPath = path.join(rootDir, 'scripts', 'run-tauri-cli.mjs');
  const workspaceTargetDirPath = path.join(rootDir, 'scripts', 'workspace-target-dir.mjs');

  assert.equal(existsSync(helperPath), true, 'missing scripts/release/desktop-targets.mjs');
  assert.equal(existsSync(runnerPath), true, 'missing scripts/release/run-desktop-release-build.mjs');
  assert.equal(existsSync(packagerPath), true, 'missing scripts/release/package-release-assets.mjs');

  const helper = await import(pathToFileURL(helperPath).href);
  const runner = await import(pathToFileURL(runnerPath).href);
  const packager = await import(pathToFileURL(packagerPath).href);
  const tauriRunner = await import(pathToFileURL(tauriRunnerPath).href);
  const workspaceTargetDir = await import(pathToFileURL(workspaceTargetDirPath).href);
  const expectedManagedWindowsTauriTargetDir = tauriRunner.resolveManagedWindowsTauriTargetDir({
    cwd: path.join(rootDir, 'apps', 'sdkwork-router-admin'),
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
    platform: 'win32',
  });

  assert.equal(typeof helper.parseDesktopTargetTriple, 'function');
  assert.equal(typeof helper.resolveDesktopReleaseTarget, 'function');
  assert.equal(typeof runner.createDesktopReleaseBuildPlan, 'function');
  assert.equal(typeof runner.buildDesktopReleaseFailureAnnotation, 'function');
  assert.equal(typeof runner.resolveDesktopAppDir, 'function');
  assert.equal(typeof runner.resolveDesktopReleaseBundles, 'function');
  assert.equal(typeof tauriRunner.resolveManagedWindowsTauriTargetDir, 'function');
  assert.equal(typeof runner.shouldPassExplicitDesktopReleaseTarget, 'function');
  assert.equal(typeof packager.resolveNativeBuildRoot, 'function');
  assert.equal(typeof packager.resolveNativeBuildRootCandidates, 'function');
  assert.equal(typeof packager.listNativeServiceBinaryNames, 'function');
  assert.equal(typeof packager.listNativeDesktopAppIds, 'function');
  assert.equal(typeof packager.listNativeProductServerBootstrapDataRoots, 'function');
  assert.equal(typeof packager.buildNativePortalDesktopArtifactBaseName, 'function');
  assert.equal(typeof packager.createNativePortalDesktopReleaseAssetSpec, 'function');
  assert.equal(typeof packager.buildNativeProductServerArchiveBaseName, 'function');
  assert.equal(typeof packager.createNativeProductServerReleaseAssetSpec, 'function');
  assert.equal(typeof packager.createTarCommandPlan, 'function');
  assert.equal(typeof workspaceTargetDir.resolveWorkspaceTargetDir, 'function');

  const expectedWorkspaceTargetDir = workspaceTargetDir.resolveWorkspaceTargetDir({
    workspaceRoot: rootDir,
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
    platform: 'win32',
  });

  assert.deepEqual(
    helper.parseDesktopTargetTriple('aarch64-pc-windows-msvc'),
    {
      platform: 'windows',
      arch: 'arm64',
      targetTriple: 'aarch64-pc-windows-msvc',
    },
  );
  assert.deepEqual(
    helper.resolveDesktopReleaseTarget({
      env: { SDKWORK_DESKTOP_TARGET: 'x86_64-apple-darwin' },
    }),
    {
      platform: 'macos',
      arch: 'x64',
      targetTriple: 'x86_64-apple-darwin',
    },
  );
  assert.throws(
    () => runner.resolveDesktopAppDir('console'),
    /Unsupported desktop application id: console/,
  );
  assert.throws(
    () => packager.resolveNativeBuildRoot({ appId: 'console' }),
    /Unsupported desktop application id: console/,
  );

  const portalBuildPlan = runner.createDesktopReleaseBuildPlan({
    appId: 'portal',
    appDir: path.join(rootDir, 'apps', 'sdkwork-router-portal'),
    platform: 'win32',
    arch: 'x64',
    env: {},
    targetTriple: 'aarch64-pc-windows-msvc',
  });

  assert.equal(portalBuildPlan.command, 'pnpm');
  assert.deepEqual(portalBuildPlan.args, [
    'tauri:build',
    '--target',
    'aarch64-pc-windows-msvc',
    '--bundles',
    'nsis',
  ]);
  assert.equal(portalBuildPlan.env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET, 'aarch64-pc-windows-msvc');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET_PLATFORM, 'windows');
  assert.equal(portalBuildPlan.env.SDKWORK_DESKTOP_TARGET_ARCH, 'arm64');
  assert.deepEqual(
    runner.resolveDesktopReleaseBundles({
      platform: 'darwin',
    }),
    ['dmg'],
  );
  assert.equal(
    runner.shouldPassExplicitDesktopReleaseTarget({
      targetTriple: 'x86_64-unknown-linux-gnu',
      platform: 'linux',
      arch: 'x64',
    }),
    false,
  );
  assert.deepEqual(
    runner.createDesktopReleaseBuildPlan({
      appId: 'admin',
      appDir: path.join(rootDir, 'apps', 'sdkwork-router-admin'),
      platform: 'linux',
      arch: 'x64',
      env: {
        GITHUB_ACTIONS: 'true',
      },
      targetTriple: 'x86_64-unknown-linux-gnu',
    }).args,
    [
      'tauri:build',
      '--bundles',
      'deb',
      '--verbose',
    ],
  );

  assert.equal(
    packager.resolveNativeBuildRoot({
      appId: 'admin',
      targetTriple: 'x86_64-pc-windows-msvc',
    }).replaceAll('\\', '/'),
    path.join(
      rootDir,
      'apps',
      'sdkwork-router-admin',
      'src-tauri',
      'target',
      'x86_64-pc-windows-msvc',
      'release',
      'bundle',
    ).replaceAll('\\', '/'),
  );
  assert.deepEqual(
    packager.resolveNativeBuildRootCandidates({
      appId: 'admin',
      targetTriple: 'x86_64-pc-windows-msvc',
      env: {
        USERPROFILE: 'C:/Users/admin',
        TEMP: 'C:/Temp',
      },
      platform: 'win32',
    }).map((entry) => entry.replaceAll('\\', '/')),
    [...new Set([
      path.join(
        rootDir,
        'apps',
        'sdkwork-router-admin',
        'target',
        'x86_64-pc-windows-msvc',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        rootDir,
        'apps',
        'sdkwork-router-admin',
        'target',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        expectedWorkspaceTargetDir,
        'x86_64-pc-windows-msvc',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        expectedWorkspaceTargetDir,
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        expectedManagedWindowsTauriTargetDir,
        'x86_64-pc-windows-msvc',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        expectedManagedWindowsTauriTargetDir,
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        rootDir,
        'target',
        'sdkwork-router-admin-tauri',
        'x86_64-pc-windows-msvc',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        rootDir,
        'target',
        'sdkwork-router-admin-tauri',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        rootDir,
        'apps',
        'sdkwork-router-admin',
        'src-tauri',
        'target',
        'x86_64-pc-windows-msvc',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
      path.join(
        rootDir,
        'apps',
        'sdkwork-router-admin',
        'src-tauri',
        'target',
        'release',
        'bundle',
      ).replaceAll('\\', '/'),
    ])],
  );
  assert.match(
    packager.listNativeServiceBinaryNames().join(','),
    /router-product-service/,
  );
  assert.deepEqual(packager.listNativeDesktopAppIds(), ['portal']);
  assert.deepEqual(
    packager.listNativeProductServerBootstrapDataRoots(),
    {
      data: path.join(rootDir, 'data'),
    },
  );
  assert.equal(
    packager.buildNativePortalDesktopArtifactBaseName({
      platformId: 'macos',
      archId: 'arm64',
    }),
    'sdkwork-router-portal-desktop-macos-arm64',
  );
  assert.deepEqual(
    packager.createNativePortalDesktopReleaseAssetSpec({
      platformId: 'macos',
      archId: 'arm64',
    }),
    {
      appId: 'portal',
      artifactKind: 'dmg',
      fileName: 'sdkwork-router-portal-desktop-macos-arm64.dmg',
      checksumFileName: 'sdkwork-router-portal-desktop-macos-arm64.dmg.sha256.txt',
      manifestFileName: 'sdkwork-router-portal-desktop-macos-arm64.manifest.json',
      expectedBundleDirectory: 'dmg',
      expectedFileSuffix: '.dmg',
    },
  );
  assert.equal(
    packager.buildNativeProductServerArchiveBaseName({
      platformId: 'linux',
      archId: 'arm64',
    }),
    'sdkwork-api-router-product-server-linux-arm64',
  );
  assert.deepEqual(
    packager.createNativeProductServerReleaseAssetSpec({
      platformId: 'linux',
      archId: 'arm64',
    }),
    {
      productId: 'sdkwork-api-router-product-server',
      fileName: 'sdkwork-api-router-product-server-linux-arm64.tar.gz',
      checksumFileName: 'sdkwork-api-router-product-server-linux-arm64.tar.gz.sha256.txt',
      manifestFileName: 'sdkwork-api-router-product-server-linux-arm64.manifest.json',
    },
  );
  assert.deepEqual(
    packager.createTarCommandPlan({
      archivePath: 'C:\\release\\bundle.tar.gz',
      workingDirectory: 'C:\\release',
      entryName: 'bundle',
      platform: 'win32',
      tarFlavor: 'bsd',
    }),
    {
      command: 'tar',
      args: ['-czf', 'C:\\release\\bundle.tar.gz', '-C', 'C:\\release', 'bundle'],
      shell: true,
    },
  );
  assert.deepEqual(
    packager.createTarCommandPlan({
      archivePath: 'C:\\release\\bundle.tar.gz',
      workingDirectory: 'C:\\release',
      entryName: 'bundle',
      platform: 'win32',
      tarFlavor: 'gnu',
    }),
    {
      command: 'tar',
      args: ['--force-local', '-czf', 'C:\\release\\bundle.tar.gz', '-C', 'C:\\release', 'bundle'],
      shell: true,
    },
  );
  assert.equal(
    packager.shouldIncludeDesktopBundleFile('windows', 'nsis/sdkwork-router-portal.exe'),
    true,
  );
  assert.equal(
    packager.shouldIncludeDesktopBundleFile('windows', 'msi/sdkwork-router-portal.msi'),
    false,
  );
  assert.equal(
    packager.shouldIncludeDesktopBundleFile('linux', 'deb/sdkwork-router-portal_0.1.0_amd64.deb'),
    true,
  );
  assert.equal(
    packager.shouldIncludeDesktopBundleFile('macos', 'dmg/sdkwork-router-portal.dmg'),
    true,
  );
  assert.equal(
    packager.shouldIncludeDesktopBundleFile('macos', 'macos/sdkwork-router-portal.app'),
    false,
  );
  assert.equal(
    runner.buildDesktopReleaseFailureAnnotation({
      appId: 'admin',
      targetTriple: 'x86_64-unknown-linux-gnu',
      error: new Error('bundle missing 50%\nnext line'),
    }),
    '::error title=run-desktop-release-build::[admin x86_64-unknown-linux-gnu] bundle missing 50%25%0Anext line',
  );
  assert.match(
    runner.buildDesktopReleaseFailureAnnotation({
      appId: 'admin',
      targetTriple: 'x86_64-unknown-linux-gnu',
      error: new Error(`${'prefix-'.repeat(2000)}actual-final-error`),
    }),
    /actual-final-error/,
  );
});

test('native desktop packager skips empty bundle roots and selects the first root that contains artifacts', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.resolveAvailableNativeBuildRoot, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const emptyRoot = path.join(stagingRoot, 'candidate-empty');
    const populatedRoot = path.join(stagingRoot, 'candidate-populated');
    mkdirSync(emptyRoot, { recursive: true });
    mkdirSync(path.join(populatedRoot, 'nsis'), { recursive: true });
    writeFileSync(path.join(populatedRoot, 'nsis', 'sdkwork-router-portal.exe'), 'artifact', 'utf8');

    assert.equal(
      packager.resolveAvailableNativeBuildRoot({
        buildRoots: [emptyRoot, populatedRoot],
      }).replaceAll('\\', '/'),
      populatedRoot.replaceAll('\\', '/'),
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('tauri build wrapper follows the managed workspace target policy on Windows and preserves upstream cargo target dirs', async () => {
  const tauriRunnerPath = path.join(rootDir, 'scripts', 'run-tauri-cli.mjs');
  const workspaceTargetDirPath = path.join(rootDir, 'scripts', 'workspace-target-dir.mjs');
  const tauriRunner = await import(pathToFileURL(tauriRunnerPath).href);
  const workspaceTargetDir = await import(pathToFileURL(workspaceTargetDirPath).href);
  const managedWindowsTargetRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-managed-target-root-'));
  const appDir = path.join(rootDir, 'apps', 'sdkwork-router-portal');
  const managedWindowsEnv = {
    SDKWORK_WINDOWS_TARGET_ROOT: managedWindowsTargetRoot,
    USERPROFILE: 'C:/Users/admin',
    TEMP: 'C:/Temp',
  };

  try {
    const expectedWorkspaceTargetDir = workspaceTargetDir.resolveWorkspaceTargetDir({
      workspaceRoot: rootDir,
      env: managedWindowsEnv,
      platform: 'win32',
    });
    const expectedWorkspaceTempDir = workspaceTargetDir.resolveWorkspaceTempDir({
      workspaceRoot: rootDir,
      env: managedWindowsEnv,
      platform: 'win32',
    });

    assert.equal(
      tauriRunner.resolveManagedWindowsTauriTargetDir({
        cwd: appDir,
        env: managedWindowsEnv,
        platform: 'win32',
      }).replaceAll('\\', '/'),
      expectedWorkspaceTargetDir.replaceAll('\\', '/'),
    );

    const defaultPlan = tauriRunner.createTauriCliPlan({
      commandName: 'build',
      cwd: appDir,
      env: managedWindowsEnv,
      platform: 'win32',
    });
    assert.equal(
      String(defaultPlan.env.CARGO_TARGET_DIR).replaceAll('\\', '/'),
      expectedWorkspaceTargetDir.replaceAll('\\', '/'),
    );
    assert.equal(
      String(defaultPlan.env.TEMP).replaceAll('\\', '/'),
      expectedWorkspaceTempDir.replaceAll('\\', '/'),
    );
    assert.equal(
      String(defaultPlan.env.TMP).replaceAll('\\', '/'),
      expectedWorkspaceTempDir.replaceAll('\\', '/'),
    );

    const preservedPlan = tauriRunner.createTauriCliPlan({
      commandName: 'build',
      cwd: appDir,
      env: {
        ...managedWindowsEnv,
        CARGO_TARGET_DIR: expectedWorkspaceTargetDir,
      },
      platform: 'win32',
    });
    assert.equal(
      String(preservedPlan.env.CARGO_TARGET_DIR).replaceAll('\\', '/'),
      expectedWorkspaceTargetDir.replaceAll('\\', '/'),
    );
    assert.equal(
      String(preservedPlan.env.TEMP).replaceAll('\\', '/'),
      expectedWorkspaceTempDir.replaceAll('\\', '/'),
    );
  } finally {
    rmSync(managedWindowsTargetRoot, { recursive: true, force: true });
  }
});

test('native desktop packager writes one canonical installer, checksum, and manifest for the official portal desktop product', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.packageDesktopBundles, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const bundleRoot = path.join(stagingRoot, 'bundle-root');
    const outputDir = path.join(stagingRoot, 'release-output');
    mkdirSync(path.join(bundleRoot, 'nsis'), { recursive: true });
    writeFileSync(path.join(bundleRoot, 'nsis', 'Portal Setup.exe'), 'installer-bytes', 'utf8');

    const assets = packager.packageDesktopBundles({
      platformId: 'windows',
      archId: 'x64',
      targetTriple: 'x86_64-pc-windows-msvc',
      outputDir,
      resolveBuildRoots: () => [bundleRoot],
      resolveBuildRoot: () => bundleRoot,
    });

    assert.equal(Array.isArray(assets), true);
    assert.equal(assets.length, 1);
    assert.equal(assets[0].fileName, 'sdkwork-router-portal-desktop-windows-x64.exe');
    assert.equal(assets[0].checksumFileName, 'sdkwork-router-portal-desktop-windows-x64.exe.sha256.txt');
    assert.equal(assets[0].manifestFileName, 'sdkwork-router-portal-desktop-windows-x64.manifest.json');

    const installerPath = path.join(
      outputDir,
      'native',
      'windows',
      'x64',
      'desktop',
      'portal',
      'sdkwork-router-portal-desktop-windows-x64.exe',
    );
    const checksumPath = `${installerPath}.sha256.txt`;
    const manifestPath = path.join(
      outputDir,
      'native',
      'windows',
      'x64',
      'desktop',
      'portal',
      'sdkwork-router-portal-desktop-windows-x64.manifest.json',
    );

    assert.equal(existsSync(installerPath), true);
    assert.equal(readFileSync(installerPath, 'utf8'), 'installer-bytes');
    assert.equal(existsSync(checksumPath), true);
    assert.match(readFileSync(checksumPath, 'utf8'), /sdkwork-router-portal-desktop-windows-x64\.exe/);
    assert.equal(existsSync(manifestPath), true);

    const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
    assert.deepEqual(manifest, {
      type: 'portal-desktop-installer',
      productId: 'sdkwork-router-portal-desktop',
      appId: 'portal',
      platform: 'windows',
      arch: 'x64',
      target: 'x86_64-pc-windows-msvc',
      artifactKind: 'nsis',
      installerFile: 'sdkwork-router-portal-desktop-windows-x64.exe',
      checksumFile: 'sdkwork-router-portal-desktop-windows-x64.exe.sha256.txt',
      sourceBundlePath: 'nsis/Portal Setup.exe',
      embeddedRuntime: {
        routerBinary: 'router-product/bin/router-product-service.exe',
        adminSiteDir: 'router-product/sites/admin/dist',
        portalSiteDir: 'router-product/sites/portal/dist',
        bootstrapDataDir: 'router-product/data',
        releaseManifestFile: 'router-product/release-manifest.json',
        readmeFile: 'router-product/README.txt',
      },
    });
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native product-server packager writes one canonical archive, checksum, release manifest, and embedded bundle manifest', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.packageProductServerBundle, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const serviceRoot = path.join(stagingRoot, 'service-release');
    const adminSiteRoot = path.join(stagingRoot, 'sites', 'admin', 'dist');
    const portalSiteRoot = path.join(stagingRoot, 'sites', 'portal', 'dist');
    const dataRoot = path.join(stagingRoot, 'data');
    const deployRoot = path.join(stagingRoot, 'deploy');
    const outputDir = path.join(stagingRoot, 'release-output');

    mkdirSync(serviceRoot, { recursive: true });
    mkdirSync(adminSiteRoot, { recursive: true });
    mkdirSync(portalSiteRoot, { recursive: true });
    mkdirSync(dataRoot, { recursive: true });
    mkdirSync(path.join(deployRoot, 'docker'), { recursive: true });

    for (const binaryName of packager.listNativeServiceBinaryNames()) {
      writeFileSync(path.join(serviceRoot, binaryName), `binary:${binaryName}`, 'utf8');
    }
    writeFileSync(path.join(adminSiteRoot, 'index.html'), '<html>admin</html>', 'utf8');
    writeFileSync(path.join(portalSiteRoot, 'index.html'), '<html>portal</html>', 'utf8');
    writeFileSync(path.join(dataRoot, 'bootstrap.json'), '{"seed":true}', 'utf8');
    writeFileSync(path.join(deployRoot, 'docker', 'Dockerfile'), 'FROM scratch\n', 'utf8');

    const asset = packager.packageProductServerBundle({
      platformId: 'linux',
      archId: 'x64',
      targetTriple: 'x86_64-unknown-linux-gnu',
      outputDir,
      resolveServiceRoot: () => serviceRoot,
      siteAssetRoots: {
        admin: adminSiteRoot,
        portal: portalSiteRoot,
      },
      bootstrapDataRoots: {
        data: dataRoot,
      },
      deploymentAssetRoots: {
        deploy: deployRoot,
      },
    });

    assert.equal(asset.productId, 'sdkwork-api-router-product-server');
    assert.equal(asset.fileName, 'sdkwork-api-router-product-server-linux-x64.tar.gz');
    assert.equal(asset.checksumFileName, 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt');
    assert.equal(asset.manifestFileName, 'sdkwork-api-router-product-server-linux-x64.manifest.json');
    assert.equal(
      asset.outputDir.replaceAll('\\', '/'),
      path.join(outputDir, 'native', 'linux', 'x64', 'bundles').replaceAll('\\', '/'),
    );

    const archivePath = path.join(asset.outputDir, asset.fileName);
    const checksumPath = path.join(asset.outputDir, asset.checksumFileName);
    const manifestPath = path.join(asset.outputDir, asset.manifestFileName);
    assert.equal(existsSync(archivePath), true);
    assert.equal(existsSync(checksumPath), true);
    assert.equal(existsSync(manifestPath), true);
    assert.match(readFileSync(checksumPath, 'utf8'), /sdkwork-api-router-product-server-linux-x64\.tar\.gz/);
    assert.deepEqual(JSON.parse(readFileSync(manifestPath, 'utf8')), {
      type: 'product-server-archive',
      productId: 'sdkwork-api-router-product-server',
      platform: 'linux',
      arch: 'x64',
      target: 'x86_64-unknown-linux-gnu',
      archiveFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz',
      checksumFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt',
      embeddedManifestFile: 'release-manifest.json',
      services: packager.listNativeServiceBinaryNames(),
      sites: ['admin', 'portal'],
      bootstrapDataRoots: ['data'],
      deploymentAssetRoots: ['deploy'],
    });

    const extractRoot = path.join(stagingRoot, 'extracted');
    mkdirSync(extractRoot, { recursive: true });
    extractTarArchive({
      archivePath,
      outputDir: extractRoot,
      packager,
    });

    const extractedBundleRoot = path.join(extractRoot, 'sdkwork-api-router-product-server-linux-x64');
    const embeddedManifest = JSON.parse(
      readFileSync(path.join(extractedBundleRoot, 'release-manifest.json'), 'utf8'),
    );
    assert.deepEqual(embeddedManifest, {
      type: 'product-server-bundle',
      productId: 'sdkwork-api-router-product-server',
      platform: 'linux',
      arch: 'x64',
      target: 'x86_64-unknown-linux-gnu',
      services: packager.listNativeServiceBinaryNames(),
      sites: ['admin', 'portal'],
      bootstrapDataRoots: ['data'],
      deploymentAssetRoots: ['deploy'],
    });
    assert.equal(
      readFileSync(path.join(extractedBundleRoot, 'README.txt'), 'utf8').includes('SDKWork API Router Product Server Bundle'),
      true,
    );
    assert.equal(readFileSync(path.join(extractedBundleRoot, 'bin', 'router-product-service'), 'utf8'), 'binary:router-product-service');
    assert.equal(readFileSync(path.join(extractedBundleRoot, 'sites', 'admin', 'dist', 'index.html'), 'utf8'), '<html>admin</html>');
    assert.equal(readFileSync(path.join(extractedBundleRoot, 'sites', 'portal', 'dist', 'index.html'), 'utf8'), '<html>portal</html>');
    assert.equal(readFileSync(path.join(extractedBundleRoot, 'data', 'bootstrap.json'), 'utf8'), '{"seed":true}');
    assert.equal(readFileSync(path.join(extractedBundleRoot, 'deploy', 'docker', 'Dockerfile'), 'utf8'), 'FROM scratch\n');
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native release packager materializes a release catalog beside the local official assets', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.packageNativeAssets, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const outputDir = path.join(stagingRoot, 'release-output');
    const result = packager.packageNativeAssets({
      platform: 'linux',
      arch: 'x64',
      target: 'x86_64-unknown-linux-gnu',
      outputDir,
      generatedAt: '2026-04-18T12:34:56.789Z',
      packageDesktopBundlesImpl({ outputDir: targetOutputDir, platformId, archId, targetTriple }) {
        const appOutputDir = path.join(targetOutputDir, 'native', platformId, archId, 'desktop', 'portal');
        mkdirSync(appOutputDir, { recursive: true });
        writeFileSync(
          path.join(appOutputDir, 'sdkwork-router-portal-desktop-linux-x64.deb'),
          'desktop-installer',
          'utf8',
        );
        writeFileSync(
          path.join(appOutputDir, 'sdkwork-router-portal-desktop-linux-x64.deb.sha256.txt'),
          'desktopdigest  sdkwork-router-portal-desktop-linux-x64.deb\n',
          'utf8',
        );
        writeFileSync(
          path.join(appOutputDir, 'sdkwork-router-portal-desktop-linux-x64.manifest.json'),
          `${JSON.stringify({
            type: 'portal-desktop-installer',
            productId: 'sdkwork-router-portal-desktop',
            appId: 'portal',
            platform: platformId,
            arch: archId,
            target: targetTriple,
            artifactKind: 'deb',
            installerFile: 'sdkwork-router-portal-desktop-linux-x64.deb',
            checksumFile: 'sdkwork-router-portal-desktop-linux-x64.deb.sha256.txt',
            sourceBundlePath: 'deb/sdkwork-router-portal.deb',
            embeddedRuntime: {
              routerBinary: 'router-product/bin/router-product-service',
              adminSiteDir: 'router-product/sites/admin/dist',
              portalSiteDir: 'router-product/sites/portal/dist',
            },
          }, null, 2)}\n`,
          'utf8',
        );

        return [{
          appId: 'portal',
          platformId,
          archId,
          targetTriple,
          fileName: 'sdkwork-router-portal-desktop-linux-x64.deb',
          checksumFileName: 'sdkwork-router-portal-desktop-linux-x64.deb.sha256.txt',
          manifestFileName: 'sdkwork-router-portal-desktop-linux-x64.manifest.json',
          outputDir: appOutputDir,
        }];
      },
      packageProductServerBundleImpl({ outputDir: targetOutputDir, platformId, archId, targetTriple }) {
        const bundleOutputDir = path.join(targetOutputDir, 'native', platformId, archId, 'bundles');
        mkdirSync(bundleOutputDir, { recursive: true });
        writeFileSync(
          path.join(bundleOutputDir, 'sdkwork-api-router-product-server-linux-x64.tar.gz'),
          'server-archive',
          'utf8',
        );
        writeFileSync(
          path.join(bundleOutputDir, 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt'),
          'serverdigest  sdkwork-api-router-product-server-linux-x64.tar.gz\n',
          'utf8',
        );
        writeFileSync(
          path.join(bundleOutputDir, 'sdkwork-api-router-product-server-linux-x64.manifest.json'),
          `${JSON.stringify({
            type: 'product-server-archive',
            productId: 'sdkwork-api-router-product-server',
            platform: platformId,
            arch: archId,
            target: targetTriple,
            archiveFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz',
            checksumFile: 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt',
            embeddedManifestFile: 'release-manifest.json',
            services: ['router-product-service'],
            sites: ['admin', 'portal'],
            bootstrapDataRoots: ['data'],
            deploymentAssetRoots: ['deploy'],
          }, null, 2)}\n`,
          'utf8',
        );

        return {
          productId: 'sdkwork-api-router-product-server',
          platformId,
          archId,
          targetTriple,
          fileName: 'sdkwork-api-router-product-server-linux-x64.tar.gz',
          checksumFileName: 'sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt',
          manifestFileName: 'sdkwork-api-router-product-server-linux-x64.manifest.json',
          outputDir: bundleOutputDir,
        };
      },
    });

    const releaseCatalogPath = path.join(outputDir, 'release-catalog.json');
    assert.equal(existsSync(releaseCatalogPath), true);
    assert.equal(result.releaseCatalog.outputPath.replaceAll('\\', '/'), releaseCatalogPath.replaceAll('\\', '/'));
    assert.equal(result.releaseCatalog.generatedAt, '2026-04-18T12:34:56.789Z');

    const catalog = JSON.parse(readFileSync(releaseCatalogPath, 'utf8'));
    assert.equal(catalog.type, 'sdkwork-release-catalog');
    assert.equal(catalog.generatedAt, '2026-04-18T12:34:56.789Z');
    assert.equal(catalog.productCount, 2);
    assert.equal(catalog.variantCount, 2);
    assert.deepEqual(
      catalog.products.map((product) => product.productId),
      ['sdkwork-api-router-product-server', 'sdkwork-router-portal-desktop'],
    );
    assert.deepEqual(
      catalog.products.flatMap((product) => product.variants).map((variant) => ({
        variantKind: variant.variantKind,
        primaryFileSizeBytes: variant.primaryFileSizeBytes,
        checksumAlgorithm: variant.checksumAlgorithm,
      })),
      [
        {
          variantKind: 'server-archive',
          primaryFileSizeBytes: 14,
          checksumAlgorithm: 'sha256',
        },
        {
          variantKind: 'desktop-installer',
          primaryFileSizeBytes: 17,
          checksumAlgorithm: 'sha256',
        },
      ],
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native desktop packager rejects multiple official installers for one platform', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.packageDesktopBundles, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const bundleRoot = path.join(stagingRoot, 'bundle-root');
    const outputDir = path.join(stagingRoot, 'release-output');
    mkdirSync(path.join(bundleRoot, 'deb'), { recursive: true });
    writeFileSync(path.join(bundleRoot, 'deb', 'portal-one.deb'), 'one', 'utf8');
    writeFileSync(path.join(bundleRoot, 'deb', 'portal-two.deb'), 'two', 'utf8');

    assert.throws(
      () => packager.packageDesktopBundles({
        platformId: 'linux',
        archId: 'x64',
        targetTriple: 'x86_64-unknown-linux-gnu',
        outputDir,
        resolveBuildRoots: () => [bundleRoot],
        resolveBuildRoot: () => bundleRoot,
      }),
      /Expected exactly one official linux desktop installer/,
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native desktop packager rejects bundles that only contain non-canonical installer kinds', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.packageDesktopBundles, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const bundleRoot = path.join(stagingRoot, 'bundle-root');
    const outputDir = path.join(stagingRoot, 'release-output');
    mkdirSync(path.join(bundleRoot, 'msi'), { recursive: true });
    writeFileSync(path.join(bundleRoot, 'msi', 'Portal Setup.msi'), 'msi', 'utf8');

    assert.throws(
      () => packager.packageDesktopBundles({
        platformId: 'windows',
        archId: 'x64',
        targetTriple: 'x86_64-pc-windows-msvc',
        outputDir,
        resolveBuildRoots: () => [bundleRoot],
        resolveBuildRoot: () => bundleRoot,
      }),
      /Expected nsis\/\*\.exe/,
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('workspace target dir ignores missing configured Windows target roots and falls back to the managed workspace-drive root', async () => {
  const workspaceTargetDirPath = path.join(rootDir, 'scripts', 'workspace-target-dir.mjs');
  const workspaceTargetDir = await import(pathToFileURL(workspaceTargetDirPath).href);
  const existingTempRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-target-root-'));
  const missingConfiguredRoot = path.join(existingTempRoot, 'missing-configured-root');

  try {
    const resolvedTargetDir = workspaceTargetDir.resolveWorkspaceTargetDir({
      workspaceRoot: rootDir,
      env: {
        SDKWORK_WINDOWS_TARGET_ROOT: missingConfiguredRoot,
        TEMP: existingTempRoot,
      },
      platform: 'win32',
    });

    assert.equal(
      resolvedTargetDir.replaceAll('\\', '/'),
      workspaceTargetDir.resolveWorkspaceTargetDir({
        workspaceRoot: rootDir,
        env: {},
        platform: 'win32',
      }).replaceAll('\\', '/'),
    );
    assert.doesNotMatch(
      resolvedTargetDir.replaceAll('\\', '/'),
      new RegExp(`^${escapeRegExp(missingConfiguredRoot.replaceAll('\\', '/'))}(?:/|$)`),
    );
  } finally {
    rmSync(existingTempRoot, { recursive: true, force: true });
  }
});

test('native desktop packager also accepts generic repository target bundle roots used by some CI layouts', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.resolveAvailableNativeBuildRoot, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const genericTargetRoot = path.join(stagingRoot, 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle');
    const appLocalRoot = path.join(stagingRoot, 'apps', 'sdkwork-router-admin', 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle');

    mkdirSync(path.join(genericTargetRoot, 'nsis'), { recursive: true });
    mkdirSync(appLocalRoot, { recursive: true });
    writeFileSync(path.join(genericTargetRoot, 'nsis', 'sdkwork-router-admin.exe'), 'artifact', 'utf8');

    assert.equal(
      packager.resolveAvailableNativeBuildRoot({
        buildRoots: [appLocalRoot, genericTargetRoot],
      }).replaceAll('\\', '/'),
      genericTargetRoot.replaceAll('\\', '/'),
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native desktop packager also accepts app-root target bundle roots used by tauri v2 project layouts', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.resolveAvailableNativeBuildRoot, 'function');

  const stagingRoot = mkdtempSync(path.join(os.tmpdir(), 'sdkwork-router-release-packager-'));

  try {
    const appRootTarget = path.join(stagingRoot, 'apps', 'sdkwork-router-admin', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle');
    const srcTauriTarget = path.join(stagingRoot, 'apps', 'sdkwork-router-admin', 'src-tauri', 'target', 'x86_64-pc-windows-msvc', 'release', 'bundle');

    mkdirSync(path.join(appRootTarget, 'nsis'), { recursive: true });
    mkdirSync(srcTauriTarget, { recursive: true });
    writeFileSync(path.join(appRootTarget, 'nsis', 'sdkwork-router-admin.exe'), 'artifact', 'utf8');

    assert.equal(
      packager.resolveAvailableNativeBuildRoot({
        buildRoots: [srcTauriTarget, appRootTarget],
      }).replaceAll('\\', '/'),
      appRootTarget.replaceAll('\\', '/'),
    );
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
});

test('native release packager exposes GitHub annotation-safe error formatting for CI failures', async () => {
  const packagerPath = path.join(rootDir, 'scripts', 'release', 'package-release-assets.mjs');
  const packager = await import(pathToFileURL(packagerPath).href);

  assert.equal(typeof packager.buildGitHubActionsErrorAnnotation, 'function');
  assert.equal(
    packager.buildGitHubActionsErrorAnnotation({
      title: 'package:release,assets',
      error: new Error('bundle missing 50%\nnext line\rfinal line'),
    }),
    '::error title=package%3Arelease%2Cassets::bundle missing 50%25%0Anext line%0Dfinal line',
  );
});
