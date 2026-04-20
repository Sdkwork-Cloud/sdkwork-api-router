import path from 'node:path';

import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';

const INSTALLED_RUNTIME_LAYOUT_SPECS = [
  Object.freeze({
    id: 'control-root',
    description: 'installed runtime control-plane root under current/',
    layoutKind: 'control-root',
    rootDir: 'current',
    binDir: 'current/bin',
    binLibDir: 'current/bin/lib',
    startScript: 'current/bin/start.sh',
    startPs1Script: 'current/bin/start.ps1',
    serviceDirs: Object.freeze({
      systemd: 'current/service/systemd',
      launchd: 'current/service/launchd',
      windowsTask: 'current/service/windows-task',
      windowsService: 'current/service/windows-service',
    }),
    releaseManifestFile: 'current/release-manifest.json',
  }),
  Object.freeze({
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
  }),
  Object.freeze({
    id: 'portable-mutable-runtime',
    description: 'portable installs keep mutable state under the product root',
    layoutKind: 'mutable-runtime',
    installMode: 'portable',
    productRootDefaultSegments: Object.freeze(['artifacts', 'install', 'sdkwork-api-router']),
    configRoot: 'config',
    configFile: 'config/router.yaml',
    configFragmentDir: 'config/conf.d',
    envFile: 'config/router.env',
    envExampleFile: 'config/router.env.example',
    dataRoot: 'data',
    logRoot: 'log',
    runRoot: 'run',
  }),
  Object.freeze({
    id: 'system-runtime-windows',
    description: 'Windows system installs split immutable Program Files content from mutable ProgramData state',
    layoutKind: 'mutable-runtime',
    installMode: 'system',
    platform: 'windows',
    productRootBaseEnvVar: 'ProgramFiles',
    productRootBaseDefault: 'C:\\Program Files',
    productRootSegments: Object.freeze(['sdkwork-api-router']),
    configRootBaseEnvVar: 'ProgramData',
    configRootBaseDefault: 'C:\\ProgramData',
    configRootSegments: Object.freeze(['sdkwork-api-router']),
    dataRootBaseEnvVar: 'ProgramData',
    dataRootBaseDefault: 'C:\\ProgramData',
    dataRootSegments: Object.freeze(['sdkwork-api-router', 'data']),
    logRootBaseEnvVar: 'ProgramData',
    logRootBaseDefault: 'C:\\ProgramData',
    logRootSegments: Object.freeze(['sdkwork-api-router', 'log']),
    runRootBaseEnvVar: 'ProgramData',
    runRootBaseDefault: 'C:\\ProgramData',
    runRootSegments: Object.freeze(['sdkwork-api-router', 'run']),
  }),
  Object.freeze({
    id: 'system-runtime-macos',
    description: 'macOS system installs use /usr/local/lib for immutable product content and Library roots for mutable state',
    layoutKind: 'mutable-runtime',
    installMode: 'system',
    platform: 'macos',
    productRootBaseDefault: '/usr/local/lib',
    productRootSegments: Object.freeze(['sdkwork-api-router']),
    configRootBaseDefault: '/Library/Application Support',
    configRootSegments: Object.freeze(['sdkwork-api-router']),
    dataRootBaseDefault: '/Library/Application Support',
    dataRootSegments: Object.freeze(['sdkwork-api-router', 'data']),
    logRootBaseDefault: '/Library/Logs',
    logRootSegments: Object.freeze(['sdkwork-api-router']),
    runRootBaseDefault: '/Library/Application Support',
    runRootSegments: Object.freeze(['sdkwork-api-router', 'run']),
  }),
  Object.freeze({
    id: 'system-runtime-linux',
    description: 'Linux system installs use /opt for immutable product content and FHS roots for mutable state',
    layoutKind: 'mutable-runtime',
    installMode: 'system',
    platform: 'linux',
    productRootBaseDefault: '/opt',
    productRootSegments: Object.freeze(['sdkwork-api-router']),
    configRootBaseDefault: '/etc',
    configRootSegments: Object.freeze(['sdkwork-api-router']),
    dataRootBaseDefault: '/var/lib',
    dataRootSegments: Object.freeze(['sdkwork-api-router']),
    logRootBaseDefault: '/var/log',
    logRootSegments: Object.freeze(['sdkwork-api-router']),
    runRootBaseDefault: '/run',
    runRootSegments: Object.freeze(['sdkwork-api-router']),
  }),
];

