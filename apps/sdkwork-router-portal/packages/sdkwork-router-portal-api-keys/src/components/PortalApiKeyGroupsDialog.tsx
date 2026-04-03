import { useEffect, useMemo, useState, type FormEvent } from 'react';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import { Badge } from 'sdkwork-router-portal-commons/framework/display';
import {
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
} from 'sdkwork-router-portal-commons/framework/entry';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import {
  SearchInput,
  SettingsField,
} from 'sdkwork-router-portal-commons/framework/form';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from 'sdkwork-router-portal-commons/framework/overlays';
import type {
  ApiKeyGroupRecord,
  RoutingProfileRecord,
} from 'sdkwork-router-portal-types';

const DEFAULT_ACCOUNTING_MODE_VALUE = '__portal-api-key-group-no-accounting__';
const DEFAULT_ROUTING_PROFILE_VALUE = '__portal-api-key-group-no-routing-profile__';
const DEFAULT_ENVIRONMENTS = ['live', 'staging', 'test'];

type ApiKeyGroupDraft = {
  environment: string;
  name: string;
  slug: string;
  description: string;
  color: string;
  default_capability_scope: string;
  default_accounting_mode: string;
  default_routing_profile_id: string;
};

type PortalApiKeyGroupsDialogProps = {
  groups: ApiKeyGroupRecord[];
  loadingRoutingProfiles: boolean;
  onDeleteGroup: (groupId: string) => Promise<void>;
  onOpenChange: (open: boolean) => void;
  onSaveGroup: (input: {
    group_id?: string;
    environment: string;
    name: string;
    slug?: string | null;
    description?: string | null;
    color?: string | null;
    default_capability_scope?: string | null;
    default_accounting_mode?: string | null;
    default_routing_profile_id?: string | null;
  }) => Promise<void>;
  onToggleGroup: (groupId: string, active: boolean) => Promise<void>;
  open: boolean;
  profileStatus: string;
  routingProfiles: RoutingProfileRecord[];
};

