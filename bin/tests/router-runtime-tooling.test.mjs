import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..', '..');
async function loadModule() {
  return import(
    pathToFileURL(path.join(repoRoot, 'bin', 'lib', 'router-runtime-tooling.mjs')).href
  );
}

async function loadRouterOpsModule() {
  return import(
    pathToFileURL(path.join(repoRoot, 'bin', 'router-ops.mjs')).href
  );
}

test('createReleaseBuildPlan builds release binaries, web apps, and native package output', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'linux',
    arch: 'x64',
    installDependencies: false,
    includeDocs: true,
    includeConsole: true,
  });

  assert.equal(plan.target.targetTriple, 'x86_64-unknown-linux-gnu');
  assert.equal(plan.steps[0].label, 'cargo release build');
  assert.deepEqual(plan.steps[0].args.slice(-14), [
    'build',
    '--release',
    '--target',
    'x86_64-unknown-linux-gnu',
    '-p',
    'admin-api-service',
    '-p',
    'gateway-service',
    '-p',
    'portal-api-service',
    '-p',
    'router-web-service',
    '-p',
    'router-product-service',
  ]);
  assert.equal(plan.steps.some((step) => step.label === 'admin app build'), true);
  assert.equal(plan.steps.some((step) => step.label === 'portal app build'), true);
  assert.equal(plan.steps.some((step) => step.label === 'console build'), true);
  assert.equal(plan.steps.some((step) => step.label === 'docs build'), true);
  assert.equal(plan.steps.at(-1).label, 'native release package');
  assert.deepEqual(plan.steps.at(-1).args, [
    path.join(repoRoot, 'scripts', 'release', 'package-release-assets.mjs'),
    'native',
    '--platform',
    'linux',
    '--arch',
    'x64',
    '--target',
    'x86_64-unknown-linux-gnu',
    '--output-dir',
    path.join(repoRoot, 'artifacts', 'release'),
  ]);
});

test('createReleaseBuildPlan normalizes broken Windows CMake generator defaults for release cargo builds', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
      CMAKE_GENERATOR: 'Visual Studio 18 2026',
    },
    includeDocs: false,
    includeConsole: false,
  });

  assert.equal(plan.steps[0].env.CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(plan.steps[0].env.HOST_CMAKE_GENERATOR, 'Visual Studio 17 2022');
  assert.equal(
    plan.steps[0].env.CARGO_TARGET_DIR,
    path.join('C:/Users/admin', '.sdkwork-target', 'sdkwork-api-router'),
  );
});

test('createReleaseBuildPlan defaults Windows release cargo builds to a single job and propagates that to downstream steps', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
    includeDocs: false,
    includeConsole: false,
  });

  const jobIndex = plan.steps[0].args.indexOf('-j');
  assert.notEqual(jobIndex, -1, 'expected cargo build to pin an explicit job count');
  assert.equal(plan.steps[0].args[jobIndex + 1], '1');
  assert.equal(plan.steps[0].env.CARGO_BUILD_JOBS, '1');
  assert.equal(
    plan.steps.find((step) => step.label === 'admin desktop release build')?.env.CARGO_BUILD_JOBS,
    '1',
  );
});

test('createReleaseBuildPlan respects an explicit Windows cargo job override', async () => {
  const module = await loadModule();

  const plan = module.createReleaseBuildPlan({
    repoRoot,
    platform: 'win32',
    arch: 'x64',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
      CARGO_BUILD_JOBS: '4',
    },
    includeDocs: false,
    includeConsole: false,
  });

  const jobIndex = plan.steps[0].args.indexOf('-j');
  assert.notEqual(jobIndex, -1, 'expected cargo build to keep an explicit job count');
  assert.equal(plan.steps[0].args[jobIndex + 1], '4');
  assert.equal(plan.steps[0].env.CARGO_BUILD_JOBS, '4');
});