function cloneInstalledRuntimeLayoutSpec(spec) {
  return {
    ...spec,
    ...(spec.serviceDirs ? { serviceDirs: { ...spec.serviceDirs } } : {}),
    ...(spec.productRootDefaultSegments ? { productRootDefaultSegments: [...spec.productRootDefaultSegments] } : {}),
    ...(spec.productRootSegments ? { productRootSegments: [...spec.productRootSegments] } : {}),
    ...(spec.configRootSegments ? { configRootSegments: [...spec.configRootSegments] } : {}),
    ...(spec.dataRootSegments ? { dataRootSegments: [...spec.dataRootSegments] } : {}),
    ...(spec.logRootSegments ? { logRootSegments: [...spec.logRootSegments] } : {}),
    ...(spec.runRootSegments ? { runRootSegments: [...spec.runRootSegments] } : {}),
  };
}

const installedRuntimeLayoutCatalog = createStrictKeyedCatalog({
  entries: INSTALLED_RUNTIME_LAYOUT_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneInstalledRuntimeLayoutSpec,
  duplicateKeyMessagePrefix: 'duplicate installed runtime layout spec',
  missingKeyMessagePrefix: 'missing installed runtime layout spec',
});

function normalizeInstallMode(mode = 'portable') {
  const normalized = String(mode ?? 'portable').trim().toLowerCase() || 'portable';
  if (!['portable', 'system'].includes(normalized)) {
    throw new Error(`unsupported install mode: ${mode}`);
  }

  return normalized;
}

function normalizeInstalledRuntimePlatform(platform = process.platform) {
  const normalized = String(platform ?? process.platform).trim().toLowerCase();
  if (normalized === 'windows' || normalized === 'win32') {
    return 'windows';
  }
  if (normalized === 'macos' || normalized === 'darwin' || normalized === 'mac') {
    return 'macos';
  }

  return 'linux';
}

function resolveInstalledRuntimePathApi({
  mode = 'portable',
  platform = process.platform,
  pathApi,
} = {}) {
  if (pathApi) {
    return pathApi;
  }

  if (normalizeInstallMode(mode) === 'portable') {
    return path;
  }

  return normalizeInstalledRuntimePlatform(platform) === 'windows' ? path.win32 : path.posix;
}

function joinBaseAndSegments(pathApi, basePath, segments = []) {
  if (!Array.isArray(segments) || segments.length === 0) {
    return basePath;
  }

  return pathApi.join(basePath, ...segments);
}

function joinRelativePath(pathApi, basePath, relativePath) {
  const segments = String(relativePath ?? '')
    .split('/')
    .filter(Boolean);

  return joinBaseAndSegments(pathApi, basePath, segments);
}

function renderReleaseTemplate(template, releaseVersion) {
  return String(template ?? '').replaceAll('<releaseVersion>', releaseVersion);
}

function resolveSystemRuntimeLayoutSpec(platform = process.platform) {
  const normalizedPlatform = normalizeInstalledRuntimePlatform(platform);
  if (normalizedPlatform === 'windows') {
    return findInstalledRuntimeLayoutSpec('system-runtime-windows');
  }
  if (normalizedPlatform === 'macos') {
    return findInstalledRuntimeLayoutSpec('system-runtime-macos');
  }

  return findInstalledRuntimeLayoutSpec('system-runtime-linux');
}

function resolveMutableRuntimeLayoutSpec({
  mode = 'portable',
  platform = process.platform,
} = {}) {
  return normalizeInstallMode(mode) === 'portable'
    ? findInstalledRuntimeLayoutSpec('portable-mutable-runtime')
    : resolveSystemRuntimeLayoutSpec(platform);
}

function resolveConfiguredBaseRoot({
  env = process.env,
  envVar,
  defaultValue,
} = {}) {
  if (envVar) {
    const configured = String(env?.[envVar] ?? '').trim();
    if (configured.length > 0) {
      return configured;
    }
  }

  return defaultValue;
}

function resolveSystemMutableRoot(pathApi, spec, {
  env = process.env,
  rootKey,
} = {}) {
  const baseRoot = resolveConfiguredBaseRoot({
    env,
    envVar: spec[`${rootKey}BaseEnvVar`],
    defaultValue: spec[`${rootKey}BaseDefault`],
  });

  return joinBaseAndSegments(pathApi, baseRoot, spec[`${rootKey}Segments`]);
}

function normalizeInstalledRuntimeProductRoot(installRoot, {
  pathApi,
  controlSpec = findInstalledRuntimeLayoutSpec('control-root'),
} = {}) {
  if (!installRoot) {
    return installRoot;
  }

  const controlRootName = String(controlSpec.rootDir ?? '')
    .split('/')
    .filter(Boolean)
    .at(-1);
  const normalizedInstallRoot = pathApi.normalize(String(installRoot));

  return pathApi.basename(normalizedInstallRoot) === controlRootName
    ? pathApi.dirname(normalizedInstallRoot)
    : normalizedInstallRoot;
}

