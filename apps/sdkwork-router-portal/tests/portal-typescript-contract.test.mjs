import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const sharedUiRoot = path.resolve(appRoot, '../../../sdkwork-ui/sdkwork-ui-pc-react');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal typecheck source-links shared-ui imports to the workspace source tree without shims', () => {
  const tsconfig = read('tsconfig.json');

  assert.equal(existsSync(path.join(appRoot, 'src', 'types', 'sdkwork-ui-pc-react-shim.d.ts')), false);
  assert.doesNotMatch(tsconfig, /sdkwork-ui-pc-react-shim\.d\.ts/);
  assert.match(tsconfig, /"allowImportingTsExtensions"\s*:\s*true/);
  assert.match(
    tsconfig,
    /"@sdkwork\/ui-pc-react"\s*:\s*\[\s*"\.\.\/\.\.\/\.\.\/sdkwork-ui\/sdkwork-ui-pc-react\/src\/index\.ts"\s*\]/,
  );
  assert.match(
    tsconfig,
    /"@sdkwork\/ui-pc-react\/theme"\s*:\s*\[\s*"\.\.\/\.\.\/\.\.\/sdkwork-ui\/sdkwork-ui-pc-react\/src\/theme\/index\.ts"\s*\]/,
  );
  assert.match(
    tsconfig,
    /"@sdkwork\/ui-pc-react\/\*"\s*:\s*\[\s*"\.\.\/\.\.\/\.\.\/sdkwork-ui\/sdkwork-ui-pc-react\/src\/\*"\s*\]/,
  );
  assert.equal(existsSync(path.join(sharedUiRoot, 'src', 'index.ts')), true);
  assert.equal(existsSync(path.join(sharedUiRoot, 'src', 'theme', 'index.ts')), true);
});
