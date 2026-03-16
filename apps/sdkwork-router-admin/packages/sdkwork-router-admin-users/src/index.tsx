import { useDeferredValue, useEffect, useState } from 'react';
import type { FormEvent } from 'react';

import {
  DataTable,
  InlineButton,
  Pill,
  SectionHero,
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, ManagedUser } from 'sdkwork-router-admin-types';

const bootstrapOperatorEmail = 'admin@sdkwork.local';
const bootstrapPortalEmail = 'portal@sdkwork.local';

type SaveOperatorUserInput = {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  active: boolean;
};

type SavePortalUserInput = {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
};

type UsersPageProps = AdminPageProps & {
  onSaveOperatorUser: (input: SaveOperatorUserInput) => Promise<void> | void;
  onSavePortalUser: (input: SavePortalUserInput) => Promise<void> | void;
  onToggleOperatorUser: (userId: string, active: boolean) => Promise<void> | void;
  onTogglePortalUser: (userId: string, active: boolean) => Promise<void> | void;
  onDeleteOperatorUser: (userId: string) => Promise<void> | void;
  onDeletePortalUser: (userId: string) => Promise<void> | void;
};

type OperatorDraft = {
  id?: string;
  email: string;
  display_name: string;
  password: string;
  active: boolean;
};

type PortalDraft = {
  id?: string;
  email: string;
  display_name: string;
  password: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
};

function defaultTenantId(snapshot: AdminPageProps['snapshot']): string {
  return snapshot.tenants[0]?.id ?? 'tenant_local_demo';
}

function defaultProjectId(
  snapshot: AdminPageProps['snapshot'],
  tenantId: string,
): string {
  return (
    snapshot.projects.find((project) => project.tenant_id === tenantId)?.id
    ?? snapshot.projects[0]?.id
    ?? 'project_local_demo'
  );
}

function emptyOperatorDraft(): OperatorDraft {
  return {
    email: '',
    display_name: '',
    password: '',
    active: true,
  };
}

function emptyPortalDraft(snapshot: AdminPageProps['snapshot']): PortalDraft {
  const tenantId = defaultTenantId(snapshot);
  return {
    email: '',
    display_name: '',
    password: '',
    workspace_tenant_id: tenantId,
    workspace_project_id: defaultProjectId(snapshot, tenantId),
    active: true,
  };
}

function matchesFilters(
  user: ManagedUser,
  deferredQuery: string,
  statusFilter: 'all' | 'active' | 'disabled',
): boolean {
  const statusMatches = statusFilter === 'all'
    || (statusFilter === 'active' && user.active)
    || (statusFilter === 'disabled' && !user.active);
  if (!statusMatches) {
    return false;
  }

  const haystack = [
    user.display_name,
    user.email,
    user.workspace_tenant_id ?? '',
    user.workspace_project_id ?? '',
  ]
    .join(' ')
    .toLowerCase();
  return haystack.includes(deferredQuery);
}

