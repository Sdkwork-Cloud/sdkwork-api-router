type TauriWindowController = {
  close?: () => Promise<void>;
  maximize?: () => Promise<void>;
  minimize?: () => Promise<void>;
  toggleMaximize?: () => Promise<void>;
};

type TauriInternalsLike = {
  invoke?: <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
};

type TauriWindowLike = Window & {
  __TAURI__?: unknown;
  __TAURI_INTERNALS__?: TauriInternalsLike;
  isTauri?: boolean;
};

function resolveWindow(): TauriWindowLike | null {
  if (typeof window === 'undefined') {
    return null;
  }

  return window as TauriWindowLike;
}

export function isTauriDesktop(): boolean {
  const currentWindow = resolveWindow();
  return Boolean(
    currentWindow?.isTauri
      || currentWindow?.__TAURI__
      || currentWindow?.__TAURI_INTERNALS__,
  );
}

export async function invokeDesktopCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const invoke = resolveWindow()?.__TAURI_INTERNALS__?.invoke;
  if (typeof invoke !== 'function') {
    throw new Error('Tauri invoke bridge is unavailable.');
  }

  return invoke<T>(command, args);
}

async function getCurrentTauriWindow(): Promise<TauriWindowController | null> {
  if (!isTauriDesktop()) {
    return null;
  }

  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    return getCurrentWindow();
  } catch {
    return null;
  }
}

export async function minimizeDesktopWindow(): Promise<void> {
  const currentWindow = await getCurrentTauriWindow();
  await currentWindow?.minimize?.();
}

export async function maximizeDesktopWindow(): Promise<void> {
  const currentWindow = await getCurrentTauriWindow();

  if (!currentWindow) {
    return;
  }

  if (currentWindow.toggleMaximize) {
    await currentWindow.toggleMaximize();
    return;
  }

  await currentWindow.maximize?.();
}

export async function closeDesktopWindow(): Promise<void> {
  const currentWindow = await getCurrentTauriWindow();
  await currentWindow?.close?.();
}