function resolveRouterBinaryName(platform = process.platform) {
  return normalizeInstalledRuntimePlatform(platform) === 'windows'
    ? 'router-product-service.exe'
    : 'router-product-service';
}

export function listInstalledRuntimeLayoutSpecs() {
  return installedRuntimeLayoutCatalog.list();
}

export function findInstalledRuntimeLayoutSpec(layoutId) {
  return installedRuntimeLayoutCatalog.find(layoutId);
}

export function listInstalledRuntimeLayoutSpecsByIds(layoutIds = []) {
  return installedRuntimeLayoutCatalog.listByKeys(layoutIds);
}

export function resolveInstalledRuntimeDefaultProductRoot({
  repoRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  pathApi,
} = {}) {
  const normalizedMode = normalizeInstallMode(mode);
  const resolvedPathApi = resolveInstalledRuntimePathApi({
    mode: normalizedMode,
    platform,
    pathApi,
  });
  const spec = resolveMutableRuntimeLayoutSpec({
    mode: normalizedMode,
    platform,
  });

  if (normalizedMode === 'portable') {
    if (!repoRoot) {
      throw new Error('portable installed runtime default product root requires repoRoot');
    }

    return joinBaseAndSegments(resolvedPathApi, repoRoot, spec.productRootDefaultSegments);
  }

  const productBaseRoot = resolveConfiguredBaseRoot({
    env,
    envVar: spec.productRootBaseEnvVar,
    defaultValue: spec.productRootBaseDefault,
  });

  return joinBaseAndSegments(resolvedPathApi, productBaseRoot, spec.productRootSegments);
}

export function materializeInstalledRuntimeControlLayout({
  productRoot,
  platform = process.platform,
  pathApi,
} = {}) {
  const spec = findInstalledRuntimeLayoutSpec('control-root');
  const resolvedPathApi = resolveInstalledRuntimePathApi({
    mode: 'system',
    platform,
    pathApi,
  });

  return {
    productRoot,
    controlRoot: joinRelativePath(resolvedPathApi, productRoot, spec.rootDir),
    binDir: joinRelativePath(resolvedPathApi, productRoot, spec.binDir),
    binLibDir: joinRelativePath(resolvedPathApi, productRoot, spec.binLibDir),
    startScript: joinRelativePath(resolvedPathApi, productRoot, spec.startScript),
    startPs1Script: joinRelativePath(resolvedPathApi, productRoot, spec.startPs1Script),
    serviceSystemdDir: joinRelativePath(resolvedPathApi, productRoot, spec.serviceDirs.systemd),
    serviceLaunchdDir: joinRelativePath(resolvedPathApi, productRoot, spec.serviceDirs.launchd),
    serviceWindowsTaskDir: joinRelativePath(resolvedPathApi, productRoot, spec.serviceDirs.windowsTask),
    serviceWindowsServiceDir: joinRelativePath(resolvedPathApi, productRoot, spec.serviceDirs.windowsService),
    releaseManifestFile: joinRelativePath(resolvedPathApi, productRoot, spec.releaseManifestFile),
  };
}

export function materializeInstalledRuntimeReleasePayloadLayout({
  productRoot,
  platform = process.platform,
  releaseVersion = '0.1.0',
  pathApi,
} = {}) {
  const spec = findInstalledRuntimeLayoutSpec('versioned-release-payload');
  const resolvedPathApi = resolveInstalledRuntimePathApi({
    mode: 'system',
    platform,
    pathApi,
  });
  const activeReleaseVersion = String(releaseVersion ?? '').trim() || '0.1.0';

  return {
    productRoot,
    releasesRoot: joinRelativePath(resolvedPathApi, productRoot, spec.releasesRoot),
    releaseVersion: activeReleaseVersion,
    releaseRoot: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.releaseRootTemplate, activeReleaseVersion),
    ),
    releaseBinDir: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.releaseBinDirTemplate, activeReleaseVersion),
    ),
    staticDataDir: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.staticDataDirTemplate, activeReleaseVersion),
    ),
    releaseDeployDir: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.releaseDeployDirTemplate, activeReleaseVersion),
    ),
    releasePayloadManifestFile: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.releasePayloadManifestFileTemplate, activeReleaseVersion),
    ),
    releasePayloadReadmeFile: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.releasePayloadReadmeFileTemplate, activeReleaseVersion),
    ),
    sitesAdminDir: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.sitesAdminDirTemplate, activeReleaseVersion),
    ),
    sitesPortalDir: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.sitesPortalDirTemplate, activeReleaseVersion),
    ),
    adminSiteDistDir: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.adminSiteDistDirTemplate, activeReleaseVersion),
    ),
    portalSiteDistDir: joinRelativePath(
      resolvedPathApi,
      productRoot,
      renderReleaseTemplate(spec.portalSiteDistDirTemplate, activeReleaseVersion),
    ),
  };
}

