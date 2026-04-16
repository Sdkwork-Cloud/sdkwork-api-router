import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');
const workspaceRoot = path.resolve(appRoot, '..', '..');

function readWorkspace(relativePath) {
  return readFileSync(path.join(workspaceRoot, relativePath), 'utf8');
}

test('zh portal API reference stays readable and documents auto-provisioned commercial account routes', () => {
  const portalApiDoc = readWorkspace('docs/zh/api-reference/portal-api.md');

  assert.match(portalApiDoc, /^# 门户 API/m);
  assert.match(portalApiDoc, /GET \/portal\/billing\/account-history/);
  assert.match(
    portalApiDoc,
    /portal 商业账户相关路由会在账户缺失时自动补齐工作区主商业账户/i,
  );
  assert.doesNotMatch(portalApiDoc, /# 闂ㄦ埛 API/);
  assert.doesNotMatch(portalApiDoc, /闂|鍩|瀹舵棌|鏈€灏/);
});
