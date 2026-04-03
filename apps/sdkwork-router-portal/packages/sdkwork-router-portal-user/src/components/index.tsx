import type { ReactNode } from 'react';
import { formatDateTime, usePortalI18n } from 'sdkwork-router-portal-commons';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

const summaryCardClassName =
  'rounded-[20px] border border-zinc-200 bg-zinc-50/85 p-4 dark:border-zinc-800 dark:bg-zinc-900/70';
const detailCardClassName =
  'rounded-[20px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70';
const sectionCardClassName =
  'overflow-hidden rounded-[24px] border border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950';
const detailCardTitleClassName = 'text-sm font-semibold text-zinc-950 dark:text-zinc-50';
const detailCardCopyClassName = 'text-sm leading-6 text-zinc-600 dark:text-zinc-300';

export function UserSummaryCard({
  detail,
  title,
  value,
}: {
  detail: string;
  title: string;
  value: string;
}) {
  return (
    <article className={summaryCardClassName}>
      <span className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
        {title}
      </span>
      <strong className="mt-3 block text-xl text-zinc-950 dark:text-zinc-50">{value}</strong>
      <p className={`mt-2 ${detailCardCopyClassName}`}>{detail}</p>
    </article>
  );
}

export function UserDetailCard({
  badge,
  children,
  description,
  title,
}: {
  badge?: ReactNode;
  children?: ReactNode;
  description?: string;
  title: string;
}) {
  return (
    <article className={detailCardClassName}>
      <div className={badge ? 'flex items-center justify-between gap-3' : undefined}>
        <strong className={detailCardTitleClassName}>{title}</strong>
        {badge}
      </div>
      {description ? <p className={`mt-2 ${detailCardCopyClassName}`}>{description}</p> : null}
      {children ? <div className={description ? 'mt-4' : 'mt-3'}>{children}</div> : null}
    </article>
  );
}

export function UserSectionCard({
  actions,
  children,
  description,
  title,
}: {
  actions?: ReactNode;
  children: ReactNode;
  description?: string;
  title: string;
}) {
  return (
    <section className={sectionCardClassName}>
      <div className="flex flex-wrap items-start justify-between gap-4 border-b border-zinc-200 bg-zinc-50/80 px-5 py-4 dark:border-zinc-800 dark:bg-zinc-900/70">
        <div className="min-w-0 space-y-1">
          <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{title}</h2>
          {description ? (
            <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">{description}</p>
          ) : null}
        </div>
        {actions ? <div className="flex shrink-0 flex-wrap gap-2">{actions}</div> : null}
      </div>
      <div className="p-5">{children}</div>
    </section>
  );
}

export function UserProfileFacts({
  workspace,
}: {
  workspace: PortalWorkspaceSummary | null;
}) {
  const { t } = usePortalI18n();

  if (!workspace) {
    return null;
  }

  return (
    <dl className="grid gap-3">
      {[
        { label: t('Name'), value: workspace.user.display_name ?? t('Unavailable') },
        { label: t('Email'), value: workspace.user.email },
        { label: t('Workspace'), value: `${workspace.tenant.name} / ${workspace.project.name}` },
        { label: t('Created'), value: formatDateTime(workspace.user.created_at_ms) },
      ].map((item) => (
        <div
          className="rounded-[18px] border border-zinc-200 bg-zinc-50/80 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/70"
          key={item.label}
        >
          <dt className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
            {item.label}
          </dt>
          <dd className="mt-2 text-sm font-medium text-zinc-950 dark:text-zinc-50">
            {item.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}
