import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal typecheck resolves shared-ui root imports through the official dist type entrypoints', () => {
  const tsconfig = read('tsconfig.json');

  assert.equal(existsSync(path.join(appRoot, 'src', 'types', 'sdkwork-ui-pc-react-shim.d.ts')), false);
  assert.doesNotMatch(tsconfig, /sdkwork-ui-pc-react-shim\.d\.ts/);
  assert.match(
    tsconfig,
    /"@sdkwork\/ui-pc-react"\s*:\s*\[\s*"\.\.\/\.\.\/\.\.\/sdkwork-ui\/sdkwork-ui-pc-react\/dist\/src\/index\.d\.ts"\s*\]/,
  );
  assert.match(
    tsconfig,
    /"@sdkwork\/ui-pc-react\/theme"\s*:\s*\[\s*"\.\.\/\.\.\/\.\.\/sdkwork-ui\/sdkwork-ui-pc-react\/dist\/src\/theme\/index\.d\.ts"\s*\]/,
  );
  assert.match(
    tsconfig,
    /"@sdkwork\/ui-pc-react\/\*"\s*:\s*\[\s*"\.\.\/\.\.\/\.\.\/sdkwork-ui\/sdkwork-ui-pc-react\/dist\/src\/\*"\s*\]/,
  );
});
