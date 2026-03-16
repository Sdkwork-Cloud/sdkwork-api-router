import { useState } from 'react';
import type { FormEvent } from 'react';

import {
  DataTable,
  InlineButton,
  SectionHero,
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, CreatedGatewayApiKey } from 'sdkwork-router-admin-types';

export function TenantsPage({
  snapshot,
  onSaveTenant,
  onSaveProject,
  onCreateApiKey,
  onUpdateApiKeyStatus,
  onDeleteApiKey,
  onDeleteTenant,
  onDeleteProject,
}: AdminPageProps & {
  onSaveTenant: (input: { id: string; name: string }) => Promise<void>;
  onSaveProject: (input: { tenant_id: string; id: string; name: string }) => Promise<void>;
  onCreateApiKey: (input: {
    tenant_id: string;
    project_id: string;
    environment: string;
  }) => Promise<CreatedGatewayApiKey>;
  onUpdateApiKeyStatus: (hashedKey: string, active: boolean) => Promise<void>;
  onDeleteApiKey: (hashedKey: string) => Promise<void>;
  onDeleteTenant: (tenantId: string) => Promise<void>;
  onDeleteProject: (projectId: string) => Promise<void>;
}) {
  const [tenantDraft, setTenantDraft] = useState({ id: '', name: '' });
  const [projectDraft, setProjectDraft] = useState({
    tenant_id: snapshot.tenants[0]?.id ?? 'tenant_local_demo',
    id: '',
    name: '',
  });
  const [apiKeyDraft, setApiKeyDraft] = useState({
    tenant_id: snapshot.tenants[0]?.id ?? 'tenant_local_demo',
    project_id: snapshot.projects[0]?.id ?? 'project_local_demo',
    environment: 'production',
  });
  const [lastIssuedKey, setLastIssuedKey] = useState<CreatedGatewayApiKey | null>(null);

  const selectedProjectUsage = snapshot.usageSummary.projects.find(
    (project) => project.project_id === projectDraft.id,
  );
  const selectedProjectBilling = snapshot.billingSummary.projects.find(
    (project) => project.project_id === projectDraft.id,
  );
  const selectedProjectTokens = snapshot.usageRecords
    .filter((record) => record.project_id === projectDraft.id)
    .reduce((sum, record) => sum + record.total_tokens, 0);
  const portalProjects = new Set(
    snapshot.portalUsers
      .map((user) => user.workspace_project_id)
      .filter((projectId): projectId is string => Boolean(projectId)),
  );
  const portalTenants = new Set(
    snapshot.portalUsers
      .map((user) => user.workspace_tenant_id)
      .filter((tenantId): tenantId is string => Boolean(tenantId)),
  );

  async function handleTenantSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveTenant(tenantDraft);
    setTenantDraft({ id: '', name: '' });
  }

  async function handleProjectSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveProject(projectDraft);
    setProjectDraft({
      tenant_id: projectDraft.tenant_id,
      id: '',
      name: '',
    });
  }

  async function handleApiKeySubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const created = await onCreateApiKey(apiKeyDraft);
    setLastIssuedKey(created);
  }

  const availableApiKeyProjects = snapshot.projects.filter(
    (project) => project.tenant_id === apiKeyDraft.tenant_id,
  );

  return (
    <div className="adminx-page-grid">
      <SectionHero
        eyebrow="Workspace"
        title="Manage tenants, projects, and gateway key inventories."
        detail="Tenant and project entities are backed by the live admin API. The workbench now supports direct editing from the registries below, so operators can correct ownership and naming without retyping ids."
      />

      <section className="adminx-stat-grid">
        <StatCard
          label="Tenants"
          value={String(snapshot.tenants.length)}
          detail="Distinct tenant workspaces managed by the router."
        />
        <StatCard
          label="Projects"
          value={String(snapshot.projects.length)}
          detail="Projects linked to tenants and routing or billing posture."
        />
        <StatCard
          label="Gateway keys"
          value={String(snapshot.apiKeys.length)}
          detail="Issued gateway keys across environments."
        />
      </section>

      <div className="adminx-users-grid">
        <Surface
          title={tenantDraft.id ? 'Edit tenant' : 'Create tenant'}
          detail="Submitting an existing tenant id performs a safe upsert."
        >
          <form className="adminx-form-grid" onSubmit={(event) => void handleTenantSubmit(event)}>
            <label className="adminx-field">
              <span>Tenant id</span>
              <input
                value={tenantDraft.id}
                onChange={(event) => setTenantDraft((current) => ({ ...current, id: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Tenant name</span>
              <input
                value={tenantDraft.name}
                onChange={(event) => setTenantDraft((current) => ({ ...current, name: event.target.value }))}
                required
              />
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                {tenantDraft.id ? 'Save tenant' : 'Create tenant'}
              </InlineButton>
              <InlineButton onClick={() => setTenantDraft({ id: '', name: '' })}>
                Clear form
              </InlineButton>
            </div>
          </form>
        </Surface>

      <Surface
        title={projectDraft.id ? 'Edit project' : 'Create project'}
        detail="Projects remain the routing, usage, and billing ownership boundary."
      >
          <form className="adminx-form-grid" onSubmit={(event) => void handleProjectSubmit(event)}>
            <label className="adminx-field">
              <span>Tenant id</span>
              {snapshot.tenants.length ? (
                <select
                  value={projectDraft.tenant_id}
                  onChange={(event) => setProjectDraft((current) => ({ ...current, tenant_id: event.target.value }))}
                >
                  {snapshot.tenants.map((tenant) => (
                    <option key={tenant.id} value={tenant.id}>
                      {tenant.name} ({tenant.id})
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  value={projectDraft.tenant_id}
                  onChange={(event) => setProjectDraft((current) => ({ ...current, tenant_id: event.target.value }))}
                  required
                />
              )}
            </label>
            <label className="adminx-field">
              <span>Project id</span>
              <input
                value={projectDraft.id}
                onChange={(event) => setProjectDraft((current) => ({ ...current, id: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Project name</span>
              <input
                value={projectDraft.name}
                onChange={(event) => setProjectDraft((current) => ({ ...current, name: event.target.value }))}
                required
              />
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                {projectDraft.id ? 'Save project' : 'Create project'}
              </InlineButton>
              <InlineButton
                onClick={() => setProjectDraft({
                  tenant_id: snapshot.tenants[0]?.id ?? 'tenant_local_demo',
                  id: '',
                  name: '',
                })}
              >
                Clear form
              </InlineButton>
            </div>
          </form>

          <div className="adminx-note">
            <strong>Selected project posture</strong>
            <p>
              Requests: {selectedProjectUsage?.request_count ?? 0}
              {' | '}
              Usage units: {selectedProjectBilling?.used_units ?? 0}
              {' | '}
              Tokens: {selectedProjectTokens}
            </p>
          </div>
        </Surface>
      </div>

      <Surface
        title="Issue gateway key"
        detail="Mint a project-scoped API key for an environment and reveal the plaintext once for secure handoff."
      >
        <form className="adminx-form-grid" onSubmit={(event) => void handleApiKeySubmit(event)}>
          <label className="adminx-field">
            <span>Tenant</span>
            {snapshot.tenants.length ? (
              <select
                value={apiKeyDraft.tenant_id}
                onChange={(event) => {
                  const nextTenantId = event.target.value;
                  setApiKeyDraft((current) => ({
                    ...current,
                    tenant_id: nextTenantId,
                    project_id: snapshot.projects.find((project) => project.tenant_id === nextTenantId)?.id ?? '',
                  }));
                }}
              >
                {snapshot.tenants.map((tenant) => (
                  <option key={tenant.id} value={tenant.id}>
                    {tenant.name} ({tenant.id})
                  </option>
                ))}
              </select>
            ) : (
              <input
                value={apiKeyDraft.tenant_id}
                onChange={(event) => setApiKeyDraft((current) => ({ ...current, tenant_id: event.target.value }))}
                required
              />
            )}
          </label>
          <label className="adminx-field">
            <span>Project</span>
            {availableApiKeyProjects.length ? (
              <select
                value={apiKeyDraft.project_id}
                onChange={(event) => setApiKeyDraft((current) => ({ ...current, project_id: event.target.value }))}
              >
                {availableApiKeyProjects.map((project) => (
                  <option key={project.id} value={project.id}>
                    {project.name} ({project.id})
                  </option>
                ))}
              </select>
            ) : (
              <input
                value={apiKeyDraft.project_id}
                onChange={(event) => setApiKeyDraft((current) => ({ ...current, project_id: event.target.value }))}
                required
              />
            )}
          </label>
          <label className="adminx-field">
            <span>Environment</span>
            <select
              value={apiKeyDraft.environment}
              onChange={(event) => setApiKeyDraft((current) => ({ ...current, environment: event.target.value }))}
            >
              <option value="production">Production</option>
              <option value="staging">Staging</option>
              <option value="development">Development</option>
            </select>
          </label>
          <div className="adminx-form-actions">
            <InlineButton tone="primary" type="submit">
              Issue gateway key
            </InlineButton>
          </div>
        </form>

        {lastIssuedKey ? (
          <div className="adminx-note">
            <strong>Last issued key</strong>
            <p>
              {lastIssuedKey.project_id}
              {' | '}
              {lastIssuedKey.environment}
              {' | '}
              hashed: {lastIssuedKey.hashed}
            </p>
            <code>{lastIssuedKey.plaintext}</code>
          </div>
        ) : (
          <div className="adminx-note">
            <strong>Secure handoff</strong>
            <p>The plaintext key is shown only immediately after issuance. Store it securely before leaving this page.</p>
          </div>
        )}
      </Surface>

      <Surface
        title="Deletion posture"
        detail="Project deletion retires gateway keys and quota policies for the project. Tenant deletion is only allowed after its projects are cleared and portal users are re-bound."
      >
        <div className="adminx-note">
          <strong>Operational safety</strong>
          <p>Portal users must be reassigned before deleting the tenant or project they are bound to. Usage and billing history remain available as audit data even after workspace ownership records are retired.</p>
        </div>
      </Surface>

      <Surface title="Tenant registry" detail="Live tenant catalog from the admin API.">
        <DataTable
          columns={[
            { key: 'id', label: 'Tenant id', render: (tenant) => <strong>{tenant.id}</strong> },
            { key: 'name', label: 'Name', render: (tenant) => tenant.name },
            {
              key: 'actions',
              label: 'Actions',
              render: (tenant) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => setTenantDraft({ id: tenant.id, name: tenant.name })}>
                    Edit
                  </InlineButton>
                  <InlineButton
                    disabled={
                      snapshot.projects.some((project) => project.tenant_id === tenant.id)
                      || portalTenants.has(tenant.id)
                    }
                    onClick={() => void onDeleteTenant(tenant.id)}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.tenants}
          empty="No tenants available."
          getKey={(tenant) => tenant.id}
        />
      </Surface>

      <Surface
        title="Project registry"
        detail="Projects are the routing, billing, and usage ownership boundary."
      >
        <DataTable
          columns={[
            { key: 'id', label: 'Project id', render: (project) => <strong>{project.id}</strong> },
            { key: 'tenant', label: 'Tenant', render: (project) => project.tenant_id },
            { key: 'name', label: 'Name', render: (project) => project.name },
            {
              key: 'actions',
              label: 'Actions',
              render: (project) => (
                <div className="adminx-row">
                  <InlineButton
                    onClick={() => setProjectDraft({
                      tenant_id: project.tenant_id,
                      id: project.id,
                      name: project.name,
                    })}
                  >
                    Edit
                  </InlineButton>
                  <InlineButton
                    disabled={portalProjects.has(project.id)}
                    onClick={() => void onDeleteProject(project.id)}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.projects}
          empty="No projects available."
          getKey={(project) => project.id}
        />
      </Surface>

      <Surface title="Gateway key inventory" detail="Keys are presented as hashed records only.">
        <DataTable
          columns={[
            { key: 'project', label: 'Project', render: (key) => key.project_id },
            { key: 'tenant', label: 'Tenant', render: (key) => key.tenant_id },
            { key: 'environment', label: 'Environment', render: (key) => key.environment },
            {
              key: 'active',
              label: 'Status',
              render: (key) => (
                <span>{key.active ? 'active' : 'revoked'}</span>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (key) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => void onUpdateApiKeyStatus(key.hashed_key, !key.active)}>
                    {key.active ? 'Revoke' : 'Restore'}
                  </InlineButton>
                  <InlineButton onClick={() => void onDeleteApiKey(key.hashed_key)}>
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.apiKeys}
          empty="No gateway keys available."
          getKey={(key) => key.hashed_key}
        />
      </Surface>
    </div>
  );
}