export function materializeInstalledRuntimeMutableLayout({
  productRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  pathApi,
} = {}) {
  const normalizedMode = normalizeInstallMode(mode);
  const resolvedPathApi = resolveInstalledRuntimePathApi({
    mode: normalizedMode,
    platform,
    pathApi,
  });
  const spec = resolveMutableRuntimeLayoutSpec({
    mode: normalizedMode,
    platform,
  });

  if (normalizedMode === 'portable') {
    return {
      productRoot,
      configRoot: joinRelativePath(resolvedPathApi, productRoot, spec.configRoot),
      configFile: joinRelativePath(resolvedPathApi, productRoot, spec.configFile),
      configFragmentDir: joinRelativePath(resolvedPathApi, productRoot, spec.configFragmentDir),
      envFile: joinRelativePath(resolvedPathApi, productRoot, spec.envFile),
      envExampleFile: joinRelativePath(resolvedPathApi, productRoot, spec.envExampleFile),
      dataRoot: joinRelativePath(resolvedPathApi, productRoot, spec.dataRoot),
      logRoot: joinRelativePath(resolvedPathApi, productRoot, spec.logRoot),
      runRoot: joinRelativePath(resolvedPathApi, productRoot, spec.runRoot),
    };
  }

  const configRoot = resolveSystemMutableRoot(resolvedPathApi, spec, {
    env,
    rootKey: 'configRoot',
  });
  const dataRoot = resolveSystemMutableRoot(resolvedPathApi, spec, {
    env,
    rootKey: 'dataRoot',
  });
  const logRoot = resolveSystemMutableRoot(resolvedPathApi, spec, {
    env,
    rootKey: 'logRoot',
  });
  const runRoot = resolveSystemMutableRoot(resolvedPathApi, spec, {
    env,
    rootKey: 'runRoot',
  });

  return {
    productRoot,
    configRoot,
    configFile: resolvedPathApi.join(configRoot, 'router.yaml'),
    configFragmentDir: resolvedPathApi.join(configRoot, 'conf.d'),
    envFile: resolvedPathApi.join(configRoot, 'router.env'),
    envExampleFile: resolvedPathApi.join(configRoot, 'router.env.example'),
    dataRoot,
    logRoot,
    runRoot,
  };
}

export function materializeInstalledRuntimeLayout({
  installRoot,
  repoRoot,
  mode = 'portable',
  platform = process.platform,
  env = process.env,
  releaseVersion = '0.1.0',
  pathApi,
} = {}) {
  const normalizedMode = normalizeInstallMode(mode);
  const resolvedPathApi = resolveInstalledRuntimePathApi({
    mode: normalizedMode,
    platform,
    pathApi,
  });
  const productRoot = normalizeInstalledRuntimeProductRoot(installRoot, {
    pathApi: resolvedPathApi,
  }) ?? resolveInstalledRuntimeDefaultProductRoot({
    repoRoot,
    mode: normalizedMode,
    platform,
    env,
    pathApi: resolvedPathApi,
  });
  const controlLayout = materializeInstalledRuntimeControlLayout({
    productRoot,
    platform,
    pathApi: resolvedPathApi,
  });
  const releasePayloadLayout = materializeInstalledRuntimeReleasePayloadLayout({
    productRoot,
    platform,
    releaseVersion,
    pathApi: resolvedPathApi,
  });
  const mutableLayout = materializeInstalledRuntimeMutableLayout({
    productRoot,
    mode: normalizedMode,
    platform,
    env,
    pathApi: resolvedPathApi,
  });

  const { productRoot: _controlProductRoot, ...resolvedControlLayout } = controlLayout;
  const { productRoot: _releaseProductRoot, ...resolvedReleasePayloadLayout } = releasePayloadLayout;
  const { productRoot: _mutableProductRoot, ...resolvedMutableLayout } = mutableLayout;

  return {
    mode: normalizedMode,
    runtimePlatform: normalizeInstalledRuntimePlatform(platform),
    installRoot: productRoot,
    ...resolvedControlLayout,
    ...resolvedReleasePayloadLayout,
    ...resolvedMutableLayout,
    routerBinary: resolvedPathApi.join(
      resolvedReleasePayloadLayout.releaseBinDir,
      resolveRouterBinaryName(platform),
    ),
  };
}
