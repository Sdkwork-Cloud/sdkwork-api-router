#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import { buildNativeProductServerArchiveBaseName } from './package-release-assets.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_TIMEOUT_MS = 20 * 60 * 1000;

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `${text.slice(0, Math.max(0, maxLength - 12))}...[truncated]`;
}

function readOptionValue(token, next) {
  if (!next || next.startsWith('--')) {
    throw new Error(`${token} requires a value`);
  }

  return next;
}

function normalizePlatform(value) {
  const normalized = String(value ?? '').trim().toLowerCase();
  if (normalized === 'linux') {
    return normalized;
  }

  throw new Error(`publish-ghcr-image only supports linux bundles, received ${value}`);
}

function normalizeArch(value) {
  const normalized = String(value ?? '').trim().toLowerCase();
  if (normalized === 'x64' || normalized === 'arm64') {
    return normalized;
  }

  throw new Error(`unsupported linux image architecture: ${value}`);
}

function toPortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/');
}

function resolveBundlePath({
  repoRoot = rootDir,
  platform,
  arch,
  bundlePath = '',
} = {}) {
  const normalizedBundlePath = String(bundlePath ?? '').trim();
  if (normalizedBundlePath.length > 0) {
    return path.isAbsolute(normalizedBundlePath)
      ? normalizedBundlePath
      : path.resolve(repoRoot, normalizedBundlePath);
  }

  const archiveBaseName = buildNativeProductServerArchiveBaseName({
    platformId: platform,
    archId: arch,
  });
  return path.resolve(
    repoRoot,
    'artifacts',
    'release',
    'native',
    platform,
    arch,
    'bundles',
    `${archiveBaseName}.tar.gz`,
  );
}

function resolveMetadataPath({
  repoRoot = rootDir,
  metadataPath = '',
} = {}) {
  const normalizedMetadataPath = String(metadataPath ?? '').trim();
  if (normalizedMetadataPath.length === 0) {
    return '';
  }

  return path.isAbsolute(normalizedMetadataPath)
    ? normalizedMetadataPath
    : path.resolve(repoRoot, normalizedMetadataPath);
}

function resolveImageRepository({
  imageRepository = '',
  env = process.env,
} = {}) {
  const normalizedImageRepository = String(imageRepository ?? '').trim();
  if (normalizedImageRepository.length > 0) {
    return normalizedImageRepository.toLowerCase();
  }

  const repositoryOwner = String(
    env.GITHUB_REPOSITORY_OWNER
      || String(env.GITHUB_REPOSITORY ?? '').split('/')[0]
      || '',
  ).trim();
  if (repositoryOwner.length === 0) {
    throw new Error(
      'imageRepository is required unless GITHUB_REPOSITORY_OWNER or GITHUB_REPOSITORY is available',
    );
  }

  return `ghcr.io/${repositoryOwner.toLowerCase()}/sdkwork-api-router`;
}

function resolveImageTag({
  releaseTag,
  platform,
  arch,
  imageTag = '',
} = {}) {
  const normalizedImageTag = String(imageTag ?? '').trim();
  if (normalizedImageTag.length > 0) {
    return normalizedImageTag;
  }

  return `${releaseTag}-${platform}-${arch}`;
}

function buildCommandFailure(label, result) {
  const fragments = [];
  if (result?.error) {
    fragments.push(`error: ${result.error.message}`);
  }
  if (String(result?.stdout ?? '').trim()) {
    fragments.push(`stdout: ${truncateText(result.stdout)}`);
  }
  if (String(result?.stderr ?? '').trim()) {
    fragments.push(`stderr: ${truncateText(result.stderr)}`);
  }

  return new Error(
    `${label} failed with exit code ${result?.status ?? 'unknown'}${fragments.length > 0 ? `\n${fragments.join('\n')}` : ''}`,
  );
}

function runCommand(command, args, {
  cwd = rootDir,
  env = process.env,
  label = `${command} ${args.join(' ')}`,
  timeoutMs = DEFAULT_TIMEOUT_MS,
  spawnSyncImpl = spawnSync,
} = {}) {
  const result = spawnSyncImpl(command, args, {
    cwd,
    env,
    encoding: 'utf8',
    shell: false,
    timeout: timeoutMs,
  });

  if (result.error || result.status !== 0) {
    throw buildCommandFailure(label, result);
  }

  return result;
}

