import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('commercial account provisioning errors are localized before account, settlements, or billing surfaces show status copy', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const accountPage = read('packages/sdkwork-router-portal-account/src/pages/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const settlementsPage = read('packages/sdkwork-router-portal-settlements/src/pages/index.tsx');

  assert.match(
    accountPage,
    /setStatus\(\s*t\(resolveCommercialAccountProvisioningStatus\(error\)\s*\?\?\s*portalErrorMessage\(error\)\)\s*\)/,
  );
  assert.match(
    billingPage,
    /setStatus\(\s*t\(resolveCommercialAccountProvisioningStatus\(error\)\s*\?\?\s*portalErrorMessage\(error\)\)\s*\)/,
  );
  assert.match(
    settlementsPage,
    /setStatus\(\s*t\(resolveCommercialAccountProvisioningStatus\(error\)\s*\?\?\s*portalErrorMessage\(error\)\)\s*\)/,
  );

  for (const key of [
    'Workspace commercial account is being prepared for this workspace.',
    'Balances, settlements, and pricing evidence will appear once workspace commercial provisioning finishes.',
  ]) {
    assert.match(commons, new RegExp(`'${key.replace(/[.*+?^${}()|[\\]\\\\]/g, '\\\\$&')}'`));
  }

  assert.doesNotMatch(accountPage, /setStatus\(portalErrorMessage\(error\)\)/);
  assert.doesNotMatch(billingPage, /setStatus\(portalErrorMessage\(error\)\)/);
  assert.doesNotMatch(settlementsPage, /setStatus\(portalErrorMessage\(error\)\)/);
});
