import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';

const DESKTOP_RELEASE_PLATFORM_SPECS = [
  Object.freeze({
    platform: 'windows',
    aliases: Object.freeze(['win32', 'windows']),
    bundles: Object.freeze(['nsis']),
    officialInstaller: Object.freeze({
      artifactKind: 'nsis',
      expectedBundleDirectory: 'nsis',
      expectedFileSuffix: '.exe',
    }),
  }),
  Object.freeze({
    platform: 'linux',
    aliases: Object.freeze(['linux']),
    bundles: Object.freeze(['deb']),
    officialInstaller: Object.freeze({
      artifactKind: 'deb',
      expectedBundleDirectory: 'deb',
      expectedFileSuffix: '.deb',
    }),
  }),
  Object.freeze({
    platform: 'macos',
    aliases: Object.freeze(['darwin', 'macos']),
    bundles: Object.freeze(['dmg']),
    officialInstaller: Object.freeze({
      artifactKind: 'dmg',
      expectedBundleDirectory: 'dmg',
      expectedFileSuffix: '.dmg',
    }),
  }),
];

const DESKTOP_RELEASE_ARCH_SPECS = [
  Object.freeze({
    arch: 'x64',
    aliases: Object.freeze(['x64', 'x86_64', 'amd64']),
  }),
  Object.freeze({
    arch: 'arm64',
    aliases: Object.freeze(['arm64', 'aarch64']),
  }),
];

const DESKTOP_RELEASE_TARGET_TRIPLE_SPECS = [
  Object.freeze({
    targetTriple: 'x86_64-pc-windows-msvc',
    platform: 'windows',
    arch: 'x64',
  }),
  Object.freeze({
    targetTriple: 'aarch64-pc-windows-msvc',
    platform: 'windows',
    arch: 'arm64',
  }),
  Object.freeze({
    targetTriple: 'x86_64-unknown-linux-gnu',
    platform: 'linux',
    arch: 'x64',
  }),
  Object.freeze({
    targetTriple: 'aarch64-unknown-linux-gnu',
    platform: 'linux',
    arch: 'arm64',
  }),
  Object.freeze({
    targetTriple: 'x86_64-apple-darwin',
    platform: 'macos',
    arch: 'x64',
  }),
  Object.freeze({
    targetTriple: 'aarch64-apple-darwin',
    platform: 'macos',
    arch: 'arm64',
  }),
];

function cloneDesktopReleasePlatformSpec(spec) {
  return {
    platform: spec.platform,
    aliases: [...spec.aliases],
    bundles: [...spec.bundles],
    officialInstaller: {
      ...spec.officialInstaller,
    },
  };
}

function cloneDesktopReleaseArchSpec(spec) {
  return {
    arch: spec.arch,
    aliases: [...spec.aliases],
  };
}

function cloneDesktopReleaseTargetTripleSpec(spec) {
  return {
    ...spec,
  };
}

function cloneAliasSpec(spec) {
  return {
    ...spec,
  };
}

function createPlatformArchKey(platform, arch) {
  return `${platform}:${arch}`;
}

const desktopReleasePlatformSpecCatalog = createStrictKeyedCatalog({
  entries: DESKTOP_RELEASE_PLATFORM_SPECS,
  getKey: (spec) => spec.platform,
  clone: cloneDesktopReleasePlatformSpec,
  duplicateKeyMessagePrefix: 'duplicate desktop release platform spec',
  missingKeyMessagePrefix: 'missing desktop release platform spec',
});

const desktopReleaseArchSpecCatalog = createStrictKeyedCatalog({
  entries: DESKTOP_RELEASE_ARCH_SPECS,
  getKey: (spec) => spec.arch,
  clone: cloneDesktopReleaseArchSpec,
  duplicateKeyMessagePrefix: 'duplicate desktop release arch spec',
  missingKeyMessagePrefix: 'missing desktop release arch spec',
});

const desktopReleaseTargetTripleSpecCatalog = createStrictKeyedCatalog({
  entries: DESKTOP_RELEASE_TARGET_TRIPLE_SPECS,
  getKey: (spec) => spec.targetTriple,
  clone: cloneDesktopReleaseTargetTripleSpec,
  duplicateKeyMessagePrefix: 'duplicate desktop release target triple spec',
  missingKeyMessagePrefix: 'missing desktop release target triple spec',
});

const desktopReleasePlatformAliasCatalog = createStrictKeyedCatalog({
  entries: DESKTOP_RELEASE_PLATFORM_SPECS.flatMap((spec) =>
    spec.aliases.map((alias) => Object.freeze({
      alias,
      platform: spec.platform,
    }))),
  getKey: (spec) => spec.alias,
  clone: cloneAliasSpec,
  duplicateKeyMessagePrefix: 'duplicate desktop release platform alias',
  missingKeyMessagePrefix: 'missing desktop release platform alias',
});

const desktopReleaseArchAliasCatalog = createStrictKeyedCatalog({
  entries: DESKTOP_RELEASE_ARCH_SPECS.flatMap((spec) =>
    spec.aliases.map((alias) => Object.freeze({
      alias,
      arch: spec.arch,
    }))),
  getKey: (spec) => spec.alias,
  clone: cloneAliasSpec,
  duplicateKeyMessagePrefix: 'duplicate desktop release arch alias',
  missingKeyMessagePrefix: 'missing desktop release arch alias',
});

