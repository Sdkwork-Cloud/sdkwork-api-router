import type { ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import { DesktopShellFrame } from 'sdkwork-router-portal-commons/framework/shell';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import { isTauriDesktop } from '../lib/desktop';
import {
  PORTAL_COLLAPSED_SIDEBAR_WIDTH,
  clampSidebarWidth,
} from '../lib/portalPreferences';
import { usePortalShellStore } from '../store/usePortalShellStore';
import { PortalBrandMark } from './PortalBrandMark';
import { PortalNavigationRail } from './PortalNavigationRail';
import { PortalTopNavigation } from './PortalTopNavigation';
import { WindowControls } from './WindowControls';

export function PortalDesktopShell({
  children,
  onOpenSettings,
  workspace,
}: {
  children: ReactNode;
  onOpenSettings: () => void;
  workspace: PortalWorkspaceSummary | null;
}) {
  const { t } = usePortalI18n();
  const navigate = useNavigate();
  const desktopMode = isTauriDesktop();
  const isSidebarCollapsed = usePortalShellStore((state) => state.isSidebarCollapsed);
  const sidebarWidth = usePortalShellStore((state) => state.sidebarWidth);
  const currentSidebarWidth = isSidebarCollapsed
    ? PORTAL_COLLAPSED_SIDEBAR_WIDTH
    : clampSidebarWidth(sidebarWidth);

  return (
    <div className="relative flex h-screen min-h-0 flex-col overflow-hidden [background:var(--portal-shell-background)] font-sans text-[var(--portal-text-primary)] transition-colors duration-300 isolate">
      <DesktopShellFrame
        actions={(
          <Button
            className="h-10 rounded-2xl border border-zinc-200/80 bg-white/88 px-4 text-zinc-700 shadow-[0_1px_0_rgba(15,23,42,0.04)] hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950/88 dark:text-zinc-200 dark:hover:bg-zinc-900 dark:hover:text-white"
            data-slot="portal-header-download-action"
            onClick={() => navigate('/downloads')}
            size="sm"
            variant="ghost"
          >
            {t('Download App')}
          </Button>
        )}
        bordered={false}
        brandMark={<PortalBrandMark />}
        center={<PortalTopNavigation />}
        centerMaxWidth="min(96rem, calc(100% - 8rem))"
        className="flex h-full min-h-0 flex-1 flex-col [&_[data-sdk-region='body']]:min-h-0 [&_[data-sdk-region='content']]:min-h-0 [&_[data-sdk-region='content']]:overflow-hidden [&_[data-sdk-region='sidebar']]:border-r-0 [&_[data-sdk-region='sidebar']]:bg-transparent [&_[data-sdk-region='sidebar']]:transition-[width] [&_[data-sdk-region='sidebar']]:duration-200 [&_[data-sdk-region='sidebar']]:ease-out [&_[data-sdk-region='sidebar']]:relative [&_[data-sdk-region='sidebar']]:overflow-hidden [&_[data-sdk-region='header']]:border-b-0 [&_[data-sdk-pattern='desktop-title-bar']]:bg-white/72 [&_[data-sdk-pattern='desktop-title-bar']]:backdrop-blur-xl dark:[&_[data-sdk-pattern='desktop-title-bar']]:bg-zinc-950/78"
        content={(
          <main className="scrollbar-hide relative min-h-0 min-w-0 flex-1 overflow-x-hidden overflow-y-auto bg-[var(--portal-content-background)]">
            <div className="flex min-h-full w-full flex-col gap-6 px-4 py-5 md:px-6 xl:px-8">
              {children}
            </div>
          </main>
        )}
        size="default"
        slotProps={{
          brand: {
            className: 'flex min-w-0 items-center gap-2.5',
          },
          centerShell: {
            className:
              'pointer-events-none absolute left-1/2 top-1/2 flex w-full -translate-x-1/2 -translate-y-1/2 items-center justify-center px-3 md:px-6 xl:px-8',
          },
          content: {
            className: 'flex h-full min-h-0 min-w-0 flex-col overflow-hidden',
          },
          leading: {
            className: 'flex min-w-0 flex-[0_0_auto] items-center gap-3',
          },
          trailing: {
            className: 'ml-auto flex h-full flex-[0_0_auto] items-center justify-end gap-2',
          },
          title: {
            className: 'truncate text-sm font-semibold tracking-[0.01em] text-zinc-950 dark:text-zinc-50',
          },
        }}
        sidebar={(
          <PortalNavigationRail
            onOpenSettings={onOpenSettings}
            workspace={workspace}
          />
        )}
        sidebarWidth={currentSidebarWidth}
        title={t('SDKWork Router')}
        windowControls={desktopMode ? <WindowControls /> : null}
      />
    </div>
  );
}
