import { useEffect, useMemo, useState, type FormEvent } from 'react';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import { Badge } from 'sdkwork-router-portal-commons/framework/display';
import {
  Checkbox,
  Input,
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
  RoutingProfileRecord,
} from 'sdkwork-router-portal-types';

import { buildRoutingStrategyLabel } from '../services';

type RoutingProfileDraft = {
  name: string;
  slug: string;
  description: string;
  active: boolean;
};

type CurrentRoutingPosture = {
  strategy: string;
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy: boolean;
  preferred_region?: string | null;
};

type PortalRoutingProfilesDialogProps = {
  currentPosture: CurrentRoutingPosture;
  loadingProfiles: boolean;
  onApplyProfile: (profile: RoutingProfileRecord) => void;
  onCreateProfile: (input: {
    name: string;
    slug?: string | null;
    description?: string | null;
    active?: boolean;
    strategy?: string;
    ordered_provider_ids?: string[];
    default_provider_id?: string | null;
    max_cost?: number | null;
    max_latency_ms?: number | null;
    require_healthy?: boolean;
    preferred_region?: string | null;
  }) => Promise<void>;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  profileStatus: string;
  profiles: RoutingProfileRecord[];
};

function createDraft(): RoutingProfileDraft {
  return {
    name: '',
    slug: '',
    description: '',
    active: true,
  };
}

function sortProfiles(profiles: RoutingProfileRecord[]): RoutingProfileRecord[] {
  return [...profiles].sort((left, right) =>
    Number(right.active) - Number(left.active)
    || right.updated_at_ms - left.updated_at_ms
    || left.name.localeCompare(right.name)
    || left.profile_id.localeCompare(right.profile_id),
  );
}