export function UsersPage({
  snapshot,
  onSaveOperatorUser,
  onSavePortalUser,
  onToggleOperatorUser,
  onTogglePortalUser,
  onDeleteOperatorUser,
  onDeletePortalUser,
}: UsersPageProps) {
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'disabled'>('all');
  const [operatorDraft, setOperatorDraft] = useState<OperatorDraft>(() => emptyOperatorDraft());
  const [portalDraft, setPortalDraft] = useState<PortalDraft>(() => emptyPortalDraft(snapshot));
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  useEffect(() => {
    setPortalDraft((current) => {
      const nextTenantId = current.workspace_tenant_id || defaultTenantId(snapshot);
      const availableProjects = snapshot.projects.filter(
        (project) => project.tenant_id === nextTenantId,
      );
      const nextProjectId = availableProjects.some(
        (project) => project.id === current.workspace_project_id,
      )
        ? current.workspace_project_id
        : defaultProjectId(snapshot, nextTenantId);

      if (
        nextTenantId === current.workspace_tenant_id
        && nextProjectId === current.workspace_project_id
      ) {
        return current;
      }

      return {
        ...current,
        workspace_tenant_id: nextTenantId,
        workspace_project_id: nextProjectId,
      };
    });
  }, [snapshot.projects, snapshot.tenants]);

  const filteredOperators = snapshot.operatorUsers.filter((user) =>
    matchesFilters(user, deferredQuery, statusFilter));
  const filteredPortalUsers = snapshot.portalUsers.filter((user) =>
    matchesFilters(user, deferredQuery, statusFilter));
  const availableProjects = snapshot.projects.filter(
    (project) => project.tenant_id === portalDraft.workspace_tenant_id,
  );
  const selectedProjectTraffic = snapshot.usageSummary.projects.find(
    (project) => project.project_id === portalDraft.workspace_project_id,
  );
  const selectedProjectBilling = snapshot.billingSummary.projects.find(
    (project) => project.project_id === portalDraft.workspace_project_id,
  );
  const selectedProjectTokens = snapshot.usageRecords
    .filter((record) => record.project_id === portalDraft.workspace_project_id)
    .reduce((sum, record) => sum + record.total_tokens, 0);

  const totalUsageUnits = snapshot.portalUsers.reduce((sum, user) => sum + user.usage_units, 0);
  const totalPortalTokens = snapshot.portalUsers.reduce((sum, user) => sum + user.total_tokens, 0);
  const disabledUsers = snapshot.operatorUsers.concat(snapshot.portalUsers).filter((user) => !user.active).length;

  async function handleOperatorSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveOperatorUser({
      id: operatorDraft.id,
      email: operatorDraft.email,
      display_name: operatorDraft.display_name,
      password: operatorDraft.password.trim() || undefined,
      active: operatorDraft.active,
    });
    setOperatorDraft(emptyOperatorDraft());
  }

  async function handlePortalSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSavePortalUser({
      id: portalDraft.id,
      email: portalDraft.email,
      display_name: portalDraft.display_name,
      password: portalDraft.password.trim() || undefined,
      workspace_tenant_id: portalDraft.workspace_tenant_id,
      workspace_project_id: portalDraft.workspace_project_id,
      active: portalDraft.active,
    });
    setPortalDraft(emptyPortalDraft(snapshot));
  }

  function editOperator(user: ManagedUser) {
    setOperatorDraft({
      id: user.id,
      email: user.email,
      display_name: user.display_name,
      password: '',
      active: user.active,
    });
  }

  function editPortal(user: ManagedUser) {
    setPortalDraft({
      id: user.id,
      email: user.email,
      display_name: user.display_name,
      password: '',
      workspace_tenant_id: user.workspace_tenant_id ?? defaultTenantId(snapshot),
      workspace_project_id: user.workspace_project_id ?? defaultProjectId(snapshot, user.workspace_tenant_id ?? defaultTenantId(snapshot)),
      active: user.active,
    });
  }

  return (
    <div className="adminx-page-grid">
      <SectionHero
        eyebrow="Identity"
        title="Operate users with live status, workspace binding, and usage visibility."
        detail="The user workbench is now backed by the admin control plane. Operators and portal identities are managed separately, while portal traffic and usage remain tied to real workspace projects."
        actions={(
          <div className="adminx-row">
            <Pill tone="live">Live-backed identities</Pill>
            <Pill tone="live">Live coupon campaigns</Pill>
          </div>
        )}
      />

      <section className="adminx-stat-grid">
        <StatCard
          label="Operator users"
          value={String(snapshot.operatorUsers.length)}
          detail="Super-admin and support operators with live access posture."
        />
        <StatCard
          label="Portal users"
          value={String(snapshot.portalUsers.length)}
          detail="Portal identities mapped to tenant and project scopes."
        />
        <StatCard
          label="Portal requests"
          value={String(snapshot.portalUsers.reduce((sum, user) => sum + user.request_count, 0))}
          detail="Aggregated request counts from linked project traffic."
        />
        <StatCard
          label="Metered units"
          value={String(totalUsageUnits)}
          detail="Metering units billed against bound portal projects."
        />
        <StatCard
          label="Portal tokens"
          value={String(totalPortalTokens)}
          detail="Total prompt and completion tokens accumulated across portal workspaces."
        />
        <StatCard
          label="Disabled users"
          value={String(disabledUsers)}
          detail="Suspended identities waiting for restore or review."
        />
      </section>

      <Surface
        title="Operational filters"
        detail="Search across both user populations and focus on the identities that need intervention."
      >
        <div className="adminx-form-grid">
          <label className="adminx-field">
            <span>Search users</span>
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="name, email, tenant, project"
            />
          </label>
          <label className="adminx-field">
            <span>Status</span>
            <select
              value={statusFilter}
              onChange={(event) => setStatusFilter(event.target.value as 'all' | 'active' | 'disabled')}
            >
              <option value="all">All users</option>
              <option value="active">Active only</option>
              <option value="disabled">Disabled only</option>
            </select>
          </label>
          <div className="adminx-note">
            <strong>Identity operations</strong>
            <p>Leaving the password field blank preserves the current secret when editing an existing user. The bootstrap admin and demo portal users stay protected so quickstart access cannot be accidentally removed.</p>
          </div>
        </div>
      </Surface>

      <div className="adminx-users-grid">
        <Surface
          title={operatorDraft.id ? 'Edit operator access' : 'Provision operator access'}
          detail="Operators manage catalog, traffic, and runtime posture. Keep this population tightly controlled."
        >
          <form className="adminx-form-grid" onSubmit={(event) => void handleOperatorSubmit(event)}>
            <label className="adminx-field">
              <span>Display name</span>
              <input
                value={operatorDraft.display_name}
                onChange={(event) => setOperatorDraft((current) => ({ ...current, display_name: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Email</span>
              <input
                value={operatorDraft.email}
                onChange={(event) => setOperatorDraft((current) => ({ ...current, email: event.target.value }))}
                type="email"
                required
              />
            </label>
            <label className="adminx-field">
              <span>{operatorDraft.id ? 'New password' : 'Password'}</span>
              <input
                value={operatorDraft.password}
                onChange={(event) => setOperatorDraft((current) => ({ ...current, password: event.target.value }))}
                type="password"
                placeholder={operatorDraft.id ? 'Leave blank to keep current password' : 'Set a strong password'}
              />
            </label>
            <label className="adminx-field">
              <span>Status</span>
              <select
                value={operatorDraft.active ? 'active' : 'disabled'}
                onChange={(event) => setOperatorDraft((current) => ({
                  ...current,
                  active: event.target.value === 'active',
                }))}
              >
                <option value="active">Active</option>
                <option value="disabled">Disabled</option>
              </select>
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                {operatorDraft.id ? 'Save operator' : 'Create operator'}
              </InlineButton>
              <InlineButton onClick={() => setOperatorDraft(emptyOperatorDraft())}>
                Clear form
              </InlineButton>
            </div>
          </form>
        </Surface>

        <Surface
          title={portalDraft.id ? 'Edit portal user' : 'Provision portal user'}
          detail="Portal users are bound to a tenant and project so usage, billing, and request counts stay attributable."
        >
          <form className="adminx-form-grid" onSubmit={(event) => void handlePortalSubmit(event)}>
            <label className="adminx-field">
              <span>Display name</span>
              <input
                value={portalDraft.display_name}
                onChange={(event) => setPortalDraft((current) => ({ ...current, display_name: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Email</span>
              <input
                value={portalDraft.email}
                onChange={(event) => setPortalDraft((current) => ({ ...current, email: event.target.value }))}
                type="email"
                required
              />
            </label>
            <label className="adminx-field">
              <span>{portalDraft.id ? 'New password' : 'Password'}</span>
              <input
                value={portalDraft.password}
                onChange={(event) => setPortalDraft((current) => ({ ...current, password: event.target.value }))}
                type="password"
                placeholder={portalDraft.id ? 'Leave blank to keep current password' : 'Set a strong password'}
              />
            </label>
            <label className="adminx-field">
              <span>Workspace tenant</span>
              {snapshot.tenants.length ? (
                <select
                  value={portalDraft.workspace_tenant_id}
                  onChange={(event) => {
                    const nextTenantId = event.target.value;
                    setPortalDraft((current) => ({
                      ...current,
                      workspace_tenant_id: nextTenantId,
                      workspace_project_id: defaultProjectId(snapshot, nextTenantId),
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
                  value={portalDraft.workspace_tenant_id}
                  onChange={(event) => setPortalDraft((current) => ({ ...current, workspace_tenant_id: event.target.value }))}
                />
              )}
            </label>
            <label className="adminx-field">
              <span>Workspace project</span>
              {availableProjects.length ? (
                <select
                  value={portalDraft.workspace_project_id}
                  onChange={(event) => setPortalDraft((current) => ({ ...current, workspace_project_id: event.target.value }))}
                >
                  {availableProjects.map((project) => (
                    <option key={project.id} value={project.id}>
                      {project.name} ({project.id})
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  value={portalDraft.workspace_project_id}
                  onChange={(event) => setPortalDraft((current) => ({ ...current, workspace_project_id: event.target.value }))}
                />
              )}
            </label>
            <label className="adminx-field">
              <span>Status</span>
              <select
                value={portalDraft.active ? 'active' : 'disabled'}
                onChange={(event) => setPortalDraft((current) => ({
                  ...current,
                  active: event.target.value === 'active',
                }))}
              >
                <option value="active">Active</option>
                <option value="disabled">Disabled</option>
              </select>
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                {portalDraft.id ? 'Save portal user' : 'Create portal user'}
              </InlineButton>
              <InlineButton onClick={() => setPortalDraft(emptyPortalDraft(snapshot))}>
                Clear form
              </InlineButton>
            </div>
          </form>

          <div className="adminx-note">
            <strong>Selected workspace posture</strong>
            <p>
              Requests: {selectedProjectTraffic?.request_count ?? 0}
              {' | '}
              Usage units: {selectedProjectBilling?.used_units ?? 0}
              {' | '}
              Tokens: {selectedProjectTokens}
            </p>
          </div>
        </Surface>
      </div>

      <Surface
        title="Operator roster"
        detail="Manage super-admin and support operator accounts with controlled activation and password rotation."
      >
        <DataTable
          columns={[
            {
              key: 'operator',
              label: 'Operator',
              render: (user) => (
                <div className="adminx-table-cell-stack">
                  <strong>{user.display_name}</strong>
                  <span>{user.id}</span>
                </div>
              ),
            },
            { key: 'email', label: 'Email', render: (user) => user.email },
            {
              key: 'created',
              label: 'Role',
              render: () => <Pill tone="live">operator</Pill>,
            },
            {
              key: 'status',
              label: 'Status',
              render: (user) => (
                <Pill tone={user.active ? 'live' : 'danger'}>
                  {user.active ? 'active' : 'disabled'}
                </Pill>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (user) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => editOperator(user)}>Edit</InlineButton>
                  <InlineButton onClick={() => void onToggleOperatorUser(user.id, !user.active)}>
                    {user.active ? 'Disable' : 'Restore'}
                  </InlineButton>
                  <InlineButton
                    disabled={
                      user.email === bootstrapOperatorEmail
                      || user.id === snapshot.sessionUser?.id
                    }
                    onClick={() => void onDeleteOperatorUser(user.id)}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={filteredOperators}
          empty="No operator users match the current filter."
          getKey={(user) => user.id}
        />
      </Surface>

      <Surface
        title="Portal roster"
        detail="Inspect portal users with workspace binding, request count, token usage, metered units, and activation status."
      >
        <DataTable
          columns={[
            {
              key: 'portal-user',
              label: 'Portal user',
              render: (user) => (
                <div className="adminx-table-cell-stack">
                  <strong>{user.display_name}</strong>
                  <span>{user.email}</span>
                </div>
              ),
            },
            {
              key: 'workspace',
              label: 'Workspace',
              render: (user) => (
                <div className="adminx-table-cell-stack">
                  <span>{user.workspace_tenant_id ?? '-'}</span>
                  <span>{user.workspace_project_id ?? '-'}</span>
                </div>
              ),
            },
            { key: 'requests', label: 'Requests', render: (user) => user.request_count },
            { key: 'tokens', label: 'Tokens', render: (user) => user.total_tokens },
            { key: 'units', label: 'Metered units', render: (user) => user.usage_units },
            {
              key: 'status',
              label: 'Status',
              render: (user) => (
                <Pill tone={user.active ? 'live' : 'danger'}>
                  {user.active ? 'active' : 'disabled'}
                </Pill>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (user) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => editPortal(user)}>Edit</InlineButton>
                  <InlineButton onClick={() => void onTogglePortalUser(user.id, !user.active)}>
                    {user.active ? 'Disable' : 'Restore'}
                  </InlineButton>
                  <InlineButton
                    disabled={user.email === bootstrapPortalEmail}
                    onClick={() => void onDeletePortalUser(user.id)}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={filteredPortalUsers}
          empty="No portal users match the current filter."
          getKey={(user) => user.id}
        />
      </Surface>
    </div>
  );
}
