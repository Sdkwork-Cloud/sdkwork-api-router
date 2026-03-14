import { useEffect, useState } from 'react';
import { listRoutingDecisionLogs, simulateRoute } from 'sdkwork-api-admin-sdk';
import type { RoutingDecisionLog, RoutingSimulationResult } from 'sdkwork-api-types';

const defaultSimulation: RoutingSimulationResult = {
  selected_provider_id: 'n/a',
  candidate_ids: [],
  slo_applied: false,
  slo_degraded: false,
  assessments: [],
};

export function RouteSimulationPage() {
  const [simulation, setSimulation] = useState<RoutingSimulationResult>(defaultSimulation);
  const [decisionLogs, setDecisionLogs] = useState<RoutingDecisionLog[]>([]);
  const [status, setStatus] = useState('Running default route simulation for gpt-4.1...');

  useEffect(() => {
    let cancelled = false;

    void Promise.all([simulateRoute('chat_completion', 'gpt-4.1', 11), listRoutingDecisionLogs()])
      .then(([result, logs]) => {
        if (!cancelled) {
          setSimulation(result);
          setDecisionLogs(logs.slice(0, 8));
          setStatus('Current simulation resolved from catalog-backed routing, and recent gateway decisions were loaded.');
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
        <article className="metric-card">
          <span className="metric-label">Strategy</span>
          <strong>{simulation.strategy ?? 'static_fallback'}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Selection Seed</span>
          <strong>{simulation.selection_seed ?? 'n/a'}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">SLO State</span>
          <strong>
            {simulation.slo_applied ? (simulation.slo_degraded ? 'degraded' : 'compliant') : 'inactive'}
          </strong>
        </article>
      </div>

      <article className="detail-card">
        <h3>Decision Reason</h3>
        <p>{simulation.selection_reason ?? 'No routing explanation returned yet.'}</p>
      </article>

      <article className="detail-card">
        <h3>Candidate Providers</h3>
        <ul className="compact-list">
          {simulation.assessments.map((assessment) => (
            <li key={assessment.provider_id}>
              <div>
                <strong>{assessment.provider_id}</strong>
                <span>
                  {assessment.provider_id === simulation.selected_provider_id ? 'selected' : 'standby'}
                </span>
              </div>
              <div>
                <span>{assessment.available ? 'available' : 'unavailable'}</span>
                <span>{assessment.health}</span>
                <span>policy #{assessment.policy_rank + 1}</span>
                <span>weight {assessment.weight ?? 100}</span>
                {assessment.cost !== undefined ? <span>cost {assessment.cost}</span> : null}
                {assessment.latency_ms !== undefined ? (
                  <span>latency {assessment.latency_ms}ms</span>
                ) : null}
                {assessment.slo_eligible !== undefined ? (
                  <span>{assessment.slo_eligible ? 'SLO eligible' : 'SLO excluded'}</span>
                ) : null}
              </div>
              <div>
                {assessment.reasons.length ? assessment.reasons.join(', ') : 'No detailed reasons returned.'}
                {assessment.slo_violations.length ? ` | SLO: ${assessment.slo_violations.join(', ')}` : ''}
              </div>
            </li>
          ))}
          {!simulation.assessments.length && (
            <li className="empty">No candidates returned from the admin simulation endpoint.</li>
          )}
        </ul>
      </article>

      <article className="detail-card">
        <h3>Recent Decision Logs</h3>
        <ul className="compact-list">
          {decisionLogs.map((log) => (
            <li key={log.decision_id}>
              <div>
                <strong>{log.selected_provider_id}</strong>
                <span>{log.decision_source}</span>
              </div>
              <div>
                <span>{log.capability}</span>
                <span>{log.route_key}</span>
                <span>{log.strategy}</span>
                <span>{formatSloState(log)}</span>
                {log.selection_seed !== undefined ? <span>seed {log.selection_seed}</span> : null}
              </div>
              <div>{log.selection_reason ?? 'No persisted selection reason.'}</div>
            </li>
          ))}
          {!decisionLogs.length && (
            <li className="empty">No persisted routing decision logs returned from the admin API.</li>
          )}
        </ul>
      </article>
    </section>
  );
}

function formatSloState(log: RoutingDecisionLog): string {
  if (!log.slo_applied) {
    return 'SLO inactive';
  }

  return log.slo_degraded ? 'SLO degraded' : 'SLO compliant';
}
