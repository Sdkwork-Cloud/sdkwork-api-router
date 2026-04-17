import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('check-rust-verification-matrix exposes grouped cross-platform cargo plans with stable target caching', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-rust-verification-matrix.mjs')).href,
  );

  assert.deepEqual(module.VERIFICATION_GROUPS, [
    'interface-openapi',
    'gateway-service',
    'admin-service',
    'portal-service',
    'dependency-audit',
    'product-runtime',
    'workspace',
  ]);
  assert.equal(typeof module.createRustVerificationPlan, 'function');

  const interfacePlan = module.createRustVerificationPlan({
    workspaceRoot,
    group: 'interface-openapi',
    platform: 'linux',
    env: {},
  });

  assert.equal(interfacePlan.length, 3);
  assert.equal(interfacePlan[0].label, 'gateway interface openapi route');
  assert.equal(interfacePlan[1].label, 'admin interface openapi route');
  assert.equal(interfacePlan[2].label, 'portal interface openapi route');
  assert.equal(interfacePlan[0].command, 'rustup');
  assert.deepEqual(interfacePlan[0].args, [
    'run',
    'stable',
    'cargo',
    'test',
    '-j',
    '1',
    '-p',
    'sdkwork-api-interface-http',
    '--test',
    'openapi_route',
  ]);
  assert.equal(interfacePlan[0].env.RUSTFLAGS, undefined);
  assert.match(String(interfacePlan[0].env.CARGO_TARGET_DIR ?? ''), /target$/i);

  const productPlan = module.createRustVerificationPlan({
    workspaceRoot,
    group: 'product-runtime',
    platform: 'linux',
    env: {},
  });
  assert.equal(productPlan.length, 2);
  assert.deepEqual(productPlan[0].args, [
    'run',
    'stable',
    'cargo',
    'check',
    '-j',
    '1',
    '-p',
    'sdkwork-api-product-runtime',
  ]);
  assert.deepEqual(productPlan[1].args, [
    'run',
    'stable',
    'cargo',
    'check',
    '-j',
    '1',
    '-p',
    'router-product-service',
  ]);

  const auditPlan = module.createRustVerificationPlan({
    workspaceRoot,
    group: 'dependency-audit',
    platform: 'linux',
    env: {},
  });
  assert.equal(auditPlan.length, 1);
  assert.equal(auditPlan[0].label, 'workspace dependency audit');
  assert.equal(auditPlan[0].command, process.execPath);
  assert.deepEqual(auditPlan[0].args, [
    path.join(workspaceRoot, 'scripts', 'check-rust-dependency-audit.mjs'),
  ]);

  const windowsPlan = module.createRustVerificationPlan({
    workspaceRoot,
    group: 'gateway-service',
    platform: 'win32',
    env: {
      TEMP: 'C:/Temp',
      USERPROFILE: process.env.USERPROFILE ?? '',
      PATH: process.env.PATH ?? '',
    },
  });
  assert.equal(windowsPlan.length, 1);
  assert.equal(typeof windowsPlan[0].command, 'string');
  assert.deepEqual(windowsPlan[0].args.slice(-5), ['check', '-j', '1', '-p', 'gateway-service']);
  assert.equal(windowsPlan[0].env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(windowsPlan[0].env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.match(String(windowsPlan[0].env.CARGO_TARGET_DIR ?? ''), /^C:\\Temp\\sdkwork-target\\/i);
  assert.doesNotMatch(
    String(windowsPlan[0].env.CARGO_TARGET_DIR ?? ''),
    /[\\/]bin[\\/]\.sdkwork-target-vs2022/i,
  );
  assert.equal(windowsPlan[0].env.RUSTFLAGS, undefined);

  const workspacePlan = module.createRustVerificationPlan({
    workspaceRoot,
    group: 'workspace',
    platform: 'win32',
    env: {
      TEMP: 'C:/Temp',
      USERPROFILE: process.env.USERPROFILE ?? '',
      PATH: process.env.PATH ?? '',
    },
  });
  assert.equal(workspacePlan.length, 1);
  assert.deepEqual(workspacePlan[0].args.slice(-3), ['--workspace', '-j', '1']);
  assert.match(String(workspacePlan[0].env.CARGO_TARGET_DIR ?? ''), /^C:\\Temp\\sdkwork-target\\/i);

  const fallbackWindowsPlan = module.createRustVerificationPlan({
    workspaceRoot,
    group: 'gateway-service',
    platform: 'win32',
    env: {
      USERPROFILE: '',
      PATH: process.env.PATH ?? '',
    },
  });
  assert.equal(fallbackWindowsPlan[0].command, 'rustup.exe');
  assert.equal(fallbackWindowsPlan[0].shell, true);
});

test('check-rust-verification-matrix plan json omits inherited environment secrets', () => {
  const secret = 'sdkwork-matrix-secret';
  const output = execFileSync(
    process.execPath,
    [
      path.join(workspaceRoot, 'scripts', 'check-rust-verification-matrix.mjs'),
      '--group',
      'dependency-audit',
      '--plan-format',
      'json',
    ],
    {
      cwd: workspaceRoot,
      env: {
        ...process.env,
        SDKWORK_TEST_SECRET: secret,
      },
      encoding: 'utf8',
    },
  );

  assert.doesNotMatch(output, new RegExp(secret));
  assert.doesNotMatch(output, /"env":/);
});
