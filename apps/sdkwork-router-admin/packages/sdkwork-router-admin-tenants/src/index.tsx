import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  Input,
  Label,
  StatusBadge,
  type DataTableColumn,
} from '@sdkwork/ui-pc-react';
import { Plus, Search } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminPageProps, CreatedGatewayApiKey } from 'sdkwork-router-admin-types';

import { ApiKeyDialog } from './page/ApiKeyDialog';
import { PlaintextApiKeyDialog } from './page/PlaintextApiKeyDialog';
import { ProjectDialog } from './page/ProjectDialog';
import { TenantDialog } from './page/TenantDialog';
import { TenantsDetailDrawer } from './page/TenantsDetailDrawer';
import { TenantsRegistrySection } from './page/TenantsRegistrySection';
import {
  ConfirmActionDialog,
  buildTenantDirectoryRows,
  createApiKeyDraft,
  defaultProjectId,
  defaultTenantId,
  type ApiKeyDraft,
  type RevealedApiKey,
  type TenantDirectoryRow,
} from './page/shared';

export function TenantsPage({
  snapshot,
  onSaveTenant,
  onSaveProject,
  onCreateApiKey,
  onDeleteTenant,
}: AdminPageProps & {
  onSaveTenant: (input: { id: string; name: string }) => Promise<void>;
  onSaveProject: (input: { tenant_id: string; id: string; name: string }) => Promise<void>;
  onCreateApiKey: (input: {
    tenant_id: string;
    project_id: string;
    environment: string;
    label?: string;
    notes?: string;
    expires_at_ms?: number | null;
  }) => Promise<CreatedGatewayApiKey>;
  onUpdateApiKeyStatus: (hashedKey: string, active: boolean) => Promise<void>;
  onDeleteApiKey: (hashedKey: string) => Promise<void>;
  onDeleteTenant: (tenantId: string) => Promise<void>;
  onDeleteProject: (projectId: string) => Promise<void>;
}) {
  const { formatNumber, t } = useAdminI18n();
  const [tenantDraft, setTenantDraft] = useState({ id: '', name: '' });
  const [projectDraft, setProjectDraft] = useState({
    tenant_id: defaultTenantId(snapshot),
    id: '',
    name: '',
  });
  const [apiKeyDraft, setApiKeyDraft] = useState<ApiKeyDraft>(() =>
    createApiKeyDraft(snapshot),
  );
  const [search, setSearch] = useState('');
  const [selectedTenantId, setSelectedTenantId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const [isTenantDialogOpen, setIsTenantDialogOpen] = useState(false);
  const [isProjectDialogOpen, setIsProjectDialogOpen] = useState(false);
  const [isApiKeyDialogOpen, setIsApiKeyDialogOpen] = useState(false);
  const [revealedApiKey, setRevealedApiKey] = useState<RevealedApiKey>(null);
  const [pendingDelete, setPendingDelete] = useState<{
    id: string;
    label: string;
  } | null>(null);
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  const selectedProjectUsage = snapshot.usageSummary.projects.find(
    (project) => project.project_id === projectDraft.id,
  );
  const selectedProjectBilling = snapshot.billingSummary.projects.find(
    (project) => project.project_id === projectDraft.id,
  );
  const selectedProjectTokens = snapshot.usageRecords
    .filter((record) => record.project_id === projectDraft.id)
    .reduce((sum, record) => sum + record.total_tokens, 0);

  const tenantRows = useMemo(
    () => buildTenantDirectoryRows(snapshot, deferredQuery),
    [
      deferredQuery,
      snapshot.apiKeys,
      snapshot.portalUsers,
      snapshot.projects,
      snapshot.tenants,
      snapshot.usageRecords,
    ],
  );

  useEffect(() => {
    if (selectedTenantId && !tenantRows.some((tenant) => tenant.id === selectedTenantId)) {
      setSelectedTenantId(null);
      setIsDetailDrawerOpen(false);
    }
  }, [selectedTenantId, tenantRows]);

  const selectedTenant = tenantRows.find((tenant) => tenant.id === selectedTenantId) ?? null;

  function resetTenantDialog() {
    setTenantDraft({ id: '', name: '' });
    setIsTenantDialogOpen(false);
  }

  function resetProjectDialog() {
    const tenantId = defaultTenantId(snapshot);

    setProjectDraft({
      tenant_id: tenantId,
      id: '',
      name: '',
    });
    setIsProjectDialogOpen(false);
  }

  function resetApiKeyDialog() {
    setApiKeyDraft(createApiKeyDraft(snapshot));
    setIsApiKeyDialogOpen(false);
  }

  function handleTenantDialogOpenChange(open: boolean) {
    if (!open) {
      resetTenantDialog();
      return;
    }

    setIsTenantDialogOpen(true);
  }

  function handleProjectDialogOpenChange(open: boolean) {
    if (!open) {
      resetProjectDialog();
      return;
    }

    setIsProjectDialogOpen(true);
  }

  function handleApiKeyDialogOpenChange(open: boolean) {
    if (!open) {
      resetApiKeyDialog();
      return;
    }

    setIsApiKeyDialogOpen(true);
  }

  function openTenantDialog(tenant?: TenantDirectoryRow) {
    setTenantDraft(
      tenant
        ? {
            id: tenant.id,
            name: tenant.name,
          }
        : { id: '', name: '' },
    );
    setIsTenantDialogOpen(true);
  }

  function openProjectDialog(tenant?: TenantDirectoryRow) {
    const tenantId = tenant?.id ?? defaultTenantId(snapshot);

    setProjectDraft({
      tenant_id: tenantId,
      id: '',
      name: '',
    });
    setIsProjectDialogOpen(true);
  }

  function openApiKeyDialog(tenant?: TenantDirectoryRow) {
    const tenantId = tenant?.id ?? defaultTenantId(snapshot);

    setApiKeyDraft(
      createApiKeyDraft(snapshot, {
        tenant_id: tenantId,
        project_id: defaultProjectId(snapshot, tenantId),
      }),
    );
    setIsApiKeyDialogOpen(true);
  }

  function openDetailDrawer(tenant: TenantDirectoryRow) {
    setSelectedTenantId(tenant.id);
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedTenantId(null);
    }
  }

  async function handleTenantSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveTenant(tenantDraft);
    resetTenantDialog();
  }

  async function handleProjectSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveProject(projectDraft);
    resetProjectDialog();
  }

  async function handleApiKeySubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedLabel = apiKeyDraft.label.trim();
    const normalizedNotes = apiKeyDraft.notes.trim();
    const normalizedExpiresAt = apiKeyDraft.expires_at_ms.trim();
    const parsedExpiresAt =
      normalizedExpiresAt === '' ? undefined : Number(normalizedExpiresAt);

    const created = await onCreateApiKey({
      tenant_id: apiKeyDraft.tenant_id,
      project_id: apiKeyDraft.project_id,
      environment: apiKeyDraft.environment,
      label: normalizedLabel || undefined,
      notes: normalizedNotes || undefined,
      expires_at_ms:
        parsedExpiresAt !== undefined
        && Number.isFinite(parsedExpiresAt)
        && Number.isInteger(parsedExpiresAt)
          ? parsedExpiresAt
          : undefined,
    });
    setRevealedApiKey(created);
    resetApiKeyDialog();
  }

  async function confirmDelete() {
    if (!pendingDelete) {
      return;
    }

    await onDeleteTenant(pendingDelete.id);
    setPendingDelete(null);
    setSelectedTenantId(null);
    setIsDetailDrawerOpen(false);
  }

  const availableApiKeyProjects = snapshot.projects.filter(
    (project) => project.tenant_id === apiKeyDraft.tenant_id,
  );

  const columns = useMemo<DataTableColumn<TenantDirectoryRow>[]>(
    () => [
      {
        id: 'tenant',
        header: t('Tenant'),
        cell: (tenant) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {tenant.name}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {tenant.id}
            </div>
          </div>
        ),
      },
      {
        id: 'projects',
        header: t('Projects'),
        cell: (tenant) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{t('{count} attached', { count: formatNumber(tenant.projectCount) })}</div>
            <div>{tenant.projectSummary}</div>
          </div>
        ),
      },
      {
        id: 'portal-users',
        align: 'right',
        header: t('Portal users'),
        cell: (tenant) => formatNumber(tenant.portalUserCount),
        width: 120,
      },
      {
        id: 'gateway',
        header: t('Gateway posture'),
        cell: (tenant) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>
              {t('{active} active / {total} total', {
                active: formatNumber(tenant.activeApiKeyCount),
                total: formatNumber(tenant.apiKeyCount),
              })}
            </div>
            <div>{tenant.environmentSummary}</div>
          </div>
        ),
      },
      {
        id: 'traffic',
        header: t('Traffic'),
        cell: (tenant) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{t('{count} requests', { count: formatNumber(tenant.requestCount) })}</div>
            <div>{t('{count} tokens', { count: formatNumber(tenant.tokenCount) })}</div>
          </div>
        ),
      },
      {
        id: 'readiness',
        header: t('Readiness'),
        cell: (tenant) => (
          <StatusBadge
            showIcon
            status={tenant.canIssueApiKey ? 'ready' : 'incomplete'}
            variant={tenant.canIssueApiKey ? 'success' : 'warning'}
          />
        ),
        width: 140,
      },
    ],
    [formatNumber, t],
  );

  const totalProjects = tenantRows.reduce((sum, tenant) => sum + tenant.projectCount, 0);
  const totalPortalUsers = tenantRows.reduce(
    (sum, tenant) => sum + tenant.portalUserCount,
    0,
  );
  const activeApiKeyCount = tenantRows.reduce(
    (sum, tenant) => sum + tenant.activeApiKeyCount,
    0,
  );

  return (
    <>
      <div className="flex h-full min-h-0 flex-col gap-4 p-4 lg:p-5">
        <Card className="shrink-0">
          <CardContent className="p-4">
            <form
              className="flex flex-wrap items-center gap-3"
              onSubmit={(event) => event.preventDefault()}
            >
              <div className="min-w-[18rem] flex-[1.5]">
                <Label className="sr-only" htmlFor="tenants-search">
                  {t('Search tenants')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="tenants-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setSearch(event.target.value)
                    }
                    placeholder={t('tenant, project, environment, key label')}
                    value={search}
                  />
                </div>
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatNumber(tenantRows.length) })}
                  {' | '}
                  {t('{count} projects', { count: formatNumber(totalProjects) })}
                  {' | '}
                  {t('{count} active keys', { count: formatNumber(activeApiKeyCount) })}
                </div>
                <Button onClick={() => openTenantDialog()} type="button" variant="primary">
                  <Plus className="w-4 h-4" />
                  {t('New tenant')}
                </Button>
                <Button onClick={() => openProjectDialog()} type="button" variant="outline">
                  <Plus className="w-4 h-4" />
                  {t('New project')}
                </Button>
                <Button onClick={() => openApiKeyDialog()} type="button" variant="outline">
                  <Plus className="w-4 h-4" />
                  {t('Issue gateway key')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <TenantsRegistrySection
            activeApiKeyCount={activeApiKeyCount}
            columns={columns}
            filteredTenants={tenantRows}
            onOpenApiKeyDialog={openApiKeyDialog}
            onOpenProjectDialog={openProjectDialog}
            onOpenTenantDialog={openTenantDialog}
            onRequestDelete={(tenant) =>
              setPendingDelete({
                id: tenant.id,
                label: `${tenant.name} (${tenant.id})`,
              })
            }
            onSelectTenant={openDetailDrawer}
            selectedTenantId={selectedTenantId}
            totalPortalUsers={totalPortalUsers}
            totalProjects={totalProjects}
          />
        </div>
      </div>

      <TenantsDetailDrawer
        canDelete={
          selectedTenant
            ? selectedTenant.projectCount === 0 && selectedTenant.portalUserCount === 0
            : false
        }
        onDelete={() => {
          if (!selectedTenant) {
            return;
          }
          setPendingDelete({
            id: selectedTenant.id,
            label: `${selectedTenant.name} (${selectedTenant.id})`,
          });
        }}
        onEdit={() => {
          if (!selectedTenant) {
            return;
          }
          setIsDetailDrawerOpen(false);
          openTenantDialog(selectedTenant);
        }}
        onIssueKey={() => {
          if (!selectedTenant) {
            return;
          }
          setIsDetailDrawerOpen(false);
          openApiKeyDialog(selectedTenant);
        }}
        onNewProject={() => {
          if (!selectedTenant) {
            return;
          }
          setIsDetailDrawerOpen(false);
          openProjectDialog(selectedTenant);
        }}
        onOpenChange={handleDetailDrawerOpenChange}
        open={isDetailDrawerOpen}
        selectedTenant={selectedTenant}
      />

      <TenantDialog
        draft={tenantDraft}
        onOpenChange={handleTenantDialogOpenChange}
        onSubmit={(event) => void handleTenantSubmit(event)}
        open={isTenantDialogOpen}
        setDraft={setTenantDraft}
      />

      <ProjectDialog
        draft={projectDraft}
        onOpenChange={handleProjectDialogOpenChange}
        onSubmit={(event) => void handleProjectSubmit(event)}
        open={isProjectDialogOpen}
        selectedProjectBilling={selectedProjectBilling}
        selectedProjectTokens={selectedProjectTokens}
        selectedProjectUsage={selectedProjectUsage}
        setDraft={setProjectDraft}
        snapshot={snapshot}
      />

      <ApiKeyDialog
        availableProjects={availableApiKeyProjects}
        draft={apiKeyDraft}
        onOpenChange={handleApiKeyDialogOpenChange}
        onSubmit={(event) => void handleApiKeySubmit(event)}
        open={isApiKeyDialogOpen}
        setDraft={setApiKeyDraft}
        snapshot={snapshot}
      />

      <PlaintextApiKeyDialog
        onClose={() => setRevealedApiKey(null)}
        revealedApiKey={revealedApiKey}
      />

      <ConfirmActionDialog
        confirmLabel={t('Delete now')}
        description={
          pendingDelete
            ? t(
                'Delete {label}. This permanently removes the selected resource from the workspace registry.',
                { label: pendingDelete.label },
              )
            : ''
        }
        onConfirm={() => void confirmDelete()}
        onOpenChange={(open) => {
          if (!open) {
            setPendingDelete(null);
          }
        }}
        open={Boolean(pendingDelete)}
        title={t('Delete workspace resource')}
      />
    </>
  );
}
