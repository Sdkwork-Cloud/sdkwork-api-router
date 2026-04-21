import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const workspaceRoot = path.resolve(import.meta.dirname, '..');

test('desktop asset preflight recognizes missing frontend dependency failures that require reinstall recovery', async () => {
  const module = await import(
    pathToFileURL(
      path.join(workspaceRoot, 'scripts', 'build-router-desktop-assets.mjs'),
    ).href,
  );

  assert.equal(typeof module.shouldReinstallFrontendDependenciesAfterBuildFailure, 'function');

  assert.equal(
    module.shouldReinstallFrontendDependenciesAfterBuildFailure({
      status: 1,
      stderr: `
[vite]: Rollup failed to resolve import "@radix-ui/react-roving-focus" from "D:/repo/apps/sdkwork-router-admin/node_modules/@radix-ui/react-tabs/dist/index.mjs".
This is most likely unintended because it can break your application at runtime.
`,
    }),
    true,
  );

  assert.equal(
    module.shouldReinstallFrontendDependenciesAfterBuildFailure({
      status: 1,
      stderr: 'src/App.tsx(19,7): error TS2322: Type string is not assignable to number',
    }),
    false,
  );
});
