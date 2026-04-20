import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'installed-runtime-layout-catalog.mjs'),
    ).href,
  );
}

test('installed runtime layout catalog exposes strict governed control, payload, and mutable-root layouts', async () => {
  const module = await loadModule();

  assert.equal(typeof module.listInstalledRuntimeLayoutSpecs, 'function');
  assert.equal(typeof module.findInstalledRuntimeLayoutSpec, 'function');
  assert.equal(typeof module.listInstalledRuntimeLayoutSpecsByIds, 'function');
  assert.equal(typeof module.resolveInstalledRuntimeDefaultProductRoot, 'function');
  assert.equal(typeof module.materializeInstalledRuntimeControlLayout, 'function');
  assert.equal(typeof module.materializeInstalledRuntimeReleasePayloadLayout, 'function');
  assert.equal(typeof module.materializeInstalledRuntimeMutableLayout, 'function');
  assert.equal(typeof module.materializeInstalledRuntimeLayout, 'function');

  assert.deepEqual(
    module.listInstalledRuntimeLayoutSpecs(),
    [
      {
        id: 'control-root',
        description: 'installed runtime control-plane root under current/',
        layoutKind: 'control-root',
        rootDir: 'current',
        binDir: 'current/bin',
        binLibDir: 'current/bin/lib',
        startScript: 'current/bin/start.sh',
        startPs1Script: 'current/bin/start.ps1',
        serviceDirs: {
          systemd: 'current/service/systemd',
          launchd: 'current/service/launchd',
          windowsTask: 'current/service/windows-task',
          windowsService: 'current/service/windows-service',
        },
        releaseManifestFile: 'current/release-manifest.json',
      },
      {
        id: 'versioned-release-payload',
        description: 'versioned installed release payload rooted under releases/<releaseVersion>/',
        layoutKind: 'release-payload',
        releasesRoot: 'releases',
        releaseRootTemplate: 'releases/<releaseVersion>',
        releaseBinDirTemplate: 'releases/<releaseVersion>/bin',
        staticDataDirTemplate: 'releases/<releaseVersion>/data',
        releaseDeployDirTemplate: 'releases/<releaseVersion>/deploy',
        releasePayloadManifestFileTemplate: 'releases/<releaseVersion>/release-manifest.json',
        releasePayloadReadmeFileTemplate: 'releases/<releaseVersion>/README.txt',
        sitesAdminDirTemplate: 'releases/<releaseVersion>/sites/admin',
        sitesPortalDirTemplate: 'releases/<releaseVersion>/sites/portal',
        adminSiteDistDirTemplate: 'releases/<releaseVersion>/sites/admin/dist',
        portalSiteDistDirTemplate: 'releases/<releaseVersion>/sites/portal/dist',
      },
      {
        id: 'portable-mutable-runtime',
        description: 'portable installs keep mutable state under the product root',
        layoutKind: 'mutable-runtime',
        installMode: 'portable',
        productRootDefaultSegments: ['artifacts', 'install', 'sdkwork-api-router'],
        configRoot: 'config',
        configFile: 'config/router.yaml',
        configFragmentDir: 'config/conf.d',
        envFile: 'config/router.env',
        envExampleFile: 'config/router.env.example',
        dataRoot: 'data',
        logRoot: 'log',
        runRoot: 'run',
      },
      {
        id: 'system-runtime-windows',
        description: 'Windows system installs split immutable Program Files content from mutable ProgramData state',
        layoutKind: 'mutable-runtime',
        installMode: 'system',
        platform: 'windows',
        productRootBaseEnvVar: 'ProgramFiles',
        productRootBaseDefault: 'C:\\Program Files',
        productRootSegments: ['sdkwork-api-router'],
        configRootBaseEnvVar: 'ProgramData',
        configRootBaseDefault: 'C:\\ProgramData',
        configRootSegments: ['sdkwork-api-router'],
        dataRootBaseEnvVar: 'ProgramData',
        dataRootBaseDefault: 'C:\\ProgramData',
        dataRootSegments: ['sdkwork-api-router', 'data'],
        logRootBaseEnvVar: 'ProgramData',
        logRootBaseDefault: 'C:\\ProgramData',
        logRootSegments: ['sdkwork-api-router', 'log'],
        runRootBaseEnvVar: 'ProgramData',
        runRootBaseDefault: 'C:\\ProgramData',
        runRootSegments: ['sdkwork-api-router', 'run'],
      },
      {
        id: 'system-runtime-macos',
        description: 'macOS system installs use /usr/local/lib for immutable product content and Library roots for mutable state',
        layoutKind: 'mutable-runtime',
        installMode: 'system',
        platform: 'macos',
        productRootBaseDefault: '/usr/local/lib',
        productRootSegments: ['sdkwork-api-router'],
        configRootBaseDefault: '/Library/Application Support',
        configRootSegments: ['sdkwork-api-router'],
        dataRootBaseDefault: '/Library/Application Support',
        dataRootSegments: ['sdkwork-api-router', 'data'],
        logRootBaseDefault: '/Library/Logs',
        logRootSegments: ['sdkwork-api-router'],
        runRootBaseDefault: '/Library/Application Support',
        runRootSegments: ['sdkwork-api-router', 'run'],
      },
      {
        id: 'system-runtime-linux',
        description: 'Linux system installs use /opt for immutable product content and FHS roots for mutable state',
        layoutKind: 'mutable-runtime',
        installMode: 'system',
        platform: 'linux',
        productRootBaseDefault: '/opt',
        productRootSegments: ['sdkwork-api-router'],
        configRootBaseDefault: '/etc',
        configRootSegments: ['sdkwork-api-router'],
        dataRootBaseDefault: '/var/lib',
        dataRootSegments: ['sdkwork-api-router'],
        logRootBaseDefault: '/var/log',
        logRootSegments: ['sdkwork-api-router'],
        runRootBaseDefault: '/run',
        runRootSegments: ['sdkwork-api-router'],
      },
    ],
  );

  const controlSpec = module.findInstalledRuntimeLayoutSpec('control-root');
  controlSpec.serviceDirs.systemd = 'mutated-locally';
  assert.deepEqual(
    module.findInstalledRuntimeLayoutSpec('control-root'),
    {
      id: 'control-root',
      description: 'installed runtime control-plane root under current/',
      layoutKind: 'control-root',
      rootDir: 'current',
      binDir: 'current/bin',
      binLibDir: 'current/bin/lib',
      startScript: 'current/bin/start.sh',
      startPs1Script: 'current/bin/start.ps1',
      serviceDirs: {
        systemd: 'current/service/systemd',
        launchd: 'current/service/launchd',
        windowsTask: 'current/service/windows-task',
        windowsService: 'current/service/windows-service',
      },
      releaseManifestFile: 'current/release-manifest.json',
    },
  );

  assert.deepEqual(
    module.listInstalledRuntimeLayoutSpecsByIds([
      'system-runtime-linux',
      'portable-mutable-runtime',
    ]).map(({ id }) => id),
    [
      'system-runtime-linux',
      'portable-mutable-runtime',
    ],
  );

  assert.equal(
    module.resolveInstalledRuntimeDefaultProductRoot({
      repoRoot,
      mode: 'portable',
      platform: 'linux',
    }).replaceAll('\\', '/'),
    path.join(repoRoot, 'artifacts', 'install', 'sdkwork-api-router').replaceAll('\\', '/'),
  );
  assert.equal(
    module.resolveInstalledRuntimeDefaultProductRoot({
      repoRoot,
      mode: 'system',
      platform: 'win32',
      env: {
        ProgramFiles: 'D:\\Programs',
        ProgramData: 'D:\\ProgramData',
      },
    }).replaceAll('\\', '/'),
    'D:/Programs/sdkwork-api-router',
  );

  assert.deepEqual(
    Object.fromEntries(
      Object.entries(module.materializeInstalledRuntimeControlLayout({
        productRoot: '/opt/sdkwork-api-router',
        platform: 'linux',
      })).map(([key, value]) => [key, String(value).replaceAll('\\', '/')]),
    ),
    {
      productRoot: '/opt/sdkwork-api-router',
      controlRoot: '/opt/sdkwork-api-router/current',
      binDir: '/opt/sdkwork-api-router/current/bin',
      binLibDir: '/opt/sdkwork-api-router/current/bin/lib',
      startScript: '/opt/sdkwork-api-router/current/bin/start.sh',
      startPs1Script: '/opt/sdkwork-api-router/current/bin/start.ps1',
      serviceSystemdDir: '/opt/sdkwork-api-router/current/service/systemd',
      serviceLaunchdDir: '/opt/sdkwork-api-router/current/service/launchd',
      serviceWindowsTaskDir: '/opt/sdkwork-api-router/current/service/windows-task',
      serviceWindowsServiceDir: '/opt/sdkwork-api-router/current/service/windows-service',
      releaseManifestFile: '/opt/sdkwork-api-router/current/release-manifest.json',
    },
  );

  assert.deepEqual(
    Object.fromEntries(
      Object.entries(module.materializeInstalledRuntimeReleasePayloadLayout({
        productRoot: '/opt/sdkwork-api-router',
        platform: 'linux',
        releaseVersion: '1.2.3',
      })).map(([key, value]) => [key, String(value).replaceAll('\\', '/')]),
    ),
    {
      productRoot: '/opt/sdkwork-api-router',
      releasesRoot: '/opt/sdkwork-api-router/releases',
      releaseVersion: '1.2.3',
      releaseRoot: '/opt/sdkwork-api-router/releases/1.2.3',
      releaseBinDir: '/opt/sdkwork-api-router/releases/1.2.3/bin',
      staticDataDir: '/opt/sdkwork-api-router/releases/1.2.3/data',
      releaseDeployDir: '/opt/sdkwork-api-router/releases/1.2.3/deploy',
      releasePayloadManifestFile: '/opt/sdkwork-api-router/releases/1.2.3/release-manifest.json',
      releasePayloadReadmeFile: '/opt/sdkwork-api-router/releases/1.2.3/README.txt',
      sitesAdminDir: '/opt/sdkwork-api-router/releases/1.2.3/sites/admin',
      sitesPortalDir: '/opt/sdkwork-api-router/releases/1.2.3/sites/portal',
      adminSiteDistDir: '/opt/sdkwork-api-router/releases/1.2.3/sites/admin/dist',
      portalSiteDistDir: '/opt/sdkwork-api-router/releases/1.2.3/sites/portal/dist',
    },
  );

  assert.deepEqual(
    Object.fromEntries(
      Object.entries(module.materializeInstalledRuntimeMutableLayout({
        productRoot: '/opt/sdkwork-api-router',
        mode: 'system',
        platform: 'linux',
        env: {},
      })).map(([key, value]) => [key, String(value).replaceAll('\\', '/')]),
    ),
    {
      productRoot: '/opt/sdkwork-api-router',
      configRoot: '/etc/sdkwork-api-router',
      configFile: '/etc/sdkwork-api-router/router.yaml',
      configFragmentDir: '/etc/sdkwork-api-router/conf.d',
      envFile: '/etc/sdkwork-api-router/router.env',
      envExampleFile: '/etc/sdkwork-api-router/router.env.example',
      dataRoot: '/var/lib/sdkwork-api-router',
      logRoot: '/var/log/sdkwork-api-router',
      runRoot: '/run/sdkwork-api-router',
    },
  );

  assert.deepEqual(
    Object.fromEntries(
      Object.entries(module.materializeInstalledRuntimeLayout({
        installRoot: '/opt/sdkwork-api-router/current',
        mode: 'system',
        platform: 'linux',
        env: {},
        releaseVersion: '1.2.3',
      })).map(([key, value]) => [key, Array.isArray(value) ? value : String(value).replaceAll('\\', '/')]),
    ),
    {
      mode: 'system',
      runtimePlatform: 'linux',
      installRoot: '/opt/sdkwork-api-router',
      controlRoot: '/opt/sdkwork-api-router/current',
      releasesRoot: '/opt/sdkwork-api-router/releases',
      releaseVersion: '1.2.3',
      releaseRoot: '/opt/sdkwork-api-router/releases/1.2.3',
      binDir: '/opt/sdkwork-api-router/current/bin',
      binLibDir: '/opt/sdkwork-api-router/current/bin/lib',
      startScript: '/opt/sdkwork-api-router/current/bin/start.sh',
      startPs1Script: '/opt/sdkwork-api-router/current/bin/start.ps1',
      releaseBinDir: '/opt/sdkwork-api-router/releases/1.2.3/bin',
      staticDataDir: '/opt/sdkwork-api-router/releases/1.2.3/data',
      serviceSystemdDir: '/opt/sdkwork-api-router/current/service/systemd',
      serviceLaunchdDir: '/opt/sdkwork-api-router/current/service/launchd',
      serviceWindowsTaskDir: '/opt/sdkwork-api-router/current/service/windows-task',
      serviceWindowsServiceDir: '/opt/sdkwork-api-router/current/service/windows-service',
      releaseDeployDir: '/opt/sdkwork-api-router/releases/1.2.3/deploy',
      releasePayloadManifestFile: '/opt/sdkwork-api-router/releases/1.2.3/release-manifest.json',
      releasePayloadReadmeFile: '/opt/sdkwork-api-router/releases/1.2.3/README.txt',
      sitesAdminDir: '/opt/sdkwork-api-router/releases/1.2.3/sites/admin',
      sitesPortalDir: '/opt/sdkwork-api-router/releases/1.2.3/sites/portal',
      adminSiteDistDir: '/opt/sdkwork-api-router/releases/1.2.3/sites/admin/dist',
      portalSiteDistDir: '/opt/sdkwork-api-router/releases/1.2.3/sites/portal/dist',
      configRoot: '/etc/sdkwork-api-router',
      configFile: '/etc/sdkwork-api-router/router.yaml',
      configFragmentDir: '/etc/sdkwork-api-router/conf.d',
      envFile: '/etc/sdkwork-api-router/router.env',
      envExampleFile: '/etc/sdkwork-api-router/router.env.example',
      dataRoot: '/var/lib/sdkwork-api-router',
      logRoot: '/var/log/sdkwork-api-router',
      runRoot: '/run/sdkwork-api-router',
      routerBinary: '/opt/sdkwork-api-router/releases/1.2.3/bin/router-product-service',
      releaseManifestFile: '/opt/sdkwork-api-router/current/release-manifest.json',
    },
  );

  assert.throws(
    () => module.findInstalledRuntimeLayoutSpec('missing-installed-runtime-layout'),
    /missing installed runtime layout spec.*missing-installed-runtime-layout/i,
  );
});

