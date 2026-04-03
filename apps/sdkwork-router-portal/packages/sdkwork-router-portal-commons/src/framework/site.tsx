import type { ReactNode } from 'react';

import { StatCard } from './display';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from './layout';

function joinClassNames(...values: Array<string | undefined>) {
  return values.filter(Boolean).join(' ');
}

export function PortalSiteHero({
  actions,
  aside,
  className,
  description,
  eyebrow,
  title,
}: {
  actions?: ReactNode;
  aside?: ReactNode;
  className?: string;
  description: ReactNode;
  eyebrow?: ReactNode;
  title: ReactNode;
}) {
  return (
    <section
      className={joinClassNames(
        'grid gap-6 rounded-[32px] border border-zinc-200 bg-white p-8 shadow-sm dark:border-zinc-800 dark:bg-zinc-950',
        aside ? 'lg:grid-cols-[minmax(0,1.25fr)_minmax(18rem,0.75fr)]' : undefined,
        className,
      )}
      data-slot="portal-site-hero"
    >
      <div className="space-y-5">
        {eyebrow ? (
          <p className="text-xs font-semibold uppercase tracking-[0.22em] text-primary-600">
            {eyebrow}
          </p>
        ) : null}
        <div className="space-y-3">
          <h1 className="text-4xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
            {title}
          </h1>
          <p className="max-w-3xl text-sm leading-7 text-zinc-600 dark:text-zinc-300">
            {description}
          </p>
        </div>
        {actions ? (
          <div className="flex flex-wrap gap-3">
            {actions}
          </div>
        ) : null}
      </div>

      {aside ? (
        <div className="space-y-4">
          {aside}
        </div>
      ) : null}
    </section>
  );
}

export function PortalSitePanel({
  children,
  className,
  description,
  title,
}: {
  children: ReactNode;
  className?: string;
  description?: ReactNode;
  title?: ReactNode;
}) {
  return (
    <Card
      className={joinClassNames(
        'rounded-[32px] border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950',
        className,
      )}
      data-slot="portal-site-panel"
    >
      {title || description ? (
        <CardHeader>
          {title ? <CardTitle>{title}</CardTitle> : null}
          {description ? <CardDescription>{description}</CardDescription> : null}
        </CardHeader>
      ) : null}
      <CardContent className="space-y-4">{children}</CardContent>
    </Card>
  );
}

export function PortalSiteMetricCard({
  description,
  label,
  value,
}: {
  description: ReactNode;
  label: ReactNode;
  value: ReactNode;
}) {
  return (
    <StatCard
      description={description}
      label={label}
      value={value}
    />
  );
}
