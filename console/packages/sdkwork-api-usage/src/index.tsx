import { useEffect, useState } from 'react';
import { listLedgerEntries, listQuotaPolicies, listUsageRecords } from 'sdkwork-api-admin-sdk';
import type { LedgerEntry, QuotaPolicyRecord, UsageRecord } from 'sdkwork-api-types';

interface UsageSnapshot {
  usageRecords: UsageRecord[];
  ledgerEntries: LedgerEntry[];
  quotaPolicies: QuotaPolicyRecord[];
}

const emptySnapshot: UsageSnapshot = {
  usageRecords: [],
  ledgerEntries: [],
  quotaPolicies: [],
};

function totalLedgerAmount(entries: LedgerEntry[]): string {
  const total = entries.reduce((sum, entry) => sum + entry.amount, 0);
  return total.toFixed(2);
}

function totalEnabledQuotaUnits(policies: QuotaPolicyRecord[]): string {
  const total = policies
    .filter((policy) => policy.enabled)
    .reduce((sum, policy) => sum + policy.max_units, 0);
  return total.toLocaleString();
}

export function RequestExplorerPage() {
  const [snapshot, setSnapshot] = useState<UsageSnapshot>(emptySnapshot);
  const [status, setStatus] = useState('Loading request telemetry...');

  useEffect(() => {
    let cancelled = false;

    void Promise.all([listUsageRecords(), listLedgerEntries(), listQuotaPolicies()])
      .then(([usageRecords, ledgerEntries, quotaPolicies]) => {
        if (cancelled) {
          return;
        }

        setSnapshot({ usageRecords, ledgerEntries, quotaPolicies });
        setStatus('Gateway telemetry and quota policies are streaming from admin usage and billing APIs.');
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('Admin API unavailable. Usage explorer is in offline mode.');
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Usage Ledger</p>
          <h2>Recent requests and booked cost</h2>
        </div>
        <p className="status">{status}</p>
      </div>

      <div className="metric-grid">
        <article className="metric-card">
          <span className="metric-label">Usage Events</span>
          <strong>{snapshot.usageRecords.length}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Ledger Entries</span>
          <strong>{snapshot.ledgerEntries.length}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Booked Amount</span>
          <strong>{totalLedgerAmount(snapshot.ledgerEntries)}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Enabled Quota Units</span>
          <strong>{totalEnabledQuotaUnits(snapshot.quotaPolicies)}</strong>
        </article>
      </div>

      <div className="detail-grid">
        <article className="detail-card">
          <h3>Usage Records</h3>
          <ul className="compact-list">
            {snapshot.usageRecords.map((record, index) => (
              <li key={`${record.project_id}:${record.model}:${index}`}>
                <strong>{record.model}</strong>
                <span>{record.provider}</span>
              </li>
            ))}
            {!snapshot.usageRecords.length && (
              <li className="empty">No gateway requests have been recorded yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Billing Ledger</h3>
          <ul className="compact-list">
            {snapshot.ledgerEntries.map((entry, index) => (
              <li key={`${entry.project_id}:${entry.units}:${index}`}>
                <strong>{entry.project_id}</strong>
                <span>
                  {entry.units} units / {entry.amount.toFixed(2)}
                </span>
              </li>
            ))}
            {!snapshot.ledgerEntries.length && (
              <li className="empty">No billing entries have been booked yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Quota Policies</h3>
          <ul className="compact-list">
            {snapshot.quotaPolicies.map((policy) => (
              <li key={policy.policy_id}>
                <strong>{policy.project_id}</strong>
                <span>
                  {policy.max_units.toLocaleString()} units / {policy.enabled ? 'enabled' : 'disabled'}
                </span>
              </li>
            ))}
            {!snapshot.quotaPolicies.length && (
              <li className="empty">No quota policies have been configured yet.</li>
            )}
          </ul>
        </article>
      </div>
    </section>
  );
}
