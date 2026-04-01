import process from 'node:process';
const viteWindowsRealpathPreloadOption = `--import=${new URL(
  './vite-windows-realpath-preload.mjs',
  import.meta.url,
).href}`;

function withWindowsNodeOptions(env, platform) {
  if (platform !== 'win32') {
    return env;
  }

  const currentNodeOptions = String(env.NODE_OPTIONS ?? '').trim();
  if (currentNodeOptions.includes(viteWindowsRealpathPreloadOption)) {
    return env;
  }

  return {
    ...env,
    NODE_OPTIONS: currentNodeOptions
      ? `${currentNodeOptions} ${viteWindowsRealpathPreloadOption}`
      : viteWindowsRealpathPreloadOption,
  };
}

export function pnpmExecutable(platform = process.platform) {
  return platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

export function pnpmSpawnOptions({
  platform = process.platform,
  env = process.env,
  cwd,
  stdio = 'inherit',
} = {}) {
  const effectiveEnv = withWindowsNodeOptions(env, platform);
  const options = {
    env: effectiveEnv,
    shell: platform === 'win32',
    stdio,
    windowsHide: platform === 'win32',
  };

  if (cwd) {
    options.cwd = cwd;
  }

  return options;
}