function extractArchive({
  bundlePath,
  extractRoot,
  cwd = rootDir,
  env = process.env,
  spawnSyncImpl = spawnSync,
} = {}) {
  runCommand('tar', ['-xzf', bundlePath, '-C', extractRoot], {
    cwd,
    env,
    label: 'extract packaged server bundle',
    spawnSyncImpl,
  });
}

function resolveExtractedBundleRoot({
  extractRoot,
  bundlePath,
} = {}) {
  const expectedBundleRoot = path.join(
    extractRoot,
    path.basename(bundlePath).replace(/\.tar\.gz$/u, ''),
  );
  if (existsSync(expectedBundleRoot) && statSync(expectedBundleRoot).isDirectory()) {
    return expectedBundleRoot;
  }

  const extractedDirectories = readdirSync(extractRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(extractRoot, entry.name));
  if (extractedDirectories.length === 1) {
    return extractedDirectories[0];
  }

  throw new Error(
    `unable to resolve extracted product bundle root under ${extractRoot}; expected ${expectedBundleRoot}`,
  );
}

function resolvePublishedDigest({
  imageRef,
  imageRepository,
  pushOutput = '',
  cwd = rootDir,
  env = process.env,
  spawnSyncImpl = spawnSync,
} = {}) {
  const pushDigestMatch = String(pushOutput ?? '').match(/digest:\s*(sha256:[a-f0-9]+)/iu);
  if (pushDigestMatch) {
    return pushDigestMatch[1];
  }

  const inspectResult = runCommand(
    'docker',
    ['image', 'inspect', imageRef, '--format', '{{json .RepoDigests}}'],
    {
      cwd,
      env,
      label: `inspect pushed image digest for ${imageRef}`,
      spawnSyncImpl,
    },
  );
  const repoDigests = JSON.parse(String(inspectResult.stdout ?? '[]').trim() || '[]');
  const digestEntry = Array.isArray(repoDigests)
    ? repoDigests.find((entry) => String(entry).startsWith(`${imageRepository}@sha256:`))
    : '';
  if (!digestEntry) {
    throw new Error(`unable to resolve pushed image digest for ${imageRef}`);
  }

  return String(digestEntry).split('@')[1];
}

export function createImagePublishMetadata({
  releaseTag,
  platform,
  arch,
  bundlePath,
  imageRepository,
  imageTag,
  imageRef,
  digest,
} = {}) {
  return {
    version: 1,
    type: 'sdkwork-ghcr-image-publish',
    generatedAt: new Date().toISOString(),
    releaseTag,
    platform,
    arch,
    bundlePath: toPortablePath(bundlePath),
    imageRepository,
    imageTag,
    imageRef,
    digest,
  };
}

export function createGhcrImagePublishPlan({
  repoRoot = rootDir,
  releaseTag,
  platform,
  arch,
  bundlePath = '',
  imageRepository = '',
  imageTag = '',
  metadataPath = '',
  env = process.env,
} = {}) {
  const normalizedReleaseTag = String(releaseTag ?? '').trim();
  if (normalizedReleaseTag.length === 0) {
    throw new Error('releaseTag is required');
  }

  const normalizedPlatform = normalizePlatform(platform);
  const normalizedArch = normalizeArch(arch);
  const resolvedBundlePath = resolveBundlePath({
    repoRoot,
    platform: normalizedPlatform,
    arch: normalizedArch,
    bundlePath,
  });
  const resolvedImageRepository = resolveImageRepository({
    imageRepository,
    env,
  });
  const resolvedImageTag = resolveImageTag({
    releaseTag: normalizedReleaseTag,
    platform: normalizedPlatform,
    arch: normalizedArch,
    imageTag,
  });
  const imageRef = `${resolvedImageRepository}:${resolvedImageTag}`;
  const resolvedMetadataPath = resolveMetadataPath({
    repoRoot,
    metadataPath,
  });

  return {
    releaseTag: normalizedReleaseTag,
    platform: normalizedPlatform,
    arch: normalizedArch,
    bundlePath: resolvedBundlePath,
    imageRepository: resolvedImageRepository,
    imageTag: resolvedImageTag,
    imageRef,
    metadataPath: resolvedMetadataPath,
    buildArgs: [
      'build',
      '-f',
      'deploy/docker/Dockerfile',
      '-t',
      imageRef,
      '.',
    ],
    pushArgs: [
      'push',
      imageRef,
    ],
    inspectArgs: [
      'image',
      'inspect',
      imageRef,
      '--format',
      '{{json .RepoDigests}}',
    ],
  };
}