const desktopReleaseTargetTripleByPlatformArchCatalog = createStrictKeyedCatalog({
  entries: DESKTOP_RELEASE_TARGET_TRIPLE_SPECS.map((spec) => Object.freeze({
    key: createPlatformArchKey(spec.platform, spec.arch),
    ...spec,
  })),
  getKey: (spec) => spec.key,
  clone: cloneDesktopReleaseTargetTripleSpec,
  duplicateKeyMessagePrefix: 'duplicate desktop release target triple platform/arch pair',
  missingKeyMessagePrefix: 'missing desktop release target triple platform/arch pair',
});

export const DESKTOP_TARGET_ENV_VAR = 'SDKWORK_DESKTOP_TARGET';
export const DESKTOP_TARGET_PLATFORM_ENV_VAR = 'SDKWORK_DESKTOP_TARGET_PLATFORM';
export const DESKTOP_TARGET_ARCH_ENV_VAR = 'SDKWORK_DESKTOP_TARGET_ARCH';

export function listDesktopReleasePlatformSpecs() {
  return desktopReleasePlatformSpecCatalog.list();
}

export function findDesktopReleasePlatformSpec(platformId) {
  return desktopReleasePlatformSpecCatalog.find(platformId);
}

export function listDesktopReleasePlatformSpecsByIds(platformIds = []) {
  return desktopReleasePlatformSpecCatalog.listByKeys(platformIds);
}

export function listDesktopReleaseArchSpecs() {
  return desktopReleaseArchSpecCatalog.list();
}

export function findDesktopReleaseArchSpec(archId) {
  return desktopReleaseArchSpecCatalog.find(archId);
}

export function listDesktopReleaseArchSpecsByIds(archIds = []) {
  return desktopReleaseArchSpecCatalog.listByKeys(archIds);
}

export function listDesktopReleaseTargetTripleSpecs() {
  return desktopReleaseTargetTripleSpecCatalog.list();
}

export function findDesktopReleaseTargetTripleSpec(targetTriple) {
  return desktopReleaseTargetTripleSpecCatalog.find(targetTriple);
}

export function listDesktopReleaseTargetTripleSpecsByIds(targetTriples = []) {
  return desktopReleaseTargetTripleSpecCatalog.listByKeys(targetTriples);
}

export function normalizeDesktopPlatform(platform = process.platform) {
  const aliasSpec = desktopReleasePlatformAliasCatalog.find(
    String(platform ?? '').trim().toLowerCase(),
  );
  return aliasSpec.platform;
}

export function normalizeDesktopArch(arch = process.arch) {
  const aliasSpec = desktopReleaseArchAliasCatalog.find(
    String(arch ?? '').trim().toLowerCase(),
  );
  return aliasSpec.arch;
}

export function resolveDesktopReleaseBundleKinds({
  platform = process.platform,
} = {}) {
  return findDesktopReleasePlatformSpec(
    normalizeDesktopPlatform(platform),
  ).bundles;
}

export function resolveDesktopOfficialInstallerRule({
  platform = process.platform,
} = {}) {
  return findDesktopReleasePlatformSpec(
    normalizeDesktopPlatform(platform),
  ).officialInstaller;
}

export function buildDesktopTargetTriple({ platform, arch }) {
  const normalizedPlatform = normalizeDesktopPlatform(platform);
  const normalizedArch = normalizeDesktopArch(arch);
  const spec = desktopReleaseTargetTripleByPlatformArchCatalog.find(
    createPlatformArchKey(normalizedPlatform, normalizedArch),
  );
  return spec.targetTriple;
}

export function parseDesktopTargetTriple(targetTriple) {
  return findDesktopReleaseTargetTripleSpec(String(targetTriple ?? '').trim());
}

export function resolveDesktopReleaseTarget({
  targetTriple,
  platform,
  arch,
  env = process.env,
} = {}) {
  const requestedTargetTriple = firstNonEmpty(
    targetTriple,
    env?.[DESKTOP_TARGET_ENV_VAR],
  );

  if (requestedTargetTriple) {
    return parseDesktopTargetTriple(requestedTargetTriple);
  }

  const resolvedPlatform = normalizeDesktopPlatform(
    firstNonEmpty(platform, env?.[DESKTOP_TARGET_PLATFORM_ENV_VAR], process.platform),
  );
  const resolvedArch = normalizeDesktopArch(
    firstNonEmpty(arch, env?.[DESKTOP_TARGET_ARCH_ENV_VAR], process.arch),
  );

  return {
    platform: resolvedPlatform,
    arch: resolvedArch,
    targetTriple: buildDesktopTargetTriple({
      platform: resolvedPlatform,
      arch: resolvedArch,
    }),
  };
}

export function buildDesktopReleaseEnv({
  env = process.env,
  targetTriple,
  platform,
  arch,
} = {}) {
  const target = resolveDesktopReleaseTarget({
    targetTriple,
    platform,
    arch,
    env,
  });

  return {
    ...env,
    [DESKTOP_TARGET_ENV_VAR]: target.targetTriple,
    [DESKTOP_TARGET_PLATFORM_ENV_VAR]: target.platform,
    [DESKTOP_TARGET_ARCH_ENV_VAR]: target.arch,
  };
}

function firstNonEmpty(...values) {
  for (const value of values) {
    if (typeof value === 'string' && value.trim().length > 0) {
      return value.trim();
    }
  }

  return '';
}
