import { NavLink } from 'react-router-dom';
import { usePortalI18n } from 'sdkwork-router-portal-commons';

type TopNavItem = {
  key: string;
  href: string;
  labelKey: string;
};

const topNavItems: TopNavItem[] = [
  { key: 'home', href: '/', labelKey: 'Home' },
  { key: 'console', href: '/console/dashboard', labelKey: 'Console' },
  { key: 'models', href: '/models', labelKey: 'Models' },
  { key: 'docs', href: '/docs', labelKey: 'Docs' },
  { key: 'downloads', href: '/downloads', labelKey: 'Download Center' },
];

export function PortalTopNavigation() {
  const { t } = usePortalI18n();

  return (
    <nav
      aria-label={t('Top navigation')}
      className="flex min-w-0 w-full flex-1 items-center justify-start"
      data-slot="portal-top-navigation"
    >
      <div className="inline-flex w-full min-w-0 items-center justify-start gap-1.5 max-w-[min(100%,72rem)] overflow-hidden px-1">
        {topNavItems.map((item) => (
          <NavLink
            key={item.key}
            end={item.href === '/'}
            to={item.href}
            className={({ isActive }) =>
              `inline-flex h-9 flex-none items-center justify-center whitespace-nowrap rounded-xl px-4 text-center text-[13px] font-medium leading-none transition-colors md:px-5 ${
                isActive
                  ? 'bg-zinc-950/[0.08] text-zinc-950 shadow-[inset_0_0_0_1px_rgba(24,24,27,0.08)] dark:bg-white/[0.08] dark:text-white dark:shadow-[inset_0_0_0_1px_rgba(255,255,255,0.12)]'
                  : 'text-zinc-500 hover:bg-zinc-950/[0.04] hover:text-zinc-950 dark:text-zinc-400 dark:hover:bg-white/[0.06] dark:hover:text-white'
              }`
            }
          >
            <span className="truncate">{t(item.labelKey)}</span>
          </NavLink>
        ))}
      </div>
    </nav>
  );
}
