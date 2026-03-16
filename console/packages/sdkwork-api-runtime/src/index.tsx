import { startTransition, useEffect, useState } from 'react';
import {
  listExtensionRuntimeStatuses,
  listProviderHealthSnapshots,
  reloadExtensionRuntimes,
} from 'sdkwork-api-admin-sdk';
import type {
  ExtensionRuntimeReloadRequest,
  ExtensionRuntimeReloadResult,
  ExtensionRuntimeReloadScope,
  ExtensionRuntimeStatusRecord,
  ProviderHealthSnapshot,
  RuntimeMode,
} from 'sdkwork-api-types';

const activeMode: RuntimeMode = 'embedded';

function formatObservedAt(timestampMs: number): string {
  return new Date(timestampMs).toLocaleString();
}

function latestSnapshotsByProvider(snapshots: ProviderHealthSnapshot[]): ProviderHealthSnapshot[] {
  const latest = new Map<string, ProviderHealthSnapshot>();
  for (const snapshot of snapshots) {
    if (!latest.has(snapshot.provider_id)) {
      latest.set(snapshot.provider_id, snapshot);
    }
  }
  return Array.from(latest.values());
}

function latestRuntimeMessage(runtime: ExtensionRuntimeStatusRecord): string {
  return runtime.message ?? (runtime.healthy ? 'healthy' : 'unhealthy');
}

function describeReloadScope(scope: ExtensionRuntimeReloadScope): string {
  switch (scope) {
    case 'all':
      return 'all managed runtimes';
    case 'extension':
      return 'one extension family';
    case 'instance':
      return 'one connector instance';
  }
}

function describeReloadTarget(result: ExtensionRuntimeReloadResult): string {
  if (result.requested_instance_id) {
    return result.requested_instance_id;
  }
  if (result.requested_extension_id) {
    return result.requested_extension_id;
  }
  return 'all managed runtimes';
}

function describeRequestedReload(request?: ExtensionRuntimeReloadRequest): string {
  if (request?.instance_id) {
    return `connector instance ${request.instance_id}`;
  }
  if (request?.extension_id) {
    return `extension ${request.extension_id}`;
  }
  return 'all managed extension runtimes';
}

function runtimeReloadAction(runtime: ExtensionRuntimeStatusRecord): {
  label: string;
  request: ExtensionRuntimeReloadRequest;
} {
  if (runtime.runtime === 'connector' && runtime.instance_id) {
    return {
      label: 'Reload this instance',
      request: { instance_id: runtime.instance_id },
    };
  }

  return {
    label: 'Reload this extension',
    request: { extension_id: runtime.extension_id },
  };
}

