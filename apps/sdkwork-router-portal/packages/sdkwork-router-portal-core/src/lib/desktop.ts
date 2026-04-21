import {
  closeDesktopWindow,
  isTauriDesktop,
  maximizeDesktopWindow,
  minimizeDesktopWindow,
} from 'sdkwork-router-portal-portal-api';

export { isTauriDesktop };

export async function minimizeWindow(): Promise<void> {
  await minimizeDesktopWindow();
}

export async function maximizeWindow(): Promise<void> {
  await maximizeDesktopWindow();
}

export async function closeWindow(): Promise<void> {
  await closeDesktopWindow();
}
