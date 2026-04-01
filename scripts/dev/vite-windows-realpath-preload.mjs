import childProcess from 'node:child_process';
import { EventEmitter } from 'node:events';
import { syncBuiltinESMExports } from 'node:module';
import process from 'node:process';

function isNetUseProbe(command) {
  return typeof command === 'string' && command.trim().toLowerCase() === 'net use';
}

if (process.platform === 'win32') {
  const originalExec = childProcess.exec.bind(childProcess);

  childProcess.exec = function patchedExec(command, options, callback) {
    let resolvedCallback = callback;
    if (typeof options === 'function') {
      resolvedCallback = options;
    }

    if (isNetUseProbe(command)) {
      const child = new EventEmitter();
      child.kill = () => true;
      child.killed = false;

      queueMicrotask(() => {
        child.emit('spawn');
        if (typeof resolvedCallback === 'function') {
          resolvedCallback(null, '', '');
        }
        child.emit('exit', 0, null);
        child.emit('close', 0, null);
      });

      return child;
    }

    return originalExec(command, options, resolvedCallback);
  };

  syncBuiltinESMExports();
}
