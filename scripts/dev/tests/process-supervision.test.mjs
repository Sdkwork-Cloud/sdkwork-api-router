import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import test from 'node:test';

import {
  createSignalController,
  didChildExitFail,
  waitForChildExit,
} from '../process-supervision.mjs';

class FakeChild extends EventEmitter {
  constructor() {
    super();
    this.exitCode = null;
    this.signalCode = null;
    this.killed = false;
    this.killCalls = [];
  }

  kill(signal) {
    this.killed = true;
    this.killCalls.push(signal);
    return true;
  }

  exit(code = 0, signal = null) {
    this.exitCode = code;
    this.signalCode = signal;
    this.emit('exit', code, signal);
  }
}

test('didChildExitFail only treats non-zero exit codes or signals as fatal', () => {
  assert.equal(didChildExitFail(0, null), false);
  assert.equal(didChildExitFail(1, null), true);
  assert.equal(didChildExitFail(null, 'SIGTERM'), true);
});

test('waitForChildExit waits for graceful termination before resolving', async () => {
  const child = new FakeChild();

  const waitPromise = waitForChildExit(child, {
    forceKillAfterMs: 100,
    settleAfterForceMs: 20,
  });
  setTimeout(() => child.exit(0, null), 10);

  await waitPromise;
  assert.deepEqual(child.killCalls, ['SIGTERM']);
});

test('waitForChildExit escalates to SIGKILL after the graceful timeout', async () => {
  const child = new FakeChild();

  await waitForChildExit(child, {
    forceKillAfterMs: 5,
    settleAfterForceMs: 5,
  });

  assert.deepEqual(child.killCalls, ['SIGTERM', 'SIGKILL']);
});

test('createSignalController exits only after supervised children stop', async () => {
  const child = new FakeChild();
  const exitCalls = [];
  const logs = [];
  const controller = createSignalController({
    label: 'start-workspace',
    children: [child],
    logger: (message) => logs.push(message),
    exit: (code) => exitCalls.push(code),
    forceKillAfterMs: 100,
    settleAfterForceMs: 20,
  });

  const shutdownPromise = controller.shutdown('SIGTERM', 7);
  assert.deepEqual(exitCalls, []);

  setTimeout(() => child.exit(0, null), 10);
  await shutdownPromise;

  assert.deepEqual(exitCalls, [7]);
  assert.match(logs.join('\n'), /received SIGTERM, stopping child processes/);
});
