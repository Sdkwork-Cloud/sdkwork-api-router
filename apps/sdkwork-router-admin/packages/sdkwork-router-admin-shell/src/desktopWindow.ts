import {
  closeDesktopWindow,
  isTauriDesktop,
  minimizeDesktopWindow,
  toggleMaximizeDesktopWindow,
} from 'sdkwork-router-admin-admin-api';

export { isTauriDesktop };

export async function minimizeWindow(): Promise<void> {
  await minimizeDesktopWindow();
}

export async function toggleMaximizeWindow(): Promise<void> {
  await toggleMaximizeDesktopWindow();
}

export async function closeWindow(): Promise<void> {
  await closeDesktopWindow();
}
