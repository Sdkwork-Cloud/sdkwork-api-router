import {
  createSdkworkTheme,
  SdkworkThemeProvider,
  useSdkworkTheme,
  type SdkworkColorMode,
  type SdkworkTheme,
  type SdkworkThemeOverrides,
  type SdkworkThemeSelection,
} from '@sdkwork/ui-pc-react/theme';
import { useEffect, useMemo, useState, type PropsWithChildren } from 'react';

import { useAdminAppStore } from 'sdkwork-router-admin-core';
import type { ThemeColor, ThemeMode } from 'sdkwork-router-admin-types';

const ADMIN_THEME_BRANDS: Record<ThemeColor, NonNullable<SdkworkThemeOverrides['brand']>> = {
  'tech-blue': {
    primary: '#2563eb',
    primaryHover: '#1d4ed8',
    primarySoft: 'rgb(37 99 235 / 0.16)',
    accent: '#60a5fa',
  },
  lobster: {
    primary: '#ea580c',
    primaryHover: '#c2410c',
    primarySoft: 'rgb(234 88 12 / 0.16)',
    accent: '#fb7185',
  },
  'green-tech': {
    primary: '#059669',
    primaryHover: '#047857',
    primarySoft: 'rgb(5 150 105 / 0.16)',
    accent: '#2dd4bf',
  },
  zinc: {
    primary: '#52525b',
    primaryHover: '#3f3f46',
    primarySoft: 'rgb(82 82 91 / 0.16)',
    accent: '#a1a1aa',
  },
  violet: {
    primary: '#7c3aed',
    primaryHover: '#6d28d9',
    primarySoft: 'rgb(124 58 237 / 0.16)',
    accent: '#c084fc',
  },
  rose: {
    primary: '#e11d48',
    primaryHover: '#be123c',
    primarySoft: 'rgb(225 29 72 / 0.16)',
    accent: '#fb7185',
  },
};

function resolveColorMode(themeMode: ThemeMode): SdkworkColorMode {
  if (
    themeMode === 'dark'
    || (themeMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  ) {
    return 'dark';
  }

  return 'light';
}

function resolveThemeSelection(themeMode: ThemeMode): SdkworkThemeSelection {
  return themeMode;
}

function resolveThemeOverrides(
  themeColor: ThemeColor,
  colorMode: SdkworkColorMode,
): SdkworkThemeOverrides {
  const theme = createSdkworkTheme({
    colorMode,
    brand: ADMIN_THEME_BRANDS[themeColor],
  });

  return {
    brand: theme.brand,
  };
}

function AdminThemeBridge({
  selection,
}: {
  selection: SdkworkThemeSelection;
}) {
  const { setThemeSelection } = useSdkworkTheme();

  useEffect(() => {
    setThemeSelection(selection);
  }, [selection, setThemeSelection]);

  return null;
}

export function AdminThemeProvider({ children }: PropsWithChildren) {
  const { themeMode, themeColor } = useAdminAppStore();
  const [resolvedColorMode, setResolvedColorMode] = useState<SdkworkColorMode>(() =>
    resolveColorMode(themeMode),
  );

  useEffect(() => {
    const applyThemeMode = () => {
      setResolvedColorMode(resolveColorMode(themeMode));
    };

    applyThemeMode();

    if (themeMode !== 'system') {
      return undefined;
    }

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', applyThemeMode);
    return () => mediaQuery.removeEventListener('change', applyThemeMode);
  }, [themeMode]);

  useEffect(() => {
    const root = document.documentElement;
    const colorMode = resolvedColorMode;
    root.setAttribute('data-theme', themeColor);
    root.setAttribute('data-sdk-color-mode', colorMode);
    root.classList.toggle('dark', colorMode === 'dark');
  }, [resolvedColorMode, themeColor]);

  const themeSelection = resolveThemeSelection(themeMode);
  const overrides = useMemo(
    () => resolveThemeOverrides(themeColor, resolvedColorMode),
    [resolvedColorMode, themeColor],
  );

  return (
    <SdkworkThemeProvider defaultTheme={themeSelection} overrides={overrides}>
      <AdminThemeBridge selection={themeSelection} />
      {children}
    </SdkworkThemeProvider>
  );
}

export function useAdminShellTheme(): SdkworkTheme {
  const { themeMode, themeColor } = useAdminAppStore();
  const resolvedColorMode = resolveColorMode(themeMode);

  return useMemo(
    () => createSdkworkTheme({
      colorMode: resolvedColorMode,
      brand: ADMIN_THEME_BRANDS[themeColor],
    }),
    [resolvedColorMode, themeColor],
  );
}

export const ThemeManager = AdminThemeProvider;
