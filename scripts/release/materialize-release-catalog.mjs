#!/usr/bin/env node

import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

function toPortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/');
}

function resolveAssetsRoot(assetsRoot = path.join(rootDir, 'artifacts', 'release')) {
  return path.resolve(String(assetsRoot ?? '').trim() || path.join(rootDir, 'artifacts', 'release'));
}

function resolveOutputPath({
  assetsRoot = path.join(rootDir, 'artifacts', 'release'),
  outputPath,
} = {}) {
  const resolvedAssetsRoot = resolveAssetsRoot(assetsRoot);
  const normalizedOutputPath = String(outputPath ?? '').trim();
  if (normalizedOutputPath.length > 0) {
    return path.resolve(normalizedOutputPath);
  }

  return path.join(resolvedAssetsRoot, 'release-catalog.json');
}

function resolveGeneratedAt(generatedAt = '') {
  const normalizedGeneratedAt = String(generatedAt ?? '').trim();
  return normalizedGeneratedAt.length > 0
    ? normalizedGeneratedAt
    : new Date().toISOString();
}

function listFilesRecursively(sourceDir, relativePrefix = '') {
  const entries = readdirSync(sourceDir, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const relativePath = path.join(relativePrefix, entry.name);
    const absolutePath = path.join(sourceDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFilesRecursively(absolutePath, relativePath));
      continue;
    }
    if (entry.isFile()) {
      files.push({
        absolutePath,
        relativePath,
      });
    }
  }

  return files;
}

function parseChecksumFile({ checksumText, expectedFileName }) {
  const normalizedText = String(checksumText ?? '').trim();
  const match = /^(\S+)\s+\s*(\S+)$/.exec(normalizedText);
  if (!match) {
    throw new Error(`invalid checksum file format for ${expectedFileName}`);
  }

  const [, sha256, fileName] = match;
  if (fileName !== expectedFileName) {
    throw new Error(
      `checksum file target mismatch: expected ${expectedFileName}, received ${fileName}`,
    );
  }

  return sha256;
}

function resolveVariantDescriptor(manifest = {}) {
  if (manifest.type === 'product-server-archive') {
    return {
      variantKind: 'server-archive',
      primaryFile: String(manifest.archiveFile ?? '').trim(),
    };
  }

  if (manifest.type === 'portal-desktop-installer') {
    return {
      variantKind: 'desktop-installer',
      primaryFile: String(manifest.installerFile ?? '').trim(),
    };
  }

  throw new Error(`unsupported release asset manifest type: ${manifest.type}`);
}

function createCatalogEntryFromManifest({
  assetsRoot,
  manifestPath,
  manifest,
  readFile = readFileSync,
  exists = existsSync,
  statFile = statSync,
} = {}) {
  const outputDirectory = path.dirname(path.relative(assetsRoot, manifestPath));
  const manifestFile = path.basename(manifestPath);
  const variantDescriptor = resolveVariantDescriptor(manifest);
  const primaryFile = variantDescriptor.primaryFile;

  const checksumFile = String(manifest.checksumFile ?? '').trim();
  if (primaryFile.length === 0 || checksumFile.length === 0) {
    throw new Error(`release asset manifest ${manifestFile} must declare primary and checksum files`);
  }

  const primaryPath = path.join(path.dirname(manifestPath), primaryFile);
  const checksumPath = path.join(path.dirname(manifestPath), checksumFile);
  if (!exists(primaryPath)) {
    throw new Error(`release catalog primary asset is missing: ${primaryPath}`);
  }
  if (!exists(checksumPath)) {
    throw new Error(`release catalog checksum asset is missing: ${checksumPath}`);
  }
  const checksumText = readFile(checksumPath, 'utf8');
  const sha256 = parseChecksumFile({
    checksumText,
    expectedFileName: primaryFile,
  });
  const primaryFileSizeBytes = Number(statFile(primaryPath).size ?? 0);

  return {
    productId: String(manifest.productId ?? '').trim(),
    platform: String(manifest.platform ?? '').trim(),
    arch: String(manifest.arch ?? '').trim(),
    outputDirectory: toPortablePath(outputDirectory),
    variantKind: variantDescriptor.variantKind,
    primaryFile,
    primaryFileSizeBytes,
    checksumFile,
    checksumAlgorithm: 'sha256',
    manifestFile,
    sha256,
    manifest,
  };
}

