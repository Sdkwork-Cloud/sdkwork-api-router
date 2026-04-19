import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import { createRouterProductLaunchPlan } from '../scripts/run-router-product.mjs';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

const desktopApps = [
  {
    name: 'portal',
    root: path.join(workspaceRoot, 'apps', 'sdkwork-router-portal'),
    capabilityPath: path.join(
      workspaceRoot,
      'apps',
      'sdkwork-router-portal',
      'src-tauri',
      'capabilities',
      'main.json',
    ),
    buildRsPath: path.join(
      workspaceRoot,
      'apps',
      'sdkwork-router-portal',
      'src-tauri',
      'build.rs',
    ),
    mainRsPath: path.join(
      workspaceRoot,
      'apps',
      'sdkwork-router-portal',
      'src-tauri',
      'src',
      'main.rs',
    ),
    helperPath: path.join(
      workspaceRoot,
      'apps',
      'sdkwork-router-portal',
      'packages',
      'sdkwork-router-portal-core',
      'src',
      'lib',
      'desktop.ts',
    ),
  },
  {
    name: 'admin',
    root: path.join(workspaceRoot, 'apps', 'sdkwork-router-admin'),
    capabilityPath: path.join(
      workspaceRoot,
      'apps',
      'sdkwork-router-admin',
      'src-tauri',
      'capabilities',
      'main.json',
    ),
    buildRsPath: path.join(
      workspaceRoot,
      'apps',
      'sdkwork-router-admin',
      'src-tauri',
      'build.rs',
    ),
    mainRsPath: path.join(
      workspaceRoot,
      'apps',
      'sdkwork-router-admin',
      'src-tauri',
      'src',
      'main.rs',
    ),
    helperPath: path.join(
      workspaceRoot,
      'apps',
      'sdkwork-router-admin',
      'packages',
      'sdkwork-router-admin-shell',
      'src',
      'desktopWindow.ts',
    ),
  },
  {
    name: 'console',
    root: path.join(workspaceRoot, 'console'),
    capabilityPath: path.join(
      workspaceRoot,
      'console',
      'src-tauri',
      'capabilities',
      'main.json',
    ),
    buildRsPath: path.join(workspaceRoot, 'console', 'src-tauri', 'build.rs'),
    mainRsPath: path.join(workspaceRoot, 'console', 'src-tauri', 'src', 'main.rs'),
    helperPath: null,
  },
];

function read(filePath) {
  return readFileSync(filePath, 'utf8');
}

function parseGenerateHandlerCommands(mainRsSource) {
  const match = mainRsSource.match(/generate_handler!\[(?<body>[\s\S]*?)\]/m);
  assert.ok(match?.groups?.body, 'expected tauri::generate_handler![...] in main.rs');

  return match.groups.body
    .split(',')
    .map((entry) => entry.trim())
    .filter(Boolean)
    .map((entry) => entry.split('::').pop());
}

function parseBuildManifestCommands(buildRsSource) {
  const match = buildRsSource.match(
    /AppManifest::new\(\)\s*\.commands\(&\[(?<body>[\s\S]*?)\]\)/m,
  );
  if (!match?.groups?.body) {
    return [];
  }

  return [...match.groups.body.matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
}

function parseCapability(filePath) {
  assert.ok(existsSync(filePath), `expected capability file at ${filePath}`);
  return JSON.parse(read(filePath));
}

function parseWindowPermissions(helperSource) {
  if (!helperSource) {
    return [];
  }

  const permissions = new Set();
  const methodPermissionPairs = [
    ['toggleMaximize', 'core:window:allow-toggle-maximize'],
    ['maximize', 'core:window:allow-maximize'],
    ['minimize', 'core:window:allow-minimize'],
    ['close', 'core:window:allow-close'],
    ['isMaximized', 'core:window:allow-is-maximized'],
    ['unmaximize', 'core:window:allow-unmaximize'],
  ];

  for (const [methodName, permission] of methodPermissionPairs) {
    if (helperSource.includes(methodName)) {
      permissions.add(permission);
    }
  }

  return [...permissions];
}

function permissionIdForCommand(commandName) {
  return `allow-${commandName.replaceAll('_', '-')}`;
}

for (const app of desktopApps) {
  test(`${app.name} desktop capability is least-privilege and covers frontend/runtime usage`, () => {
    const buildRs = read(app.buildRsPath);
    const mainRs = read(app.mainRsPath);
    const capability = parseCapability(app.capabilityPath);
    const helperSource = app.helperPath ? read(app.helperPath) : '';

    const runtimeCommands = parseGenerateHandlerCommands(mainRs);
    const manifestCommands = parseBuildManifestCommands(buildRs);
    const expectedAppPermissions = runtimeCommands.map(permissionIdForCommand);
    const expectedWindowPermissions = parseWindowPermissions(helperSource);
    const capabilityPermissions = new Set(capability.permissions ?? []);

    assert.equal(capability.identifier, 'main');
    assert.deepEqual(capability.windows, ['main']);
    assert.ok(
      capability.description && capability.description.length > 0,
      'capability should describe the granted scope',
    );
    assert.equal(
      capability.remote,
      undefined,
      'desktop IPC permissions must not be granted to remote windows by default',
    );
    assert.equal(
      capability.local ?? true,
      true,
      'desktop capability should remain limited to local app content',
    );
    assert.ok(
      !capabilityPermissions.has('core:default'),
      'desktop capability must not grant the broad core:default permission set',
    );
    assert.deepEqual(
      manifestCommands.sort(),
      [...runtimeCommands].sort(),
      'build.rs app manifest must cover every command exposed through generate_handler',
    );

    for (const permission of expectedAppPermissions) {
      assert.ok(
        capabilityPermissions.has(permission),
        `capability must include app permission ${permission}`,
      );
    }

    for (const permission of expectedWindowPermissions) {
      assert.ok(
        capabilityPermissions.has(permission),
        `capability must include window permission ${permission}`,
      );
    }
  });
}

test('server mode stays outside the Tauri desktop permission surface', () => {
  const plan = createRouterProductLaunchPlan({
    workspaceRoot,
    mode: 'server',
    install: false,
    platform: 'linux',
    env: {},
  });

  assert.equal(plan.length, 1);
  assert.equal(plan[0].label, 'portal product server');
  assert.ok(
    plan[0].args.includes('server:start'),
    'server mode should boot the server entrypoint instead of tauri:dev',
  );
  assert.ok(
    !plan[0].args.includes('tauri:dev'),
    'server mode must not depend on Tauri capability files or desktop IPC',
  );
});
