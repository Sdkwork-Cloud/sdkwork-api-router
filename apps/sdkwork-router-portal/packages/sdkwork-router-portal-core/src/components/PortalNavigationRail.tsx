import {
  Activity,
  ChevronUp,
  Coins,
  Gauge,
  KeyRound,
  LogOut,
  PanelLeftClose,
  PanelLeftOpen,
  ReceiptText,
  Route,
  Server,
  Settings2,
  UserRound,
  WalletCards,
  type LucideIcon,
} from 'lucide-react';
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from 'react';
import { NavLink, useLocation, useNavigate } from 'react-router-dom';
import { usePortalI18n } from 'sdkwork-router-portal-commons';
import type {
  PortalRouteGroupKey,
  PortalRouteKey,
  PortalWorkspaceSummary,
} from 'sdkwork-router-portal-types';

import {
  cancelSidebarRoutePrefetch,
  prefetchSidebarRoute,
  scheduleSidebarRoutePrefetch,
} from '../application/router/routePrefetch';
import { resolvePortalPath } from '../application/router/routeManifest';
import { PORTAL_ROUTE_PATHS } from '../application/router/routePaths';
import {
  PORTAL_MIN_SIDEBAR_WIDTH,
  clampSidebarWidth,
} from '../lib/portalPreferences';
import { portalSidebarRoutes } from '../routes';
import { usePortalAuthStore } from '../store/usePortalAuthStore';
import { usePortalShellStore } from '../store/usePortalShellStore';

const routeIcons: Record<PortalRouteKey, LucideIcon> = {
  gateway: Server,
  dashboard: Gauge,
  routing: Route,
  'api-keys': KeyRound,
  usage: Activity,
  user: UserRound,
  credits: Coins,
  recharge: WalletCards,
  billing: WalletCards,
  settlements: ReceiptText,
  account: ReceiptText,
};

const routeGroupOrder: PortalRouteGroupKey[] = ['operations', 'access', 'revenue'];

const routeGroupLabelKeys: Record<PortalRouteGroupKey, string> = {
  operations: 'Operations',
  access: 'Access',
  revenue: 'Revenue',
};

interface SidebarNavItem {
  key: PortalRouteKey;
  labelKey: string;
  eyebrowKey: string;
  detailKey: string;
  path: string;
  icon: LucideIcon;
}

interface SidebarNavGroup {
  key: PortalRouteGroupKey;
  section: string;
  items: SidebarNavItem[];
}

function resolveUserDisplayName(workspace: PortalWorkspaceSummary | null) {
  return workspace?.user.display_name || workspace?.user.email || null;
}

function buildInitials(label: string) {
  const parts = label
    .split(/\s+/)
    .map((part) => part.trim())
    .filter(Boolean)
    .slice(0, 2);

  if (parts.length === 0) {
    return 'PR';
  }

  return parts.map((part) => part[0]?.toUpperCase() ?? '').join('') || 'PR';
}

