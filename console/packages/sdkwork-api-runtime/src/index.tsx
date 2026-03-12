import type { RuntimeMode } from 'sdkwork-api-types';

const activeMode: RuntimeMode = 'embedded';

export function RuntimeStatusPage() {
  return (
    <section className="panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Runtime</p>
          <h2>Host mode and packaging posture</h2>
        </div>
        <p className="status">
          The desktop shell can embed the runtime in-process or target the standalone HTTP host.
        </p>
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
      </div>
    </section>
  );
}
