import assert from 'node:assert/strict';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export async function assertDesktopReleaseSigningContracts({
  repoRoot,
} = {}) {
  const helper = await import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'release', 'run-desktop-release-signing.mjs'),
    ).href,
  );

  assert.equal(typeof helper.parseArgs, 'function');
  assert.equal(typeof helper.resolveDesktopSigningHook, 'function');
  assert.equal(typeof helper.resolveDesktopSigningBundleFiles, 'function');
  assert.equal(typeof helper.createDesktopReleaseSigningPlan, 'function');
  assert.equal(typeof helper.createDesktopReleaseSigningEvidence, 'function');
  assert.equal(typeof helper.executeDesktopReleaseSigningPlan, 'function');
  assert.equal(
    helper.DESKTOP_SIGNING_REQUIRED_ENV,
    'SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED',
  );
  assert.equal(
    helper.DESKTOP_GENERIC_SIGN_HOOK_ENV,
    'SDKWORK_RELEASE_DESKTOP_SIGN_HOOK',
  );
  assert.deepEqual(helper.DESKTOP_PLATFORM_SIGN_HOOK_ENVS, {
    windows: 'SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK',
    linux: 'SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK',
    macos: 'SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK',
  });
}
