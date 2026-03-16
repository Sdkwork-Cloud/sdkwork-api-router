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
    <div className="adminx-hero">
      <div>
        <p className="adminx-eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="adminx-hero-detail">{detail}</p>
      </div>
      {actions ? <div className="adminx-hero-actions">{actions}</div> : null}
    </div>
  );
}

export function Surface({
  title,
  detail,
  children,
}: {
  title: string;
  detail?: string;
  children: ReactNode;
}) {
  return (
    <section className="adminx-surface">
      <div className="adminx-surface-heading">
        <div>
          <h2>{title}</h2>
          {detail ? <p>{detail}</p> : null}
        </div>
      </div>
      {children}
    </section>
  );
}

export function StatCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <article className="adminx-stat-card">
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
  tone?: 'default' | 'live' | 'seed' | 'danger';
  children: ReactNode;
}) {
  return <span className={`adminx-pill adminx-pill-${tone ?? 'default'}`}>{children}</span>;
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
    <div className="adminx-table-wrap">
      <table className="adminx-table">
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
              <td colSpan={columns.length} className="adminx-empty">
                {empty}
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>
    </div>
  );
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
  tone?: 'primary' | 'secondary';
  type?: 'button' | 'submit';
  disabled?: boolean;
}) {
  return (
    <button
      className={`adminx-button adminx-button-${tone ?? 'secondary'}`}
      disabled={disabled}
      onClick={onClick}
      type={type ?? 'button'}
    >
      {children}
    </button>
  );
}
