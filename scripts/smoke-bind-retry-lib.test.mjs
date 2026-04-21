import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import path from 'node:path';
import test from 'node:test';
import { pathToFileURL } from 'node:url';

const repoRoot = path.resolve(import.meta.dirname, '..');

test('smoke bind retry helper classifies cross-platform bind conflict messages', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'smoke-bind-retry-lib.mjs')).href,
  );

  assert.equal(typeof module.isBindConflictError, 'function');
  assert.equal(module.isBindConflictError(new Error('listen EADDRINUSE: address already in use')), true);
  assert.equal(module.isBindConflictError(new Error('Port 4173 is already in use')), true);
  assert.equal(
    module.isBindConflictError(
      new Error('Only one usage of each socket address (protocol/network address/port) is normally permitted. (os error 10048)'),
    ),
    true,
  );
  assert.equal(module.isBindConflictError(new Error('database bootstrap failed')), false);
});

test('smoke bind retry helper retries bind conflicts and returns the later successful result', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'smoke-bind-retry-lib.mjs')).href,
  );

  const allocations = ['127.0.0.1:4101', '127.0.0.1:4102'];
  const attempted = [];
  let delayCalls = 0;
  let retryNotices = 0;

  const result = await module.runWithBindConflictRetry({
    label: 'preview smoke',
    maxAttempts: 2,
    retryDelayMs: 0,
    allocate: async ({ attempt }) => allocations[attempt - 1],
    run: async ({ allocation }) => {
      attempted.push(allocation);
      if (allocation === '127.0.0.1:4101') {
        throw new Error(`Port ${allocation.split(':').at(-1)} is already in use`);
      }
      return {
        ok: true,
        allocation,
      };
    },
    delayImpl: async () => {
      delayCalls += 1;
    },
    onRetry: () => {
      retryNotices += 1;
    },
  });

  assert.deepEqual(attempted, ['127.0.0.1:4101', '127.0.0.1:4102']);
  assert.equal(delayCalls, 1);
  assert.equal(retryNotices, 1);
  assert.deepEqual(result, {
    ok: true,
    allocation: '127.0.0.1:4102',
  });
});

test('smoke bind retry helper does not retry non-bind failures', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'smoke-bind-retry-lib.mjs')).href,
  );

  let allocationCalls = 0;
  let delayCalls = 0;

  await assert.rejects(
    () => module.runWithBindConflictRetry({
      label: 'preview smoke',
      maxAttempts: 3,
      retryDelayMs: 0,
      allocate: async () => {
        allocationCalls += 1;
        return '127.0.0.1:5101';
      },
      run: async () => {
        throw new Error('unexpected HTML output');
      },
      delayImpl: async () => {
        delayCalls += 1;
      },
    }),
    /unexpected HTML output/,
  );

  assert.equal(allocationCalls, 1);
  assert.equal(delayCalls, 0);
});

test('smoke bind retry helper exposes child failure watchers for early process exits', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'smoke-bind-retry-lib.mjs')).href,
  );

  assert.equal(typeof module.createChildFailureWatcher, 'function');
  assert.equal(typeof module.raceAgainstChildFailure, 'function');

  const child = new EventEmitter();
  const watcher = module.createChildFailureWatcher(child, {
    label: 'preview server',
  });

  child.emit('exit', 1, null);

  await assert.rejects(
    watcher.promise,
    /preview server exited before smoke completed with code 1/,
  );
});

test('smoke bind retry helper can race successful work against a child failure watcher', async () => {
  const module = await import(
    pathToFileURL(path.join(repoRoot, 'scripts', 'smoke-bind-retry-lib.mjs')).href,
  );

  const child = new EventEmitter();
  const watcher = module.createChildFailureWatcher(child, {
    label: 'preview server',
  });

  const result = await module.raceAgainstChildFailure(Promise.resolve('ok'), watcher);
  watcher.stop();

  assert.equal(result, 'ok');
});
