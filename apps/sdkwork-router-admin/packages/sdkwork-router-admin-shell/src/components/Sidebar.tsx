import {
  Activity,
  Blocks,
  Building2,
  ChevronUp,
  CircleUserRound,
  Gauge,
  LogIn,
  LogOut,
  PanelLeftClose,
  PanelLeftOpen,
  ServerCog,
  Settings2,
  ShieldCheck,
  TicketPercent,
  TimerReset,
  Users,
  Waypoints,
  type LucideIcon,
} from 'lucide-react';
import { motion } from 'motion/react';
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from 'react';
import { NavLink, useLocation, useNavigate } from 'react-router-dom';

import {
  ADMIN_ROUTE_PATHS,
  adminRoutePathByKey,
  adminRoutes,
  useAdminAppStore,
  useAdminI18n,
  useAdminWorkbench,
} from 'sdkwork-router-admin-core';
import type { AdminRouteKey } from 'sdkwork-router-admin-types';

import {
  cancelSidebarRoutePrefetch,
  prefetchSidebarRoute,
  scheduleSidebarRoutePrefetch,
} from '../application/router/routePrefetch';

const COLLAPSED_SIDEBAR_WIDTH = 72;
const MIN_SIDEBAR_WIDTH = 220;
const MAX_SIDEBAR_WIDTH = 360;

interface SidebarNavItem {
  key: AdminRouteKey;
  to: string;
  icon: LucideIcon;
  label: string;
}

interface SidebarNavGroup {
  section: string;
  items: SidebarNavItem[];
}

const iconByRoute: Record<AdminRouteKey, LucideIcon> = {
  overview: Gauge,
  users: Users,
  tenants: Building2,
  coupons: TicketPercent,
  'api-keys': ShieldCheck,
  'rate-limits': TimerReset,
  'route-config': Waypoints,
  'model-mapping': Waypoints,
  'usage-records': Activity,
  catalog: Blocks,
  traffic: Activity,
  operations: ServerCog,
  settings: Settings2,
};

function clampSidebarWidth(width: number) {
  return Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, width));
}

function buildAvatarInitials(value?: string | null) {
  if (!value) {
    return 'RA';
  }

  const normalized = value.trim();
  if (!normalized) {
    return 'RA';
  }

  const parts = normalized.split(/\s+/).filter(Boolean);
  if (parts.length > 1) {
    return `${parts[0][0] ?? ''}${parts[1][0] ?? ''}`.toUpperCase();
  }

  return normalized.slice(0, 2).toUpperCase();
}

function buildLoginTarget(redirectTarget: string) {
  return `${ADMIN_ROUTE_PATHS.LOGIN}?redirect=${encodeURIComponent(redirectTarget)}`;
}