export function collectReleaseCatalogEntries({
  assetsRoot = path.join(rootDir, 'artifacts', 'release'),
  readFile = readFileSync,
  statFile = statSync,
} = {}) {
  const resolvedAssetsRoot = resolveAssetsRoot(assetsRoot);
  const manifestFiles = listFilesRecursively(resolvedAssetsRoot)
    .filter((file) => file.relativePath.endsWith('.manifest.json'))
    .map((file) => file.absolutePath)
    .sort((left, right) => left.localeCompare(right));

  return manifestFiles.map((manifestPath) => createCatalogEntryFromManifest({
    assetsRoot: resolvedAssetsRoot,
    manifestPath,
    manifest: JSON.parse(readFile(manifestPath, 'utf8')),
    readFile,
    statFile,
  }));
}

export function createReleaseCatalog({
  releaseTag = '',
  entries = [],
  generatedAt = '',
} = {}) {
  const sortedEntries = [...entries].sort((left, right) => (
    left.productId.localeCompare(right.productId)
    || left.platform.localeCompare(right.platform)
    || left.arch.localeCompare(right.arch)
    || left.outputDirectory.localeCompare(right.outputDirectory)
  ));

  const groupedProducts = [];
  for (const entry of sortedEntries) {
    let product = groupedProducts.at(-1);
    if (!product || product.productId !== entry.productId) {
      product = {
        productId: entry.productId,
        variants: [],
      };
      groupedProducts.push(product);
    }

    product.variants.push({
      platform: entry.platform,
      arch: entry.arch,
      outputDirectory: entry.outputDirectory,
      variantKind: entry.variantKind,
      primaryFile: entry.primaryFile,
      primaryFileSizeBytes: entry.primaryFileSizeBytes,
      checksumFile: entry.checksumFile,
      checksumAlgorithm: entry.checksumAlgorithm,
      manifestFile: entry.manifestFile,
      sha256: entry.sha256,
      manifest: entry.manifest,
    });
  }

  return {
    version: 1,
    type: 'sdkwork-release-catalog',
    releaseTag: String(releaseTag ?? '').trim(),
    generatedAt: resolveGeneratedAt(generatedAt),
    productCount: groupedProducts.length,
    variantCount: sortedEntries.length,
    products: groupedProducts,
  };
}

export function readReleaseCatalogFile({
  releaseCatalogPath,
  readFile = readFileSync,
} = {}) {
  const normalizedReleaseCatalogPath = path.resolve(String(releaseCatalogPath ?? '').trim());
  if (normalizedReleaseCatalogPath.length === 0) {
    throw new Error('releaseCatalogPath is required');
  }

  const catalog = JSON.parse(String(readFile(normalizedReleaseCatalogPath, 'utf8')));
  if (catalog?.type !== 'sdkwork-release-catalog') {
    throw new Error(`unsupported release catalog type at ${normalizedReleaseCatalogPath}`);
  }

  return catalog;
}

export function findReleaseCatalogVariant(catalog, {
  productId,
  variantKind,
  platform,
  arch,
} = {}) {
  const normalizedProductId = String(productId ?? '').trim();
  const normalizedVariantKind = String(variantKind ?? '').trim();
  const normalizedPlatform = String(platform ?? '').trim();
  const normalizedArch = String(arch ?? '').trim();
  const matchingVariants = Array.isArray(catalog?.products)
    ? catalog.products
      .filter((product) => String(product?.productId ?? '').trim() === normalizedProductId)
      .flatMap((product) => Array.isArray(product?.variants) ? product.variants : [])
      .filter((variant) => (
        String(variant?.variantKind ?? '').trim() === normalizedVariantKind
        && String(variant?.platform ?? '').trim() === normalizedPlatform
        && String(variant?.arch ?? '').trim() === normalizedArch
      ))
    : [];

  if (matchingVariants.length === 0) {
    return null;
  }
  if (matchingVariants.length > 1) {
    throw new Error(
      `Multiple release-catalog variants found for ${normalizedProductId}/${normalizedVariantKind}/${normalizedPlatform}/${normalizedArch}`,
    );
  }

  return matchingVariants[0];
}

function assertReleaseCatalogPathWithinAssetsRoot({
  releaseCatalogPath,
  candidatePath,
  fieldName,
} = {}) {
  const releaseAssetsRoot = path.dirname(path.resolve(releaseCatalogPath));
  const normalizedCandidatePath = path.resolve(candidatePath);
  const relativePath = path.relative(releaseAssetsRoot, normalizedCandidatePath);
  if (relativePath.startsWith('..') || path.isAbsolute(relativePath)) {
    throw new Error(`release catalog ${fieldName} escapes assets root at ${releaseCatalogPath}`);
  }
}

