#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function pnpmCommand(platform = process.platform) {
  return platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function shellForPnpm(platform = process.platform) {
  return platform === 'win32';
}

function toPortablePath(value) {
  return value.replaceAll(path.sep, '/');
}

function installLabelForRelativeDir(relativeDir) {
  const appName = path.basename(relativeDir);

  if (appName === 'sdkwork-router-admin') {
    return 'admin install';
  }

  if (appName === 'sdkwork-router-portal') {
    return 'portal install';
  }

  return `${appName} install`;
}

function installCandidatesForMode(mode) {
  switch (mode) {
    case 'server':
      return [
        toPortablePath(path.join('apps', 'sdkwork-router-admin')),
        toPortablePath(path.join('apps', 'sdkwork-router-portal')),
      ];
    case 'desktop':
    case 'service':
    case 'browser':
      return [toPortablePath(path.join('apps', 'sdkwork-router-portal'))];
    default:
      return [];
  }
}

export function parseRouterProductArgs(argv) {
  const result = {
    mode: 'desktop',
    install: false,
    dryRun: false,
    help: false,
    extraArgs: [],
  };

  let modeSet = false;
  let forwardOnly = false;
  for (const arg of argv) {
    if (forwardOnly) {
      result.extraArgs.push(arg);
      continue;
    }
    if (arg === '--') {
      forwardOnly = true;
      continue;
    }
    if (arg === '--install') {
      result.install = true;
      continue;
    }
    if (arg === '--dry-run') {
      result.dryRun = true;
      continue;
    }
    if (arg === '--help' || arg === '-h') {
      if (modeSet) {
        result.extraArgs.push(arg);
      } else {
        result.help = true;
      }
      continue;
    }
    if (!modeSet && !arg.startsWith('-')) {
      result.mode = arg;
      modeSet = true;
      continue;
    }
    result.extraArgs.push(arg);
  }

  return result;
}

function appendForwardArgs(args, extraArgs) {
  if (!extraArgs.length) {
    return args;
  }

  return [...args, '--', ...extraArgs];
}

function isForwardedHelpRequest(extraArgs = []) {
  return extraArgs.length === 1 && (extraArgs[0] === '--help' || extraArgs[0] === '-h');
}

export function createRouterProductLaunchPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  mode = 'desktop',
  install = false,
  platform = process.platform,
  env = process.env,
  extraArgs = [],
} = {}) {
  const portalRelativeDir = toPortablePath(path.join('apps', 'sdkwork-router-portal'));
  const portalAbsoluteDir = path.join(workspaceRoot, portalRelativeDir);
  const pnpm = pnpmCommand(platform);
  const shell = shellForPnpm(platform);
  const nodeCommand = process.execPath;
  const plan = [];

  for (const relativeDir of installCandidatesForMode(mode)) {
    const absoluteDir = path.join(workspaceRoot, relativeDir);
    if (!install && existsSync(path.join(absoluteDir, 'node_modules'))) {
      continue;
    }

    plan.push({
      label: installLabelForRelativeDir(relativeDir),
      command: pnpm,
      args: ['--dir', relativeDir, 'install'],
      cwd: workspaceRoot,
      env,
      shell,
      windowsHide: platform === 'win32',
    });
  }

  let launchArgs;
  let label;
  let launchCommand = pnpm;
  let launchEnv = env;
  let launchShell = shell;
  let launchWindowsHide = platform === 'win32';
  switch (mode) {
    case 'desktop':
      label = 'portal desktop runtime';
      launchArgs = appendForwardArgs(['--dir', portalRelativeDir, 'tauri:dev'], extraArgs);
      break;
    case 'service':
      label = 'portal service runtime';
      launchArgs = appendForwardArgs(['--dir', portalRelativeDir, 'tauri:dev'], extraArgs);
      launchEnv = {
        ...env,
        SDKWORK_ROUTER_BACKGROUND: '1',
        SDKWORK_ROUTER_PORTAL_START_HIDDEN: '1',
        SDKWORK_ROUTER_SERVICE_MODE: '1',
      };
      break;
    case 'server':
      label = 'server development workspace';
      launchCommand = nodeCommand;
      launchShell = false;
      launchArgs = [
        path.join(workspaceRoot, 'scripts', 'dev', 'start-workspace.mjs'),
        '--proxy-dev',
        ...extraArgs,
      ];
      break;
    case 'plan':
      plan.push({
        label: 'portal deployment plan',
        command: nodeCommand,
        args: [
          path.join(workspaceRoot, 'scripts', 'run-router-product-service.mjs'),
          '--dry-run',
          '--plan-format',
          'json',
          ...extraArgs,
        ],
        cwd: portalAbsoluteDir,
        env,
        shell: false,
      });
      return plan;
    case 'check':
      label = 'portal product check';
      launchArgs = ['--dir', portalRelativeDir, 'product:check'];
      break;
    case 'browser':
      label = 'portal browser runtime';
      launchArgs = appendForwardArgs(['--dir', portalRelativeDir, 'dev'], extraArgs);
      break;
    default:
      throw new Error(
        `Unsupported router product mode: ${mode}. Expected one of desktop, service, server, plan, check, browser.`,
      );
  }

  plan.push({
    label,
    command: launchCommand,
    args: launchArgs,
    cwd: workspaceRoot,
    env: launchEnv,
    shell: launchShell,
    windowsHide: launchWindowsHide,
  });

  return plan;
}

