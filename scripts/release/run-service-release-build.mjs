#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

import {
  RELEASE_BINARY_NAMES,
  createReleaseBuildPlan,
  withExecutable,
} from '../../bin/lib/router-runtime-tooling.mjs';
import {
  buildDesktopReleaseEnv,
  resolveDesktopReleaseTarget,
} from './desktop-targets.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..', '..');

function truncateText(value, maxLength = 4000) {
  const text = String(value ?? '').trim();
  if (text.length <= maxLength) {
    return text;
  }

  return `[truncated]${text.slice(-Math.max(0, maxLength - 11))}`;
}

function escapeGitHubActionsCommandValue(value) {
  let escaped = String(value ?? '');
  escaped = escaped.replaceAll('%', '%25');
  escaped = escaped.replaceAll('\r', '%0D');
  escaped = escaped.replaceAll('\n', '%0A');
  return escaped;
}

function appendBufferedOutput(buffer, chunk, maxLength = 6000) {
  const text = Buffer.isBuffer(chunk) ? chunk.toString('utf8') : String(chunk ?? '');
  const next = `${buffer}${text}`;
  if (next.length <= maxLength) {
    return next;
  }

  return next.slice(-maxLength);
}

function describeDirectoryState(directoryPath, maxEntries = 10) {
  if (!existsSync(directoryPath)) {
    return `${directoryPath} [missing]`;
  }

  const sample = [];
  try {
    for (const entry of readdirSync(directoryPath, { withFileTypes: true }).slice(0, maxEntries)) {
      sample.push(entry.name);
    }
  } catch {
    return `${directoryPath} [exists]`;
  }

  const suffix = sample.length > 0 ? ` [exists: ${sample.join(', ')}]` : ' [exists, empty]';
  return `${directoryPath}${suffix}`;
}

