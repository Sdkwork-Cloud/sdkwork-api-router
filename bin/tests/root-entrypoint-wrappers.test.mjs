import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { readFileSync } from 'node:fs';

const repoRoot = path.resolve(import.meta.dirname, '..', '..');

function readRepoFile(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

test('root shell entrypoints stay thin compatibility wrappers over bin-managed scripts', () => {
  const shellEntrypoints = [
    ['start.sh', 'bin/start.sh'],
    ['start-dev.sh', 'bin/start-dev.sh'],
    ['stop.sh', 'bin/stop.sh'],
    ['stop-dev.sh', 'bin/stop-dev.sh'],
    ['build.sh', 'bin/build.sh'],
    ['install.sh', 'bin/install.sh'],
  ];

  for (const [rootScript, targetScript] of shellEntrypoints) {
    const source = readRepoFile(rootScript);
    assert.match(source, new RegExp(`TARGET_SCRIPT=.*${targetScript.replace('/', '\\/')}`));
    assert.match(source, /exec "\$TARGET_SCRIPT" "\$@"/);
    assert.doesNotMatch(source, /runtime-common/);
    assert.doesNotMatch(source, /router-ops\.mjs/);
  }
});

test('root powershell entrypoints stay thin compatibility wrappers over bin-managed scripts', () => {
  const psEntrypoints = [
    ['start.ps1', 'bin\\start.ps1'],
    ['start-dev.ps1', 'bin\\start-dev.ps1'],
    ['stop.ps1', 'bin\\stop.ps1'],
    ['stop-dev.ps1', 'bin\\stop-dev.ps1'],
    ['build.ps1', 'bin\\build.ps1'],
    ['install.ps1', 'bin\\install.ps1'],
  ];

  for (const [rootScript, targetScript] of psEntrypoints) {
    const source = readRepoFile(rootScript);
    assert.match(source, new RegExp(`Join-Path \\$PSScriptRoot '${targetScript.replace(/\\/g, '\\\\')}'`));
    assert.match(source, /& \$target @args/);
    assert.doesNotMatch(source, /runtime-common/);
    assert.doesNotMatch(source, /router-ops\.mjs/);
  }
});

test('documentation states that root entrypoints are compatibility wrappers and bin is the managed source of truth', () => {
  const readme = readRepoFile('README.md');
  const lifecycle = readRepoFile(path.join('docs', 'getting-started', 'script-lifecycle.md'));

  assert.match(
    readme,
    /root-level start\/build\/install\/stop scripts are compatibility wrappers that delegate to `bin\/\*`/
  );
  assert.match(
    lifecycle,
    /root-level `start\.\*`, `build\.\*`, `install\.\*`, and `stop\.\*` scripts are thin compatibility wrappers/
  );
});