test('createInstallPlan copies product assets, runtime scripts, and service descriptors into install home', async () => {
  const module = await loadModule();
  const installRoot = path.join(repoRoot, 'artifacts', 'install', 'sdkwork-api-router', 'current');

  const plan = module.createInstallPlan({
    repoRoot,
    installRoot,
    platform: 'darwin',
  });

  assert.equal(plan.directories.includes(path.join(installRoot, 'bin')), true);
  assert.equal(plan.directories.includes(path.join(installRoot, 'sites', 'admin')), true);
  assert.equal(plan.directories.includes(path.join(installRoot, 'sites', 'portal')), true);
  assert.equal(plan.directories.includes(path.join(installRoot, 'var', 'log')), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('bin', 'start.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('bin', 'stop.ps1'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'launchd', 'com.sdkwork.api-router.plist'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'systemd', 'sdkwork-api-router.service'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'windows-task', 'sdkwork-api-router.xml'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'systemd', 'install-service.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'systemd', 'uninstall-service.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'launchd', 'install-service.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'launchd', 'uninstall-service.sh'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'windows-task', 'install-service.ps1'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('service', 'windows-task', 'uninstall-service.ps1'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('sites', 'admin', 'dist'))), true);
  assert.equal(plan.files.some((file) => file.targetPath.endsWith(path.join('sites', 'portal', 'dist'))), true);
});

test('createInstallPlan reads release binaries from the managed short Windows target directory when needed', async () => {
  const module = await loadModule();
  const installRoot = path.join(repoRoot, 'artifacts', 'install', 'sdkwork-api-router', 'current');

  const plan = module.createInstallPlan({
    repoRoot,
    installRoot,
    platform: 'win32',
    env: {
      USERPROFILE: 'C:/Users/admin',
      TEMP: 'C:/Temp',
    },
  });

  const binaryCopy = plan.files.find((file) => file.targetPath.endsWith(path.join('bin', 'router-product-service.exe')));
  assert.ok(binaryCopy, 'expected router-product-service.exe copy entry');
  assert.equal(
    binaryCopy.sourcePath,
    path.join('C:/Users/admin', '.sdkwork-target', 'sdkwork-api-router', 'x86_64-pc-windows-msvc', 'release', 'router-product-service.exe'),
  );
});

test('renderRuntimeEnvTemplate defaults release runtime to writable local data and 9980-series ports', async () => {
  const module = await loadModule();
  const installRoot = '/opt/sdkwork-api-router/current';

  const envFile = module.renderRuntimeEnvTemplate({
    installRoot,
    platform: 'linux',
  });

  assert.match(envFile, /SDKWORK_CONFIG_DIR="\/opt\/sdkwork-api-router\/current\/config"/);
  assert.match(envFile, /SDKWORK_DATABASE_URL="sqlite:\/\/\/opt\/sdkwork-api-router\/current\/var\/data\/sdkwork-api-router\.db"/);
  assert.match(envFile, /SDKWORK_WEB_BIND="0\.0\.0\.0:9983"/);
  assert.match(envFile, /SDKWORK_GATEWAY_BIND="127\.0\.0\.1:9980"/);
  assert.match(envFile, /SDKWORK_ADMIN_BIND="127\.0\.0\.1:9981"/);
  assert.match(envFile, /SDKWORK_PORTAL_BIND="127\.0\.0\.1:9982"/);
  assert.match(envFile, /SDKWORK_ADMIN_SITE_DIR="\/opt\/sdkwork-api-router\/current\/sites\/admin\/dist"/);
  assert.match(envFile, /SDKWORK_PORTAL_SITE_DIR="\/opt\/sdkwork-api-router\/current\/sites\/portal\/dist"/);
});

