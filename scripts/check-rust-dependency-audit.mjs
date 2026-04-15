#!/usr/bin/env node

import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export const ADVISORY_DB_URL = 'https://github.com/RustSec/advisory-db.git';
export const AUDIT_POLICY_PATH = path.join(__dirname, 'check-rust-dependency-audit.policy.json');

function pathApiForPlatform(platform = process.platform) {
  return platform === 'win32' ? path.win32 : path.posix;
}

export function resolveCargoHome(platform = process.platform, env = process.env) {
  const pathApi = pathApiForPlatform(platform);

  if (env.CARGO_HOME) {
    return env.CARGO_HOME;
  }

  if (platform === 'win32') {
    if (env.USERPROFILE) {
      return pathApi.join(env.USERPROFILE, '.cargo');
    }
    if (env.HOMEDRIVE && env.HOMEPATH) {
      return pathApi.join(env.HOMEDRIVE, env.HOMEPATH, '.cargo');
    }
  }

  if (env.HOME) {
    return pathApi.join(env.HOME, '.cargo');
  }

  throw new Error('unable to resolve cargo home for advisory database');
}

export function resolveAdvisoryDbPath(platform = process.platform, env = process.env) {
  return pathApiForPlatform(platform).join(resolveCargoHome(platform, env), 'advisory-db');
}

export function resolveGitRunner(platform = process.platform) {
  return {
    command: platform === 'win32' ? 'git.exe' : 'git',
    args: [],
    shell: false,
  };
}

export function resolveRustRunner(platform = process.platform, env = process.env) {
  if (platform === 'win32') {
    const rustupPath = path.win32.join(env.USERPROFILE ?? '', '.cargo', 'bin', 'rustup.exe');
    if (env.USERPROFILE && existsSync(rustupPath)) {
      return {
        command: rustupPath,
        args: ['run', 'stable', 'cargo'],
        shell: false,
      };
    }

    return {
      command: 'rustup.exe',
      args: ['run', 'stable', 'cargo'],
      shell: true,
    };
  }

  return {
    command: 'rustup',
    args: ['run', 'stable', 'cargo'],
    shell: false,
  };
}

export function readDependencyAuditPolicy(policyPath = AUDIT_POLICY_PATH) {
  const policy = JSON.parse(readFileSync(policyPath, 'utf8'));
  const allowedWarnings = policy?.allowedWarnings;
  if (!Array.isArray(allowedWarnings)) {
    throw new Error(`dependency audit policy must define an allowedWarnings array: ${policyPath}`);
  }

  const ids = new Set();
  for (const entry of allowedWarnings) {
    if (!entry || typeof entry !== 'object') {
      throw new Error(`dependency audit policy entries must be objects: ${policyPath}`);
    }
    for (const field of ['id', 'owner', 'reason', 'reviewBy']) {
      if (typeof entry[field] !== 'string' || entry[field].trim() === '') {
        throw new Error(`dependency audit policy entry is missing ${field}: ${policyPath}`);
      }
    }
    if (ids.has(entry.id)) {
      throw new Error(`dependency audit policy contains duplicate advisory id ${entry.id}: ${policyPath}`);
    }
    ids.add(entry.id);
  }

  return {
    allowedWarnings,
  };
}

export function createDependencyAuditPlan({
  workspaceRoot = path.resolve(__dirname, '..'),
  platform = process.platform,
  env = process.env,
  advisoryDbPath = resolveAdvisoryDbPath(platform, env),
  advisoryDbExists = existsSync(advisoryDbPath),
  auditPolicy = readDependencyAuditPolicy(),
} = {}) {
  const gitRunner = resolveGitRunner(platform);
  const rustRunner = resolveRustRunner(platform, env);
  const ignoredAdvisories = auditPolicy.allowedWarnings.map((entry) => entry.id);

  const advisoryDbPlan = advisoryDbExists
    ? [
        {
          label: 'fetch RustSec advisory database',
          command: gitRunner.command,
          args: [...gitRunner.args, '-C', advisoryDbPath, 'fetch', '--depth', '1', 'origin', 'main'],
          cwd: workspaceRoot,
          env,
          shell: gitRunner.shell,
          windowsHide: platform === 'win32',
        },
        {
          label: 'align RustSec advisory database to fetched main',
          command: gitRunner.command,
          args: [...gitRunner.args, '-C', advisoryDbPath, 'checkout', '--detach', '--force', 'FETCH_HEAD'],
          cwd: workspaceRoot,
          env,
          shell: gitRunner.shell,
          windowsHide: platform === 'win32',
        },
      ]
    : [
        {
          label: 'clone RustSec advisory database',
          command: gitRunner.command,
          args: [
            ...gitRunner.args,
            'clone',
            '--depth',
            '1',
            '--branch',
            'main',
            ADVISORY_DB_URL,
            advisoryDbPath,
          ],
          cwd: workspaceRoot,
          env,
          shell: gitRunner.shell,
          windowsHide: platform === 'win32',
        },
      ];

  return [
    ...advisoryDbPlan,
    {
      label: 'workspace cargo audit',
      command: rustRunner.command,
      args: [
        ...rustRunner.args,
        'audit',
        '--db',
        advisoryDbPath,
        '--no-fetch',
        '--stale',
        '--deny',
        'warnings',
        ...ignoredAdvisories.flatMap((id) => ['--ignore', id]),
      ],
      cwd: workspaceRoot,
      env,
      shell: rustRunner.shell,
      windowsHide: platform === 'win32',
    },
  ];
}

function serializePlan(plan) {
  return plan.map(({ env: _env, ...step }) => step);
}

function parseArgs(argv = process.argv.slice(2)) {
  let planFormat = '';

  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--plan-format') {
      planFormat = argv[index + 1] ?? '';
      index += 1;
      continue;
    }

    throw new Error(`unknown argument: ${value}`);
  }

  if (planFormat && planFormat !== 'json') {
    throw new Error(`unsupported plan format: ${planFormat}`);
  }

  return {
    planFormat,
  };
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
  const { planFormat } = parseArgs();
  const advisoryDbPath = resolveAdvisoryDbPath();
  mkdirSync(path.dirname(advisoryDbPath), { recursive: true });

  const plan = createDependencyAuditPlan({
    advisoryDbPath,
  });

  if (planFormat === 'json') {
    console.log(JSON.stringify(serializePlan(plan), null, 2));
    return;
  }

  for (const step of plan) {
    console.error(`[check-rust-dependency-audit] ${step.label}`);
    // eslint-disable-next-line no-await-in-loop
    await runStep(step);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(`[check-rust-dependency-audit] ${error.message}`);
    process.exit(1);
  });
}
