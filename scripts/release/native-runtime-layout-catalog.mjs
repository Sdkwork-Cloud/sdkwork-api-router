import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';

const NATIVE_RUNTIME_LAYOUT_SPECS = [
  Object.freeze({
    id: 'portal-desktop-embedded-runtime',
    layoutKind: 'desktop-embedded-runtime',
    serviceBinaryName: 'router-product-service',
    serviceBinaryDir: 'router-product/bin',
    siteTargetDirs: Object.freeze({
      admin: 'router-product/sites/admin/dist',
      portal: 'router-product/sites/portal/dist',
    }),
    bootstrapDataRootDirs: Object.freeze({
      data: 'router-product/data',
    }),
    deploymentAssetRootDirs: Object.freeze({}),
    releaseManifestFile: 'router-product/release-manifest.json',
    readmeFile: 'router-product/README.txt',
  }),
  Object.freeze({
    id: 'product-server-bundle',
    layoutKind: 'product-server-bundle',
    serviceBinaryName: 'router-product-service',
    serviceBinaryDir: 'bin',
    bundleInstallers: Object.freeze({
      shell: 'install.sh',
      powershell: 'install.ps1',
    }),
    controlScriptDir: 'control/bin',
    siteTargetDirs: Object.freeze({
      admin: 'sites/admin/dist',
      portal: 'sites/portal/dist',
    }),
    bootstrapDataRootDirs: Object.freeze({
      data: 'data',
    }),
    deploymentAssetRootDirs: Object.freeze({
      deploy: 'deploy',
    }),
    releaseManifestFile: 'release-manifest.json',
    readmeFile: 'README.txt',
  }),
];

function cloneNativeRuntimeLayoutSpec(spec) {
  return {
    ...spec,
    ...(spec.bundleInstallers ? { bundleInstallers: { ...spec.bundleInstallers } } : {}),
    siteTargetDirs: { ...spec.siteTargetDirs },
    bootstrapDataRootDirs: { ...spec.bootstrapDataRootDirs },
    deploymentAssetRootDirs: { ...spec.deploymentAssetRootDirs },
  };
}

const nativeRuntimeLayoutCatalog = createStrictKeyedCatalog({
  entries: NATIVE_RUNTIME_LAYOUT_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneNativeRuntimeLayoutSpec,
  duplicateKeyMessagePrefix: 'duplicate native runtime layout spec',
  missingKeyMessagePrefix: 'missing native runtime layout spec',
});

export function listNativeRuntimeLayoutSpecs() {
  return nativeRuntimeLayoutCatalog.list();
}

export function findNativeRuntimeLayoutSpec(layoutId) {
  return nativeRuntimeLayoutCatalog.find(layoutId);
}

export function listNativeRuntimeLayoutSpecsByIds(layoutIds = []) {
  return nativeRuntimeLayoutCatalog.listByKeys(layoutIds);
}

export function findNativePortalDesktopEmbeddedRuntimeLayoutSpec() {
  return findNativeRuntimeLayoutSpec('portal-desktop-embedded-runtime');
}

export function findNativeProductServerBundleRuntimeLayoutSpec() {
  return findNativeRuntimeLayoutSpec('product-server-bundle');
}
