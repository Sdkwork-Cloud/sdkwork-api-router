import { Check, type LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

import { cn } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import { Checkbox } from 'sdkwork-router-portal-commons/framework/entry';
import type { PortalThemeColor } from 'sdkwork-router-portal-types';

export function PortalSettingsNavButton({
  active,
  icon: Icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: LucideIcon;
  label: string;
  onClick: () => void;
}) {
  return (
    <Button
      onClick={onClick}
      className={cn(
        'h-auto w-full justify-start gap-3 rounded-xl border px-3 py-2.5 text-[14px] font-medium shadow-none transition-all duration-200',
        active
          ? 'border-zinc-200/50 bg-white text-primary-600 shadow-sm dark:border-zinc-700/50 dark:bg-zinc-800 dark:text-primary-400'
          : 'border-transparent text-zinc-600 hover:bg-zinc-200/50 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800/50 dark:hover:text-zinc-100',
      )}
      type="button"
      variant="ghost"
    >
      <Icon
        className={cn(
          'h-4 w-4',
          active ? 'text-primary-500 dark:text-primary-400' : 'text-zinc-400 dark:text-zinc-500',
        )}
      />
      {label}
    </Button>
  );
}

export function PortalSettingsPanelCard({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <div className="overflow-hidden rounded-[1.5rem] border border-zinc-200/80 bg-white shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
      <div className="border-b border-zinc-100 bg-zinc-50/50 px-6 py-5 dark:border-zinc-800/80 dark:bg-zinc-900/50">
        <div className="space-y-1">
          <h3 className="text-[15px] font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
            {title}
          </h3>
          {description ? (
            <p className="text-sm text-zinc-500 dark:text-zinc-400">{description}</p>
          ) : null}
        </div>
      </div>
      <div className="p-6">{children}</div>
    </div>
  );
}

export function PortalThemeModeChoiceCard({
  active,
  icon: Icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: LucideIcon;
  label: string;
  onClick: () => void;
}) {
  return (
    <Button
      type="button"
      onClick={onClick}
      className={cn(
        'h-auto w-full flex-col items-center justify-center gap-3 whitespace-normal rounded-xl border-2 p-4 text-center shadow-none transition-all',
        active
          ? 'border-primary-500 bg-primary-50/50 dark:bg-primary-500/10'
          : 'border-zinc-200 bg-white hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-zinc-700',
      )}
      variant="ghost"
    >
      <Icon
        className={cn(
          'h-6 w-6',
          active ? 'text-primary-500 dark:text-primary-400' : 'text-zinc-500 dark:text-zinc-400',
        )}
      />
      <span
        className={cn(
          'text-sm font-medium',
          active
            ? 'text-primary-700 dark:text-primary-300'
            : 'text-zinc-700 dark:text-zinc-300',
        )}
      >
        {label}
      </span>
    </Button>
  );
}

export function PortalThemeColorSwatch({
  active,
  color,
  label,
  onClick,
  previewClassName,
}: {
  active: boolean;
  color: PortalThemeColor;
  label: string;
  onClick: () => void;
  previewClassName: string;
}) {
  return (
    <Button
      type="button"
      onClick={onClick}
      className="group relative h-auto flex-col items-center gap-2 whitespace-normal rounded-none p-0 shadow-none hover:bg-transparent"
      variant="ghost"
    >
      <div
        className={cn(
          'flex h-10 w-10 items-center justify-center rounded-full shadow-sm ring-2 ring-offset-2 transition-all dark:ring-offset-zinc-950',
          previewClassName,
          active
            ? 'scale-110 ring-zinc-900 dark:ring-zinc-100'
            : 'ring-transparent hover:scale-105',
        )}
      >
        {active ? <Check className="h-5 w-5 text-white" /> : null}
      </div>
      <span
        className={cn(
          'text-xs font-medium',
          active
            ? 'text-zinc-900 dark:text-zinc-100'
            : 'text-zinc-500 group-hover:text-zinc-700 dark:text-zinc-400 dark:group-hover:text-zinc-300',
        )}
      >
        {label}
      </span>
      <span className="sr-only">{color}</span>
    </Button>
  );
}

export function PortalSettingsActionButton({
  emphasis = 'secondary',
  icon: Icon,
  label,
  onClick,
}: {
  emphasis?: 'primary' | 'secondary' | 'subtle';
  icon?: LucideIcon;
  label: ReactNode;
  onClick: () => void;
}) {
  const className =
    emphasis === 'primary'
      ? 'inline-flex h-10 items-center justify-center gap-2 rounded-2xl bg-zinc-950 px-4 text-sm font-semibold text-white transition hover:bg-zinc-900 dark:bg-zinc-100 dark:text-zinc-950 dark:hover:bg-zinc-200'
      : emphasis === 'subtle'
        ? 'inline-flex h-10 items-center justify-center gap-2 rounded-2xl border border-zinc-200 bg-zinc-50 px-4 text-sm font-semibold text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:hover:text-zinc-50'
        : 'inline-flex h-10 items-center justify-center gap-2 rounded-2xl border border-zinc-200 bg-white px-4 text-sm font-semibold text-zinc-600 transition hover:bg-zinc-100 hover:text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300 dark:hover:bg-zinc-900 dark:hover:text-zinc-50';

  return (
    <Button
      type="button"
      onClick={onClick}
      className={className}
      variant={emphasis === 'primary' ? 'primary' : 'secondary'}
    >
      {Icon ? <Icon className="h-4 w-4" /> : null}
      {label}
    </Button>
  );
}

export function PortalSettingsToggleRow({
  checked,
  description,
  label,
  onCheckedChange,
}: {
  checked: boolean;
  description: ReactNode;
  label: ReactNode;
  onCheckedChange: () => void;
}) {
  return (
    <label className="flex cursor-pointer items-center gap-3 rounded-xl border border-zinc-200 p-3 transition-colors hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-800/50">
      <Checkbox checked={checked} onCheckedChange={onCheckedChange} />
      <span className="grid gap-0.5">
        <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">{label}</span>
        <span className="text-xs text-zinc-500 dark:text-zinc-400">{description}</span>
      </span>
    </label>
  );
}

export function PortalSettingsIdentityCard({
  eyebrow,
  title,
  description,
}: {
  eyebrow: ReactNode;
  title: ReactNode;
  description: ReactNode;
}) {
  return (
    <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
      <div className="text-xs uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
        {eyebrow}
      </div>
      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">{title}</div>
      <div className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{description}</div>
    </div>
  );
}
