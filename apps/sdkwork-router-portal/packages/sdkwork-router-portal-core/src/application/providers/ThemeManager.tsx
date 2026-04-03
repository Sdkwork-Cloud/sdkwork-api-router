import {
  createSdkworkTheme,
  SdkworkThemeProvider,
  useSdkworkTheme,
  type SdkworkColorMode,
  type SdkworkThemeOverrides,
  type SdkworkThemeSelection,
} from '@sdkwork/ui-pc-react/theme';
import { useEffect, useMemo, useState, type PropsWithChildren } from 'react';
import type { PortalThemeColor, PortalThemeMode } from 'sdkwork-router-portal-types';
import { usePortalShellStore } from '../../store/usePortalShellStore';

const PORTAL_THEME_BRANDS: Record<PortalThemeColor, NonNullable<SdkworkThemeOverrides['brand']>> = {
  'tech-blue': {
    primary: '#2563eb',
    primaryHover: '#1d4ed8',
    primarySoft: 'rgb(37 99 235 / 0.16)',
    accent: '#60a5fa',
  },
  lobster: {
    primary: '#dc2626',
    primaryHover: '#b91c1c',
    primarySoft: 'rgb(220 38 38 / 0.16)',
    accent: '#f87171',
  },
  'green-tech': {
    primary: '#059669',
    primaryHover: '#047857',
    primarySoft: 'rgb(5 150 105 / 0.16)',
    accent: '#34d399',
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
    accent: '#a78bfa',
  },
  rose: {
    primary: '#e11d48',
    primaryHover: '#be123c',
    primarySoft: 'rgb(225 29 72 / 0.16)',
    accent: '#fb7185',
  },
};

function resolveColorMode(themeMode: PortalThemeMode): SdkworkColorMode {
  if (
    themeMode === 'dark'
    || (themeMode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  ) {
    return 'dark';
  }

  return 'light';
}

function resolveThemeSelection(themeMode: PortalThemeMode): SdkworkThemeSelection {
  return themeMode;
}

function resolveThemeOverrides(
  themeColor: PortalThemeColor,
  colorMode: SdkworkColorMode,
): SdkworkThemeOverrides {
  const theme = createSdkworkTheme({
    colorMode,
    brand: PORTAL_THEME_BRANDS[themeColor],
  });

  return {
    brand: theme.brand,
  };
}

function PortalThemeBridge({
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

export function PortalThemeProvider({ children }: PropsWithChildren) {
  const themeMode = usePortalShellStore((state) => state.themeMode);
  const themeColor = usePortalShellStore((state) => state.themeColor);
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
    root.setAttribute('data-theme', themeColor);
    root.classList.toggle('dark', resolvedColorMode === 'dark');
  }, [resolvedColorMode, themeColor]);

  const themeSelection = resolveThemeSelection(themeMode);
  const overrides = useMemo(
    () => resolveThemeOverrides(themeColor, resolvedColorMode),
    [resolvedColorMode, themeColor],
  );

  return (
    <SdkworkThemeProvider defaultTheme={themeSelection} overrides={overrides}>
      <PortalThemeBridge selection={themeSelection} />
      {children}
    </SdkworkThemeProvider>
  );
}
