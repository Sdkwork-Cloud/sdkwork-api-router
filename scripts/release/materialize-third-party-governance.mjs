#!/usr/bin/env node

import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_NODE_PACKAGE_ROOTS = [
  path.join('apps', 'sdkwork-router-admin'),
  path.join('apps', 'sdkwork-router-portal'),
];

function toPortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/');
}

function ensureDirectory(directoryPath) {
  mkdirSync(directoryPath, { recursive: true });
}

function writeJsonFile(filePath, payload) {
  ensureDirectory(path.dirname(filePath));
  writeFileSync(filePath, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
}

function normalizeLicenseExpression(value) {
  if (typeof value === 'string' && value.trim().length > 0) {
    return value.trim();
  }

  if (Array.isArray(value)) {
    const normalizedItems = value
      .map((entry) => normalizeLicenseExpression(entry))
      .filter((entry) => entry !== 'NOASSERTION');
    return normalizedItems.length > 0 ? normalizedItems.join(' OR ') : 'NOASSERTION';
  }

  if (value && typeof value === 'object') {
    if (typeof value.type === 'string' && value.type.trim().length > 0) {
      return value.type.trim();
    }
    if (typeof value.name === 'string' && value.name.trim().length > 0) {
      return value.name.trim();
    }
  }

  return 'NOASSERTION';
}

function normalizeRepositoryUrl(value) {
  if (typeof value === 'string' && value.trim().length > 0) {
    return value.trim();
  }

  if (value && typeof value === 'object' && typeof value.url === 'string' && value.url.trim().length > 0) {
    return value.url.trim();
  }

  return '';
}

function normalizeDownloadLocation(candidateUrls = []) {
  for (const candidateUrl of candidateUrls) {
    const normalized = String(candidateUrl ?? '').trim();
    if (normalized.length > 0) {
      return normalized;
    }
  }

  return 'NOASSERTION';
}

function sanitizeSpdxIdSegment(value) {
  return String(value ?? '')
    .replace(/[^A-Za-z0-9.\-]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 120) || 'unknown';
}

function buildPackageSpdxId(pkg) {
  return [
    'SPDXRef',
    sanitizeSpdxIdSegment(pkg.ecosystem),
    sanitizeSpdxIdSegment(pkg.name),
    sanitizeSpdxIdSegment(pkg.version),
  ].join('-');
}

function encodePurlName(name) {
  return String(name ?? '').replace(/^@/, '%40').replace(/\//g, '%2F');
}

function buildPackagePurl(pkg) {
  if (pkg.ecosystem === 'cargo') {
    return `pkg:cargo/${pkg.name}@${pkg.version}`;
  }

  return `pkg:npm/${encodePurlName(pkg.name)}@${pkg.version}`;
}

function isFirstPartyNodePackage(packageName) {
  const normalizedName = String(packageName ?? '').trim().toLowerCase();
  return normalizedName.startsWith('sdkwork-')
    || normalizedName.startsWith('@sdkwork/')
    || normalizedName.startsWith('@sdkwork-cloud/')
    || normalizedName.startsWith('sdkwork-router-');
}

function listFilesRecursively(rootPath, predicate, relativePrefix = '') {
  if (!existsSync(rootPath)) {
    return [];
  }

  const files = [];
  for (const entry of readdirSync(rootPath, { withFileTypes: true })) {
    const relativePath = path.join(relativePrefix, entry.name);
    const absolutePath = path.join(rootPath, entry.name);

    if (entry.isDirectory()) {
      files.push(...listFilesRecursively(absolutePath, predicate, relativePath));
      continue;
    }

    if (entry.isFile() && predicate(absolutePath, relativePath)) {
      files.push(absolutePath);
    }
  }

  return files.sort((left, right) => left.localeCompare(right));
}

function listNodePackageJsonFiles(packageRoot) {
  const nodeModulesRoot = path.join(packageRoot, 'node_modules');
  return listFilesRecursively(
    nodeModulesRoot,
    (_absolutePath, relativePath) => {
      const portableRelativePath = toPortablePath(relativePath);
      return portableRelativePath.endsWith('/package.json')
        && !portableRelativePath.includes('/.bin/')
        && !portableRelativePath.startsWith('.bin/')
        && !portableRelativePath.includes('/@types/node/');
    },
  );
}

function collectNodePackages({
  repoRoot = rootDir,
  packageRoots = DEFAULT_NODE_PACKAGE_ROOTS.map((relativePath) => path.join(rootDir, relativePath)),
} = {}) {
  const packageMap = new Map();

  for (const packageRoot of packageRoots) {
    if (!existsSync(packageRoot)) {
      continue;
    }

    for (const packageJsonPath of listNodePackageJsonFiles(packageRoot)) {
      const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf8'));
      const packageName = String(packageJson.name ?? '').trim();
      const packageVersion = String(packageJson.version ?? '').trim();

      if (!packageName || !packageVersion || isFirstPartyNodePackage(packageName)) {
        continue;
      }

      const packageKey = `npm:${packageName}@${packageVersion}`;
      if (packageMap.has(packageKey)) {
        continue;
      }

      const packageDir = path.dirname(packageJsonPath);
      const licenseFileCandidates = [
        'LICENSE',
        'LICENSE.md',
        'LICENSE.txt',
        'LICENCE',
        'LICENCE.md',
        'LICENCE.txt',
        'NOTICE',
        'NOTICE.md',
        'NOTICE.txt',
      ]
        .map((fileName) => path.join(packageDir, fileName))
        .filter((candidatePath) => existsSync(candidatePath))
        .map((candidatePath) => toPortablePath(path.relative(repoRoot, candidatePath)));

      packageMap.set(packageKey, {
        ecosystem: 'npm',
        name: packageName,
        version: packageVersion,
        licenseDeclared: normalizeLicenseExpression(
          packageJson.license ?? packageJson.licenses ?? 'NOASSERTION',
        ),
        downloadLocation: normalizeDownloadLocation([
          packageJson.homepage,
          normalizeRepositoryUrl(packageJson.repository),
        ]),
        repositoryUrl: normalizeRepositoryUrl(packageJson.repository),
        homepage: String(packageJson.homepage ?? '').trim(),
        sourcePath: toPortablePath(path.relative(repoRoot, packageJsonPath)),
        noticeFiles: licenseFileCandidates,
      });
    }
  }

  return [...packageMap.values()];
}

function parseCargoLockFile(lockText) {
  const packages = [];
  const lines = String(lockText ?? '').split(/\r?\n/u);
  let currentPackage = null;

  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (line === '[[package]]') {
      if (currentPackage) {
        packages.push(currentPackage);
      }
      currentPackage = {};
      continue;
    }

    if (!currentPackage || line.length === 0 || line.startsWith('#')) {
      continue;
    }

    const match = line.match(/^([A-Za-z0-9_-]+)\s*=\s*"(.+)"$/u);
    if (!match) {
      continue;
    }

    const [, key, value] = match;
    currentPackage[key] = value;
  }

  if (currentPackage) {
    packages.push(currentPackage);
  }

  return packages;
}

function parseCargoTomlMetadata(cargoTomlText) {
  const metadata = {};
  let inPackageSection = false;

  for (const rawLine of String(cargoTomlText ?? '').split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (line.length === 0 || line.startsWith('#')) {
      continue;
    }

    if (line.startsWith('[')) {
      inPackageSection = line === '[package]';
      continue;
    }

    if (!inPackageSection) {
      continue;
    }

    const match = line.match(/^([A-Za-z0-9_-]+)\s*=\s*"(.+)"$/u);
    if (!match) {
      continue;
    }

    const [, key, value] = match;
    metadata[key] = value;
  }

  return metadata;
}

function collectCargoPackages({
  repoRoot = rootDir,
  cargoLockPath = path.join(repoRoot, 'Cargo.lock'),
  vendorRoot = path.join(repoRoot, 'vendor'),
} = {}) {
  if (!existsSync(cargoLockPath)) {
    return [];
  }

  const packageMap = new Map();
  const cargoPackages = parseCargoLockFile(readFileSync(cargoLockPath, 'utf8'));

  for (const cargoPackage of cargoPackages) {
    const packageName = String(cargoPackage.name ?? '').trim();
    const packageVersion = String(cargoPackage.version ?? '').trim();
    const source = String(cargoPackage.source ?? '').trim();

    if (!packageName || !packageVersion || !source.startsWith('registry+')) {
      continue;
    }

    const packageKey = `cargo:${packageName}@${packageVersion}`;
    if (packageMap.has(packageKey)) {
      continue;
    }

    const vendorDir = path.join(vendorRoot, `${packageName}-${packageVersion}`);
    const cargoTomlPath = path.join(vendorDir, 'Cargo.toml');
    const cargoMetadata = existsSync(cargoTomlPath)
      ? parseCargoTomlMetadata(readFileSync(cargoTomlPath, 'utf8'))
      : {};
    const noticeFiles = [
      'LICENSE',
      'LICENSE-MIT',
      'LICENSE-APACHE',
      'COPYING',
      'NOTICE',
    ]
      .map((fileName) => path.join(vendorDir, fileName))
      .filter((candidatePath) => existsSync(candidatePath))
      .map((candidatePath) => toPortablePath(path.relative(repoRoot, candidatePath)));

    packageMap.set(packageKey, {
      ecosystem: 'cargo',
      name: packageName,
      version: packageVersion,
      licenseDeclared: normalizeLicenseExpression(cargoMetadata.license),
      downloadLocation: normalizeDownloadLocation([
        cargoMetadata.repository,
        cargoMetadata.homepage,
        'https://crates.io/',
      ]),
      repositoryUrl: String(cargoMetadata.repository ?? '').trim(),
      homepage: String(cargoMetadata.homepage ?? '').trim(),
      sourcePath: existsSync(cargoTomlPath)
        ? toPortablePath(path.relative(repoRoot, cargoTomlPath))
        : toPortablePath(path.relative(repoRoot, vendorDir)),
      noticeFiles,
    });
  }

  return [...packageMap.values()];
}

function compareGovernancePackages(left, right) {
  return [
    left.ecosystem.localeCompare(right.ecosystem),
    left.name.localeCompare(right.name),
    left.version.localeCompare(right.version),
  ].find((result) => result !== 0) ?? 0;
}

function createThirdPartyNoticesArtifact({
  packages = [],
  generatedAt = new Date().toISOString(),
} = {}) {
  const sortedPackages = [...packages].sort(compareGovernancePackages);
  const cargoPackages = sortedPackages.filter((pkg) => pkg.ecosystem === 'cargo');
  const npmPackages = sortedPackages.filter((pkg) => pkg.ecosystem === 'npm');

  const renderPackageLine = (pkg) =>
    `- ${pkg.name} ${pkg.version} [${pkg.licenseDeclared}]${pkg.downloadLocation !== 'NOASSERTION' ? ` ${pkg.downloadLocation}` : ''}`;

  const noticeText = [
    'SDKWork API Router Third-Party Notices',
    '',
    `Generated: ${generatedAt}`,
    `Package count: ${sortedPackages.length}`,
    '',
    'Cargo crates:',
    ...(cargoPackages.length > 0 ? cargoPackages.map(renderPackageLine) : ['- none']),
    '',
    'NPM packages:',
    ...(npmPackages.length > 0 ? npmPackages.map(renderPackageLine) : ['- none']),
    '',
  ].join('\n');

  return {
    version: 1,
    generatedAt,
    packageCount: sortedPackages.length,
    cargoPackageCount: cargoPackages.length,
    npmPackageCount: npmPackages.length,
    packages: sortedPackages.map((pkg) => ({
      ecosystem: pkg.ecosystem,
      name: pkg.name,
      version: pkg.version,
      licenseDeclared: pkg.licenseDeclared,
      downloadLocation: pkg.downloadLocation,
      repositoryUrl: pkg.repositoryUrl,
      homepage: pkg.homepage,
      sourcePath: pkg.sourcePath,
      noticeFiles: [...pkg.noticeFiles],
    })),
    noticeText,
  };
}

function createSpdxPackage(pkg) {
  const packageSpdxId = buildPackageSpdxId(pkg);

  return {
    SPDXID: packageSpdxId,
    name: pkg.name,
    versionInfo: pkg.version,
    supplier: 'NOASSERTION',
    downloadLocation: pkg.downloadLocation,
    filesAnalyzed: false,
    licenseConcluded: 'NOASSERTION',
    licenseDeclared: pkg.licenseDeclared,
    externalRefs: [
      {
        referenceCategory: 'PACKAGE-MANAGER',
        referenceType: 'purl',
        referenceLocator: buildPackagePurl(pkg),
      },
    ],
  };
}

function createThirdPartySbomArtifact({
  packages = [],
  generatedAt = new Date().toISOString(),
} = {}) {
  const sortedPackages = [...packages].sort(compareGovernancePackages);
  const rootPackageId = 'SPDXRef-Package-sdkwork-api-router-third-party-governance';

  return {
    spdxVersion: 'SPDX-2.3',
    dataLicense: 'CC0-1.0',
    SPDXID: 'SPDXRef-DOCUMENT',
    name: 'sdkwork-api-router-third-party-sbom',
    documentNamespace: `https://sdkwork.local/spdx/sdkwork-api-router/${encodeURIComponent(generatedAt)}`,
    creationInfo: {
      created: generatedAt,
      creators: ['Tool: sdkwork-api-router/scripts/release/materialize-third-party-governance.mjs'],
    },
    documentDescribes: [rootPackageId],
    packages: [
      {
        SPDXID: rootPackageId,
        name: 'sdkwork-api-router-third-party-governance',
        versionInfo: generatedAt,
        supplier: 'Organization: SDKWork',
        downloadLocation: 'NOASSERTION',
        filesAnalyzed: false,
        licenseConcluded: 'NOASSERTION',
        licenseDeclared: 'NOASSERTION',
      },
      ...sortedPackages.map((pkg) => createSpdxPackage(pkg)),
    ],
    relationships: [
      {
        spdxElementId: 'SPDXRef-DOCUMENT',
        relationshipType: 'DESCRIBES',
        relatedSpdxElement: rootPackageId,
      },
      ...sortedPackages.map((pkg) => ({
        spdxElementId: rootPackageId,
        relationshipType: 'DEPENDS_ON',
        relatedSpdxElement: buildPackageSpdxId(pkg),
      })),
    ],
  };
}

export function validateThirdPartySbomArtifact(payload) {
  if (!payload || typeof payload !== 'object') {
    throw new Error('third-party SBOM artifact must be an object');
  }

  if (payload.spdxVersion !== 'SPDX-2.3') {
    throw new Error('third-party SBOM artifact must declare SPDX-2.3');
  }

  if (!Array.isArray(payload.packages)) {
    throw new Error('third-party SBOM artifact must contain packages');
  }

  if (!payload.creationInfo || typeof payload.creationInfo.created !== 'string' || payload.creationInfo.created.trim().length === 0) {
    throw new Error('third-party SBOM artifact must contain creationInfo.created');
  }

  return {
    packageCount: payload.packages.length,
    spdxVersion: payload.spdxVersion,
  };
}

export function validateThirdPartyNoticesArtifact(payload) {
  if (!payload || typeof payload !== 'object') {
    throw new Error('third-party notices artifact must be an object');
  }

  if (payload.version !== 1) {
    throw new Error('third-party notices artifact must declare version 1');
  }

  if (!Array.isArray(payload.packages)) {
    throw new Error('third-party notices artifact must contain a packages array');
  }

  if (typeof payload.noticeText !== 'string' || payload.noticeText.trim().length === 0) {
    throw new Error('third-party notices artifact must contain noticeText');
  }

  if (typeof payload.packageCount !== 'number') {
    throw new Error('third-party notices artifact must contain packageCount');
  }

  return {
    packageCount: payload.packageCount,
    noticeLength: payload.noticeText.length,
  };
}

export function materializeThirdPartyGovernance({
  repoRoot = rootDir,
  generatedAt = new Date().toISOString(),
  sbomOutputPath = path.join(repoRoot, 'docs', 'release', 'third-party-sbom-latest.spdx.json'),
  noticesOutputPath = path.join(repoRoot, 'docs', 'release', 'third-party-notices-latest.json'),
  nodePackageRoots = DEFAULT_NODE_PACKAGE_ROOTS.map((relativePath) => path.join(repoRoot, relativePath)),
  cargoLockPath = path.join(repoRoot, 'Cargo.lock'),
  vendorRoot = path.join(repoRoot, 'vendor'),
} = {}) {
  const packages = [
    ...collectCargoPackages({
      repoRoot,
      cargoLockPath,
      vendorRoot,
    }),
    ...collectNodePackages({
      repoRoot,
      packageRoots: nodePackageRoots,
    }),
  ].sort(compareGovernancePackages);

  const sbom = createThirdPartySbomArtifact({
    packages,
    generatedAt,
  });
  const notices = createThirdPartyNoticesArtifact({
    packages,
    generatedAt,
  });

  validateThirdPartySbomArtifact(sbom);
  validateThirdPartyNoticesArtifact(notices);

  writeJsonFile(sbomOutputPath, sbom);
  writeJsonFile(noticesOutputPath, notices);

  return {
    generatedAt,
    packageCount: packages.length,
    sbomOutputPath,
    noticesOutputPath,
  };
}

function parseArgs(argv = process.argv.slice(2)) {
  const options = {};

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === '--repo-root') {
      options.repoRoot = path.resolve(String(argv[index + 1] ?? '').trim());
      index += 1;
      continue;
    }
    if (token === '--generated-at') {
      options.generatedAt = String(argv[index + 1] ?? '').trim();
      index += 1;
      continue;
    }
    if (token === '--sbom-output') {
      options.sbomOutputPath = path.resolve(String(argv[index + 1] ?? '').trim());
      index += 1;
      continue;
    }
    if (token === '--notices-output') {
      options.noticesOutputPath = path.resolve(String(argv[index + 1] ?? '').trim());
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  return options;
}

function runCli() {
  const result = materializeThirdPartyGovernance(parseArgs());
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
