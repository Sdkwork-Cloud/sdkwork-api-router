import type { ReactNode } from 'react';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import { DataTable } from 'sdkwork-router-portal-commons/framework/display';
import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

function formatDate(value: number | null | undefined, locale: string, emptyLabel: string): string {
  if (value === null || value === undefined) {
    return emptyLabel;
  }

  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }).format(new Date(value));
}

function formatLastUsed(value: number | null | undefined, locale: string, emptyLabel: string): string {
  if (value === null || value === undefined) {
    return emptyLabel;
  }

  return new Intl.DateTimeFormat(locale, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(value));
}

function maskValue(value: string): string {
  if (value.length <= 14) {
    return value;
  }

  return `${value.slice(0, 10)}********${value.slice(-4)}`;
}

function describeUsage(item: GatewayApiKeyRecord, t: (text: string) => string): string {
  if (!item.active) {
    return t('Revoked from gateway traffic');
  }

  if (item.last_used_at_ms) {
    return t('Gateway traffic observed');
  }

  return t('Ready for first authenticated request');
}

type PortalApiKeyTableConfigOptions = {
  items: GatewayApiKeyRecord[];
  latestCreatedKey: CreatedGatewayApiKey | null;
  locale: string;
  mutatingKey: string | null;
  onCopyLatestPlaintext: () => void;
  onCopyPlaintext: (item: GatewayApiKeyRecord) => void;
  onDelete: (item: GatewayApiKeyRecord) => void;
  onOpenDetails: (item: GatewayApiKeyRecord) => void;
  onToggleStatus: (item: GatewayApiKeyRecord) => void;
  resolveGroupLabel: (groupId: string | null | undefined) => string;
  resolvePlaintext: (item: GatewayApiKeyRecord) => string | null;
  t: (text: string) => string;
};

type PortalApiKeyTableColumn = {
  align?: 'left' | 'center' | 'right';
  cell: (item: GatewayApiKeyRecord, index: number) => ReactNode;
  className?: string;
  header: ReactNode;
  headerClassName?: string;
  id: string;
  width?: number | string;
};

type PortalApiKeyTableColumnSource = {
  align?: 'left' | 'center' | 'right';
  className?: string;
  headerClassName?: string;
  key: string;
  label: ReactNode;
  render: (item: GatewayApiKeyRecord, index: number) => ReactNode;
  width?: number | string;
};

export type PortalApiKeyTableConfig = {
  columns: PortalApiKeyTableColumn[];
  emptyState: ReactNode;
  getRowId: (item: GatewayApiKeyRecord, index: number) => string;
  rows: GatewayApiKeyRecord[];
};

