import type { PortalThemeColor, PortalThemeMode } from 'sdkwork-router-portal-types';

export const PORTAL_PREFERENCES_STORAGE_KEY = 'sdkwork-router-portal.preferences.v1';

export const PORTAL_COLLAPSED_SIDEBAR_WIDTH = 72;
export const PORTAL_DEFAULT_SIDEBAR_WIDTH = 252;
export const PORTAL_MIN_SIDEBAR_WIDTH = 220;
export const PORTAL_MAX_SIDEBAR_WIDTH = 360;

export const PORTAL_THEME_MODE_OPTIONS: Array<{ id: PortalThemeMode; labelKey: string }> = [
  { id: 'light', labelKey: 'Light' },
  { id: 'dark', labelKey: 'Dark' },
  { id: 'system', labelKey: 'System' },
];

export const PORTAL_THEME_COLOR_OPTIONS: Array<{
  id: PortalThemeColor;
  labelKey: string;
  previewClassName: string;
}> = [
  { id: 'tech-blue', labelKey: 'Tech Blue', previewClassName: 'bg-sky-500' },
  { id: 'lobster', labelKey: 'Lobster', previewClassName: 'bg-red-500' },
  { id: 'green-tech', labelKey: 'Green Tech', previewClassName: 'bg-emerald-500' },
  { id: 'zinc', labelKey: 'Zinc', previewClassName: 'bg-zinc-500' },
  { id: 'violet', labelKey: 'Violet', previewClassName: 'bg-violet-500' },
  { id: 'rose', labelKey: 'Rose', previewClassName: 'bg-rose-500' },
];

export function clampSidebarWidth(width: number): number {
  return Math.max(PORTAL_MIN_SIDEBAR_WIDTH, Math.min(PORTAL_MAX_SIDEBAR_WIDTH, width));
}