export function RuntimeStatusPage() {
  const [snapshots, setSnapshots] = useState<ProviderHealthSnapshot[]>([]);
  const [runtimeStatuses, setRuntimeStatuses] = useState<ExtensionRuntimeStatusRecord[]>([]);
  const [reloadResult, setReloadResult] = useState<ExtensionRuntimeReloadResult | null>(null);
  const [reloading, setReloading] = useState(false);
  const [status, setStatus] = useState('Loading runtime control plane and persisted health telemetry...');

  useEffect(() => {
    let cancelled = false;

    void Promise.all([listProviderHealthSnapshots(), listExtensionRuntimeStatuses()])
      .then(([snapshotResult, runtimeResult]) => {
        if (cancelled) {
          return;
        }

        startTransition(() => {
          setSnapshots(snapshotResult);
          setRuntimeStatuses(runtimeResult);
          setStatus('Runtime control plane is live. You can reload managed extension runtimes without restarting the process.');
        });
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('Admin API unavailable. Runtime page is showing the static deployment posture only.');
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const latestByProvider = latestSnapshotsByProvider(snapshots);
  const runtimeFamilies = new Set(runtimeStatuses.map((runtime) => runtime.runtime)).size;
  const healthyRuntimes = runtimeStatuses.filter((runtime) => runtime.healthy).length;

  function handleReload(request?: ExtensionRuntimeReloadRequest) {
    setReloading(true);
    setStatus(`Reloading ${describeRequestedReload(request)} and rebuilding discovered runtime state...`);

    void reloadExtensionRuntimes(request)
      .then((result) => {
        startTransition(() => {
          setReloadResult(result);
          setRuntimeStatuses(result.runtime_statuses);
          setStatus(
            `Runtime reload completed for ${describeReloadTarget(result)}. ${result.active_runtime_count} active runtimes are now loaded from ${result.loadable_package_count} trusted packages.`,
          );
        });
      })
      .catch(() => {
        setStatus('Runtime reload failed. Existing runtime state was left unchanged.');
      })
      .finally(() => {
        setReloading(false);
      });
  }

  return (
    <section className="panel panel-accent">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Runtime</p>
          <h2>Host mode, reload orchestration, and persisted health telemetry</h2>
        </div>
        <div>
          <button className="button-secondary" type="button" onClick={() => handleReload()} disabled={reloading}>
            {reloading ? 'Reloading runtimes...' : 'Reload extension runtimes'}
          </button>
          <p className="status">{status}</p>
        </div>
      </div>

      <div className="metric-grid">
        <article className="metric-card">
          <span className="metric-label">Active Mode</span>
          <strong>{activeMode}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Preferred Local Store</span>
          <strong>SQLite</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Desktop Host</span>
          <strong>Tauri</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Active Runtimes</span>
          <strong>{runtimeStatuses.length}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Healthy Runtimes</span>
          <strong>{healthyRuntimes}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Runtime Families</span>
          <strong>{runtimeFamilies || 'n/a'}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Last Reload</span>
          <strong>{reloadResult ? formatObservedAt(reloadResult.reloaded_at_ms) : 'n/a'}</strong>
        </article>
      </div>

      <div className="detail-grid">
        <article className="detail-card">
          <h3>Managed Runtime Statuses</h3>
          <ul className="compact-list">
            {runtimeStatuses.map((runtime) => {
              const action = runtimeReloadAction(runtime);

              return (
                <li key={`${runtime.runtime}:${runtime.extension_id}:${runtime.instance_id || runtime.library_path || 'global'}`}>
                  <strong>{runtime.display_name}</strong>
                  <span>
                    {runtime.runtime} / {runtime.running ? 'running' : 'stopped'} / {latestRuntimeMessage(runtime)}
                  </span>
                  <span>{runtime.runtime === 'connector' ? runtime.instance_id : runtime.extension_id}</span>
                  <button
                    className="button-secondary"
                    type="button"
                    onClick={() => handleReload(action.request)}
                    disabled={reloading}
                  >
                    {reloading ? 'Reloading...' : action.label}
                  </button>
                </li>
              );
            })}
            {!runtimeStatuses.length && (
              <li className="empty">No managed extension runtimes are currently active in this process.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Last Reload Summary</h3>
          <ul className="compact-list">
            {reloadResult ? (
              <>
                <li>
                  <strong>Applied scope</strong>
                  <span>{describeReloadScope(reloadResult.scope)}</span>
                </li>
                <li>
                  <strong>Requested target</strong>
                  <span>{describeReloadTarget(reloadResult)}</span>
                </li>
                <li>
                  <strong>Resolved extension</strong>
                  <span>{reloadResult.resolved_extension_id ?? 'n/a'}</span>
                </li>
                <li>
                  <strong>Discovered packages</strong>
                  <span>{reloadResult.discovered_package_count}</span>
                </li>
                <li>
                  <strong>Loadable packages</strong>
                  <span>{reloadResult.loadable_package_count}</span>
                </li>
                <li>
                  <strong>Active runtimes</strong>
                  <span>{reloadResult.active_runtime_count}</span>
                </li>
                <li>
                  <strong>Reloaded at</strong>
                  <span>{formatObservedAt(reloadResult.reloaded_at_ms)}</span>
                </li>
              </>
            ) : (
              <li className="empty">No explicit runtime reload has been triggered from this console session yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Latest Provider Health</h3>
          <ul className="compact-list">
            {latestByProvider.map((snapshot) => (
              <li key={snapshot.provider_id}>
                <strong>{snapshot.provider_id}</strong>
                <span>
                  {snapshot.healthy ? 'healthy' : 'unhealthy'} / {snapshot.runtime}
                </span>
              </li>
            ))}
            {!latestByProvider.length && (
              <li className="empty">No persisted runtime health snapshots have been captured yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Recent Snapshot History</h3>
          <ul className="compact-list">
            {snapshots.slice(0, 6).map((snapshot, index) => (
              <li key={`${snapshot.provider_id}:${snapshot.observed_at_ms}:${index}`}>
                <strong>{snapshot.instance_id ?? snapshot.provider_id}</strong>
                <span>
                  {formatObservedAt(snapshot.observed_at_ms)} / {snapshot.running ? 'running' : 'stopped'}
                  {' / '}
                  {snapshot.message ?? (snapshot.healthy ? 'healthy' : 'unhealthy')}
                </span>
              </li>
            ))}
            {!snapshots.length && (
              <li className="empty">
                Snapshot history will appear after the standalone supervisor or embedded host captures runtime state.
              </li>
            )}
          </ul>
        </article>
      </div>
    </section>
  );
}
