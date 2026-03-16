#!/usr/bin/env node

import { spawn } from 'node:child_process';
import {
  databaseDisplayValue,
  parseStackArgs,
  serviceEnv,
  stackHelpText,
} from './backend-launch-lib.mjs';

function cargoExecutable() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function startService(packageName, settings, children) {
  const env = serviceEnv(settings);
  const command = `${cargoExecutable()} run -p ${packageName}`;
  console.log(`[start-stack] ${command}`);

  if (settings.dryRun) {
    return;
  }

  const child = spawn(cargoExecutable(), ['run', '-p', packageName], {
    env,
    stdio: 'inherit',
  });
  children.push(child);

  child.on('exit', (code, signal) => {
    if (signal) {
      console.log(`[start-stack] ${packageName} exited with signal ${signal}`);
      return;
    }
    console.log(`[start-stack] ${packageName} exited with code ${code ?? 0}`);
  });
}

function installSignalHandlers(children) {
  let stopping = false;

  function shutdown(signal) {
    if (stopping) {
      return;
    }
    stopping = true;
    console.log(`[start-stack] received ${signal}, stopping child processes`);
    for (const child of children) {
      if (!child.killed) {
        child.kill('SIGTERM');
      }
    }
    setTimeout(() => process.exit(0), 150);
  }

  process.on('SIGINT', () => shutdown('SIGINT'));
  process.on('SIGTERM', () => shutdown('SIGTERM'));
}

const settings = parseStackArgs(process.argv.slice(2));
if (settings.help) {
  console.log(stackHelpText());
  process.exit(0);
}

console.log('[start-stack] shared configuration');
console.log(`  SDKWORK_DATABASE_URL=${databaseDisplayValue(settings)}`);
console.log(`  SDKWORK_ADMIN_BIND=${settings.adminBind}`);
console.log(`  SDKWORK_GATEWAY_BIND=${settings.gatewayBind}`);
console.log(`  SDKWORK_PORTAL_BIND=${settings.portalBind}`);

const children = [];
installSignalHandlers(children);
startService('admin-api-service', settings, children);
startService('gateway-service', settings, children);
startService('portal-api-service', settings, children);

if (settings.dryRun) {
  process.exit(0);
}
