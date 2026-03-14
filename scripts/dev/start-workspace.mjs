#!/usr/bin/env node

import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import {
  buildWorkspaceCommandPlan,
  parseWorkspaceArgs,
  workspaceHelpText,
} from './workspace-launch-lib.mjs';

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, '..', '..');
process.chdir(repositoryRoot);

function formatCommand(command, args) {
  return [command, ...args].join(' ');
}

function spawnStep(step, nodeExecutable, children) {
  const command = formatCommand(nodeExecutable, step.args);
  console.log(`[start-workspace] ${step.name}: ${command}`);

  const child = spawn(nodeExecutable, step.args, {
    cwd: repositoryRoot,
    stdio: 'inherit',
  });
  children.push(child);

  child.on('exit', (code, signal) => {
    if (signal) {
      console.log(`[start-workspace] ${step.name} exited with signal ${signal}`);
      return;
    }
    console.log(`[start-workspace] ${step.name} exited with code ${code ?? 0}`);
  });

  return child;
}

function installSignalHandlers(children) {
  let stopping = false;

  function shutdown(signal, exitCode = 0) {
    if (stopping) {
      return;
    }
    stopping = true;
    console.log(`[start-workspace] received ${signal}, stopping child processes`);
    for (const child of children) {
      if (!child.killed) {
        child.kill('SIGTERM');
      }
    }
    setTimeout(() => process.exit(exitCode), 150);
  }

  return {
    shutdown,
    register() {
      process.on('SIGINT', () => shutdown('SIGINT'));
      process.on('SIGTERM', () => shutdown('SIGTERM'));
    },
  };
}

let settings;
try {
  settings = parseWorkspaceArgs(process.argv.slice(2));
} catch (error) {
  console.error(`[start-workspace] ${error.message}`);
  console.error('');
  console.error(workspaceHelpText());
  process.exit(1);
}

if (settings.help) {
  console.log(workspaceHelpText());
  process.exit(0);
}

const plan = buildWorkspaceCommandPlan(settings);

console.log('[start-workspace] unified launch settings');
console.log(`  SDKWORK_DATABASE_URL=${settings.databaseUrl}`);
console.log(`  SDKWORK_GATEWAY_BIND=${settings.gatewayBind}`);
console.log(`  SDKWORK_ADMIN_BIND=${settings.adminBind}`);
console.log(`  SDKWORK_PORTAL_BIND=${settings.portalBind}`);
console.log(`  console_mode=${settings.preview ? 'preview' : settings.tauri ? 'tauri' : 'browser'}`);

if (settings.dryRun) {
  console.log(`[start-workspace] ${plan.backend.name}: ${formatCommand(plan.nodeExecutable, plan.backend.args)}`);
  console.log(`[start-workspace] ${plan.console.name}: ${formatCommand(plan.nodeExecutable, plan.console.args)}`);
  process.exit(0);
}

const children = [];
const controller = installSignalHandlers(children);
controller.register();

let exited = false;
for (const step of [plan.backend, plan.console]) {
  const child = spawnStep(step, plan.nodeExecutable, children);
  child.on('exit', (code, signal) => {
    if (exited) {
      return;
    }

    if (signal || (code ?? 0) !== 0) {
      exited = true;
      controller.shutdown(`${step.name} exit`, code ?? 1);
    }
  });
}