function createGroupDraft(overrides: Partial<ApiKeyGroupDraft> = {}): ApiKeyGroupDraft {
  return {
    environment: 'live',
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
  return createGroupDraft({
    environment: group.environment,
    name: group.name,
    slug: group.slug,
    description: group.description ?? '',
    color: group.color ?? '',
    default_capability_scope: group.default_capability_scope ?? '',
    default_accounting_mode: group.default_accounting_mode ?? '',
    default_routing_profile_id: group.default_routing_profile_id ?? '',
  });
}

function sortGroups(groups: ApiKeyGroupRecord[]): ApiKeyGroupRecord[] {
  return [...groups].sort((left, right) =>
    Number(right.active) - Number(left.active)
    || right.updated_at_ms - left.updated_at_ms
    || left.environment.localeCompare(right.environment)
    || left.name.localeCompare(right.name)
    || left.group_id.localeCompare(right.group_id)
  );
}

function toOptionalValue(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length ? trimmed : null;
}

function buildEnvironmentOptions(
  groups: ApiKeyGroupRecord[],
  currentEnvironment: string,
): string[] {
  const values = new Set<string>([currentEnvironment, ...DEFAULT_ENVIRONMENTS]);

  for (const group of groups) {
    if (group.environment.trim()) {
      values.add(group.environment);
    }
  }

  return [...values].sort((left, right) => {
    const leftIndex = DEFAULT_ENVIRONMENTS.indexOf(left);
    const rightIndex = DEFAULT_ENVIRONMENTS.indexOf(right);
    const resolvedLeft = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
    const resolvedRight = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
    return resolvedLeft - resolvedRight || left.localeCompare(right);
  });
}

export function PortalApiKeyGroupsDialog({
  groups,
  loadingRoutingProfiles,
  onDeleteGroup,
  onOpenChange,
  onSaveGroup,
  onToggleGroup,
  open,
  profileStatus,
  routingProfiles,
}: PortalApiKeyGroupsDialogProps) {
  const { t } = usePortalI18n();
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [draft, setDraft] = useState<ApiKeyGroupDraft>(createGroupDraft);
  const [statusMessage, setStatusMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const [pendingDeleteGroupId, setPendingDeleteGroupId] = useState<string | null>(null);

  useEffect(() => {
    if (!open) {
      setPendingDeleteGroupId(null);
      return;
    }

    setSearchQuery('');
    setSelectedGroupId(null);
    setDraft(createGroupDraft());
    setStatusMessage('');
    setPendingDeleteGroupId(null);
  }, [open]);

  const selectedGroup = useMemo(
    () => groups.find((group) => group.group_id === selectedGroupId) ?? null,
    [groups, selectedGroupId],
  );
  const pendingDeleteGroup = useMemo(
    () => groups.find((group) => group.group_id === pendingDeleteGroupId) ?? null,
    [groups, pendingDeleteGroupId],
  );
  const filteredGroups = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();

    return sortGroups(groups).filter((group) => {
      if (!normalizedQuery) {
        return true;
      }

      return [
        group.name,
        group.slug,
        group.environment,
        group.group_id,
        group.description ?? '',
        group.default_capability_scope ?? '',
        group.default_accounting_mode ?? '',
        group.default_routing_profile_id ?? '',
      ]
        .join(' ')
        .toLowerCase()
        .includes(normalizedQuery);
    });
  }, [groups, searchQuery]);
  const routingProfileOptions = useMemo(
    () =>
      [...routingProfiles]
        .filter(
          (profile) =>
            profile.active || profile.profile_id === draft.default_routing_profile_id,
        )
        .sort((left, right) =>
          Number(right.active) - Number(left.active)
          || right.updated_at_ms - left.updated_at_ms
          || left.name.localeCompare(right.name)
          || left.profile_id.localeCompare(right.profile_id),
        ),
    [draft.default_routing_profile_id, routingProfiles],
  );
  const environmentOptions = useMemo(
    () => buildEnvironmentOptions(groups, draft.environment),
    [draft.environment, groups],
  );
  const feedbackMessages = [profileStatus, statusMessage].filter(Boolean);

  useEffect(() => {
    if (!selectedGroupId) {
      return;
    }

    if (!selectedGroup) {
      setSelectedGroupId(null);
      setDraft(createGroupDraft());
      return;
    }

    setDraft(draftFromGroup(selectedGroup));
  }, [selectedGroup, selectedGroupId]);

  function handleStartCreateGroup() {
    setSelectedGroupId(null);
    setDraft(createGroupDraft({ environment: draft.environment }));
    setStatusMessage('');
    setPendingDeleteGroupId(null);
  }

  function handleSelectGroup(group: ApiKeyGroupRecord) {
    setSelectedGroupId(group.group_id);
    setDraft(draftFromGroup(group));
    setStatusMessage('');
    setPendingDeleteGroupId(null);
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!draft.name.trim()) {
      setStatusMessage(t('Name is required before this group can be saved.'));
      return;
    }

    setBusy(true);

    try {
      await onSaveGroup({
        group_id: selectedGroupId ?? undefined,
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
          ? t('Group updated. Review the refreshed policy inventory.')
          : t('Group created. Review the refreshed policy inventory.'),
      );

      if (!selectedGroupId) {
        setDraft(createGroupDraft({ environment: draft.environment }));
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
      await onToggleGroup(selectedGroup.group_id, !selectedGroup.active);
      setStatusMessage(
        selectedGroup.active
          ? t('Group disabled. New key assignments now require a different group.')
          : t('Group enabled. Keys in this workspace can bind to it again.'),
      );
    } catch (error) {
      setStatusMessage(
        error instanceof Error ? error.message : t('Failed to update API key group status.'),
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
      await onDeleteGroup(pendingDeleteGroup.group_id);
      if (selectedGroupId === pendingDeleteGroup.group_id) {
        setSelectedGroupId(null);
        setDraft(createGroupDraft({ environment: draft.environment }));
      }
      setPendingDeleteGroupId(null);
      setStatusMessage(t('Group deleted. Review the refreshed policy inventory.'));
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
        <DialogContent className="w-[min(96vw,78rem)]">
          <DialogHeader>
            <DialogTitle>{t('API key groups')}</DialogTitle>
            <DialogDescription>
              {t(
                'Manage reusable defaults for environment-scoped keys, routing profile overrides, and accounting posture inside this workspace.',
              )}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-6 lg:grid-cols-[22rem,minmax(0,1fr)]">
            <div className="space-y-4">
              <section className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/50">
                <div className="space-y-3">
                  <SearchInput
                    onChange={(event) => setSearchQuery(event.target.value)}
                    placeholder={t('Search groups')}
                    value={searchQuery}
                  />
                  <Button onClick={handleStartCreateGroup} type="button" variant="secondary">
                    {t('Create group')}
                  </Button>
                </div>
              </section>

              <div className="max-h-[60vh] space-y-3 overflow-y-auto pr-1">
                {filteredGroups.length ? (
                  filteredGroups.map((group) => (
                    <article
                      className={
                        selectedGroupId === group.group_id
                          ? 'rounded-[24px] border border-primary-500 bg-white p-4 shadow-sm dark:border-primary-500 dark:bg-zinc-950'
                          : 'rounded-[24px] border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950'
                      }
                      key={group.group_id}
                    >
                      <div className="flex flex-wrap items-start justify-between gap-3">
                        <div className="space-y-1">
                          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {group.name}
                          </div>
                          <div className="text-xs text-zinc-500 dark:text-zinc-400">
                            {group.slug}
                          </div>
                        </div>
                        <Badge variant={group.active ? 'success' : 'secondary'}>
                          {group.active ? t('Active') : t('Inactive')}
                        </Badge>
                      </div>

                      <div className="mt-3 flex flex-wrap gap-2 text-xs text-zinc-500 dark:text-zinc-400">
                        <Badge variant="outline">{group.environment}</Badge>
                        {group.default_accounting_mode ? (
                          <Badge variant="outline">{group.default_accounting_mode}</Badge>
                        ) : null}
                      </div>

                      <div className="mt-3 space-y-1 text-sm text-zinc-600 dark:text-zinc-300">
                        <div>
                          {group.default_routing_profile_id
                            ? `${t('Routing profile')}: ${group.default_routing_profile_id}`
                            : t('No routing profile override')}
                        </div>
                        <div>{group.default_capability_scope ?? t('Default scope is not set.')}</div>
                      </div>

                      <div className="mt-4 flex flex-wrap gap-2">
                        <Button
                          onClick={() => handleSelectGroup(group)}
                          type="button"
                          variant={selectedGroupId === group.group_id ? 'primary' : 'secondary'}
                        >
                          {t('Open')}
                        </Button>
                        <Button
                          onClick={() => setPendingDeleteGroupId(group.group_id)}
                          type="button"
                          variant="ghost"
                        >
                          {t('Delete group')}
                        </Button>
                      </div>
                    </article>
                  ))
                ) : (
                  <div className="rounded-[24px] border border-dashed border-zinc-300 bg-zinc-50/80 p-4 dark:border-zinc-700 dark:bg-zinc-900/40">
                    <EmptyState
                      description={t(
                        'Create the first reusable group to share routing and accounting defaults across keys.',
                      )}
                      title={t('No API key groups match the current filter.')}
                    />
                  </div>
                )}
              </div>
            </div>

            <form className="space-y-6" onSubmit={(event) => void handleSubmit(event)}>
              {feedbackMessages.map((message, index) => (
                <div
                  className="rounded-[20px] border border-zinc-200 bg-zinc-50/85 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300"
                  key={`${index}-${message}`}
                  role="status"
                >
                  {message}
                </div>
              ))}

              <section className="grid gap-4 md:grid-cols-2">
                <SettingsField label={t('Name')} layout="vertical">
                  <Input
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        name: event.target.value,
                      }))
                    }
                    required
                    value={draft.name}
                  />
                </SettingsField>

                <SettingsField label={t('Environment')} layout="vertical">
                  <Select
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        environment: value,
                      }))
                    }
                    value={draft.environment}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('Environment')} />
                    </SelectTrigger>
                    <SelectContent>
                      {environmentOptions.map((option) => (
                        <SelectItem key={option} value={option}>
                          {option}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </SettingsField>

                <SettingsField label={t('Slug')} layout="vertical">
                  <Input
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        slug: event.target.value,
                      }))
                    }
                    value={draft.slug}
                  />
                </SettingsField>

                <SettingsField label={t('Color')} layout="vertical">
                  <Input
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        color: event.target.value,
                      }))
                    }
                    value={draft.color}
                  />
                </SettingsField>
              </section>

              <section className="grid gap-4 md:grid-cols-2">
                <SettingsField label={t('Default scope')} layout="vertical">
                  <Input
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        default_capability_scope: event.target.value,
                      }))
                    }
                    value={draft.default_capability_scope}
                  />
                </SettingsField>

                <SettingsField label={t('Accounting mode')} layout="vertical">
                  <Select
                    onValueChange={(value) =>
                      setDraft((current) => ({
                        ...current,
                        default_accounting_mode:
                          value === DEFAULT_ACCOUNTING_MODE_VALUE ? '' : value,
                      }))
                    }
                    value={draft.default_accounting_mode || DEFAULT_ACCOUNTING_MODE_VALUE}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('Accounting mode')} />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value={DEFAULT_ACCOUNTING_MODE_VALUE}>
                        {t('No accounting override')}
                      </SelectItem>
                      <SelectItem value="platform_credit">{t('Platform credit')}</SelectItem>
                      <SelectItem value="byok">{t('BYOK')}</SelectItem>
                      <SelectItem value="passthrough">{t('Passthrough')}</SelectItem>
                    </SelectContent>
                  </Select>
                </SettingsField>

                <SettingsField label={t('Routing profile')} layout="vertical">
                  <div className="space-y-2">
                    <Select
                      disabled={loadingRoutingProfiles}
                      onValueChange={(value) =>
                        setDraft((current) => ({
                          ...current,
                          default_routing_profile_id:
                            value === DEFAULT_ROUTING_PROFILE_VALUE ? '' : value,
                        }))
                      }
                      value={draft.default_routing_profile_id || DEFAULT_ROUTING_PROFILE_VALUE}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('Routing profile')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value={DEFAULT_ROUTING_PROFILE_VALUE}>
                          {t('No routing profile override')}
                        </SelectItem>
                        {routingProfileOptions.map((profile) => (
                          <SelectItem key={profile.profile_id} value={profile.profile_id}>
                            {profile.active
                              ? `${profile.name} (${profile.slug})`
                              : `${profile.name} (${profile.slug}) - ${t('Inactive')}`}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    {loadingRoutingProfiles ? (
                      <div className="text-xs text-zinc-500 dark:text-zinc-400">
                        {t('Loading routing profiles...')}
                      </div>
                    ) : null}
                  </div>
                </SettingsField>

                <SettingsField label={t('Description')} layout="vertical">
                  <Textarea
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        description: event.target.value,
                      }))
                    }
                    rows={5}
                    value={draft.description}
                  />
                </SettingsField>
              </section>

              <DialogFooter className="justify-between">
                <div className="flex flex-wrap gap-2">
                  <Button onClick={() => onOpenChange(false)} type="button" variant="ghost">
                    {t('Close')}
                  </Button>
                  {selectedGroup ? (
                    <Button onClick={handleStartCreateGroup} type="button" variant="secondary">
                      {t('Create group')}
                    </Button>
                  ) : null}
                </div>

                <div className="flex flex-wrap gap-2">
                  {selectedGroup ? (
                    <Button
                      onClick={() => void handleToggleSelectedGroup()}
                      type="button"
                      variant="secondary"
                    >
                      {selectedGroup.active ? t('Disable group') : t('Enable group')}
                    </Button>
                  ) : null}
                  {selectedGroup ? (
                    <Button
                      onClick={() => setPendingDeleteGroupId(selectedGroup.group_id)}
                      type="button"
                      variant="ghost"
                    >
                      {t('Delete group')}
                    </Button>
                  ) : null}
                  <Button disabled={busy} type="submit" variant="primary">
                    {selectedGroup ? t('Save group') : t('Create group')}
                  </Button>
                </div>
              </DialogFooter>
            </form>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog
        open={Boolean(pendingDeleteGroup)}
        onOpenChange={(nextOpen) => {
          if (!nextOpen) {
            setPendingDeleteGroupId(null);
          }
        }}
      >
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{t('Delete group')}</DialogTitle>
            <DialogDescription>
              {pendingDeleteGroup
                ? t('Delete this group from the workspace?')
                : ''}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              onClick={() => setPendingDeleteGroupId(null)}
              type="button"
              variant="ghost"
            >
              {t('Close')}
            </Button>
            <Button
              disabled={busy}
              onClick={() => void handleConfirmDelete()}
              type="button"
              variant="primary"
            >
              {t('Delete group')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
