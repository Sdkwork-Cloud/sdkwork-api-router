#!/usr/bin/env node

import { spawn, spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import {
  frontendInstallStatus,
  frontendViteConfigHealthy,
  pnpmProcessSpec,
  pnpmSpawnOptions,
} from './dev/pnpm-launch-lib.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));

export function createDesktopAssetBuildPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
} = {}) {
  const appRoots = [
    path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
  ];

  return appRoots.map((cwd) => ({
    cwd,
    ...pnpmProcessSpec(['build'], { platform }),
    shell: false,
  }));
}

function ensureFrontendAppReady({
  appRoot,
  platform = process.platform,
  env = process.env,
} = {}) {
  const installStatus = frontendInstallStatus({
    appRoot,
    platform,
    requiredPackages: ['vite', 'typescript'],
    requiredBinCommands: ['vite', 'tsc'],
    verifyInstalled: () => frontendViteConfigHealthy({
      appRoot,
      command: 'build',
      env,
      platform,
    }),
  });

  if (installStatus === 'ready') {
    return;
  }

  const installProcess = pnpmProcessSpec(['--dir', appRoot, 'install'], { platform });
  const result = spawnSync(
    installProcess.command,
    installProcess.args,
    {
      ...pnpmSpawnOptions({
        platform,
        env,
      }),
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    },
  );

  if (result.error) {
    throw result.error;
  }

  if ((result.status ?? 1) !== 0) {
    throw new Error(`pnpm install exited with code ${result.status ?? 1} for ${appRoot}`);
  }
}

async function runBuild(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      stdio: 'inherit',
      shell: step.shell,
    });

    child.on('error', reject);
    child.on('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`build in ${step.cwd} exited with signal ${signal}`));
        return;
      }
      if ((code ?? 1) !== 0) {
        reject(new Error(`build in ${step.cwd} exited with code ${code}`));
        return;
      }
      resolve();
    });
  });
}

async function main() {
  const plan = createDesktopAssetBuildPlan();
  for (const step of plan) {
    ensureFrontendAppReady({
      appRoot: step.cwd,
    });
  }

  for (const step of plan) {
    // eslint-disable-next-line no-await-in-loop
    await runBuild(step);
  }
}

if (__filename === process.argv[1]) {
  main().catch((error) => {
    console.error(`[build-router-desktop-assets] ${error.message}`);
    process.exit(1);
  });
}
