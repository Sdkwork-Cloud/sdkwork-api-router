import { RefreshCw, Search } from 'lucide-react';
import { useEffect, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';

import { useAdminI18n, useAdminWorkbench } from 'sdkwork-router-admin-core';

import { ROUTE_PATHS } from '../application/router/routePaths';
import { ShellStatus } from './ShellStatus';

const routerAdminLogo = new URL('../../../../src-tauri/icons/32x32.png', import.meta.url).href;

function HeaderActionButton({
  title,
  onClick,
  children,
  className = '',
  dataSlot,
}: {
  title: string;
  onClick: () => void;
  children: ReactNode;
  className?: string;
  dataSlot?: string;
}) {
  return (
    <button
      className={`flex h-9 items-center justify-center rounded-xl [background:var(--admin-header-control-background)] px-3 text-[var(--admin-text-secondary)] transition-colors hover:[background:var(--admin-header-control-hover)] hover:text-[var(--admin-text-primary)] ${className}`}
      data-slot={dataSlot}
      data-tauri-drag-region="false"
      onClick={onClick}
      title={title}
      type="button"
    >
      {children}
    </button>
  );
}

export function AppHeader() {
  const navigate = useNavigate();
  const { loading, refreshWorkspace, status } = useAdminWorkbench();
  const { t } = useAdminI18n();

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        navigate(ROUTE_PATHS.SETTINGS);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [navigate]);

  return (
    <div className="relative z-30 border-b border-[var(--admin-border-color)] [background:var(--admin-header-background)] backdrop-blur-xl">
      <header className="relative flex h-12 items-center px-3 sm:px-4">
        <div
          className="flex min-w-0 flex-1 items-center gap-3"
          data-slot="app-header-leading"
          data-tauri-drag-region
        >
          <button
            className="flex min-w-0 items-center gap-2 rounded-xl px-1 py-1 transition-colors hover:[background:var(--admin-header-control-background)]"
            data-slot="app-header-brand"
            data-tauri-drag-region="false"
            onClick={() => navigate(ROUTE_PATHS.OVERVIEW)}
            title={t('Router Admin')}
            type="button"
          >
            <span className="flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-[10px] border border-[var(--admin-border-color)] [background:var(--admin-header-control-background)]">
              <img
                alt={t('Router Admin')}
                className="h-5 w-5 object-contain"
                src={routerAdminLogo}
              />
            </span>
            <span className="min-w-0">
              <span className="block truncate text-sm font-semibold text-[var(--admin-text-primary)]">
                {t('Router Admin')}
              </span>
            </span>
          </button>
          <HeaderActionButton
            className="gap-2 px-2.5"
            dataSlot="app-header-search"
            onClick={() => navigate(ROUTE_PATHS.SETTINGS)}
            title={t('Open workspace search')}
          >
            <Search className="h-4 w-4" />
            <span className="hidden text-xs font-medium md:inline">{t('Search')}</span>
            <span className="hidden rounded-full [background:var(--admin-header-control-background)] px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--admin-text-muted)] md:inline">
              {t('Ctrl K')}
            </span>
          </HeaderActionButton>
        </div>

        <div
          className="ml-auto flex h-full items-center justify-end gap-2"
          data-slot="app-header-trailing"
          data-tauri-drag-region="false"
        >
          <div className="hidden lg:block">
            <ShellStatus status={status} />
          </div>
          <HeaderActionButton
            className="gap-2 px-2.5"
            dataSlot="app-header-refresh"
            onClick={() => void refreshWorkspace()}
            title={loading ? t('Refreshing workspace') : t('Refresh')}
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            <span className="hidden text-xs font-medium lg:inline">{t('Refresh')}</span>
          </HeaderActionButton>
        </div>
      </header>
    </div>
  );
}
