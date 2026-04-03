import { clsx, type ClassValue } from 'clsx';
import { Search as SearchIcon } from 'lucide-react';
import {
  forwardRef,
  type ComponentPropsWithoutRef,
  type ElementRef,
  type ReactNode,
} from 'react';
import { twMerge } from 'tailwind-merge';

import {
  FilterBar as BaseFilterBar,
  FilterBarActions as BaseFilterBarActions,
  FilterBarSection,
  SettingsField,
} from '@sdkwork/ui-pc-react/components/ui/form';

import { Input } from './entry';

export { FilterBarSection, SettingsField };

function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

export type FilterBarProps = ComponentPropsWithoutRef<typeof BaseFilterBar> & {
  wrap?: boolean;
};

export const FilterBar = forwardRef<ElementRef<typeof BaseFilterBar>, FilterBarProps>(
  ({ className, wrap = true, ...props }, ref) => (
    <BaseFilterBar
      ref={ref}
      className={cn(
        wrap
          ? undefined
          : 'overflow-x-auto [&>div:last-child]:min-w-max [&>div:last-child]:flex-nowrap',
        className,
      )}
      {...props}
    />
  ),
);

FilterBar.displayName = 'FilterBar';

export type FilterBarActionsProps = ComponentPropsWithoutRef<typeof BaseFilterBarActions> & {
  wrap?: boolean;
};

export const FilterBarActions = forwardRef<
  ElementRef<typeof BaseFilterBarActions>,
  FilterBarActionsProps
>(({ className, wrap = true, ...props }, ref) => (
  <BaseFilterBarActions
    ref={ref}
    className={cn(wrap ? undefined : 'w-auto shrink-0 flex-nowrap', className)}
    {...props}
  />
));

FilterBarActions.displayName = 'FilterBarActions';

export function FilterField({
  children,
  className,
  controlClassName,
  label,
}: {
  children: ReactNode;
  className?: string;
  controlClassName?: string;
  label: string;
}) {
  return (
    <label className={cn('flex min-w-0 max-w-full items-center gap-3', className)}>
      <span className="shrink-0 whitespace-nowrap text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
        {label}
      </span>
      <span className={cn('min-w-0 flex-1', controlClassName)}>{children}</span>
    </label>
  );
}

export function SearchInput({
  className,
  iconClassName,
  inputClassName,
  style,
  type,
  ...props
}: Omit<ComponentPropsWithoutRef<'input'>, 'className'> & {
  className?: string;
  iconClassName?: string;
  inputClassName?: string;
}) {
  return (
    <span className={cn('relative block w-full', className)}>
      <span
        className={cn(
          'pointer-events-none absolute left-4 top-1/2 flex h-5 w-5 -translate-y-1/2 items-center justify-center text-zinc-400 dark:text-zinc-500',
          iconClassName,
        )}
      >
        <SearchIcon className="h-4 w-4" />
      </span>
      <Input
        {...props}
        className={inputClassName}
        style={{ ...style, paddingLeft: '2.75rem' }}
        type={type ?? 'text'}
      />
    </span>
  );
}
