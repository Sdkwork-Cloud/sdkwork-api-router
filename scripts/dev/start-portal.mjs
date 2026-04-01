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
  };

  for (const arg of argv) {
    if (arg === '--dry-run') {
      result.dryRun = true;
    } else if (arg === '--install') {
      result.install = true;
    } else if (arg === '--preview') {
      result.preview = true;
    } else if (arg === '--help' || arg === '-h') {
      result.help = true;
    }
  }

  return result;
}

function printHelp() {
  console.log(`Usage: node scripts/dev/start-portal.mjs [options]

Starts the standalone sdkwork-router-portal app.

Options:
  --install   Run pnpm install before starting
  --preview   Build and preview the portal instead of dev mode
  --dry-run   Print the commands without running them
  -h, --help  Show this help
`);
}

function runStep(args, dryRun) {
  const command = `${pnpmExecutable()} ${args.join(' ')}`;
  console.log(`[start-portal] ${command}`);

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

const portalRoot = 'apps/sdkwork-router-portal';
const needInstall = settings.install || !existsSync(`${portalRoot}/node_modules`);
if (needInstall && !runStep(['--dir', portalRoot, 'install'], settings.dryRun)) {
  process.exit(1);
}

if (settings.preview) {
  if (!runStep(['--dir', portalRoot, 'build'], settings.dryRun)) {
    process.exit(1);
  }
  if (!runStep(['--dir', portalRoot, 'preview'], settings.dryRun)) {
    process.exit(1);
  }
  process.exit(0);
}

const longRunningArgs = ['--dir', portalRoot, 'dev'];
const command = `${pnpmExecutable()} ${longRunningArgs.join(' ')}`;
console.log(`[start-portal] ${command}`);

if (settings.dryRun) {
  process.exit(0);
}

const child = spawn(pnpmExecutable(), longRunningArgs, {
  ...pnpmSpawnOptions(),
});
let shuttingDown = false;
const controller = createSignalController({
  label: 'start-portal',
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
