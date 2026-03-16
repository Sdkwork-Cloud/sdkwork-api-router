import type { ReactNode } from 'react';

export function SectionHero({
  eyebrow,
  title,
  detail,
  actions,
}: {
  eyebrow: string;
  title: string;
  detail: string;
  actions?: ReactNode;
}) {
  return (
    <header className="portalx-hero">
      <div>
        <p className="portalx-eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="portalx-hero-detail">{detail}</p>
      </div>
      {actions ? <div className="portalx-hero-actions">{actions}</div> : null}
    </header>
  );
}

export function Surface({
  title,
  detail,
  actions,
  children,
}: {
  title: string;
  detail?: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="portalx-surface">
      <div className="portalx-surface-heading">
        <div>
          <h2>{title}</h2>
          {detail ? <p>{detail}</p> : null}
        </div>
        {actions ? <div className="portalx-surface-actions">{actions}</div> : null}
      </div>
      {children}
    </section>
  );
}

export function MetricCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <article className="portalx-metric-card">
      <span>{label}</span>
      <strong>{value}</strong>
      <p>{detail}</p>
    </article>
  );
}

export function Pill({
  tone,
  children,
}: {
  tone?: 'default' | 'accent' | 'positive' | 'warning' | 'seed';
  children: ReactNode;
}) {
  return <span className={`portalx-pill portalx-pill-${tone ?? 'default'}`}>{children}</span>;
}

export function InlineButton({
  children,
  onClick,
  tone,
  type,
  disabled,
}: {
  children: ReactNode;
  onClick?: () => void;
  tone?: 'primary' | 'secondary' | 'ghost';
  type?: 'button' | 'submit';
  disabled?: boolean;
}) {
  return (
    <button
      className={`portalx-button portalx-button-${tone ?? 'secondary'}`}
      disabled={disabled}
      onClick={onClick}
      type={type ?? 'button'}
    >
      {children}
    </button>
  );
}

export function DataTable<T>({
  columns,
  rows,
  empty,
  getKey,
}: {
  columns: Array<{ key: string; label: string; render: (row: T) => ReactNode }>;
  rows: T[];
  empty: string;
  getKey: (row: T, index: number) => string;
}) {
  return (
    <div className="portalx-table-wrap">
      <table className="portalx-table">
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key}>{column.label}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={getKey(row, index)}>
              {columns.map((column) => (
                <td key={column.key}>{column.render(row)}</td>
              ))}
            </tr>
          ))}
          {!rows.length ? (
            <tr>
              <td className="portalx-empty" colSpan={columns.length}>
                {empty}
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>
    </div>
  );
}

export function EmptyState({
  title,
  detail,
}: {
  title: string;
  detail: string;
}) {
  return (
    <div className="portalx-empty-state">
      <strong>{title}</strong>
      <p>{detail}</p>
    </div>
  );
}

export function formatCurrency(amount: number): string {
  return `$${amount.toFixed(2)}`;
}

export function formatUnits(units: number): string {
  return new Intl.NumberFormat('en-US').format(units);
}

export function formatDateTime(timestamp: number): string {
  if (!timestamp) {
    return 'Pending';
  }

  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(timestamp));
}

export async function copyText(value: string): Promise<boolean> {
  if (!value) {
    return false;
  }

  try {
    await globalThis.navigator?.clipboard?.writeText(value);
    return true;
  } catch {
    return false;
  }
}