export function resolveReleaseCatalogVariantPaths({
  releaseCatalogPath,
  productId,
  variantKind,
  platform,
  arch,
  readFile = readFileSync,
} = {}) {
  const normalizedReleaseCatalogPath = path.resolve(String(releaseCatalogPath ?? '').trim());
  const catalog = readReleaseCatalogFile({
    releaseCatalogPath: normalizedReleaseCatalogPath,
    readFile,
  });
  const variant = findReleaseCatalogVariant(catalog, {
    productId,
    variantKind,
    platform,
    arch,
  });

  if (!variant) {
    throw new Error(
      `Missing release-catalog variant for ${productId}/${variantKind}/${platform}/${arch}: ${normalizedReleaseCatalogPath}`,
    );
  }

  const outputDirectory = String(variant.outputDirectory ?? '').trim();
  const primaryFile = String(variant.primaryFile ?? '').trim();
  const manifestFile = String(variant.manifestFile ?? '').trim();
  const checksumFile = String(variant.checksumFile ?? '').trim();
  if (outputDirectory.length === 0 || primaryFile.length === 0 || manifestFile.length === 0 || checksumFile.length === 0) {
    throw new Error(`release catalog variant paths are incomplete for ${productId}/${variantKind}/${platform}/${arch}`);
  }

  const variantRoot = path.resolve(path.dirname(normalizedReleaseCatalogPath), outputDirectory);
  const primaryPath = path.resolve(variantRoot, primaryFile);
  const manifestPath = path.resolve(variantRoot, manifestFile);
  const checksumPath = path.resolve(variantRoot, checksumFile);

  assertReleaseCatalogPathWithinAssetsRoot({
    releaseCatalogPath: normalizedReleaseCatalogPath,
    candidatePath: variantRoot,
    fieldName: 'outputDirectory',
  });
  assertReleaseCatalogPathWithinAssetsRoot({
    releaseCatalogPath: normalizedReleaseCatalogPath,
    candidatePath: primaryPath,
    fieldName: 'primaryFile',
  });
  assertReleaseCatalogPathWithinAssetsRoot({
    releaseCatalogPath: normalizedReleaseCatalogPath,
    candidatePath: manifestPath,
    fieldName: 'manifestFile',
  });
  assertReleaseCatalogPathWithinAssetsRoot({
    releaseCatalogPath: normalizedReleaseCatalogPath,
    candidatePath: checksumPath,
    fieldName: 'checksumFile',
  });

  return {
    releaseCatalogPath: normalizedReleaseCatalogPath,
    catalog,
    variant,
    variantRoot,
    primaryPath,
    manifestPath,
    checksumPath,
  };
}

export function materializeReleaseCatalog({
  assetsRoot = path.join(rootDir, 'artifacts', 'release'),
  releaseTag = '',
  generatedAt = '',
  outputPath,
  readFile = readFileSync,
  writeFile = writeFileSync,
  mkdir = mkdirSync,
  statFile = statSync,
} = {}) {
  const resolvedAssetsRoot = resolveAssetsRoot(assetsRoot);
  const resolvedOutputPath = resolveOutputPath({
    assetsRoot: resolvedAssetsRoot,
    outputPath,
  });
  const entries = collectReleaseCatalogEntries({
    assetsRoot: resolvedAssetsRoot,
    readFile,
    statFile,
  });
  const catalog = createReleaseCatalog({
    releaseTag,
    entries,
    generatedAt,
  });

  mkdir(path.dirname(resolvedOutputPath), { recursive: true });
  writeFile(resolvedOutputPath, `${JSON.stringify(catalog, null, 2)}\n`, 'utf8');

  return {
    assetsRoot: resolvedAssetsRoot,
    outputPath: resolvedOutputPath,
    releaseTag: catalog.releaseTag,
    generatedAt: catalog.generatedAt,
    productCount: catalog.productCount,
    variantCount: catalog.variantCount,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  const options = {};

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--assets-root') {
      options.assetsRoot = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--output') {
      options.outputPath = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--release-tag') {
      options.releaseTag = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  return options;
}

function runCli() {
  const result = materializeReleaseCatalog(parseArgs());
  console.log(JSON.stringify(result, null, 2));
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  try {
    runCli();
  } catch (error) {
    console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    process.exit(1);
  }
}