export function Sidebar() {
  const { t } = useAdminI18n();
  const navigate = useNavigate();
  const location = useLocation();
  const {
    hiddenSidebarItems,
    isSidebarCollapsed,
    sidebarWidth,
    toggleSidebar,
    setSidebarCollapsed,
    setSidebarWidth,
  } = useAdminAppStore();
  const { handleLogout, sessionUser } = useAdminWorkbench();
  const [isSidebarHovered, setIsSidebarHovered] = useState(false);
  const [isSidebarResizing, setIsSidebarResizing] = useState(false);
  const [isUserMenuOpen, setIsUserMenuOpen] = useState(false);
  const resizeStartXRef = useRef(0);
  const resizeStartWidthRef = useRef(0);
  const userMenuRef = useRef<HTMLDivElement>(null);
  const accountSettingsTarget = adminRoutePathByKey.settings;
  const loginTarget = buildLoginTarget(accountSettingsTarget);

  const resolvedSidebarWidth = clampSidebarWidth(sidebarWidth);

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

      const nextWidth = isSidebarCollapsed ? MIN_SIDEBAR_WIDTH : resolvedSidebarWidth;
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

  const navGroups = useMemo<SidebarNavGroup[]>(() => {
    const routeGroups = new Map<string, SidebarNavItem[]>();

    for (const route of adminRoutes.filter(
      (item) => item.key !== 'settings' && !hiddenSidebarItems.includes(item.key),
    )) {
      const group = route.group ?? 'Workspace';
      if (!routeGroups.has(group)) {
        routeGroups.set(group, []);
      }

      routeGroups.get(group)?.push({
        key: route.key,
        to: adminRoutePathByKey[route.key],
        icon: iconByRoute[route.key],
        label: t(route.label),
      });
    }

    return [...routeGroups.entries()].map(([section, items]) => ({
      section: t(section),
      items,
    }));
  }, [hiddenSidebarItems, t]);

  const currentSidebarWidth = isSidebarCollapsed ? COLLAPSED_SIDEBAR_WIDTH : resolvedSidebarWidth;
  const showEdgeAffordances = isSidebarHovered || isSidebarResizing;
  const isAuthenticated = Boolean(sessionUser);
  const profileName = sessionUser?.display_name ?? t('Router Admin');
  const profileDetail = sessionUser?.email ?? t('Control plane operator');
  const profileInitials = buildAvatarInitials(sessionUser?.display_name ?? sessionUser?.email);
  const userMenuTitle = isAuthenticated
    ? isUserMenuOpen
      ? t('Close user menu')
      : t('Open user menu')
    : t('Sign in');

  const handleUserControlClick = () => {
    if (!isAuthenticated) {
      navigate(loginTarget);
      return;
    }

    setIsUserMenuOpen((open) => !open);
  };

  const handleOpenAccountSettings = () => {
    setIsUserMenuOpen(false);
    prefetchSidebarRoute(accountSettingsTarget);
    navigate(accountSettingsTarget);
  };

  const handleSignOut = async () => {
    setIsUserMenuOpen(false);

    if (!isAuthenticated) {
      navigate(ADMIN_ROUTE_PATHS.LOGIN, { replace: true });
      return;
    }

    await handleLogout();
    navigate(ADMIN_ROUTE_PATHS.LOGIN, { replace: true });
  };

  return (
    <div
      className={`admin-shell-sidebar relative z-20 flex h-full shrink-0 ${
        isSidebarResizing ? '' : 'transition-[width] duration-200 ease-out'
      }`}
      style={{ width: currentSidebarWidth }}
      onMouseEnter={() => setIsSidebarHovered(true)}
      onMouseLeave={() => setIsSidebarHovered(false)}
    >
      <div className="flex h-full w-full flex-col overflow-hidden border-r border-[var(--admin-contrast-border)] [background:var(--admin-sidebar-background)] text-[var(--admin-sidebar-text)] shadow-[var(--admin-shadow-strong)]">
        <nav
          className={`admin-shell-nav-scroll scrollbar-hide flex-1 space-y-5 overflow-x-hidden overflow-y-auto ${
            isSidebarCollapsed ? 'px-2 py-4' : 'px-3 py-5'
          }`}
        >
          {navGroups.map((group) => (
            <div key={group.section}>
              {!isSidebarCollapsed ? (
                <div className="mb-3 px-3 text-[10px] font-semibold uppercase tracking-[0.22em] text-[var(--admin-sidebar-text-subtle)]">
                  {group.section}
                </div>
              ) : (
                <div className="mx-2 my-4 h-px bg-[var(--admin-sidebar-divider)]" />
              )}

              <div className="space-y-1">
                {group.items.map((item) => (
                  <NavLink
                    key={item.key}
                    title={isSidebarCollapsed ? item.label : undefined}
                    to={item.to}
                    onPointerDown={() => prefetchSidebarRoute(item.to)}
                    onMouseEnter={() => scheduleSidebarRoutePrefetch(item.to)}
                    onMouseLeave={() => cancelSidebarRoutePrefetch(item.to)}
                    onFocus={() => scheduleSidebarRoutePrefetch(item.to)}
                    onBlur={() => cancelSidebarRoutePrefetch(item.to)}
                    className={({ isActive }) =>
                      `group relative flex items-center rounded-2xl transition-all duration-200 ${
                        isSidebarCollapsed
                          ? 'mx-auto h-11 w-11 justify-center'
                          : 'justify-between px-3 py-2.5'
                      } ${
                        isActive
                          ? 'bg-[var(--admin-sidebar-item-active)] font-medium text-[var(--admin-text-on-contrast)] [box-shadow:var(--admin-sidebar-item-shadow)]'
                          : 'text-[var(--admin-sidebar-text-muted)] hover:bg-[var(--admin-sidebar-item-hover)] hover:text-[var(--admin-text-on-contrast)]'
                      }`
                    }
                  >
                    {({ isActive }) => (
                      <>
                        {isActive && !isSidebarCollapsed ? (
                          <motion.div
                            className="absolute left-0 top-1/2 h-5 w-1 -translate-y-1/2 rounded-r-full bg-primary-500"
                            layoutId="sidebar-active-indicator"
                          />
                        ) : null}
                        <div className="flex items-center gap-3">
                          <item.icon
                            className={`h-4 w-4 shrink-0 transition-colors ${
                              isActive
                                ? 'text-primary-400'
                                : 'text-[var(--admin-sidebar-icon)] group-hover:text-[var(--admin-sidebar-icon-hover)]'
                            }`}
                          />
                          {!isSidebarCollapsed ? (
                            <span className="text-[14px] tracking-tight">{item.label}</span>
                          ) : null}
                        </div>
                      </>
                    )}
                  </NavLink>
                ))}
              </div>
            </div>
          ))}
        </nav>

        <div className="flex flex-col gap-1 border-t border-[var(--admin-sidebar-divider)] p-3">
          <NavLink
            className={({ isActive }) =>
              `group relative flex items-center rounded-2xl transition-all duration-200 ${
                isSidebarCollapsed ? 'mx-auto h-11 w-11 justify-center' : 'gap-3 px-3 py-2.5'
              } ${
                isActive
                  ? 'bg-[var(--admin-sidebar-item-active)] font-medium text-[var(--admin-text-on-contrast)]'
                  : 'text-[var(--admin-sidebar-text-muted)] hover:bg-[var(--admin-sidebar-item-hover)] hover:text-[var(--admin-text-on-contrast)]'
              }`
            }
            data-slot="sidebar-footer-settings"
            title={isSidebarCollapsed ? t('Settings') : undefined}
            to={adminRoutePathByKey.settings}
            onPointerDown={() => prefetchSidebarRoute(adminRoutePathByKey.settings)}
            onMouseEnter={() => scheduleSidebarRoutePrefetch(adminRoutePathByKey.settings)}
            onMouseLeave={() => cancelSidebarRoutePrefetch(adminRoutePathByKey.settings)}
            onFocus={() => scheduleSidebarRoutePrefetch(adminRoutePathByKey.settings)}
            onBlur={() => cancelSidebarRoutePrefetch(adminRoutePathByKey.settings)}
          >
            {({ isActive }) => (
              <>
                {isActive && !isSidebarCollapsed ? (
                  <motion.div
                    className="absolute left-0 top-1/2 h-5 w-1 -translate-y-1/2 rounded-r-full bg-primary-500"
                    layoutId="sidebar-active-indicator"
                  />
                ) : null}
                <Settings2
                  className={`h-4 w-4 shrink-0 transition-colors ${
                    isActive
                      ? 'text-primary-400'
                      : 'text-[var(--admin-sidebar-icon)] group-hover:text-[var(--admin-sidebar-icon-hover)]'
                  }`}
                />
                {!isSidebarCollapsed ? (
                  <span className="text-[14px] tracking-tight">{t('Settings')}</span>
                ) : null}
              </>
            )}
          </NavLink>

          <div className="relative" ref={userMenuRef}>
            {isUserMenuOpen ? (
              <div
                className={`absolute z-40 rounded-3xl border border-[var(--admin-sidebar-popover-border)] bg-[var(--admin-sidebar-popover-background)] p-2 [box-shadow:var(--admin-sidebar-popover-shadow)] backdrop-blur-xl ${
                  isSidebarCollapsed ? 'bottom-0 left-full ml-3 w-64' : 'bottom-full left-0 right-0 mb-2'
                }`}
              >
                <div className="mb-2 rounded-2xl border border-[var(--admin-sidebar-item-border)] bg-[var(--admin-sidebar-item-surface)] p-3">
                  <div className="flex items-center gap-3">
                    <div className="flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-2xl bg-primary-500/15 text-sm font-bold text-primary-200">
                      {profileInitials}
                    </div>
                    <div className="min-w-0">
                      <div className="truncate text-sm font-semibold text-[var(--admin-text-on-contrast)]">
                        {profileName}
                      </div>
                      <div className="truncate text-xs text-[var(--admin-sidebar-text-muted)]">
                        {profileDetail}
                      </div>
                    </div>
                  </div>
                  <div className="mt-3 inline-flex items-center rounded-full border border-[var(--admin-success-border)] bg-[var(--admin-success-background)] px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--admin-success-text)]">
                    {t('Signed in')}
                  </div>
                </div>

                <button
                  className="flex w-full items-center gap-3 rounded-2xl px-3 py-2.5 text-left text-sm text-[var(--admin-sidebar-text)] transition-colors hover:bg-[var(--admin-sidebar-item-hover)] hover:text-[var(--admin-text-on-contrast)]"
                  onPointerDown={() => prefetchSidebarRoute(accountSettingsTarget)}
                  onMouseEnter={() => scheduleSidebarRoutePrefetch(accountSettingsTarget)}
                  onMouseLeave={() => cancelSidebarRoutePrefetch(accountSettingsTarget)}
                  onFocus={() => scheduleSidebarRoutePrefetch(accountSettingsTarget)}
                  onBlur={() => cancelSidebarRoutePrefetch(accountSettingsTarget)}
                  onClick={handleOpenAccountSettings}
                  type="button"
                >
                  <Settings2 className="h-4 w-4 text-[var(--admin-sidebar-icon)]" />
                  <span>{t('Profile settings')}</span>
                </button>

                <button
                  className="mt-1 flex w-full items-center gap-3 rounded-2xl px-3 py-2.5 text-left text-sm text-[var(--admin-danger-text)] transition-colors hover:bg-[var(--admin-danger-background)] hover:text-[var(--admin-danger-text-hover)]"
                  onClick={() => {
                    void handleSignOut();
                  }}
                  type="button"
                >
                  <LogOut className="h-4 w-4" />
                  <span>{t('Sign out')}</span>
                </button>
              </div>
            ) : null}

            <button
              className={`group relative flex w-full items-center rounded-2xl border border-[var(--admin-sidebar-item-border)] bg-[var(--admin-sidebar-item-surface)] text-[var(--admin-sidebar-text)] transition-all duration-200 hover:bg-[var(--admin-sidebar-item-hover)] hover:text-[var(--admin-text-on-contrast)] ${
                isSidebarCollapsed
                  ? 'mx-auto h-11 w-11 justify-center px-0'
                  : 'gap-3 px-2.5 py-2.5'
              }`}
              data-slot="sidebar-user-control"
              onClick={handleUserControlClick}
              title={isSidebarCollapsed ? userMenuTitle : undefined}
              type="button"
            >
              <div className="flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-2xl bg-[var(--admin-sidebar-item-surface-strong)] text-sm font-semibold text-[var(--admin-text-on-contrast)]">
                {isAuthenticated ? (
                  profileInitials
                ) : (
                  <CircleUserRound className="h-4 w-4 text-[var(--admin-sidebar-text)]" />
                )}
              </div>

              {!isSidebarCollapsed ? (
                <>
                  <div className="min-w-0 flex-1 text-left">
                    <div className="truncate text-sm font-semibold text-[var(--admin-text-on-contrast)]">
                      {isAuthenticated ? profileName : t('Guest')}
                    </div>
                    <div className="truncate text-xs text-[var(--admin-sidebar-text-subtle)]">
                      {isAuthenticated ? profileDetail : t('Sign in to manage the control plane')}
                    </div>
                  </div>

                  {isAuthenticated ? (
                    <ChevronUp
                      className={`h-4 w-4 shrink-0 text-[var(--admin-sidebar-icon)] transition-transform ${
                        isUserMenuOpen ? '' : 'rotate-180'
                      }`}
                    />
                  ) : (
                    <LogIn className="h-4 w-4 shrink-0 text-[var(--admin-sidebar-icon)] transition-colors group-hover:text-[var(--admin-sidebar-icon-hover)]" />
                  )}
                </>
              ) : null}
            </button>
          </div>
        </div>
      </div>

      <button
        className={`absolute right-0 top-1/2 z-30 flex h-8 w-8 -translate-y-1/2 translate-x-1/2 items-center justify-center rounded-full border border-[var(--admin-sidebar-item-border)] bg-[var(--admin-sidebar-edge-background)] text-[var(--admin-sidebar-edge-text)] [box-shadow:var(--admin-sidebar-edge-shadow)] transition-all duration-200 ${
          showEdgeAffordances
            ? 'opacity-100 hover:scale-105 hover:bg-[var(--admin-sidebar-edge-hover)]'
            : 'pointer-events-none opacity-0'
        }`}
        data-slot="sidebar-edge-control"
        onClick={toggleSidebar}
        title={isSidebarCollapsed ? t('Expand sidebar') : t('Collapse sidebar')}
        type="button"
      >
        {isSidebarCollapsed ? (
          <PanelLeftOpen className="h-4 w-4" />
        ) : (
          <PanelLeftClose className="h-4 w-4" />
        )}
      </button>

      <div
        className="admin-shell-sidebar-resize-handle absolute inset-y-0 right-0 z-20 w-3 cursor-col-resize touch-none"
        data-slot="sidebar-resize-handle"
        onPointerDown={startSidebarResize}
      />
    </div>
  );
}