test('install tooling and installed-runtime smoke consume the shared installed runtime layout catalog', () => {
  const runtimeTooling = read('bin/lib/router-runtime-tooling.mjs');
  const installedRuntimeSmokeLib = read('scripts/release/installed-runtime-smoke-lib.mjs');

  assert.match(
    runtimeTooling,
    /installed-runtime-layout-catalog\.mjs/,
    'install tooling must consume the shared installed runtime layout catalog',
  );
  assert.match(
    installedRuntimeSmokeLib,
    /installed-runtime-layout-catalog\.mjs/,
    'installed runtime smoke helpers must consume the shared installed runtime layout catalog',
  );
  assert.doesNotMatch(
    runtimeTooling,
    /path(?:Api)?\.join\(productRoot,\s*['"`]current['"`]\)|path(?:Api)?\.join\(releasesRoot,\s*activeReleaseVersion\)|path(?:Api)?\.join\(releaseRoot,\s*['"`]sites['"`],\s*['"`]admin['"`],\s*['"`]dist['"`]\)|path(?:Api)?\.join\(releaseRoot,\s*['"`]sites['"`],\s*['"`]portal['"`],\s*['"`]dist['"`]\)|path(?:Api)?\.join\(releaseRoot,\s*['"`]release-manifest\.json['"`]\)|path(?:Api)?\.join\(releaseRoot,\s*['"`]README\.txt['"`]\)/,
    'install tooling must not inline installed control/payload layout paths after catalog extraction',
  );
  assert.doesNotMatch(
    installedRuntimeSmokeLib,
    /path\.join\(runtimeHome,\s*['"`]current['"`],\s*['"`]release-manifest\.json['"`]\)|path\.join\(productRoot,\s*['"`]config['"`]\)|path\.join\(productRoot,\s*['"`]log['"`]\)|path\.join\(productRoot,\s*['"`]run['"`]\)/,
    'installed runtime smoke helpers must not inline installed control or mutable-root paths after catalog extraction',
  );
  assert.doesNotMatch(
    installedRuntimeSmokeLib,
    /path\.join\(bundleRoot,\s*['"`]control['"`],\s*['"`]release-manifest\.json['"`]\)|path\.join\(bundleRoot,\s*['"`]config['"`]\)|path\.join\(bundleRoot,\s*['"`]data['"`]\)/,
    'installed runtime smoke helpers must not inline backup bundle snapshot paths after the backup manifest contract becomes self-describing',
  );
});
