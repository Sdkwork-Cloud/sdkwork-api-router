#!/usr/bin/env node

import { existsSync } from 'node:fs';
import { spawn, spawnSync } from 'node:child_process';
import {
  createSignalController,
  didChildExitFail,
} from './process-supervision.mjs';
import {
  pnpmExecutable,
  pnpmSpawnOptions,
} from './pnpm-launch-lib.mjs';

function parseArgs(argv) {
  const result = {
    dryRun: false,
    help: false,
    install: false,
    preview: false,
    tauri: false,
  };

  for (const arg of argv) {
    if (arg === '--dry-run') {
      result.dryRun = true;
    } else if (arg === '--install') {
      result.install = true;
    } else if (arg === '--preview') {
      result.preview = true;
    } else if (arg === '--tauri') {
      result.tauri = true;
    } else if (arg === '--help' || arg === '-h') {
      result.help = true;
    }
  }

  return result;
}

function printHelp() {
  console.log(`Usage: node scripts/dev/start-admin.mjs [options]

Starts the standalone sdkwork-router-admin app.

Options:
  --install   Run pnpm install before starting
  --preview   Build and preview the admin app instead of dev mode
  --tauri     Start the admin Tauri desktop shell
  --dry-run   Print the commands without running them
  -h, --help  Show this help
`);
}

function runStep(args, dryRun) {
  const command = `${pnpmExecutable()} ${args.join(' ')}`;
  console.log(`[start-admin] ${command}`);

  if (dryRun) {
    return true;
  }

  const result = spawnSync(pnpmExecutable(), args, {
    ...pnpmSpawnOptions(),
  });
  return result.status === 0;
}

const settings = parseArgs(process.argv.slice(2));
if (settings.help) {
  printHelp();
  process.exit(0);
}

const adminRoot = 'apps/sdkwork-router-admin';
const needInstall = settings.install || !existsSync(`${adminRoot}/node_modules`);
if (needInstall && !runStep(['--dir', adminRoot, 'install'], settings.dryRun)) {
  process.exit(1);
}

if (settings.preview) {
  if (!runStep(['--dir', adminRoot, 'build'], settings.dryRun)) {
    process.exit(1);
  }
  if (!runStep(['--dir', adminRoot, 'preview'], settings.dryRun)) {
    process.exit(1);
  }
  process.exit(0);
}

const longRunningArgs = settings.tauri
  ? ['--dir', adminRoot, 'tauri:dev']
  : ['--dir', adminRoot, 'dev'];
const command = `${pnpmExecutable()} ${longRunningArgs.join(' ')}`;
console.log(`[start-admin] ${command}`);

if (settings.dryRun) {
  process.exit(0);
}

const child = spawn(pnpmExecutable(), longRunningArgs, {
  ...pnpmSpawnOptions(),
});
let shuttingDown = false;
const controller = createSignalController({
  label: 'start-admin',
  children: [child],
  onShutdownStart: () => {
    shuttingDown = true;
  },
});
controller.register();

child.on('exit', (code, signal) => {
  if (shuttingDown) {
    return;
  }

  process.exit(didChildExitFail(code, signal) ? code ?? 1 : 0);
});
