import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const devRoot = path.resolve(import.meta.dirname, '..');

test('start-console.mjs uses the shared frontend dependency-health checks before launching pnpm', () => {
  const script = readFileSync(path.join(devRoot, 'start-console.mjs'), 'utf8');

  assert.match(script, /frontendInstallStatus/);
  assert.match(script, /frontendViteConfigHealthy/);
  assert.match(script, /requiredBinCommands/);
  assert.match(script, /pnpmProcessSpec/);
  assert.match(script, /pnpmDisplayCommand/);
  assert.match(script, /shouldReuseExistingFrontendDist/);
  assert.match(script, /allowInstallReuse/);
  assert.match(script, /pnpmSpawnOptions\(\{ stdio: 'pipe' \}\)/);
});

test('start-console.ps1 delegates to the Node launcher instead of maintaining a second PowerShell-only workflow', () => {
  const script = readFileSync(path.join(devRoot, 'start-console.ps1'), 'utf8');

  assert.match(script, /scripts\/dev\/start-console\.mjs|scripts\\dev\\start-console\.mjs/);
  assert.match(script, /& node @arguments/);
  assert.match(script, /--install/);
  assert.match(script, /--preview/);
  assert.match(script, /--tauri/);
  assert.match(script, /--dry-run/);
  assert.doesNotMatch(script, /Start-Process powershell/);
});