export function parseCliArgs(argv = process.argv.slice(2)) {
  let targetTriple = '';

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];

    if (token === '--target') {
      targetTriple = String(next ?? '').trim();
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${token}`);
  }

  if (!targetTriple) {
    throw new Error('--target is required');
  }

  return {
    targetTriple,
  };
}

function findCargoReleaseBuildStep(releaseBuildPlan) {
  const cargoStep = releaseBuildPlan.steps.find((step) => step.label === 'cargo release build');
  if (!cargoStep) {
    throw new Error('release build plan did not expose the cargo release build step');
  }

  return cargoStep;
}

export function createServiceReleaseBuildPlan({
  repoRoot = rootDir,
  targetTriple,
  env = process.env,
  platform = process.platform,
} = {}) {
  const resolvedTargetTriple = String(targetTriple ?? '').trim();
  if (!resolvedTargetTriple) {
    throw new Error('targetTriple is required');
  }

  const target = resolveDesktopReleaseTarget({
    platform,
    targetTriple: resolvedTargetTriple,
    env,
  });
  const resolvedEnv = buildDesktopReleaseEnv({
    env,
    targetTriple: target.targetTriple,
  });
  const releaseBuildPlan = createReleaseBuildPlan({
    repoRoot,
    platform: target.platform,
    arch: target.arch,
    env: resolvedEnv,
    includeDocs: false,
    installDependencies: false,
    verifyRelease: false,
  });
  const cargoStep = findCargoReleaseBuildStep(releaseBuildPlan);

  return {
    ...cargoStep,
    repoRoot,
    releaseTarget: target,
  };
}

function resolveServiceReleaseOutputDir({ plan }) {
  const targetRoot = String(plan?.env?.CARGO_TARGET_DIR ?? '').trim() || path.join(plan.repoRoot, 'target');
  return path.join(targetRoot, plan.releaseTarget.targetTriple, 'release');
}

function verifyServiceReleaseBuildOutput({ plan }) {
  const releaseDir = resolveServiceReleaseOutputDir({ plan });
  const missingBinaries = RELEASE_BINARY_NAMES.filter((binaryName) => !existsSync(
    path.join(releaseDir, withExecutable(binaryName, plan.releaseTarget.platform)),
  ));

  if (missingBinaries.length > 0) {
    throw new Error(
      [
        `release service build completed without the full official binary set for ${plan.releaseTarget.targetTriple}`,
        `missing: ${missingBinaries.join(', ')}`,
        `release dir: ${describeDirectoryState(releaseDir)}`,
      ].join('\n'),
    );
  }

  return releaseDir;
}

export function buildServiceReleaseBuildFailureAnnotation({
  targetTriple = '',
  error,
} = {}) {
  const scope = String(targetTriple ?? '').trim();
  const message = truncateText(
    `${scope ? `[${scope}] ` : ''}${error instanceof Error ? error.message : String(error)}`,
    8000,
  );
  return `::error title=run-service-release-build::${escapeGitHubActionsCommandValue(message)}`;
}

function buildServiceReleaseFailure({
  reason,
  stdoutBuffer,
  stderrBuffer,
} = {}) {
  const details = [reason];
  const stderrTail = truncateText(stderrBuffer, 2000);
  const stdoutTail = truncateText(stdoutBuffer, 2000);
  if (stderrTail) {
    details.push(`stderr tail:\n${stderrTail}`);
  }
  if (stdoutTail) {
    details.push(`stdout tail:\n${stdoutTail}`);
  }

  return new Error(details.join('\n'));
}

function reportFailure({ targetTriple, error }) {
  if (process.env.GITHUB_ACTIONS === 'true') {
    console.error(buildServiceReleaseBuildFailureAnnotation({
      targetTriple,
      error,
    }));
  }
  console.error(error instanceof Error ? error.stack ?? error.message : String(error));
}

function runCli() {
  const options = parseCliArgs(process.argv.slice(2));
  const plan = createServiceReleaseBuildPlan({
    repoRoot: rootDir,
    targetTriple: options.targetTriple,
  });
  let stdoutBuffer = '';
  let stderrBuffer = '';
  const child = spawn(plan.command, plan.args, {
    cwd: plan.cwd,
    env: plan.env,
    stdio: ['inherit', 'pipe', 'pipe'],
    shell: plan.shell ?? false,
    windowsHide: plan.windowsHide ?? process.platform === 'win32',
  });

  child.stdout?.on('data', (chunk) => {
    stdoutBuffer = appendBufferedOutput(stdoutBuffer, chunk);
    process.stdout.write(chunk);
  });

  child.stderr?.on('data', (chunk) => {
    stderrBuffer = appendBufferedOutput(stderrBuffer, chunk);
    process.stderr.write(chunk);
  });

  child.on('error', (error) => {
    reportFailure({
      targetTriple: plan.releaseTarget.targetTriple,
      error: buildServiceReleaseFailure({
        reason: `[run-service-release-build] ${error.message}`,
        stdoutBuffer,
        stderrBuffer,
      }),
    });
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal) {
      reportFailure({
        targetTriple: plan.releaseTarget.targetTriple,
        error: buildServiceReleaseFailure({
          reason: `[run-service-release-build] build exited with signal ${signal}`,
          stdoutBuffer,
          stderrBuffer,
        }),
      });
      process.exit(1);
      return;
    }

    if ((code ?? 0) !== 0) {
      reportFailure({
        targetTriple: plan.releaseTarget.targetTriple,
        error: buildServiceReleaseFailure({
          reason: `[run-service-release-build] build exited with code ${code}`,
          stdoutBuffer,
          stderrBuffer,
        }),
      });
      process.exit(code ?? 1);
      return;
    }

    try {
      const releaseDir = verifyServiceReleaseBuildOutput({ plan });
      console.error(`[run-service-release-build] release dir: ${releaseDir}`);
    } catch (error) {
      reportFailure({
        targetTriple: plan.releaseTarget.targetTriple,
        error,
      });
      process.exit(1);
      return;
    }

    process.exit(0);
  });
}

if (path.resolve(process.argv[1] ?? '') === __filename) {
  try {
    runCli();
  } catch (error) {
    reportFailure({
      targetTriple: '',
      error,
    });
    process.exit(1);
  }
}
