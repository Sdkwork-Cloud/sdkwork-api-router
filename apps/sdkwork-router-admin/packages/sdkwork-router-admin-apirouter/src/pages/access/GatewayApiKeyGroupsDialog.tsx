import { useEffect, useMemo, useState, type ChangeEvent, type FormEvent } from 'react';
import {
  Button,
  Card,
  CardContent,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  FormActions,
  FormGrid,
  FormSection,
  Input,
  StatusBadge,
  Textarea,
} from '@sdkwork/ui-pc-react';
import { Search } from 'lucide-react';
import { useAdminI18n } from 'sdkwork-router-admin-core';
import type { AdminWorkspaceSnapshot, ApiKeyGroupRecord } from 'sdkwork-router-admin-types';

import { ConfirmActionDialog, DialogField, SelectField } from '../shared';

type ApiKeyGroupDraft = {
  tenant_id: string;
  project_id: string;
  environment: string;
  name: string;
  slug: string;
  description: string;
  color: string;
  default_capability_scope: string;
  default_accounting_mode: string;
  default_routing_profile_id: string;
};

type GatewayApiKeyGroupsDialogProps = {
  onDeleteApiKeyGroup: (groupId: string) => Promise<void>;
  onOpenChange: (open: boolean) => void;
  onSaveApiKeyGroup: (input: {
    group_id?: string;
    tenant_id: string;
    project_id: string;
    environment: string;
    name: string;
    slug?: string | null;
    description?: string | null;
    color?: string | null;
    default_capability_scope?: string | null;
    default_accounting_mode?: string | null;
    default_routing_profile_id?: string | null;
  }) => Promise<void>;
  onToggleApiKeyGroup: (groupId: string, active: boolean) => Promise<void>;
  open: boolean;
  preferredScope?: {
    tenant_id: string;
    project_id: string;
    environment: string;
  };
  snapshot: AdminWorkspaceSnapshot;
};

function resolvePreferredScope(
  snapshot: AdminWorkspaceSnapshot,
  preferredScope?: GatewayApiKeyGroupsDialogProps['preferredScope'],
) {
  const fallbackTenantId =
    preferredScope?.tenant_id
    || snapshot.tenants[0]?.id
    || snapshot.projects[0]?.tenant_id
    || 'tenant_local_demo';
  const fallbackProjectId =
    preferredScope?.project_id
    || snapshot.projects.find((project) => project.tenant_id === fallbackTenantId)?.id
    || snapshot.projects[0]?.id
    || 'project_local_demo';

  return {
    tenant_id: fallbackTenantId,
    project_id: fallbackProjectId,
    environment: preferredScope?.environment || 'live',
  };
}

function createGroupDraft(
  scope: ReturnType<typeof resolvePreferredScope>,
  overrides: Partial<ApiKeyGroupDraft> = {},
): ApiKeyGroupDraft {
  return {
    tenant_id: scope.tenant_id,
    project_id: scope.project_id,
    environment: scope.environment,
    name: '',
    slug: '',
    description: '',
    color: '',
    default_capability_scope: '',
    default_accounting_mode: '',
    default_routing_profile_id: '',
    ...overrides,
  };
}

function draftFromGroup(group: ApiKeyGroupRecord): ApiKeyGroupDraft {
  return createGroupDraft(
    {
      tenant_id: group.tenant_id,
      project_id: group.project_id,
      environment: group.environment,
    },
    {
      name: group.name,
      slug: group.slug,
      description: group.description ?? '',
      color: group.color ?? '',
      default_capability_scope: group.default_capability_scope ?? '',
      default_accounting_mode: group.default_accounting_mode ?? '',
      default_routing_profile_id: group.default_routing_profile_id ?? '',
    },
  );
}

function toOptionalValue(value: string): string | undefined {
  const normalized = value.trim();
  return normalized ? normalized : undefined;
}

