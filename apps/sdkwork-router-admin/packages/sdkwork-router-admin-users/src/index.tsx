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
import type { AdminPageProps, ManagedUser } from 'sdkwork-router-admin-types';

import { OperatorUserDialog } from './page/OperatorUserDialog';
import { PortalUserDialog } from './page/PortalUserDialog';
import { UsersDetailDrawer } from './page/UsersDetailDrawer';
import { UsersRegistrySection } from './page/UsersRegistrySection';
import {
  ConfirmActionDialog,
  SelectField,
  defaultProjectId,
  defaultTenantId,
  emptyOperatorDraft,
  emptyPortalDraft,
  isProtectedUser,
  matchesFilters,
  operatorDraftFromUser,
  portalDraftFromUser,
  type PendingDelete,
  type SaveOperatorUserInput,
  type SavePortalUserInput,
} from './page/shared';

type UsersPageProps = AdminPageProps & {
  onSaveOperatorUser: (input: SaveOperatorUserInput) => Promise<void> | void;
  onSavePortalUser: (input: SavePortalUserInput) => Promise<void> | void;
  onToggleOperatorUser: (userId: string, active: boolean) => Promise<void> | void;
  onTogglePortalUser: (userId: string, active: boolean) => Promise<void> | void;
  onDeleteOperatorUser: (userId: string) => Promise<void> | void;
  onDeletePortalUser: (userId: string) => Promise<void> | void;
};

