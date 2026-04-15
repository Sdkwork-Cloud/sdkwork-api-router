import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');
const auditPolicyPath = path.join(workspaceRoot, 'scripts', 'check-rust-dependency-audit.policy.json');

test('check-rust-dependency-audit plans advisory DB refresh before cargo audit and honors explicit cargo homes', async () => {
  const module = await import(
    pathToFileURL(path.join(workspaceRoot, 'scripts', 'check-rust-dependency-audit.mjs')).href,
  );
  const auditPolicy = JSON.parse(readFileSync(auditPolicyPath, 'utf8'));
  const ignoredAdvisories = auditPolicy.allowedWarnings.map((entry) => entry.id);

  assert.equal(module.ADVISORY_DB_URL, 'https://github.com/RustSec/advisory-db.git');
  assert.equal(typeof module.createDependencyAuditPlan, 'function');

  const clonedDbPlan = module.createDependencyAuditPlan({
    workspaceRoot,
    platform: 'linux',
    env: {
      HOME: '/home/router',
      CARGO_HOME: '/tmp/router-cargo-home',
    },
    advisoryDbExists: false,
  });

  assert.equal(clonedDbPlan.length, 2);
  assert.equal(clonedDbPlan[0].label, 'clone RustSec advisory database');
  assert.equal(clonedDbPlan[0].command, 'git');
  assert.deepEqual(clonedDbPlan[0].args, [
    'clone',
    '--depth',
    '1',
    '--branch',
    'main',
    'https://github.com/RustSec/advisory-db.git',
    path.posix.join('/tmp/router-cargo-home', 'advisory-db'),
  ]);
  assert.equal(clonedDbPlan[0].cwd, workspaceRoot);

  assert.equal(clonedDbPlan[1].label, 'workspace cargo audit');
  assert.equal(clonedDbPlan[1].command, 'rustup');
  assert.deepEqual(clonedDbPlan[1].args, [
    'run',
    'stable',
    'cargo',
    'audit',
    '--db',
    path.posix.join('/tmp/router-cargo-home', 'advisory-db'),
    '--no-fetch',
    '--stale',
    '--deny',
    'warnings',
    ...ignoredAdvisories.flatMap((id) => ['--ignore', id]),
  ]);

  const refreshedDbPlan = module.createDependencyAuditPlan({
    workspaceRoot,
    platform: 'linux',
    env: {
      HOME: '/home/router',
    },
    advisoryDbExists: true,
  });

  assert.equal(refreshedDbPlan[0].label, 'refresh RustSec advisory database');
  assert.deepEqual(refreshedDbPlan[0].args, [
    '-C',
    path.posix.join('/home/router', '.cargo', 'advisory-db'),
    'pull',
    '--ff-only',
    '--depth',
    '1',
    'origin',
    'main',
  ]);
  assert.deepEqual(refreshedDbPlan[1].args, [
    'run',
    'stable',
    'cargo',
    'audit',
    '--db',
    path.posix.join('/home/router', '.cargo', 'advisory-db'),
    '--no-fetch',
    '--stale',
    '--deny',
    'warnings',
    ...ignoredAdvisories.flatMap((id) => ['--ignore', id]),
  ]);

  assert.equal(
    module.resolveAdvisoryDbPath('win32', {
      USERPROFILE: 'C:\\Users\\router',
      CARGO_HOME: '',
    }),
    path.win32.join('C:\\Users\\router', '.cargo', 'advisory-db'),
  );
});

test('check-rust-dependency-audit policy records each allowed RustSec warning with owner, rationale, and review date', () => {
  const auditPolicy = JSON.parse(readFileSync(auditPolicyPath, 'utf8'));

  assert.deepEqual(
    auditPolicy.allowedWarnings.map((entry) => entry.id),
    [],
  );

  for (const entry of auditPolicy.allowedWarnings) {
    assert.equal(typeof entry.owner, 'string');
    assert.notEqual(entry.owner.trim(), '');
    assert.equal(typeof entry.reason, 'string');
    assert.notEqual(entry.reason.trim(), '');
    assert.match(entry.reviewBy, /^\d{4}-\d{2}-\d{2}$/);
  }
});

