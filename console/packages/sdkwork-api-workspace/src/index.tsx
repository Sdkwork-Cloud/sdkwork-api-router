import { useEffect, useState } from 'react';
import { listApiKeys, listProjects, listTenants } from 'sdkwork-api-admin-sdk';
import { rootSections } from 'sdkwork-api-core';
import type { GatewayApiKeyRecord, ProjectRecord, TenantRecord } from 'sdkwork-api-types';

interface WorkspaceSnapshot {
  tenants: TenantRecord[];
  projects: ProjectRecord[];
  apiKeys: GatewayApiKeyRecord[];
}

const emptySnapshot: WorkspaceSnapshot = {
  tenants: [],
  projects: [],
  apiKeys: [],
};

export function WorkspaceDashboard() {
  const [snapshot, setSnapshot] = useState<WorkspaceSnapshot>(emptySnapshot);
  const [status, setStatus] = useState('Loading workspace registry...');

  useEffect(() => {
    let cancelled = false;

    void Promise.all([listTenants(), listProjects(), listApiKeys()])
      .then(([tenants, projects, apiKeys]) => {
        if (cancelled) {
          return;
        }

        setSnapshot({ tenants, projects, apiKeys });
        setStatus('Control-plane workspace registry is live.');
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('Admin API unavailable. Start admin-api-service to inspect tenants and keys.');
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="panel panel-accent">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">{rootSections[0]?.title ?? 'Workspace'}</p>
          <h2>Tenant and project control plane</h2>
        </div>
        <p className="status">{status}</p>
      </div>

      <div className="metric-grid">
        <article className="metric-card">
          <span className="metric-label">Tenants</span>
          <strong>{snapshot.tenants.length}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Projects</span>
          <strong>{snapshot.projects.length}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Gateway Keys</span>
          <strong>{snapshot.apiKeys.length}</strong>
        </article>
      </div>

      <div className="detail-grid">
        <article className="detail-card">
          <h3>Tenants</h3>
          <ul className="compact-list">
            {snapshot.tenants.map((tenant) => (
              <li key={tenant.id}>
                <strong>{tenant.name}</strong>
                <span>{tenant.id}</span>
              </li>
            ))}
            {!snapshot.tenants.length && <li className="empty">No tenants configured yet.</li>}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Projects</h3>
          <ul className="compact-list">
            {snapshot.projects.map((project) => (
              <li key={project.id}>
                <strong>{project.name}</strong>
                <span>{project.tenant_id}</span>
              </li>
            ))}
            {!snapshot.projects.length && <li className="empty">No projects configured yet.</li>}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Gateway Keys</h3>
          <ul className="compact-list">
            {snapshot.apiKeys.map((key) => (
              <li key={key.hashed_key}>
                <strong>{key.project_id}</strong>
                <span>{key.environment}</span>
              </li>
            ))}
            {!snapshot.apiKeys.length && <li className="empty">No gateway keys issued yet.</li>}
          </ul>
        </article>
      </div>
    </section>
  );
}
