import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { AdminI18nProvider } from 'sdkwork-router-admin-core';

import { AdminThemeProvider } from './ThemeManager';

function resolveBaseName(): string {
  const baseName = import.meta.env.BASE_URL ?? '/admin/';
  return baseName === '/' ? '/' : baseName.replace(/\/$/, '');
}

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <AdminI18nProvider>
      <BrowserRouter basename={resolveBaseName()}>
        <AdminThemeProvider>
          {children}
        </AdminThemeProvider>
      </BrowserRouter>
    </AdminI18nProvider>
  );
}