test('vendored pingora-core no longer declares the retired daemonize dependency', () => {
  const pingoraCoreManifest = readFileSync(
    path.join(workspaceRoot, 'vendor', 'pingora-core-0.8.0', 'Cargo.toml'),
    'utf8',
  );

  assert.doesNotMatch(
    pingoraCoreManifest,
    /\[target\."cfg\(unix\)"\.dependencies\.daemonize\]/,
  );
});

test('workspace and interface crates no longer depend on utoipa-axum for OpenAPI path registration', () => {
  const workspaceManifest = readFileSync(path.join(workspaceRoot, 'Cargo.toml'), 'utf8');
  const adminManifest = readFileSync(
    path.join(workspaceRoot, 'crates', 'sdkwork-api-interface-admin', 'Cargo.toml'),
    'utf8',
  );
  const httpManifest = readFileSync(
    path.join(workspaceRoot, 'crates', 'sdkwork-api-interface-http', 'Cargo.toml'),
    'utf8',
  );
  const portalManifest = readFileSync(
    path.join(workspaceRoot, 'crates', 'sdkwork-api-interface-portal', 'Cargo.toml'),
    'utf8',
  );
  const adminOpenapi = readFileSync(
    path.join(workspaceRoot, 'crates', 'sdkwork-api-interface-admin', 'src', 'openapi.rs'),
    'utf8',
  );
  const httpOpenapi = readFileSync(
    path.join(workspaceRoot, 'crates', 'sdkwork-api-interface-http', 'src', 'gateway_openapi.rs'),
    'utf8',
  );

  assert.doesNotMatch(workspaceManifest, /utoipa-axum\s*=/);
  assert.doesNotMatch(adminManifest, /utoipa-axum\.workspace\s*=\s*true/);
  assert.doesNotMatch(httpManifest, /utoipa-axum\.workspace\s*=\s*true/);
  assert.doesNotMatch(portalManifest, /utoipa-axum\.workspace\s*=\s*true/);
  assert.doesNotMatch(adminOpenapi, /utoipa_axum/);
  assert.doesNotMatch(httpOpenapi, /utoipa_axum/);
});

test('check-rust-dependency-audit plan json omits inherited environment secrets', () => {
  const secret = 'sdkwork-plan-secret';
  const output = execFileSync(
    process.execPath,
    [path.join(workspaceRoot, 'scripts', 'check-rust-dependency-audit.mjs'), '--plan-format', 'json'],
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

test('lockfile only retains RustSec-warning dependencies when they are explicitly tracked by the audit policy', () => {
  const lockfile = readFileSync(path.join(workspaceRoot, 'Cargo.lock'), 'utf8');
  const auditPolicy = JSON.parse(readFileSync(auditPolicyPath, 'utf8'));

  assert.doesNotMatch(
    lockfile,
    /name = "cf-rustracing"\r?\nversion = "1\.2\.1"/,
  );
  assert.doesNotMatch(
    lockfile,
    /name = "cf-rustracing-jaeger"\r?\nversion = "1\.2\.2"/,
  );
  assert.doesNotMatch(
    lockfile,
    /name = "rand"\r?\nversion = "0\.9\.2"/,
  );
  assert.doesNotMatch(
    lockfile,
    /name = "derivative"\r?\nversion = "2\.2\.0"/,
  );

  assert.doesNotMatch(
    lockfile,
    /name = "paste"\r?\nversion = "1\.0\.15"/,
  );
  assert.doesNotMatch(
    lockfile,
    /name = "daemonize"\r?\nversion = "0\.5\.0"/,
  );
  assert.doesNotMatch(
    lockfile,
    /name = "rand"\r?\nversion = "0\.8\.5"/,
  );
});