export function buildPortalApiKeyTableConfig({
  items,
  latestCreatedKey,
  locale,
  mutatingKey,
  onCopyLatestPlaintext,
  onCopyPlaintext,
  onDelete,
  onOpenDetails,
  onToggleStatus,
  resolveGroupLabel,
  resolvePlaintext,
  t,
}: PortalApiKeyTableConfigOptions): PortalApiKeyTableConfig {
  const sourceColumns: PortalApiKeyTableColumnSource[] = [
    {
      key: 'name',
      label: t('Name'),
      render: (item: GatewayApiKeyRecord) => (
        <div className="min-w-[16rem]">
          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
            {item.label}
          </div>
          {item.notes ? (
            <div className="mt-2 max-w-[24rem] text-xs leading-6 text-zinc-500 dark:text-zinc-400">
              {item.notes}
            </div>
          ) : null}
        </div>
      ),
    },
    {
      key: 'key',
      label: t('API key'),
      render: (item: GatewayApiKeyRecord) => {
        const isLatestCreatedKey = latestCreatedKey?.hashed === item.hashed_key;
        const plaintext = resolvePlaintext(item);
        const hasVisiblePlaintext = Boolean(plaintext);
        const displayValue = plaintext ?? item.hashed_key;

        return (
          <div className="flex min-w-[14rem] items-start gap-3">
            <div className="flex-1 rounded-2xl border border-zinc-200 bg-zinc-50 px-3 py-2 text-sm font-medium text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-200">
              {maskValue(displayValue)}
            </div>
            {hasVisiblePlaintext ? (
              <Button
                onClick={() =>
                  isLatestCreatedKey
                    ? onCopyLatestPlaintext()
                    : onCopyPlaintext(item)
                }
                type="button"
                variant="secondary"
              >
                {t('Copy key')}
              </Button>
            ) : (
              <span className="inline-flex h-9 items-center justify-center rounded-2xl border border-primary-500/15 bg-primary-500/10 px-3 text-xs font-semibold text-primary-600 dark:border-primary-500/20 dark:text-primary-300">
                {t('Write-only')}
              </span>
            )}
          </div>
        );
      },
    },
    {
      key: 'source',
      label: t('Source'),
      render: () => (
        <span className="inline-flex min-w-[8rem] items-center justify-center rounded-full border border-primary-500/15 bg-primary-500/10 px-3 py-1 text-xs font-semibold text-primary-600 dark:border-primary-500/20 dark:text-primary-300">
          {t('Portal managed')}
        </span>
      ),
    },
    {
      key: 'environment',
      label: t('Environment'),
      render: (item: GatewayApiKeyRecord) => (
        <span className="inline-flex min-w-[8rem] items-center justify-center rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-semibold text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300">
          {item.environment}
        </span>
      ),
    },
    {
      key: 'group',
      label: t('Key group'),
      render: (item: GatewayApiKeyRecord) => (
        <div className="min-w-[12rem]">
          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
            {resolveGroupLabel(item.api_key_group_id)}
          </div>
          <div className="mt-1 font-mono text-xs text-zinc-500 dark:text-zinc-400">
            {item.api_key_group_id ?? t('No group binding')}
          </div>
        </div>
      ),
    },
    {
      key: 'usage',
      label: t('Usage'),
      render: (item: GatewayApiKeyRecord) => (
        <div className="min-w-[11rem]">
          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
            {formatLastUsed(item.last_used_at_ms, locale, t('Not yet'))}
          </div>
          <div className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
            {t('Last authenticated use')}
          </div>
          <div className="mt-2 text-xs font-semibold text-primary-500">
            {describeUsage(item, t)}
          </div>
        </div>
      ),
    },
    {
      key: 'expires_at',
      label: t('Expires at'),
      render: (item: GatewayApiKeyRecord) => formatDate(item.expires_at_ms, locale, t('Never')),
    },
    {
      key: 'status',
      label: t('Status'),
      render: (item: GatewayApiKeyRecord) => (
        <span
          className={
            item.active
              ? 'inline-flex items-center rounded-full border border-emerald-400/20 bg-emerald-400/10 px-3 py-1 text-xs font-semibold text-emerald-700 dark:text-emerald-300'
              : 'inline-flex items-center rounded-full border border-amber-400/20 bg-amber-400/10 px-3 py-1 text-xs font-semibold text-amber-700 dark:text-amber-300'
          }
        >
          {item.active ? t('Active') : t('Inactive')}
        </span>
      ),
    },
    {
      key: 'created_at',
      label: t('Created at'),
      render: (item: GatewayApiKeyRecord) => formatDate(item.created_at_ms, locale, t('Never')),
    },
    {
      key: 'actions',
      label: t('Actions'),
      render: (item: GatewayApiKeyRecord) => (
        <div className="flex min-w-[17rem] flex-wrap gap-2">
          <Button
            onClick={() => onOpenDetails(item)}
            type="button"
            variant="secondary"
          >
            {t('View details')}
          </Button>
          <Button
            disabled={mutatingKey === item.hashed_key}
            onClick={() => onToggleStatus(item)}
            type="button"
            variant="secondary"
          >
            {item.active ? t('Disable') : t('Enable')}
          </Button>
          <Button
            disabled={mutatingKey === item.hashed_key}
            onClick={() => onDelete(item)}
            type="button"
            variant="danger"
          >
            {t('Delete')}
          </Button>
        </div>
      ),
    },
  ];

  const columns: PortalApiKeyTableColumn[] = sourceColumns.map((column) => ({
    align: column.align,
    cell: (item, index) => column.render(item, index),
    className: column.className,
    header: column.label,
    headerClassName: column.headerClassName,
    id: column.key,
    width: column.width,
  }));

  return {
    columns,
    emptyState: (
      <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
        <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
          {t('No API keys yet')}
        </strong>
        <p className="text-sm text-zinc-500 dark:text-zinc-400">
          {t('Create your first key to connect a client or service to the SDKWork Router gateway.')}
        </p>
      </div>
    ),
    getRowId: (item: GatewayApiKeyRecord) => item.hashed_key,
    rows: items,
  };
}

export function PortalApiKeyTable({
  className,
  footer,
  items,
  latestCreatedKey,
  mutatingKey,
  onCopyLatestPlaintext,
  onCopyPlaintext,
  onDelete,
  onOpenDetails,
  resolvePlaintext,
  resolveGroupLabel,
  onToggleStatus,
  ...props
}: {
  className?: string;
  footer?: ReactNode;
  items: GatewayApiKeyRecord[];
  latestCreatedKey: CreatedGatewayApiKey | null;
  mutatingKey: string | null;
  onCopyLatestPlaintext: () => void;
  onCopyPlaintext: (item: GatewayApiKeyRecord) => void;
  onDelete: (item: GatewayApiKeyRecord) => void;
  onOpenDetails: (item: GatewayApiKeyRecord) => void;
  resolveGroupLabel: (groupId: string | null | undefined) => string;
  resolvePlaintext: (item: GatewayApiKeyRecord) => string | null;
  onToggleStatus: (item: GatewayApiKeyRecord) => void;
} & React.HTMLAttributes<HTMLDivElement>) {
  const { locale, t } = usePortalI18n();
  const table = buildPortalApiKeyTableConfig({
    items,
    latestCreatedKey,
    locale,
    mutatingKey,
    onCopyLatestPlaintext,
    onCopyPlaintext,
    onDelete,
    onOpenDetails,
    onToggleStatus,
    resolveGroupLabel,
    resolvePlaintext,
    t,
  });

  return <DataTable {...table} className={className} footer={footer} {...props} />;
}
