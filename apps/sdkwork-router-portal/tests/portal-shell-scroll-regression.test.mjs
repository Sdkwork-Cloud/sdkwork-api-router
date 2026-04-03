import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal desktop shell keeps scrolling inside the viewport-bound content region', () => {
  const shell = read('packages/sdkwork-router-portal-core/src/components/PortalDesktopShell.tsx');

  assert.match(
    shell,
    /className="relative flex h-screen min-h-0 flex-col overflow-hidden \[background:var\(--portal-shell-background\)\] font-sans/,
  );
  assert.match(
    shell,
    /className="flex h-full min-h-0 flex-1 flex-col \[&_\[data-sdk-region='body'\]\]:min-h-0 \[&_\[data-sdk-region='content'\]\]:min-h-0 \[&_\[data-sdk-region='content'\]\]:overflow-hidden/,
  );
  assert.match(
    shell,
    /slotProps=\{\{[\s\S]*content:\s*\{\s*className:\s*'flex h-full min-h-0 min-w-0 flex-col overflow-hidden'/,
  );
  assert.match(
    shell,
    /<main className="scrollbar-hide relative min-h-0 min-w-0 flex-1 overflow-x-hidden overflow-y-auto bg-\[var\(--portal-content-background\)\]">/,
  );
});
