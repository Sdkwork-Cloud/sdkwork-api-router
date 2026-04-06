import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadOrderAuditLookupModule() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-commercial',
      'src',
      'orderAuditLookup.ts',
    ),
  );
}

test('admin commercial order audit lookup normalizes operator input before opening the drawer', () => {
  const {
    hasCommercialOrderAuditLookupValue,
    normalizeCommercialOrderAuditLookupValue,
  } = loadOrderAuditLookupModule();

  assert.equal(
    normalizeCommercialOrderAuditLookupValue('  order-refunded  '),
    'order-refunded',
  );
  assert.equal(normalizeCommercialOrderAuditLookupValue('\n\torder-1\t'), 'order-1');
  assert.equal(hasCommercialOrderAuditLookupValue(' order-1 '), true);
  assert.equal(hasCommercialOrderAuditLookupValue('   '), false);
});
