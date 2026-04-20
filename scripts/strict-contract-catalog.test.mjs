import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

async function loadModule() {
  return import(
    pathToFileURL(
      path.join(repoRoot, 'scripts', 'strict-contract-catalog.mjs'),
    ).href,
  );
}

test('strict contract catalog exports keyed and contract catalog factories', async () => {
  const module = await loadModule();

  assert.equal(typeof module.createStrictKeyedCatalog, 'function');
  assert.equal(typeof module.createStrictContractCatalog, 'function');
});

test('strict keyed catalog rejects duplicate keys and missing keys', async () => {
  const module = await loadModule();

  assert.throws(
    () => module.createStrictKeyedCatalog({
      entries: ['alpha', 'alpha'],
      getKey: (entry) => entry,
      duplicateKeyMessagePrefix: 'duplicate keyed entry',
    }),
    /duplicate keyed entry: alpha/i,
  );

  const catalog = module.createStrictKeyedCatalog({
    entries: ['alpha', 'beta'],
    getKey: (entry) => entry,
    missingKeyMessagePrefix: 'missing keyed entry',
  });

  assert.equal(catalog.find('beta'), 'beta');
  assert.throws(
    () => catalog.find('missing'),
    /missing keyed entry: missing/i,
  );
});

test('strict keyed catalog preserves requested key order and clone semantics', async () => {
  const module = await loadModule();

  const catalog = module.createStrictKeyedCatalog({
    entries: [
      {
        id: 'release-workflow-test',
        args: ['--test', 'scripts/release/tests/release-workflow.test.mjs'],
      },
      {
        id: 'publish-ghcr-manifest-test',
        args: ['--test', 'scripts/release/tests/publish-ghcr-manifest.test.mjs'],
      },
    ],
    getKey: (entry) => entry.id,
    clone: (entry) => ({
      ...entry,
      args: [...entry.args],
    }),
    missingKeyMessagePrefix: 'missing keyed definition',
  });

  const orderedDefinitions = catalog.listByKeys([
    'publish-ghcr-manifest-test',
    'release-workflow-test',
  ]);
  assert.deepEqual(
    orderedDefinitions.map(({ id }) => id),
    [
      'publish-ghcr-manifest-test',
      'release-workflow-test',
    ],
  );

  orderedDefinitions[0].args.push('--mutated');
  assert.deepEqual(
    catalog.find('publish-ghcr-manifest-test').args,
    ['--test', 'scripts/release/tests/publish-ghcr-manifest.test.mjs'],
  );

  const listedDefinitions = catalog.list();
  listedDefinitions[1].args.push('--list-mutated');
  assert.deepEqual(
    catalog.find('release-workflow-test').args,
    ['--test', 'scripts/release/tests/release-workflow.test.mjs'],
  );

  assert.throws(
    () => catalog.listByKeys(['missing-definition']),
    /missing keyed definition: missing-definition/i,
  );
});

test('strict contract catalog keeps id-based wrapper behavior', async () => {
  const module = await loadModule();

  const catalog = module.createStrictContractCatalog({
    contracts: [
      {
        id: 'workflow-step',
        message: 'workflow step contract',
      },
    ],
    duplicateIdMessagePrefix: 'duplicate workflow contract',
    missingIdMessagePrefix: 'missing workflow contract',
  });

  const contract = catalog.find('workflow-step');
  assert.deepEqual(contract, {
    id: 'workflow-step',
    message: 'workflow step contract',
  });

  contract.message = 'mutated';
  assert.equal(
    catalog.find('workflow-step').message,
    'workflow step contract',
  );

  assert.throws(
    () => module.createStrictContractCatalog({
      contracts: [{ id: 'workflow-step' }, { id: 'workflow-step' }],
      duplicateIdMessagePrefix: 'duplicate workflow contract',
    }),
    /duplicate workflow contract: workflow-step/i,
  );

  assert.throws(
    () => catalog.find('missing-workflow-step'),
    /missing workflow contract: missing-workflow-step/i,
  );
});