function printHelp() {
  console.log(`Usage: node scripts/run-router-product.mjs [mode] [options] [mode-args...]

Start the sdkwork-router-portal product through the unified root entrypoint.

Modes:
  desktop  Start the Tauri desktop host and embedded router product runtime (default)
  service  Start the desktop host in tray-managed service mode
  server   Start the full server development workspace (backend + admin + portal + unified web host)
  plan     Print the resolved server deployment plan through the portal entrypoint
  check    Run the integrated product verification flow
  browser  Start the standalone portal browser dev server

Options:
  --install   Run the required frontend pnpm installs before starting
  --dry-run   Print the planned commands without running them
  -h, --help  Show this help

Examples:
  node scripts/run-router-product.mjs
  node scripts/run-router-product.mjs service
  node scripts/run-router-product.mjs server --gateway-bind 0.0.0.0:9980 --web-bind 127.0.0.1:9983
  node scripts/run-router-product.mjs plan --roles web
  node scripts/run-router-product.mjs check

For the standalone integrated router-product-service CLI, use:
  pnpm --dir apps/sdkwork-router-portal server:start -- --help
`);
}

async function runStep(step) {
  await new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      env: step.env,
      stdio: 'inherit',
      shell: step.shell ?? false,
      windowsHide: step.windowsHide ?? process.platform === 'win32',
    });

    child.on('error', reject);
    child.on('exit', (code, signal) => {
      if (signal) {
        reject(new Error(`${step.label} exited with signal ${signal}`));
        return;
      }
      if ((code ?? 1) !== 0) {
        reject(new Error(`${step.label} exited with code ${code}`));
        return;
      }
      resolve();
    });
  });
}

async function main() {
  const settings = parseRouterProductArgs(process.argv.slice(2));
  if (settings.help) {
    printHelp();
    return;
  }

  const plan = createRouterProductLaunchPlan({
    mode: settings.mode,
    install: settings.install,
    extraArgs: settings.extraArgs,
  });
  const suppressPlanLogging = isForwardedHelpRequest(settings.extraArgs) && !settings.dryRun;

  for (const step of plan) {
    if (!suppressPlanLogging) {
      const rendered = `${step.command} ${step.args.join(' ')}`;
      console.error(`[run-router-product] ${rendered}`);
    }
    if (settings.dryRun) {
      continue;
    }
    // eslint-disable-next-line no-await-in-loop
    await runStep(step);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === __filename) {
  main().catch((error) => {
    console.error(`[run-router-product] ${error.message}`);
    process.exit(1);
  });
}
