import './App.css';

import { appName } from 'sdkwork-api-core';
import { ChannelRegistryPage } from 'sdkwork-api-channel';
import { RouteSimulationPage } from 'sdkwork-api-routing';
import { RuntimeStatusPage } from 'sdkwork-api-runtime';
import { RequestExplorerPage } from 'sdkwork-api-usage';
import { WorkspaceDashboard } from 'sdkwork-api-workspace';

export function App() {
  return (
    <main className="app-shell">
      <header className="hero">
        <div className="hero-copy">
          <p className="eyebrow">SDKWork API Gateway Console</p>
          <h1>{appName}</h1>
          <p className="hero-text">
            Observe the control plane, catalog mesh, routing decisions, and usage telemetry from a
            single embedded-ready dashboard.
          </p>
        </div>
        <aside className="hero-aside">
          <span>Axum</span>
          <span>SQLite</span>
          <span>Tauri</span>
          <span>OpenAI API</span>
        </aside>
      </header>

      <WorkspaceDashboard />
      <ChannelRegistryPage />
      <RouteSimulationPage />
      <RequestExplorerPage />
      <RuntimeStatusPage />
    </main>
  );
}