test('service descriptors start the production runtime in foreground mode from the installed home', async () => {
  const module = await loadModule();
  const installRoot = '/opt/sdkwork-api-router/current';

  const systemdUnit = module.renderSystemdUnit({
    installRoot,
    serviceName: 'sdkwork-api-router',
  });
  const launchdPlist = module.renderLaunchdPlist({
    installRoot,
    serviceName: 'com.sdkwork.api-router',
  });
  const windowsTaskXml = module.renderWindowsTaskXml({
    installRoot: 'C:/sdkwork/api-router/current',
    taskName: 'sdkwork-api-router',
  });

  assert.match(systemdUnit, /ExecStart="\/opt\/sdkwork-api-router\/current\/bin\/start\.sh" --foreground --home "\/opt\/sdkwork-api-router\/current"/);
  assert.match(systemdUnit, /EnvironmentFile=-\/opt\/sdkwork-api-router\/current\/config\/router\.env/);
  assert.match(systemdUnit, /WorkingDirectory=\/opt\/sdkwork-api-router\/current/);

  assert.match(launchdPlist, /<string>\/opt\/sdkwork-api-router\/current\/bin\/start\.sh<\/string>/);
  assert.match(launchdPlist, /<string>--foreground<\/string>/);
  assert.match(launchdPlist, /<string>\/opt\/sdkwork-api-router\/current<\/string>/);
  assert.match(launchdPlist, /<key>KeepAlive<\/key>/);

  assert.match(windowsTaskXml, /powershell\.exe/);
  assert.match(windowsTaskXml, /start\.ps1/);
  assert.match(windowsTaskXml, /-Foreground/);
  assert.match(windowsTaskXml, /sdkwork-api-router/);
});

test('rendered runtime env and systemd unit safely handle install roots with spaces', async () => {
  const module = await loadModule();
  const installRoot = '/opt/sdkwork router/current build';

  const envFile = module.renderRuntimeEnvTemplate({
    installRoot,
    platform: 'linux',
  });
  const systemdUnit = module.renderSystemdUnit({
    installRoot,
    serviceName: 'sdkwork-api-router',
  });

  assert.match(envFile, /^SDKWORK_CONFIG_DIR="\/opt\/sdkwork router\/current build\/config"$/m);
  assert.match(envFile, /^SDKWORK_DATABASE_URL="sqlite:\/\/\/opt\/sdkwork router\/current build\/var\/data\/sdkwork-api-router\.db"$/m);
  assert.match(envFile, /^SDKWORK_ADMIN_SITE_DIR="\/opt\/sdkwork router\/current build\/sites\/admin\/dist"$/m);
  assert.match(envFile, /^SDKWORK_ROUTER_BINARY="\/opt\/sdkwork router\/current build\/bin\/router-product-service"$/m);

  assert.match(systemdUnit, /WorkingDirectory=\/opt\/sdkwork\\ router\/current\\ build/);
  assert.match(systemdUnit, /EnvironmentFile=-\/opt\/sdkwork\\ router\/current\\ build\/config\/router\.env/);
  assert.match(systemdUnit, /ExecStart="\/opt\/sdkwork router\/current build\/bin\/start\.sh" --foreground --home "\/opt\/sdkwork router\/current build"/);
});

test('router-ops install rejects --home without a following value', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['install', '--home']),
      /--home requires a value/,
    );
  });
});

test('router-ops install rejects --home when the next token is another flag', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['install', '--home', '--dry-run']),
      /--home requires a value/,
    );
  });
});

test('router-ops rejects build-only flags during install', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['install', '--skip-docs']),
      /--skip-docs is only supported for the build command/,
    );
  });
});

test('router-ops rejects install-only flags during build', () => {
  return loadRouterOpsModule().then(({ parseArgs }) => {
    assert.throws(
      () => parseArgs(['build', '--home', 'artifacts/install/custom']),
      /--home is only supported for the install command/,
    );
  });
});

test('start.ps1 keeps the public -Home switch via alias instead of binding to the built-in HOME variable', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');

  assert.match(script, /\[Alias\('Home'\)\]\s*\r?\n\s*\[string\]\$RuntimeHome = ''/);
  assert.doesNotMatch(script, /\[string\]\$Home = ''/);
});

test('stop.ps1 keeps the public -Home switch via alias instead of binding to the built-in HOME variable', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'stop.ps1'), 'utf8');

  assert.match(script, /\[Alias\('Home'\)\]\s*\r?\n\s*\[string\]\$RuntimeHome = ''/);
  assert.doesNotMatch(script, /\[string\]\$Home = ''/);
});