export function UsersPage({
  snapshot,
  onSaveOperatorUser,
  onSavePortalUser,
  onToggleOperatorUser,
  onTogglePortalUser,
  onDeleteOperatorUser,
  onDeletePortalUser,
}: UsersPageProps) {
  const { formatNumber, t } = useAdminI18n();
  const [search, setSearch] = useState('');
  const [roleFilter, setRoleFilter] = useState<'all' | 'operator' | 'portal'>('all');
  const [statusFilter, setStatusFilter] =
    useState<'all' | 'active' | 'disabled'>('all');
  const [operatorDraft, setOperatorDraft] = useState(() => emptyOperatorDraft());
  const [portalDraft, setPortalDraft] = useState(() => emptyPortalDraft(snapshot));
  const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);
  const [isOperatorDialogOpen, setIsOperatorDialogOpen] = useState(false);
  const [isPortalDialogOpen, setIsPortalDialogOpen] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<PendingDelete>(null);
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

  const filteredUsers = useMemo(
    () =>
      [...snapshot.operatorUsers, ...snapshot.portalUsers]
        .filter((user) =>
          matchesFilters(user, deferredQuery, roleFilter, statusFilter),
        )
        .sort(
          (left, right) =>
            left.role.localeCompare(right.role)
            || left.display_name.localeCompare(right.display_name)
            || left.email.localeCompare(right.email),
        ),
    [deferredQuery, roleFilter, snapshot.operatorUsers, snapshot.portalUsers, statusFilter],
  );

  useEffect(() => {
    if (selectedUserId && !filteredUsers.some((user) => user.id === selectedUserId)) {
      setSelectedUserId(null);
      setIsDetailDrawerOpen(false);
    }
  }, [filteredUsers, selectedUserId]);

  const selectedUser = filteredUsers.find((user) => user.id === selectedUserId) ?? null;
  const availableProjects = snapshot.projects.filter(
    (project) => project.tenant_id === portalDraft.workspace_tenant_id,
  );
  const selectedProject = snapshot.projects.find(
    (project) => project.id === portalDraft.workspace_project_id,
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
  const selectedUserProject = snapshot.projects.find(
    (project) => project.id === selectedUser?.workspace_project_id,
  );
  const selectedUserTraffic = snapshot.usageSummary.projects.find(
    (project) => project.project_id === selectedUser?.workspace_project_id,
  );
  const selectedUserBilling = snapshot.billingSummary.projects.find(
    (project) => project.project_id === selectedUser?.workspace_project_id,
  );

  const columns = useMemo<DataTableColumn<ManagedUser>[]>(
    () => [
      {
        id: 'user',
        header: t('User'),
        cell: (user) => (
          <div className="space-y-1">
            <div className="font-medium text-[var(--sdk-color-text-primary)]">
              {user.display_name}
            </div>
            <div className="text-sm text-[var(--sdk-color-text-secondary)]">
              {user.email}
            </div>
          </div>
        ),
      },
      {
        id: 'type',
        header: t('Type'),
        cell: (user) => (
          <StatusBadge
            showIcon
            status={user.role}
            variant={user.role === 'operator' ? 'success' : 'secondary'}
          />
        ),
        width: 140,
      },
      {
        id: 'workspace',
        header: t('Workspace'),
        cell: (user) => (
          <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
            <div>{user.workspace_tenant_id ?? t('control-plane')}</div>
            <div>{user.workspace_project_id ?? t('shared operator context')}</div>
          </div>
        ),
      },
      {
        id: 'requests',
        header: t('Requests'),
        align: 'right',
        cell: (user) => formatNumber(user.request_count),
        width: 120,
      },
      {
        id: 'tokens',
        header: t('Tokens'),
        align: 'right',
        cell: (user) => formatNumber(user.total_tokens),
        width: 140,
      },
      {
        id: 'status',
        header: t('Status'),
        cell: (user) => (
          <StatusBadge
            showIcon
            status={user.active ? 'active' : 'disabled'}
            variant={user.active ? 'success' : 'danger'}
          />
        ),
        width: 140,
      },
    ],
    [formatNumber, t],
  );

  function resetOperatorDialog() {
    setIsOperatorDialogOpen(false);
    setOperatorDraft(emptyOperatorDraft());
  }

  function resetPortalDialog() {
    setIsPortalDialogOpen(false);
    setPortalDraft(emptyPortalDraft(snapshot));
  }

  function handleOperatorDialogOpenChange(open: boolean) {
    if (!open) {
      resetOperatorDialog();
      return;
    }

    setIsOperatorDialogOpen(true);
  }

  function handlePortalDialogOpenChange(open: boolean) {
    if (!open) {
      resetPortalDialog();
      return;
    }

    setIsPortalDialogOpen(true);
  }

  function openOperatorDialog(user?: ManagedUser) {
    setOperatorDraft(user ? operatorDraftFromUser(user) : emptyOperatorDraft());
    setIsOperatorDialogOpen(true);
  }

  function openPortalDialog(user?: ManagedUser) {
    setPortalDraft(user ? portalDraftFromUser(user, snapshot) : emptyPortalDraft(snapshot));
    setIsPortalDialogOpen(true);
  }

  function openDetailDrawer(user: ManagedUser) {
    setSelectedUserId(user.id);
    setIsDetailDrawerOpen(true);
  }

  function handleDetailDrawerOpenChange(open: boolean) {
    setIsDetailDrawerOpen(open);
    if (!open) {
      setSelectedUserId(null);
    }
  }

  async function handleOperatorSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveOperatorUser({
      id: operatorDraft.id,
      email: operatorDraft.email.trim(),
      display_name: operatorDraft.display_name.trim(),
      password: operatorDraft.password.trim() || undefined,
      active: operatorDraft.active,
    });
    resetOperatorDialog();
  }

  async function handlePortalSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSavePortalUser({
      id: portalDraft.id,
      email: portalDraft.email.trim(),
      display_name: portalDraft.display_name.trim(),
      password: portalDraft.password.trim() || undefined,
      workspace_tenant_id: portalDraft.workspace_tenant_id,
      workspace_project_id: portalDraft.workspace_project_id,
      active: portalDraft.active,
    });
    resetPortalDialog();
  }

  async function confirmDelete() {
    if (!pendingDelete) {
      return;
    }

    if (pendingDelete.kind === 'operator') {
      await onDeleteOperatorUser(pendingDelete.user.id);
    } else {
      await onDeletePortalUser(pendingDelete.user.id);
    }

    setPendingDelete(null);
    setSelectedUserId(null);
    setIsDetailDrawerOpen(false);
  }

  const operatorCount = filteredUsers.filter((user) => user.role === 'operator').length;
  const portalCount = filteredUsers.length - operatorCount;
  const activeCount = filteredUsers.filter((user) => user.active).length;

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
                <Label className="sr-only" htmlFor="users-search">
                  {t('Search users')}
                </Label>
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                  <Input
                    className="pl-9"
                    id="users-search"
                    onChange={(event: ChangeEvent<HTMLInputElement>) =>
                      setSearch(event.target.value)
                    }
                    placeholder={t('name, email, tenant, project')}
                    value={search}
                  />
                </div>
              </div>

              <div className="min-w-[12rem]">
                <SelectField<'all' | 'operator' | 'portal'>
                  label={t('User type')}
                  labelVisibility="sr-only"
                  onValueChange={setRoleFilter}
                  options={[
                    { label: t('All users'), value: 'all' },
                    { label: t('Operators'), value: 'operator' },
                    { label: t('Portal users'), value: 'portal' },
                  ]}
                  value={roleFilter}
                />
              </div>

              <div className="min-w-[12rem]">
                <SelectField<'all' | 'active' | 'disabled'>
                  label={t('Status')}
                  labelVisibility="sr-only"
                  onValueChange={setStatusFilter}
                  options={[
                    { label: t('All statuses'), value: 'all' },
                    { label: t('Active'), value: 'active' },
                    { label: t('Disabled'), value: 'disabled' },
                  ]}
                  value={statusFilter}
                />
              </div>

              <div className="ml-auto flex flex-wrap items-center self-center gap-2">
                <div className="hidden text-sm text-[var(--sdk-color-text-secondary)] xl:block">
                  {t('{count} visible', { count: formatNumber(filteredUsers.length) })}
                  {' | '}
                  {t('{count} active', { count: formatNumber(activeCount) })}
                </div>
                <Button onClick={() => openOperatorDialog()} type="button" variant="primary">
                  <Plus className="w-4 h-4" />
                  {t('New Operator')}
                </Button>
                <Button onClick={() => openPortalDialog()} type="button" variant="outline">
                  <Plus className="w-4 h-4" />
                  {t('New Portal User')}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <div className="min-h-0 flex-1">
          <UsersRegistrySection
            activeCount={activeCount}
            columns={columns}
            filteredUsers={filteredUsers}
            onOpenOperatorDialog={openOperatorDialog}
            onOpenPortalDialog={openPortalDialog}
            onRequestDelete={(user) =>
              setPendingDelete({
                kind: user.role,
                user,
              })
            }
            onSelectUser={openDetailDrawer}
            onToggleOperatorUser={onToggleOperatorUser}
            onTogglePortalUser={onTogglePortalUser}
            operatorCount={operatorCount}
            portalCount={portalCount}
            selectedUserId={selectedUserId}
            sessionUserId={snapshot.sessionUser?.id ?? null}
          />
        </div>
      </div>

      <UsersDetailDrawer
        isProtected={
          selectedUser ? isProtectedUser(selectedUser, snapshot.sessionUser?.id ?? null) : false
        }
        onDelete={() => {
          if (!selectedUser) {
            return;
          }
          setPendingDelete({
            kind: selectedUser.role,
            user: selectedUser,
          });
        }}
        onEdit={() => {
          if (!selectedUser) {
            return;
          }
          setIsDetailDrawerOpen(false);
          if (selectedUser.role === 'operator') {
            openOperatorDialog(selectedUser);
            return;
          }
          openPortalDialog(selectedUser);
        }}
        onOpenChange={handleDetailDrawerOpenChange}
        onToggleStatus={() => {
          if (!selectedUser) {
            return;
          }
          if (selectedUser.role === 'operator') {
            void onToggleOperatorUser(selectedUser.id, !selectedUser.active);
            return;
          }
          void onTogglePortalUser(selectedUser.id, !selectedUser.active);
        }}
        open={isDetailDrawerOpen}
        user={selectedUser}
        userBilling={selectedUserBilling}
        userProject={selectedUserProject}
        userTraffic={selectedUserTraffic}
      />

      <OperatorUserDialog
        draft={operatorDraft}
        onOpenChange={handleOperatorDialogOpenChange}
        onSubmit={(event) => void handleOperatorSubmit(event)}
        open={isOperatorDialogOpen}
        setDraft={setOperatorDraft}
      />

      <PortalUserDialog
        availableProjects={availableProjects}
        draft={portalDraft}
        onOpenChange={handlePortalDialogOpenChange}
        onSubmit={(event) => void handlePortalSubmit(event)}
        open={isPortalDialogOpen}
        selectedProject={selectedProject}
        selectedProjectBilling={selectedProjectBilling}
        selectedProjectTokens={selectedProjectTokens}
        selectedProjectTraffic={selectedProjectTraffic}
        setDraft={setPortalDraft}
        snapshot={snapshot}
      />

      <ConfirmActionDialog
        confirmLabel={t('Delete now')}
        description={
          pendingDelete
            ? t(
                'Remove {name} ({email}) from the directory. This action cannot be undone from this console.',
                {
                  email: pendingDelete.user.email,
                  name: pendingDelete.user.display_name,
                },
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
        title={
          pendingDelete?.kind === 'operator'
            ? t('Delete operator account')
            : t('Delete portal account')
        }
      />
    </>
  );
}
