import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { PortalI18nProvider } from 'sdkwork-router-portal-commons';

import { PortalThemeProvider } from './ThemeManager';

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <PortalThemeProvider>
      <PortalI18nProvider>
        <BrowserRouter basename="/portal">{children}</BrowserRouter>
      </PortalI18nProvider>
    </PortalThemeProvider>
  );
}