test('runtime-common.ps1 avoids assigning to the built-in HOST variable while resolving health URLs', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.doesNotMatch(script, /\$host\s*=/i);
  assert.match(script, /\$bindHost\s*=/);
});

test('start.ps1 and stop.ps1 resolve router-product-service through a shared PowerShell binary-name helper', () => {
  const startScript = readFileSync(path.join(repoRoot, 'bin', 'start.ps1'), 'utf8');
  const stopScript = readFileSync(path.join(repoRoot, 'bin', 'stop.ps1'), 'utf8');

  assert.match(startScript, /\$binaryName = Get-RouterBinaryName -BaseName 'router-product-service'/);
  assert.match(stopScript, /\$binaryName = Get-RouterBinaryName -BaseName 'router-product-service'/);
  assert.doesNotMatch(startScript, /\$binaryName = 'router-product-service\.exe'/);
  assert.doesNotMatch(stopScript, /\$binaryName = 'router-product-service\.exe'/);
});

test('runtime-common.ps1 includes platform-aware PowerShell process and binary helpers', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.match(script, /function Test-RouterWindowsPlatform/);
  assert.match(script, /function Get-RouterBinaryName/);
  assert.match(script, /ps -o pid= -o ppid=/);
  assert.match(script, /Stop-Process -Id \$processId/);
});

test('runtime-common.ps1 carries startup summary helpers with unified links, direct links, and seeded credentials', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.ps1'), 'utf8');

  assert.match(script, /function Get-RouterStartupSummaryLines/);
  assert.match(script, /function Write-RouterStartupSummary/);
  assert.match(script, /Unified Access/);
  assert.match(script, /Direct Service Access/);
  assert.match(script, /\/api\/v1\/health/);
  assert.match(script, /\/admin\/health/);
  assert.match(script, /\/portal\/health/);
  assert.match(script, /admin@sdkwork\.local/);
  assert.match(script, /portal@sdkwork\.local/);
  assert.match(script, /ChangeMe123!/);
});

test('runtime-common.sh carries matching startup summary and seeded credential helpers for shell entrypoints', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'lib', 'runtime-common.sh'), 'utf8');

  assert.match(script, /router_startup_summary/);
  assert.match(script, /admin@sdkwork\.local/);
  assert.match(script, /portal@sdkwork\.local/);
  assert.match(script, /ChangeMe123!/);
});

test('start-dev.ps1 defaults the managed dev entrypoint to preview mode and supports explicit browser mode', () => {
  const script = readFileSync(path.join(repoRoot, 'bin', 'start-dev.ps1'), 'utf8');

  assert.match(script, /\[switch\]\$Browser/);
  assert.match(script, /elseif \(-not \$Preview\) \{\s*\$Preview = \$true\s*\}/);
  assert.match(script, /if \(\$Browser\) \{\s*\$Preview = \$false\s*\$Tauri = \$false\s*\}/);
  assert.match(script, /if \(\$Preview\) \{ \$startArgs \+= '--preview' \}/);
});

test('PowerShell source-dev wrappers use the 9980-series defaults', () => {
  const workspaceScript = readFileSync(path.join(repoRoot, 'scripts', 'dev', 'start-workspace.ps1'), 'utf8');
  const serversScript = readFileSync(path.join(repoRoot, 'scripts', 'dev', 'start-servers.ps1'), 'utf8');

  assert.match(workspaceScript, /\$AdminBind = "127\.0\.0\.1:9981"/);
  assert.match(workspaceScript, /\$GatewayBind = "127\.0\.0\.1:9980"/);
  assert.match(workspaceScript, /\$PortalBind = "127\.0\.0\.1:9982"/);
  assert.match(workspaceScript, /\$WebBind = "0\.0\.0\.0:9983"/);
  assert.match(serversScript, /\$AdminBind = "127\.0\.0\.1:9981"/);
  assert.match(serversScript, /\$GatewayBind = "127\.0\.0\.1:9980"/);
  assert.match(serversScript, /\$PortalBind = "127\.0\.0\.1:9982"/);
});
