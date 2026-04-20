import { createStrictKeyedCatalog } from '../strict-contract-catalog.mjs';

const NATIVE_RELEASE_PRODUCT_SPECS = [
  Object.freeze({
    id: 'product-server',
    productId: 'sdkwork-api-router-product-server',
    packageKind: 'server-archive',
    baseNamePrefix: 'sdkwork-api-router-product-server',
    archiveFileExtension: '.tar.gz',
    outputPathSegments: Object.freeze(['bundles']),
    archiveManifestType: 'product-server-archive',
    embeddedManifestType: 'product-server-bundle',
  }),
  Object.freeze({
    id: 'portal-desktop',
    productId: 'sdkwork-router-portal-desktop',
    packageKind: 'desktop-installer',
    appId: 'portal',
    baseNamePrefix: 'sdkwork-router-portal-desktop',
    outputPathSegments: Object.freeze(['desktop', 'portal']),
    manifestType: 'portal-desktop-installer',
  }),
];

function cloneNativeReleaseProductSpec(spec) {
  return {
    ...spec,
    outputPathSegments: [...spec.outputPathSegments],
  };
}

const nativeReleaseProductSpecCatalog = createStrictKeyedCatalog({
  entries: NATIVE_RELEASE_PRODUCT_SPECS,
  getKey: (spec) => spec.id,
  clone: cloneNativeReleaseProductSpec,
  duplicateKeyMessagePrefix: 'duplicate native release product spec',
  missingKeyMessagePrefix: 'missing native release product spec',
});

const nativeDesktopReleaseProductAppCatalog = createStrictKeyedCatalog({
  entries: NATIVE_RELEASE_PRODUCT_SPECS
    .filter((spec) => typeof spec.appId === 'string' && spec.appId.length > 0)
    .map((spec) => Object.freeze({
      appId: spec.appId,
      productSpecId: spec.id,
    })),
  getKey: (spec) => spec.appId,
  duplicateKeyMessagePrefix: 'duplicate native desktop release product app id',
  missingKeyMessagePrefix: 'missing native desktop release product app id',
});

export function listNativeReleaseProductSpecs() {
  return nativeReleaseProductSpecCatalog.list();
}

export function findNativeReleaseProductSpec(productSpecId) {
  return nativeReleaseProductSpecCatalog.find(productSpecId);
}

export function listNativeReleaseProductSpecsByIds(productSpecIds = []) {
  return nativeReleaseProductSpecCatalog.listByKeys(productSpecIds);
}

export function findNativeDesktopReleaseProductSpecByAppId(appId) {
  const alias = nativeDesktopReleaseProductAppCatalog.find(appId);
  return findNativeReleaseProductSpec(alias.productSpecId);
}
