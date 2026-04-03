import type { LucideIcon } from 'lucide-react';

import { Button } from 'sdkwork-router-portal-commons/framework/actions';

export function ApiKeyModeChoiceCard({
  detail,
  icon: Icon,
  onClick,
  selected,
  title,
}: {
  detail: string;
  icon?: LucideIcon;
  onClick: () => void;
  selected: boolean;
  title: string;
}) {
  return (
    <Button
      type="button"
      onClick={onClick}
      className={
        selected
          ? 'h-auto w-full justify-start whitespace-normal rounded-[24px] border border-primary-500/35 bg-primary-500/8 p-4 text-left shadow-[0_12px_30px_rgba(59,130,246,0.10)] hover:bg-primary-500/10'
          : 'h-auto w-full justify-start whitespace-normal rounded-[24px] border border-zinc-200 bg-white p-4 text-left shadow-none hover:border-zinc-300 hover:bg-white dark:border-zinc-800 dark:bg-zinc-950 dark:hover:border-zinc-700 dark:hover:bg-zinc-950'
      }
      variant="ghost"
    >
      <div className="flex items-start gap-3">
        {Icon ? (
          <span
            className={
              selected
                ? 'inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-primary-500 text-white'
                : 'inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-zinc-100 text-zinc-600 dark:bg-zinc-900 dark:text-zinc-300'
            }
          >
            <Icon className="h-4 w-4" />
          </span>
        ) : null}
        <div>
          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">{title}</div>
          <p className="mt-1 text-xs leading-6 text-zinc-500 dark:text-zinc-400">{detail}</p>
        </div>
      </div>
    </Button>
  );
}
