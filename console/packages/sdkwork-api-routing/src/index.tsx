import { useEffect, useState } from 'react';
import { simulateRoute } from 'sdkwork-api-admin-sdk';
import type { RoutingSimulationResult } from 'sdkwork-api-types';

const defaultSimulation: RoutingSimulationResult = {
  selected_provider_id: 'n/a',
  candidate_ids: [],
};

export function RouteSimulationPage() {
  const [simulation, setSimulation] = useState<RoutingSimulationResult>(defaultSimulation);
  const [status, setStatus] = useState('Running default route simulation for gpt-4.1...');

  useEffect(() => {
    let cancelled = false;

    void simulateRoute('chat_completion', 'gpt-4.1')
      .then((result) => {
        if (!cancelled) {
          setSimulation(result);
          setStatus('Current simulation resolved from catalog-backed routing.');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('Admin API unavailable. Route simulation requires the control plane.');
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="panel panel-highlight">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Routing</p>
          <h2>Default simulation for `chat_completion:gpt-4.1`</h2>
        </div>
        <p className="status">{status}</p>
      </div>

      <div className="metric-grid">
        <article className="metric-card">
          <span className="metric-label">Selected Provider</span>
          <strong>{simulation.selected_provider_id}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Candidate Count</span>
          <strong>{simulation.candidate_ids.length}</strong>
        </article>
      </div>

      <article className="detail-card">
        <h3>Candidate Providers</h3>
        <ul className="compact-list">
          {simulation.candidate_ids.map((candidateId) => (
            <li key={candidateId}>
              <strong>{candidateId}</strong>
              <span>{candidateId === simulation.selected_provider_id ? 'selected' : 'standby'}</span>
            </li>
          ))}
          {!simulation.candidate_ids.length && (
            <li className="empty">No candidates returned from the admin simulation endpoint.</li>
          )}
        </ul>
      </article>
    </section>
  );
}
