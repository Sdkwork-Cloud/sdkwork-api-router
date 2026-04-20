#!/usr/bin/env node

import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import {
  databaseDisplayValue,
  parseStackArgs,
  renderSourceDevRouterConfig,
  serviceEnv,
  stackHelpText,
} from './backend-launch-lib.mjs';
import {
  createSupervisorKeepAlive,
  createSignalController,
  didChildExitFail,
} from './process-supervision.mjs';

function cargoExecutable() {
  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

const repoRoot = path.resolve(import.meta.dirname, '..', '..');

function resolveBackendLaunchSpec(packageName, env) {
  const usePrebuiltBinaries =
    process.platform === 'win32' && env.SDKWORK_ROUTER_USE_PREBUILT_BACKEND_BINARIES === '1';
  if (usePrebuiltBinaries && env.CARGO_TARGET_DIR) {
    const binaryPath = path.resolve(env.CARGO_TARGET_DIR, 'debug', `${packageName}.exe`);
    if (existsSync(binaryPath)) {
      return {
        command: binaryPath,
        args: [],
      };
    }
  }

  return {
    command: cargoExecutable(),
    args: ['run', '-p', packageName],
  };
}

function sourceConfigRunLabel(now = new Date(), pid = process.pid) {
  const iso = now.toISOString().replaceAll(':', '').replaceAll('.', '').replace('T', '-');
  return `${iso}-${pid}`;
}

function createSourceDevConfigPlan(settings, {
  repositoryRoot = repoRoot,
  now = new Date(),
  pid = process.pid,
} = {}) {
  const runRoot = path.join(
    repositoryRoot,
    'artifacts',
    'runtime',
    'source-workspace',
    sourceConfigRunLabel(now, pid),
  );
  const configDir = path.join(runRoot, 'config');
  const configFile = path.join(configDir, 'router.yaml');
  mkdirSync(configDir, { recursive: true });
  writeFileSync(configFile, renderSourceDevRouterConfig(settings), 'utf8');

  return {
    runRoot,
    configDir,
    configFile,
  };
}

function startService(packageName, settings, sourceConfig, children, onFailure) {
  const env = serviceEnv(settings, process.env, {
    sourceConfigDir: sourceConfig.configDir,
    sourceConfigFile: sourceConfig.configFile,
  });
  const launchSpec = resolveBackendLaunchSpec(packageName, env);
  const command = [launchSpec.command, ...launchSpec.args].join(' ');
  console.log(`[start-stack] ${command}`);

  if (settings.dryRun) {
    return;
  }

  const child = spawn(launchSpec.command, launchSpec.args, {
    env,
    stdio: 'inherit',
  });
  children.push(child);

  child.on('exit', (code, signal) => {
    if (signal) {
      console.log(`[start-stack] ${packageName} exited with signal ${signal}`);
    } else {
      console.log(`[start-stack] ${packageName} exited with code ${code ?? 0}`);
    }

    onFailure(`${packageName} exit`, didChildExitFail(code, signal) ? code ?? 1 : 0);
  });
}

const settings = parseStackArgs(process.argv.slice(2));
if (settings.help) {
  console.log(stackHelpText());
  process.exit(0);
}
const sourceConfig = createSourceDevConfigPlan(settings);

console.log('[start-stack] shared configuration');
console.log(`  SDKWORK_DATABASE_URL=${databaseDisplayValue(settings)}`);
console.log(`  SDKWORK_ADMIN_BIND=${settings.adminBind}`);
console.log(`  SDKWORK_GATEWAY_BIND=${settings.gatewayBind}`);
console.log(`  SDKWORK_PORTAL_BIND=${settings.portalBind}`);
console.log(`  SDKWORK_CONFIG_DIR=${sourceConfig.configDir}`);
console.log(`  SDKWORK_CONFIG_FILE=${sourceConfig.configFile}`);

const children = [];
let exited = false;
const releaseKeepAlive = createSupervisorKeepAlive();
const controller = createSignalController({
  label: 'start-stack',
  children,
  onShutdownStart: () => {
    exited = true;
    releaseKeepAlive();
  },
});
controller.register();

function stopOnFailure(reason, exitCode) {
  if (exited) {
    return;
  }

  exited = true;
  releaseKeepAlive();
  void controller.shutdown(reason, exitCode);
}

startService('admin-api-service', settings, sourceConfig, children, stopOnFailure);
startService('gateway-service', settings, sourceConfig, children, stopOnFailure);
startService('portal-api-service', settings, sourceConfig, children, stopOnFailure);

if (settings.dryRun) {
  process.exit(0);
}