function toOptionalValue(value: string): string | null {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

export function PortalRoutingProfilesDialog({
  currentPosture,
  loadingProfiles,
  onApplyProfile,
  onCreateProfile,
  onOpenChange,
  open,
  profileStatus,
  profiles,
}: PortalRoutingProfilesDialogProps) {
  const { t } = usePortalI18n();
  const [searchQuery, setSearchQuery] = useState('');
  const [draft, setDraft] = useState<RoutingProfileDraft>(createDraft);
  const [busy, setBusy] = useState(false);
  const [localStatus, setLocalStatus] = useState('');

  useEffect(() => {
    if (!open) {
      return;
    }

    setSearchQuery('');
    setDraft(createDraft());
    setBusy(false);
    setLocalStatus('');
  }, [open]);

  const filteredProfiles = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();

    return sortProfiles(profiles).filter((profile) => {
      if (!normalizedQuery) {
        return true;
      }

      return [
        profile.name,
        profile.slug,
        profile.profile_id,
        profile.description ?? '',
        profile.strategy,
        profile.default_provider_id ?? '',
        profile.preferred_region ?? '',
        ...profile.ordered_provider_ids,
      ]
        .join(' ')
        .toLowerCase()
        .includes(normalizedQuery);
    });
  }, [profiles, searchQuery]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!draft.name.trim()) {
      setLocalStatus(t('Name is required before this profile can be saved.'));
      return;
    }

    setBusy(true);
    setLocalStatus('');

    try {
      await onCreateProfile({
        name: draft.name.trim(),
        slug: toOptionalValue(draft.slug),
        description: toOptionalValue(draft.description),
        active: draft.active,
        strategy: currentPosture.strategy,
        ordered_provider_ids: currentPosture.ordered_provider_ids,
        default_provider_id: currentPosture.default_provider_id ?? null,
        max_cost: currentPosture.max_cost ?? null,
        max_latency_ms: currentPosture.max_latency_ms ?? null,
        require_healthy: currentPosture.require_healthy,
        preferred_region: currentPosture.preferred_region ?? null,
      });
      setDraft(createDraft());
    } catch (error) {
      setLocalStatus(
        error instanceof Error ? error.message : t('Failed to save routing profile.'),
      );
    } finally {
      setBusy(false);
    }
  }

  const feedbackMessages = [profileStatus, localStatus].filter(Boolean);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(96vw,84rem)]">
        <DialogHeader>
          <DialogTitle>{t('Routing profiles')}</DialogTitle>
          <DialogDescription>
            {t(
              'Review reusable routing profiles for this workspace and save the current routing posture as a reusable profile for API key groups.',
            )}
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-6 lg:grid-cols-[22rem,minmax(0,1fr)]">
          <div className="space-y-4">
            <SearchInput
              onChange={(event) => setSearchQuery(event.target.value)}
              placeholder={t('Search routing profiles')}
              value={searchQuery}
            />

            {loadingProfiles ? (
              <div className="rounded-[24px] border border-zinc-200 bg-zinc-50/80 px-4 py-5 text-sm text-zinc-500 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300">
                {t('Loading routing profiles...')}
              </div>
            ) : filteredProfiles.length ? (
              <div className="max-h-[60vh] space-y-3 overflow-y-auto pr-1">
                {filteredProfiles.map((profile) => (
                  <article
                    key={profile.profile_id}
                    className="rounded-[24px] border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/70"
                  >
                    <div className="flex flex-wrap items-start justify-between gap-3">
                      <div className="space-y-1">
                        <strong className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                          {profile.name}
                        </strong>
                        <p className="text-xs text-zinc-500 dark:text-zinc-400">
                          {profile.slug}
                        </p>
                      </div>
                      <Badge variant={profile.active ? 'success' : 'secondary'}>
                        {profile.active ? t('Active') : t('Inactive')}
                      </Badge>
                    </div>

                    <div className="mt-3 space-y-1 text-sm text-zinc-500 dark:text-zinc-400">
                      <div>{buildRoutingStrategyLabel(profile.strategy)}</div>
                      <div>
                        {t('Default provider')}: {profile.default_provider_id ?? t('Auto fallback')}
                      </div>
                      <div>
                        {t('Preferred region')}: {profile.preferred_region ?? t('Auto')}
                      </div>
                    </div>

                    {profile.description ? (
                      <p className="mt-3 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                        {profile.description}
                      </p>
                    ) : null}

                    <div className="mt-4 flex flex-wrap gap-2">
                      <Badge variant="outline">
                        {`${profile.ordered_provider_ids.length} ${t('Provider roster')}`}
                      </Badge>
                      <Button onClick={() => onApplyProfile(profile)} variant="secondary">
                        {t('Use as posture')}
                      </Button>
                    </div>
                  </article>
                ))}
              </div>
            ) : (
              <EmptyState
                description={t(
                  'Save the current routing posture as a reusable profile, then bind it to API key groups.',
                )}
                title={t('No routing profiles yet')}
              />
            )}
          </div>

          <form className="space-y-5" onSubmit={(event) => void handleSubmit(event)}>
            {feedbackMessages.length ? (
              <div className="space-y-2">
                {feedbackMessages.map((message) => (
                  <div
                    key={message}
                    className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300"
                  >
                    {message}
                  </div>
                ))}
              </div>
            ) : null}

            <section className="space-y-4 rounded-[24px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
              <div className="space-y-1">
                <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Current posture')}
                </strong>
                <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(
                    'The current routing strategy, provider order, and guardrails will be captured into a reusable profile for this workspace.',
                  )}
                </p>
              </div>

              <div className="grid gap-3 md:grid-cols-2">
                <div className="rounded-[18px] border border-zinc-200 bg-white/90 px-4 py-3 text-sm dark:border-zinc-800 dark:bg-zinc-950/70">
                  <div className="text-xs uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t('Strategy')}
                  </div>
                  <div className="mt-1 font-medium text-zinc-950 dark:text-zinc-50">
                    {buildRoutingStrategyLabel(currentPosture.strategy)}
                  </div>
                </div>
                <div className="rounded-[18px] border border-zinc-200 bg-white/90 px-4 py-3 text-sm dark:border-zinc-800 dark:bg-zinc-950/70">
                  <div className="text-xs uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t('Default provider')}
                  </div>
                  <div className="mt-1 font-medium text-zinc-950 dark:text-zinc-50">
                    {currentPosture.default_provider_id ?? t('Auto fallback')}
                  </div>
                </div>
                <div className="rounded-[18px] border border-zinc-200 bg-white/90 px-4 py-3 text-sm dark:border-zinc-800 dark:bg-zinc-950/70">
                  <div className="text-xs uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t('Preferred region')}
                  </div>
                  <div className="mt-1 font-medium text-zinc-950 dark:text-zinc-50">
                    {currentPosture.preferred_region ?? t('Auto')}
                  </div>
                </div>
                <div className="rounded-[18px] border border-zinc-200 bg-white/90 px-4 py-3 text-sm dark:border-zinc-800 dark:bg-zinc-950/70">
                  <div className="text-xs uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                    {t('Require healthy providers')}
                  </div>
                  <div className="mt-1 font-medium text-zinc-950 dark:text-zinc-50">
                    {currentPosture.require_healthy ? t('Enabled') : t('Disabled')}
                  </div>
                </div>
              </div>
            </section>

            <section className="space-y-4 rounded-[24px] border border-zinc-200 bg-white/90 p-5 dark:border-zinc-800 dark:bg-zinc-950/70">
              <div className="space-y-1">
                <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Save current posture')}
                </strong>
                <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t(
                    'Create a reusable routing profile from the current project posture without opening the admin control plane.',
                  )}
                </p>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <SettingsField label={t('Name')} layout="vertical">
                  <Input
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        name: event.target.value,
                      }))
                    }
                    placeholder={t('Example: Balanced production posture')}
                    value={draft.name}
                  />
                </SettingsField>

                <SettingsField label={t('Slug')} layout="vertical">
                  <Input
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        slug: event.target.value,
                      }))
                    }
                    placeholder={t('Example: Balanced production posture')}
                    value={draft.slug}
                  />
                </SettingsField>

                <SettingsField className="md:col-span-2" label={t('Description')} layout="vertical">
                  <Textarea
                    onChange={(event) =>
                      setDraft((current) => ({
                        ...current,
                        description: event.target.value,
                      }))
                    }
                    rows={4}
                    value={draft.description}
                  />
                </SettingsField>
              </div>

              <label className="flex items-start gap-3 rounded-[20px] border border-zinc-200 bg-zinc-50/80 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/60">
                <Checkbox
                  checked={draft.active}
                  onCheckedChange={(nextChecked) =>
                    setDraft((current) => ({
                      ...current,
                      active: nextChecked === true,
                    }))
                  }
                />
                <div className="space-y-1">
                  <div className="font-medium text-zinc-950 dark:text-zinc-50">
                    {t('Active')}
                  </div>
                  <div className="text-sm text-zinc-600 dark:text-zinc-300">
                    {t('Keep the new profile immediately selectable by API key groups after creation.')}
                  </div>
                </div>
              </label>
            </section>

            <DialogFooter>
              <Button onClick={() => onOpenChange(false)} variant="ghost">
                {t('Close')}
              </Button>
              <Button disabled={busy} type="submit" variant="primary">
                {busy ? t('Saving...') : t('Save as profile')}
              </Button>
            </DialogFooter>
          </form>
        </div>
      </DialogContent>
    </Dialog>
  );
}