export function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    releaseTag: '',
    platform: '',
    arch: '',
    bundlePath: '',
    imageRepository: '',
    imageTag: '',
    metadataPath: '',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--release-tag') {
      options.releaseTag = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--platform') {
      options.platform = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--arch') {
      options.arch = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--bundle-path') {
      options.bundlePath = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--image-repository') {
      options.imageRepository = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--image-tag') {
      options.imageTag = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--metadata-path') {
      options.metadataPath = readOptionValue(token, next);
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!options.releaseTag) {
    throw new Error('--release-tag is required');
  }
  if (!options.platform) {
    throw new Error('--platform is required');
  }
  if (!options.arch) {
    throw new Error('--arch is required');
  }

  return {
    releaseTag: String(options.releaseTag).trim(),
    platform: normalizePlatform(options.platform),
    arch: normalizeArch(options.arch),
    bundlePath: options.bundlePath,
    imageRepository: options.imageRepository,
    imageTag: options.imageTag,
    metadataPath: options.metadataPath,
  };
}

export function publishGhcrImage({
  repoRoot = rootDir,
  releaseTag,
  platform,
  arch,
  bundlePath = '',
  imageRepository = '',
  imageTag = '',
  metadataPath = '',
  env = process.env,
  spawnSyncImpl = spawnSync,
  exists = existsSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
  mkdtemp = mkdtempSync,
  rm = rmSync,
} = {}) {
  const plan = createGhcrImagePublishPlan({
    repoRoot,
    releaseTag,
    platform,
    arch,
    bundlePath,
    imageRepository,
    imageTag,
    metadataPath,
    env,
  });
  if (!exists(plan.bundlePath)) {
    throw new Error(`missing packaged product bundle: ${plan.bundlePath}`);
  }

  const extractRoot = mkdtemp(path.join(os.tmpdir(), 'sdkwork-router-ghcr-'));
  try {
    extractArchive({
      bundlePath: plan.bundlePath,
      extractRoot,
      cwd: repoRoot,
      env,
      spawnSyncImpl,
    });
    const bundleRoot = resolveExtractedBundleRoot({
      extractRoot,
      bundlePath: plan.bundlePath,
    });
    const dockerfilePath = path.join(bundleRoot, 'deploy', 'docker', 'Dockerfile');
    if (!exists(dockerfilePath)) {
      throw new Error(`missing Dockerfile in extracted bundle: ${dockerfilePath}`);
    }

    runCommand('docker', plan.buildArgs, {
      cwd: bundleRoot,
      env,
      label: `build GHCR image ${plan.imageRef}`,
      spawnSyncImpl,
    });
    const pushResult = runCommand('docker', plan.pushArgs, {
      cwd: bundleRoot,
      env,
      label: `push GHCR image ${plan.imageRef}`,
      spawnSyncImpl,
    });
    const digest = resolvePublishedDigest({
      imageRef: plan.imageRef,
      imageRepository: plan.imageRepository,
      pushOutput: `${pushResult.stdout ?? ''}\n${pushResult.stderr ?? ''}`,
      cwd: bundleRoot,
      env,
      spawnSyncImpl,
    });

    const metadata = createImagePublishMetadata({
      releaseTag: plan.releaseTag,
      platform: plan.platform,
      arch: plan.arch,
      bundlePath: plan.bundlePath,
      imageRepository: plan.imageRepository,
      imageTag: plan.imageTag,
      imageRef: plan.imageRef,
      digest,
    });

    if (plan.metadataPath) {
      mkdir(path.dirname(plan.metadataPath), { recursive: true });
      writeFile(plan.metadataPath, `${JSON.stringify(metadata, null, 2)}\n`, 'utf8');
    }

    return metadata;
  } finally {
    rm(extractRoot, { recursive: true, force: true });
  }
}

function isDirectExecution({
  argv1 = process.argv[1] ?? '',
  moduleFile = __filename,
  platform = process.platform,
} = {}) {
  if (!argv1) {
    return false;
  }

  const resolvedArgv1 = path.resolve(argv1);
  const resolvedModuleFile = path.resolve(moduleFile);
  if (platform === 'win32') {
    return resolvedArgv1.toLowerCase() === resolvedModuleFile.toLowerCase();
  }

  return resolvedArgv1 === resolvedModuleFile;
}

if (isDirectExecution()) {
  const options = parseArgs();
  const metadata = publishGhcrImage(options);
  process.stdout.write(`${JSON.stringify(metadata, null, 2)}\n`);
}