export function PortalNavigationRail({
  onOpenSettings,
  workspace,
}: {
  onOpenSettings: () => void;
  workspace: PortalWorkspaceSummary | null;
}) {
  const { t } = usePortalI18n();
  const navigate = useNavigate();
  const location = useLocation();
  const storedWorkspace = usePortalAuthStore((state) => state.workspace);
  const signOut = usePortalAuthStore((state) => state.signOut);
  const hiddenSidebarItems = usePortalShellStore((state) => state.hiddenSidebarItems);
  const isSidebarCollapsed = usePortalShellStore((state) => state.isSidebarCollapsed);
  const sidebarWidth = usePortalShellStore((state) => state.sidebarWidth);
  const toggleSidebar = usePortalShellStore((state) => state.toggleSidebar);
  const setSidebarCollapsed = usePortalShellStore((state) => state.setSidebarCollapsed);
  const setSidebarWidth = usePortalShellStore((state) => state.setSidebarWidth);
  const resolvedWorkspace = workspace ?? storedWorkspace;
  const [isSidebarHovered, setIsSidebarHovered] = useState(false);
  const [isSidebarResizing, setIsSidebarResizing] = useState(false);
  const [isUserMenuOpen, setIsUserMenuOpen] = useState(false);
  const resizeStartXRef = useRef(0);
  const resizeStartWidthRef = useRef(0);
  const userMenuRef = useRef<HTMLDivElement>(null);

  const resolvedSidebarWidth = clampSidebarWidth(sidebarWidth);
  const userDisplayName = resolveUserDisplayName(resolvedWorkspace) ?? t('Portal workspace');
  const userEmail = resolvedWorkspace?.user.email ?? t('Awaiting workspace session');
  const userInitials = buildInitials(userDisplayName);
  const userMenuTitle = isUserMenuOpen ? t('Close') : t('Open account');

  useEffect(() => {
    if (resolvedSidebarWidth !== sidebarWidth) {
      setSidebarWidth(resolvedSidebarWidth);
    }
  }, [resolvedSidebarWidth, setSidebarWidth, sidebarWidth]);

  useEffect(() => {
    if (!isSidebarResizing) {
      return;
    }

    const previousCursor = document.body.style.cursor;
    const previousUserSelect = document.body.style.userSelect;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';

    const handlePointerMove = (event: PointerEvent) => {
      const nextWidth = clampSidebarWidth(
        resizeStartWidthRef.current + (event.clientX - resizeStartXRef.current),
      );
      setSidebarWidth(nextWidth);
    };

    const handlePointerUp = () => {
      setIsSidebarResizing(false);
    };

    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);

    return () => {
      document.body.style.cursor = previousCursor;
      document.body.style.userSelect = previousUserSelect;
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
  }, [isSidebarResizing, setSidebarWidth]);

  useEffect(() => {
    setIsUserMenuOpen(false);
  }, [isSidebarCollapsed, location.pathname, location.search]);

  useEffect(() => {
    if (!isUserMenuOpen) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      if (!userMenuRef.current?.contains(event.target as Node)) {
        setIsUserMenuOpen(false);
      }
    };

    window.addEventListener('pointerdown', handlePointerDown);
    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
    };
  }, [isUserMenuOpen]);

  const startSidebarResize = useCallback(
    (event: ReactPointerEvent<HTMLDivElement>) => {
      event.preventDefault();
      event.stopPropagation();

      const nextWidth = isSidebarCollapsed ? PORTAL_MIN_SIDEBAR_WIDTH : resolvedSidebarWidth;
      resizeStartXRef.current = event.clientX;
      resizeStartWidthRef.current = nextWidth;

      if (isSidebarCollapsed) {
        setSidebarCollapsed(false);
        setSidebarWidth(nextWidth);
      }

      setIsSidebarResizing(true);
    },
    [isSidebarCollapsed, resolvedSidebarWidth, setSidebarCollapsed, setSidebarWidth],
  );

  const navGroups: SidebarNavGroup[] = routeGroupOrder
    .map((groupKey) => ({
      key: groupKey,
      section: t(routeGroupLabelKeys[groupKey]),
      items: portalSidebarRoutes
        .filter(
          (route) =>
            route.group === groupKey && !hiddenSidebarItems.includes(route.key),
        )
        .map((route) => ({
          key: route.key,
          labelKey: route.labelKey,
          eyebrowKey: route.eyebrowKey,
          detailKey: route.detailKey,
          path: resolvePortalPath(route.key),
          icon: routeIcons[route.key],
        })),
    }))
    .filter((group) => group.items.length > 0);

  const activeGroupKey =
    navGroups.find((group) =>
      group.items.some(
        (item) =>
          location.pathname === item.path || location.pathname.startsWith(`${item.path}/`),
      ),
    )?.key ?? null;

  const showEdgeAffordances = isSidebarHovered || isSidebarResizing;

  const handleOpenSettings = () => {
    setIsUserMenuOpen(false);
    onOpenSettings();
  };

  const handleOpenUserDetails = () => {
    setIsUserMenuOpen(false);
    navigate(resolvePortalPath('user'));
  };

  const handleSignOut = async () => {
    setIsUserMenuOpen(false);
    await signOut();
    navigate(PORTAL_ROUTE_PATHS.login, { replace: true });
  };

  return (
    <div
      className="relative z-20 flex h-full w-full"
      onMouseEnter={() => setIsSidebarHovered(true)}
      onMouseLeave={() => setIsSidebarHovered(false)}
    >
      <div className="flex h-full w-full flex-col overflow-hidden border-r border-zinc-900/90 bg-zinc-950 [background:var(--portal-sidebar-background)] [border-color:var(--portal-sidebar-border)] text-zinc-300 shadow-[var(--portal-sidebar-shadow)]">
        <nav
          className={`scrollbar-hide relative flex-1 overflow-x-hidden overflow-y-auto ${
            isSidebarCollapsed ? 'px-2 py-4' : 'px-3 py-5'
          }`}
        >
          <div className={isSidebarCollapsed ? 'space-y-4' : 'space-y-5'}>
            {navGroups.map((group) => {
              const isGroupActive = activeGroupKey === group.key;

              return (
                <section
                  key={group.section}
                  className={isSidebarCollapsed ? 'space-y-2.5' : 'space-y-2'}
                  data-slot="sidebar-nav-group"
                >
                  <div
                    className={`px-3 text-[11px] font-semibold uppercase tracking-[0.18em] ${
                      isSidebarCollapsed
                        ? 'sr-only'
                        : isGroupActive
                          ? 'text-zinc-300'
                          : 'text-zinc-500'
                    }`}
                    data-slot="sidebar-nav-group-header"
                  >
                    {group.section}
                  </div>

                  <div
                    className={isSidebarCollapsed ? 'space-y-1.5' : 'space-y-1'}
                    data-slot="sidebar-nav-list"
                  >
                    {group.items.map((item) => (
                      <NavLink
                        data-slot="sidebar-nav-item"
                        key={item.key}
                        title={isSidebarCollapsed ? t(item.labelKey) : undefined}
                        to={item.path}
                        onBlur={() => cancelSidebarRoutePrefetch(item.path)}
                        onFocus={() => scheduleSidebarRoutePrefetch(item.path)}
                        onMouseEnter={() => scheduleSidebarRoutePrefetch(item.path)}
                        onMouseLeave={() => cancelSidebarRoutePrefetch(item.path)}
                        onPointerDown={() => prefetchSidebarRoute(item.path)}
                        className={({ isActive }) =>
                          `group relative flex items-center rounded-2xl transition-colors duration-150 ${
                            isSidebarCollapsed
                              ? 'mx-auto h-11 w-11 justify-center'
                              : 'h-10 gap-3 px-3'
                          } ${
                            isActive
                              ? 'bg-white/[0.08] text-white'
                              : 'text-zinc-400 hover:bg-white/[0.05] hover:text-zinc-200'
                          }`
                        }
                      >
                        {({ isActive }) => (
                          <>
                            <item.icon
                              className={`h-4 w-4 shrink-0 ${
                                isActive ? 'text-white' : 'text-zinc-500 group-hover:text-zinc-200'
                              }`}
                            />
                            {!isSidebarCollapsed ? (
                              <span className="truncate text-sm tracking-tight text-current">
                                {t(item.labelKey)}
                              </span>
                            ) : null}
                          </>
                        )}
                      </NavLink>
                    ))}
                  </div>
                </section>
              );
            })}
          </div>
        </nav>

        <div className="border-t border-white/6 p-3">
          <div
            className="relative"
            data-slot="sidebar-profile-shell"
            ref={userMenuRef}
          >
            {isUserMenuOpen ? (
              <div
                className={`absolute z-40 rounded-2xl border border-white/10 bg-zinc-950/96 p-2 shadow-[0_22px_52px_rgba(2,6,23,0.4)] backdrop-blur-xl ${
                  isSidebarCollapsed ? 'bottom-0 left-full ml-3 w-64' : 'bottom-full left-0 right-0 mb-2'
                }`}
              >
                <div className="mb-2 rounded-xl border border-white/8 bg-white/[0.04] p-3">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-white/[0.08] text-sm font-bold text-white">
                      {userInitials}
                    </div>
                    <div className="min-w-0">
                      <div className="truncate text-sm font-semibold text-white">
                        {userDisplayName}
                      </div>
                      <div className="truncate text-xs text-zinc-400">{userEmail}</div>
                    </div>
                  </div>
                  <div className="mt-3 inline-flex items-center rounded-full border border-white/8 bg-white/[0.05] px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-300">
                    {resolvedWorkspace?.project.name ?? t('Portal workspace')}
                  </div>
                </div>

                <button
                  type="button"
                  onClick={handleOpenUserDetails}
                  className="flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm text-zinc-300 transition-colors hover:bg-white/[0.06] hover:text-white"
                >
                  <UserRound className="h-4 w-4 text-zinc-500" />
                  <span>{t('User details')}</span>
                </button>

                <button
                  type="button"
                  onClick={handleOpenSettings}
                  className="mt-1 flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm text-zinc-300 transition-colors hover:bg-white/[0.06] hover:text-white"
                >
                  <Settings2 className="h-4 w-4 text-zinc-500" />
                  <span>{t('Settings')}</span>
                </button>

                <button
                  type="button"
                  onClick={() => {
                    void handleSignOut();
                  }}
                  className="mt-1 flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm text-rose-300 transition-colors hover:bg-rose-500/10 hover:text-rose-200"
                >
                  <LogOut className="h-4 w-4" />
                  <span>{t('Sign out')}</span>
                </button>
              </div>
            ) : null}

            <button
              type="button"
              className={`group relative flex w-full items-center rounded-2xl text-zinc-300 transition-colors duration-150 hover:bg-white/[0.05] hover:text-white ${
                isSidebarCollapsed
                  ? 'mx-auto h-11 w-11 justify-center px-0'
                  : 'gap-3 px-3 py-2.5'
              }`}
              data-slot="sidebar-user-control"
              onClick={() => setIsUserMenuOpen((open) => !open)}
              title={isSidebarCollapsed ? userMenuTitle : undefined}
            >
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-white/[0.08] text-sm font-semibold text-white">
                {userInitials}
              </div>

              {!isSidebarCollapsed ? (
                <>
                  <div className="min-w-0 flex-1 text-left">
                    <div className="truncate text-sm font-semibold text-white">
                      {userDisplayName}
                    </div>
                    <div className="truncate text-xs text-zinc-500">{userEmail}</div>
                  </div>
                  <ChevronUp
                    className={`h-4 w-4 shrink-0 text-zinc-500 transition-transform ${
                      isUserMenuOpen ? '' : 'rotate-180'
                    }`}
                  />
                </>
              ) : null}
            </button>
          </div>
        </div>
      </div>

      <button
        type="button"
        data-slot="sidebar-edge-control"
        title={isSidebarCollapsed ? t('Expand sidebar') : t('Collapse sidebar')}
        onClick={toggleSidebar}
        className={`absolute right-0 top-1/2 z-30 flex h-8 w-8 -translate-y-1/2 translate-x-1/2 items-center justify-center rounded-full border border-white/10 bg-zinc-950/95 text-zinc-200 shadow-[0_12px_28px_rgba(2,6,23,0.32)] backdrop-blur-xl transition-all duration-200 dark:bg-zinc-900 ${
          showEdgeAffordances
            ? 'opacity-100 hover:scale-105 hover:bg-zinc-900'
            : 'pointer-events-none opacity-0'
        }`}
      >
        {isSidebarCollapsed ? (
          <PanelLeftOpen className="h-4 w-4" />
        ) : (
          <PanelLeftClose className="h-4 w-4" />
        )}
      </button>

      <div
        data-slot="sidebar-resize-handle"
        onPointerDown={startSidebarResize}
        className="absolute inset-y-0 right-0 z-20 w-3 cursor-col-resize touch-none"
      />
    </div>
  );
}