export function GatewayApiKeyGroupsDialog({
  onDeleteApiKeyGroup,
  onOpenChange,
  onSaveApiKeyGroup,
  onToggleApiKeyGroup,
  open,
  preferredScope,
  snapshot,
}: GatewayApiKeyGroupsDialogProps) {
  const { t } = useAdminI18n();
  const resolvedScope = useMemo(
    () => resolvePreferredScope(snapshot, preferredScope),
    [preferredScope, snapshot],
  );
  const [search, setSearch] = useState('');
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [draft, setDraft] = useState<ApiKeyGroupDraft>(() => createGroupDraft(resolvedScope));
  const [busy, setBusy] = useState(false);
  const [statusMessage, setStatusMessage] = useState('');
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null);

  useEffect(() => {
    if (!open) {
      setPendingDeleteId(null);
      return;
    }

    setSearch('');
    setSelectedGroupId(null);
    setDraft(createGroupDraft(resolvedScope));
    setStatusMessage('');
    setPendingDeleteId(null);
  }, [open]);

  const selectedGroup = useMemo(
    () => snapshot.apiKeyGroups.find((group) => group.group_id === selectedGroupId) ?? null,
    [selectedGroupId, snapshot.apiKeyGroups],
  );

  useEffect(() => {
    if (!selectedGroupId) {
      return;
    }

    if (!selectedGroup) {
      setSelectedGroupId(null);
      setDraft(createGroupDraft(resolvedScope));
      return;
    }

    setDraft(draftFromGroup(selectedGroup));
  }, [resolvedScope, selectedGroup, selectedGroupId]);

  const availableProjects = useMemo(
    () => snapshot.projects.filter((project) => project.tenant_id === draft.tenant_id),
    [draft.tenant_id, snapshot.projects],
  );
  const routingProfiles = useMemo(
    () => {
      const routingProfiles = snapshot.routingProfiles
        .filter(
          (profile) =>
            profile.tenant_id === draft.tenant_id && profile.project_id === draft.project_id,
        );
      const visibleRoutingProfiles = routingProfiles.filter(
        (profile) =>
          profile.active || profile.profile_id === draft.default_routing_profile_id,
      );

      return visibleRoutingProfiles
        .sort((left, right) => {
          if (left.active !== right.active) {
            return left.active ? -1 : 1;
          }
          return left.name.localeCompare(right.name);
        });
    },
    [
      draft.default_routing_profile_id,
      draft.project_id,
      draft.tenant_id,
      snapshot.routingProfiles,
    ],
  );

  const environmentOptions = useMemo(() => {
    const values = new Set<string>([
      draft.environment,
      'live',
      'staging',
      'test',
      'production',
    ]);
    for (const key of snapshot.apiKeys) {
      values.add(key.environment);
    }
    for (const group of snapshot.apiKeyGroups) {
      values.add(group.environment);
    }

    return [...values]
      .filter((value) => value.trim().length > 0)
      .map((value) => ({
        label: value,
        value,
      }));
  }, [draft.environment, snapshot.apiKeyGroups, snapshot.apiKeys]);

  const filteredGroups = useMemo(() => {
    const normalizedSearch = search.trim().toLowerCase();

    return [...snapshot.apiKeyGroups]
      .sort((left, right) => {
        if (left.active !== right.active) {
          return left.active ? -1 : 1;
        }
        return right.updated_at_ms - left.updated_at_ms;
      })
      .filter((group) => {
        if (!normalizedSearch) {
          return true;
        }

        return [
          group.name,
          group.slug,
          group.group_id,
          group.tenant_id,
          group.project_id,
          group.environment,
          group.description ?? '',
          group.default_capability_scope ?? '',
          group.default_accounting_mode ?? '',
          group.default_routing_profile_id ?? '',
        ]
          .join(' ')
          .toLowerCase()
          .includes(normalizedSearch);
      });
  }, [search, snapshot.apiKeyGroups]);

  const pendingDeleteGroup = useMemo(
    () => snapshot.apiKeyGroups.find((group) => group.group_id === pendingDeleteId) ?? null,
    [pendingDeleteId, snapshot.apiKeyGroups],
  );

  function handleStartCreateGroup() {
    setSelectedGroupId(null);
    setDraft(
      createGroupDraft({
        tenant_id: draft.tenant_id,
        project_id: draft.project_id,
        environment: draft.environment,
      }),
    );
    setStatusMessage('');
  }

  function handleSelectGroup(group: ApiKeyGroupRecord) {
    setSelectedGroupId(group.group_id);
    setDraft(draftFromGroup(group));
    setStatusMessage('');
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);

    try {
      await onSaveApiKeyGroup({
        group_id: selectedGroupId ?? undefined,
        tenant_id: draft.tenant_id.trim(),
        project_id: draft.project_id.trim(),
        environment: draft.environment.trim(),
        name: draft.name.trim(),
        slug: toOptionalValue(draft.slug),
        description: toOptionalValue(draft.description),
        color: toOptionalValue(draft.color),
        default_capability_scope: toOptionalValue(draft.default_capability_scope),
        default_accounting_mode: toOptionalValue(draft.default_accounting_mode),
        default_routing_profile_id: toOptionalValue(draft.default_routing_profile_id),
      });

      setStatusMessage(
        selectedGroupId
          ? t('Group updated. Review the refreshed policy state on the left.')
          : t('Group created. Review the refreshed policy state on the left.'),
      );

      if (!selectedGroupId) {
        setDraft(
          createGroupDraft({
            tenant_id: draft.tenant_id,
            project_id: draft.project_id,
            environment: draft.environment,
          }),
        );
      }
    } catch (error) {
      setStatusMessage(
        error instanceof Error ? error.message : t('Failed to save API key group.'),
      );
    } finally {
      setBusy(false);
    }
  }

  async function handleToggleSelectedGroup() {
    if (!selectedGroup) {
      return;
    }

    setBusy(true);

    try {
      await onToggleApiKeyGroup(selectedGroup.group_id, !selectedGroup.active);
      setStatusMessage(
        selectedGroup.active
          ? t('Group disabled. New key assignments now require a different policy group.')
          : t('Group enabled. Keys in this workspace scope can bind to it again.'),
      );
    } catch (error) {
      setStatusMessage(
        error instanceof Error
          ? error.message
          : t('Failed to update API key group status.'),
      );
    } finally {
      setBusy(false);
    }
  }

  async function handleConfirmDelete() {
    if (!pendingDeleteGroup) {
      return;
    }

    setBusy(true);

    try {
      await onDeleteApiKeyGroup(pendingDeleteGroup.group_id);
      if (selectedGroupId === pendingDeleteGroup.group_id) {
        setSelectedGroupId(null);
        setDraft(createGroupDraft(resolvedScope));
      }
      setStatusMessage(t('Group deleted. Review the refreshed policy inventory.'));
      setPendingDeleteId(null);
    } catch (error) {
      setStatusMessage(
        error instanceof Error ? error.message : t('Failed to delete API key group.'),
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="w-[min(96vw,82rem)]">
          <DialogHeader>
            <DialogTitle>{t('API key groups')}</DialogTitle>
            <DialogDescription>
              {t('Define reusable policy groups for workspace-scoped key issuance, routing posture, and accounting defaults.')}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-6 lg:grid-cols-[22rem,minmax(0,1fr)]">
            <div className="space-y-4">
              <Card>
                <CardContent className="space-y-3 p-4">
                  <div className="relative">
                    <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--sdk-color-text-muted)]" />
                    <Input
                      className="pl-9"
                      onChange={(event) => setSearch(event.target.value)}
                      placeholder={t('Search groups')}
                      value={search}
                    />
                  </div>
                  <Button onClick={handleStartCreateGroup} type="button" variant="outline">
                    {t('Create group')}
                  </Button>
                </CardContent>
              </Card>

              <div className="max-h-[60vh] space-y-3 overflow-y-auto pr-1">
                {filteredGroups.length ? (
                  filteredGroups.map((group) => (
                    <Card
                      className={
                        selectedGroupId === group.group_id
                          ? 'border-[var(--sdk-color-primary-500)] shadow-sm'
                          : undefined
                      }
                      key={group.group_id}
                    >
                      <CardContent className="space-y-3 p-4">
                        <div className="flex items-start justify-between gap-3">
                          <div className="space-y-1">
                            <div className="font-semibold text-[var(--sdk-color-text-primary)]">
                              {group.name}
                            </div>
                            <div className="text-xs text-[var(--sdk-color-text-secondary)]">
                              {group.slug}
                            </div>
                          </div>
                          <StatusBadge
                            label={group.active ? t('Active') : t('Inactive')}
                            showIcon
                            status={group.active ? 'active' : 'paused'}
                            variant={group.active ? 'success' : 'secondary'}
                          />
                        </div>

                        <div className="space-y-1 text-sm text-[var(--sdk-color-text-secondary)]">
                          <div>
                            {group.tenant_id} / {group.project_id}
                          </div>
                          <div>{group.environment}</div>
                          <div>{group.default_capability_scope ?? t('No default scope')}</div>
                        </div>

                        <div className="flex flex-wrap gap-2">
                          <Button
                            onClick={() => handleSelectGroup(group)}
                            size="sm"
                            type="button"
                            variant={selectedGroupId === group.group_id ? 'primary' : 'outline'}
                          >
                            {t('Open')}
                          </Button>
                          <Button
                            onClick={() => setPendingDeleteId(group.group_id)}
                            size="sm"
                            type="button"
                            variant="ghost"
                          >
                            {t('Delete')}
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  ))
                ) : (
                  <Card>
                    <CardContent className="space-y-1 p-4 text-sm text-[var(--sdk-color-text-secondary)]">
                      <div className="font-medium text-[var(--sdk-color-text-primary)]">
                        {t('No groups match the current filter')}
                      </div>
                      <div>{t('Broaden the query or create a new policy group for this workspace scope.')}</div>
                    </CardContent>
                  </Card>
                )}
              </div>
            </div>

            <form className="space-y-6" onSubmit={(event) => void handleSubmit(event)}>
              {statusMessage ? (
                <div className="rounded-[var(--sdk-radius-panel)] border border-[var(--sdk-color-border-default)] bg-[var(--sdk-color-surface-panel-muted)] px-4 py-3 text-sm text-[var(--sdk-color-text-secondary)]">
                  {statusMessage}
                </div>
              ) : null}

              <FormSection
                description={t('Pin the workspace boundary for this reusable key governance group.')}
                title={selectedGroup ? t('Edit group') : t('Create group')}
              >
                <FormGrid columns={2}>
                  {snapshot.tenants.length ? (
                    <SelectField
                      label={t('Tenant')}
                      onValueChange={(value) =>
                        setDraft((current) => ({
                          ...current,
                          tenant_id: value,
                          project_id:
                            snapshot.projects.find((project) => project.tenant_id === value)?.id
                            ?? current.project_id,
                        }))
                      }
                      options={snapshot.tenants.map((tenant) => ({
                        label: `${tenant.name} (${tenant.id})`,
                        value: tenant.id,
                      }))}
                      value={draft.tenant_id}
                    />
                  ) : (
                    <DialogField htmlFor="api-key-group-tenant" label={t('Tenant')}>
                      <Input
                        id="api-key-group-tenant"
                        onChange={(event: ChangeEvent<HTMLInputElement>) =>
                          setDraft((current) => ({
                            ...current,
                            tenant_id: event.target.value,
                          }))
                        }
                        required
                        value={draft.tenant_id}
                      />
                    </DialogField>
                  )}

                  {availableProjects.length ? (
                    <SelectField
                      label={t('Project')}
                      onValueChange={(value) =>
                        setDraft((current) => ({
                          ...current,
                          project_id: value,
                        }))
                      }
                      options={availableProjects.map((project) => ({
                        label: `${project.name} (${project.id})`,
                        value: project.id,
                      }))}
                      value={draft.project_id}
                    />
                  ) : (
                    <DialogField htmlFor="api-key-group-project" label={t('Project')}>
                      <Input
                        id="api-key-group-project"
                        onChange={(event: ChangeEvent<HTMLInputElement>) =>
                          setDraft((current) => ({
                            ...current,
                            project_id: event.target.value,
                          }))
                        }
                        required
                        value={draft.project_id}
                      />
                    </DialogField>
                  )}

                  <SelectField
                    label={t('Environment')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        environment: value,
                      }))
                    }
                    options={environmentOptions}
                    value={draft.environment}
                  />

                  <DialogField htmlFor="api-key-group-name" label={t('Name')}>
                    <Input
                      id="api-key-group-name"
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          name: event.target.value,
                        }))
                      }
                      required
                      value={draft.name}
                    />
                  </DialogField>

                  <DialogField
                    description={t('Leave empty to derive a slug from the group name.')}
                    htmlFor="api-key-group-slug"
                    label={t('Slug')}
                  >
                    <Input
                      id="api-key-group-slug"
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          slug: event.target.value,
                        }))
                      }
                      value={draft.slug}
                    />
                  </DialogField>

                  <DialogField htmlFor="api-key-group-color" label={t('Color')}>
                    <Input
                      id="api-key-group-color"
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          color: event.target.value,
                        }))
                      }
                      placeholder="#2563eb"
                      value={draft.color}
                    />
                  </DialogField>
                </FormGrid>
              </FormSection>

              <FormSection
                description={t('Define the defaults that each bound API key should inherit from this group policy.')}
                title={t('Policy defaults')}
              >
                <FormGrid columns={2}>
                  <DialogField
                    description={t('Examples: chat,responses or images,audio.')}
                    htmlFor="api-key-group-default-scope"
                    label={t('Default scope')}
                  >
                    <Input
                      id="api-key-group-default-scope"
                      onChange={(event: ChangeEvent<HTMLInputElement>) =>
                        setDraft((current) => ({
                          ...current,
                          default_capability_scope: event.target.value,
                        }))
                      }
                      value={draft.default_capability_scope}
                    />
                  </DialogField>

                  <SelectField
                    label={t('Accounting mode')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        default_accounting_mode: value,
                      }))
                    }
                    options={[
                      { label: t('No accounting override'), value: '' },
                      { label: 'platform_credit', value: 'platform_credit' },
                      { label: 'byok', value: 'byok' },
                      { label: 'passthrough', value: 'passthrough' },
                    ]}
                    value={draft.default_accounting_mode}
                  />

                  <SelectField
                    description={t('Bind to an active routing profile inside the same workspace scope when needed.')}
                    label={t('Routing profile')}
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        default_routing_profile_id: value,
                      }))
                    }
                    options={[
                      {
                        label: t('No routing profile override'),
                        value: '',
                      },
                      ...routingProfiles.map((profile) => ({
                        label: `${profile.name} (${profile.slug})${profile.active ? '' : ` • ${t('Inactive')}`}`,
                        value: profile.profile_id,
                      })),
                    ]}
                    value={draft.default_routing_profile_id}
                  />

                  <DialogField htmlFor="api-key-group-description" label={t('Description')}>
                    <Textarea
                      id="api-key-group-description"
                      onChange={(event: ChangeEvent<HTMLTextAreaElement>) =>
                        setDraft((current) => ({
                          ...current,
                          description: event.target.value,
                        }))
                      }
                      rows={4}
                      value={draft.description}
                    />
                  </DialogField>
                </FormGrid>
              </FormSection>

              <FormActions>
                <Button
                  onClick={() => onOpenChange(false)}
                  type="button"
                  variant="outline"
                >
                  {t('Close')}
                </Button>
                {selectedGroup ? (
                  <Button
                    onClick={handleStartCreateGroup}
                    type="button"
                    variant="outline"
                  >
                    {t('Create group')}
                  </Button>
                ) : null}
                {selectedGroup ? (
                  <Button
                    onClick={() => void handleToggleSelectedGroup()}
                    type="button"
                    variant="outline"
                  >
                    {selectedGroup.active ? t('Disable group') : t('Enable group')}
                  </Button>
                ) : null}
                {selectedGroup ? (
                  <Button
                    onClick={() => setPendingDeleteId(selectedGroup.group_id)}
                    type="button"
                    variant="danger"
                  >
                    {t('Delete group')}
                  </Button>
                ) : null}
                <Button disabled={busy} type="submit" variant="primary">
                  {selectedGroup ? t('Save group') : t('Create group')}
                </Button>
              </FormActions>
            </form>
          </div>
        </DialogContent>
      </Dialog>

      <ConfirmActionDialog
        confirmLabel={t('Delete group')}
        description={
          pendingDeleteGroup
            ? t(
                'Delete {name}. Keys already bound to this group will need a new policy assignment before future updates.',
                { name: pendingDeleteGroup.name },
              )
            : ''
        }
        onConfirm={() => void handleConfirmDelete()}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) {
            setPendingDeleteId(null);
          }
        }}
        open={Boolean(pendingDeleteGroup)}
        title={t('Delete API key group')}
      />
    </>
  );
}
