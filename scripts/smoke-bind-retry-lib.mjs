import net from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';

export const DEFAULT_BIND_RETRY_ATTEMPTS = 3;
export const DEFAULT_BIND_RETRY_DELAY_MS = 250;

const BIND_CONFLICT_PATTERNS = [
  /\bEADDRINUSE\b/i,
  /\bWSAEADDRINUSE\b/i,
  /\bAddrInUse\b/i,
  /address already in use/i,
  /only one usage of each socket address/i,
  /port \d+ is already in use/i,
  /\bos error 48\b/i,
  /\bos error 98\b/i,
  /\bos error 10048\b/i,
];

export function resolvePositiveInteger(value, fallback) {
  const parsed = Number.parseInt(String(value ?? ''), 10);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : fallback;
}

export async function findAvailableTcpPort({
  host = '127.0.0.1',
} = {}) {
  return await new Promise((resolve, reject) => {
    const server = net.createServer();
    server.unref();
    server.on('error', reject);
    server.listen(0, host, () => {
      const address = server.address();
      const port = typeof address === 'object' && address ? address.port : 0;
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve(port);
      });
    });
  });
}

export async function allocateAvailableTcpPorts(count, {
  host = '127.0.0.1',
} = {}) {
  return await Promise.all(
    Array.from({ length: count }, () => findAvailableTcpPort({ host })),
  );
}

export function isBindConflictError(error) {
  const message = error instanceof Error
    ? `${error.message}\n${error.stack ?? ''}`
    : String(error ?? '');

  return BIND_CONFLICT_PATTERNS.some((pattern) => pattern.test(message));
}

export function createChildFailureWatcher(child, {
  label = 'child process',
  createSpawnError = null,
  createExitError = null,
} = {}) {
  let active = true;

  const promise = new Promise((_, reject) => {
    child.once('error', (error) => {
      if (!active) {
        return;
      }

      reject(
        typeof createSpawnError === 'function'
          ? createSpawnError(error)
          : (error instanceof Error ? error : new Error(String(error))),
      );
    });

    child.once('exit', (code, signal) => {
      if (!active) {
        return;
      }

      if (typeof createExitError === 'function') {
        reject(createExitError({ code, signal }));
        return;
      }

      if (signal) {
        reject(new Error(`${label} exited before smoke completed with signal ${signal}`));
        return;
      }

      reject(new Error(`${label} exited before smoke completed with code ${code ?? 0}`));
    });
  });

  return {
    promise,
    stop() {
      active = false;
    },
  };
}

export async function raceAgainstChildFailure(taskPromise, childFailureWatcher) {
  return await Promise.race([
    taskPromise,
    childFailureWatcher.promise,
  ]);
}

export async function runWithBindConflictRetry({
  label = 'smoke',
  maxAttempts = DEFAULT_BIND_RETRY_ATTEMPTS,
  retryDelayMs = DEFAULT_BIND_RETRY_DELAY_MS,
  allocate = async () => undefined,
  run,
  shouldRetry = isBindConflictError,
  delayImpl = delay,
  onRetry = ({ label: attemptLabel, attempt, maxAttempts: totalAttempts }) => {
    console.warn(
      `[${attemptLabel}] bind collision detected during attempt ${attempt}/${totalAttempts}; retrying with a fresh allocation`,
    );
  },
} = {}) {
  if (typeof run !== 'function') {
    throw new Error('runWithBindConflictRetry requires a run function');
  }

  let lastError = null;
  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    const allocation = await allocate({ attempt, maxAttempts });

    try {
      return await run({
        attempt,
        maxAttempts,
        allocation,
      });
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (!shouldRetry(lastError) || attempt >= maxAttempts) {
        throw lastError;
      }

      onRetry({
        label,
        attempt,
        maxAttempts,
        error: lastError,
        allocation,
      });
      // eslint-disable-next-line no-await-in-loop
      await delayImpl(retryDelayMs);
    }
  }

  throw lastError ?? new Error(`${label} failed without a captured error`);
}
