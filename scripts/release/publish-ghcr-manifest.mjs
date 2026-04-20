#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

const DEFAULT_TIMEOUT_MS = 5 * 60 * 1000;

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

function toPortablePath(value) {
  return String(value ?? '').replaceAll('\\', '/');
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

function parseManifestDigest(inspectText = '') {
  const match = String(inspectText ?? '').match(/Digest:\s*(sha256:[a-f0-9]+)/iu);
  return match ? match[1] : '';
}

function parseManifestPayload(rawManifestText = '') {
  const normalizedRawManifestText = String(rawManifestText ?? '').trim();
  if (!normalizedRawManifestText) {
    return {
      manifest: null,
      platformCount: 0,
      mediaType: '',
      digest: '',
    };
  }

  const manifest = JSON.parse(normalizedRawManifestText);
  return {
    manifest,
    platformCount: Array.isArray(manifest?.manifests) ? manifest.manifests.length : 0,
    mediaType: String(manifest?.mediaType ?? '').trim(),
    digest: `sha256:${createHash('sha256').update(normalizedRawManifestText).digest('hex')}`,
  };
}

export function createGhcrManifestPublishPlan({
  releaseTag,
  imageRepository = '',
  targetImageTag = '',
  sourceImageRefs = [],
  env = process.env,
} = {}) {
  const normalizedReleaseTag = String(releaseTag ?? '').trim();
  if (normalizedReleaseTag.length === 0) {
    throw new Error('releaseTag is required');
  }

  const resolvedImageRepository = resolveImageRepository({
    imageRepository,
    env,
  });
  const resolvedTargetImageTag = String(targetImageTag ?? '').trim() || normalizedReleaseTag;
  const resolvedSourceImageRefs = sourceImageRefs.length > 0
    ? sourceImageRefs.map((value) => String(value ?? '').trim()).filter(Boolean)
    : [
      `${resolvedImageRepository}:${normalizedReleaseTag}-linux-x64`,
      `${resolvedImageRepository}:${normalizedReleaseTag}-linux-arm64`,
    ];
  if (resolvedSourceImageRefs.length === 0) {
    throw new Error('at least one source image reference is required');
  }

  const targetImageRef = `${resolvedImageRepository}:${resolvedTargetImageTag}`;
  return {
    releaseTag: normalizedReleaseTag,
    imageRepository: resolvedImageRepository,
    targetImageTag: resolvedTargetImageTag,
    targetImageRef,
    sourceImageRefs: resolvedSourceImageRefs,
    createArgs: [
      'buildx',
      'imagetools',
      'create',
      '-t',
      targetImageRef,
      ...resolvedSourceImageRefs,
    ],
    inspectArgs: [
      'buildx',
      'imagetools',
      'inspect',
      targetImageRef,
    ],
    inspectRawArgs: [
      'buildx',
      'imagetools',
      'inspect',
      targetImageRef,
      '--raw',
    ],
  };
}

export function createGhcrManifestPublishMetadata({
  releaseTag,
  imageRepository,
  targetImageTag,
  targetImageRef,
  sourceImageRefs = [],
  inspectText = '',
  rawManifestText = '',
} = {}) {
  const parsedManifest = parseManifestPayload(rawManifestText);
  const digest = parseManifestDigest(inspectText) || parsedManifest.digest;

  return {
    version: 1,
    type: 'sdkwork-ghcr-image-manifest-publish',
    generatedAt: new Date().toISOString(),
    releaseTag,
    imageRepository,
    targetImageTag,
    targetImageRef,
    sourceImageRefs: sourceImageRefs.map((value) => String(value ?? '').trim()).filter(Boolean),
    digest,
    manifestMediaType: parsedManifest.mediaType,
    platformCount: parsedManifest.platformCount,
  };
}

export function parseArgs(argv = process.argv.slice(2)) {
  const options = {
    releaseTag: '',
    imageRepository: '',
    targetImageTag: '',
    sourceImageRefs: [],
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
    if (token === '--image-repository') {
      options.imageRepository = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--target-image-tag') {
      options.targetImageTag = readOptionValue(token, next);
      index += 1;
      continue;
    }
    if (token === '--source-image-ref') {
      options.sourceImageRefs.push(readOptionValue(token, next));
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

  return {
    releaseTag: String(options.releaseTag).trim(),
    imageRepository: options.imageRepository,
    targetImageTag: options.targetImageTag,
    sourceImageRefs: options.sourceImageRefs,
    metadataPath: options.metadataPath,
  };
}

export function publishGhcrManifest({
  repoRoot = rootDir,
  releaseTag,
  imageRepository = '',
  targetImageTag = '',
  sourceImageRefs = [],
  metadataPath = '',
  env = process.env,
  spawnSyncImpl = spawnSync,
  mkdir = mkdirSync,
  writeFile = writeFileSync,
} = {}) {
  const plan = createGhcrManifestPublishPlan({
    releaseTag,
    imageRepository,
    targetImageTag,
    sourceImageRefs,
    env,
  });
  const resolvedMetadataPath = resolveMetadataPath({
    repoRoot,
    metadataPath,
  });

  runCommand('docker', plan.createArgs, {
    cwd: repoRoot,
    env,
    label: `publish GHCR multi-arch manifest ${plan.targetImageRef}`,
    spawnSyncImpl,
  });
  const inspectResult = runCommand('docker', plan.inspectArgs, {
    cwd: repoRoot,
    env,
    label: `inspect GHCR multi-arch manifest ${plan.targetImageRef}`,
    spawnSyncImpl,
  });
  const inspectRawResult = runCommand('docker', plan.inspectRawArgs, {
    cwd: repoRoot,
    env,
    label: `inspect raw GHCR multi-arch manifest ${plan.targetImageRef}`,
    spawnSyncImpl,
  });

  const metadata = createGhcrManifestPublishMetadata({
    releaseTag: plan.releaseTag,
    imageRepository: plan.imageRepository,
    targetImageTag: plan.targetImageTag,
    targetImageRef: plan.targetImageRef,
    sourceImageRefs: plan.sourceImageRefs,
    inspectText: `${inspectResult.stdout ?? ''}\n${inspectResult.stderr ?? ''}`,
    rawManifestText: String(inspectRawResult.stdout ?? ''),
  });

  if (resolvedMetadataPath) {
    mkdir(path.dirname(resolvedMetadataPath), { recursive: true });
    writeFile(resolvedMetadataPath, `${JSON.stringify(metadata, null, 2)}\n`, 'utf8');
  }

  return metadata;
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
  const metadata = publishGhcrManifest(options);
  process.stdout.write(`${JSON.stringify(metadata, null, 2)}\n`);
}
