import { startTransition, useEffect, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';

import { usePortalI18n } from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  Badge,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons/framework/display';
import { Checkbox, Input } from 'sdkwork-router-portal-commons/framework/entry';
import { SettingsField } from 'sdkwork-router-portal-commons/framework/form';
import {
  Card,
  CardContent,
} from 'sdkwork-router-portal-commons/framework/layout';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from 'sdkwork-router-portal-commons/framework/overlays';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import {
  UserDetailCard,
  UserProfileFacts,
  UserSectionCard,
  UserSummaryCard,
} from '../components';
import { changePortalPassword } from '../repository';
import {
  buildPortalUserViewModel,
  buildUserWorkspaceSummary,
  passwordsMatch,
  readPortalUserPreferenceState,
  sanitizePhoneNumber,
  sanitizeWeChatId,
  writePortalUserPreferenceState,
} from '../services';
import type {
  PortalUserPageProps,
  PortalUserPreferenceState,
  PortalUserPrivacyPreferenceId,
} from '../types';

type UserCenterTab = 'profile' | 'privacy' | 'security';

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;

function resolveWorkspaceMemberLabel(active: boolean | undefined, t: TranslateFn) {
  return active ? t('Active') : t('Restricted');
}

export function PortalUserPage({ workspace, onNavigate }: PortalUserPageProps) {
  const { t } = usePortalI18n();
  const defaultStatus = t('User center details stay aligned with the active workspace session.');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [status, setStatus] = useState(defaultStatus);
  const [submitting, setSubmitting] = useState(false);
  const [passwordDialogOpen, setPasswordDialogOpen] = useState(false);
  const [phoneDialogOpen, setPhoneDialogOpen] = useState(false);
  const [wechatDialogOpen, setWechatDialogOpen] = useState(false);
  const [phoneNumber, setPhoneNumber] = useState('');
  const [wechatId, setWechatId] = useState('');
  const [activeTab, setActiveTab] = useState<UserCenterTab>('profile');
  const [preferences, setPreferences] = useState<PortalUserPreferenceState>(() =>
    readPortalUserPreferenceState(workspace),
  );

  useEffect(() => {
    const nextPreferences = readPortalUserPreferenceState(workspace);
    setPreferences(nextPreferences);
    setPhoneNumber(nextPreferences.phone_number);
    setWechatId(nextPreferences.wechat_id);
    setStatus(defaultStatus);
  }, [
    defaultStatus,
    workspace?.project.id,
    workspace?.tenant.id,
    workspace?.user.id,
  ]);

  useEffect(() => {
    writePortalUserPreferenceState(workspace, preferences);
  }, [
    preferences,
    workspace?.project.id,
    workspace?.tenant.id,
    workspace?.user.id,
  ]);

  const viewModel = buildPortalUserViewModel(
    workspace,
    preferences,
    newPassword,
    confirmPassword,
  );
  const workspaceSummary = buildUserWorkspaceSummary(workspace);
  const userStatusLabel = resolveWorkspaceMemberLabel(workspace?.user.active, t);

  async function handleSubmitPassword(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!passwordsMatch(newPassword, confirmPassword)) {
      setStatus(t('New password confirmation does not match.'));
      return;
    }

    if (!viewModel.can_submit_password) {
      setStatus(t('Password rotation does not yet satisfy the visible user security policy.'));
      return;
    }

    setSubmitting(true);
    setStatus(t('Updating password...'));

    try {
      await changePortalPassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setStatus(t('Password updated. Use the new password the next time you sign in.'));
      setPasswordDialogOpen(false);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  function handleSubmitPhone(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const nextPhoneNumber = sanitizePhoneNumber(phoneNumber);

    if (!nextPhoneNumber) {
      setStatus(t('Phone number is required before binding can be saved.'));
      return;
    }

    startTransition(() => {
      setPreferences((current) => ({
        ...current,
        phone_number: nextPhoneNumber,
      }));
    });
    setStatus(t('Phone binding updated in the current user center.'));
    setPhoneDialogOpen(false);
  }

  function handleRemovePhone() {
    startTransition(() => {
      setPreferences((current) => ({
        ...current,
        phone_number: '',
      }));
    });
    setPhoneNumber('');
    setStatus(t('Phone binding removed from the current user center.'));
    setPhoneDialogOpen(false);
  }

  function handleSubmitWeChat(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const nextWechatId = sanitizeWeChatId(wechatId);

    if (!nextWechatId) {
      setStatus(t('WeChat ID is required before binding can be saved.'));
      return;
    }

    startTransition(() => {
      setPreferences((current) => ({
        ...current,
        wechat_id: nextWechatId,
      }));
    });
    setStatus(t('WeChat binding updated in the current user center.'));
    setWechatDialogOpen(false);
  }

  function handleRemoveWeChat() {
    startTransition(() => {
      setPreferences((current) => ({
        ...current,
        wechat_id: '',
      }));
    });
    setWechatId('');
    setStatus(t('WeChat binding removed from the current user center.'));
    setWechatDialogOpen(false);
  }

  function handleTogglePrivacyPreference(preferenceId: PortalUserPrivacyPreferenceId) {
    startTransition(() => {
      setPreferences((current) => ({
        ...current,
        privacy_preferences: {
          ...current.privacy_preferences,
          [preferenceId]: !current.privacy_preferences[preferenceId],
        },
      }));
    });
    setStatus(t('Privacy preferences updated in the current user center.'));
  }

  return (
    <>
      <Dialog open={passwordDialogOpen} onOpenChange={setPasswordDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('Change password')}</DialogTitle>
            <DialogDescription>
              {t('Change the account password without leaving the user center.')}
            </DialogDescription>
          </DialogHeader>
          <form className="grid gap-4" onSubmit={handleSubmitPassword}>
            <SettingsField label={t('Current password')} layout="vertical">
              <Input
                autoComplete="current-password"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setCurrentPassword(event.target.value)}
                required
                type="password"
                value={currentPassword}
              />
            </SettingsField>
            <SettingsField label={t('New password')} layout="vertical">
              <Input
                autoComplete="new-password"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setNewPassword(event.target.value)}
                required
                type="password"
                value={newPassword}
              />
            </SettingsField>
            <SettingsField label={t('Confirm new password')} layout="vertical">
              <Input
                autoComplete="new-password"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setConfirmPassword(event.target.value)}
                required
                type="password"
                value={confirmPassword}
              />
            </SettingsField>

            <div className="grid gap-3 rounded-[20px] border border-zinc-200 bg-zinc-50/85 p-4 dark:border-zinc-800 dark:bg-zinc-900/70">
              {viewModel.password_policy.map((item) => (
                <div className="flex items-center justify-between gap-3" key={item.id}>
                  <span className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {item.label}
                  </span>
                  <Badge variant={item.met ? 'success' : 'warning'}>
                    {item.met ? t('Met') : t('Pending')}
                  </Badge>
                </div>
              ))}
            </div>

            <DialogFooter>
              <Button onClick={() => setPasswordDialogOpen(false)} type="button" variant="ghost">
                {t('Cancel')}
              </Button>
              <Button type="submit" variant="primary">
                {submitting ? t('Saving...') : t('Save password')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={phoneDialogOpen} onOpenChange={setPhoneDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('Phone binding')}</DialogTitle>
            <DialogDescription>
              {t('Add a recovery phone for password resets and important notices.')}
            </DialogDescription>
          </DialogHeader>
          <form className="grid gap-4" onSubmit={handleSubmitPhone}>
            <SettingsField label={t('Phone number')} layout="vertical">
              <Input
                autoComplete="tel"
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setPhoneNumber(event.target.value)}
                placeholder={t('Phone number')}
                type="tel"
                value={phoneNumber}
              />
            </SettingsField>

            <DialogFooter>
              {preferences.phone_number ? (
                <Button onClick={handleRemovePhone} type="button" variant="secondary">
                  {t('Remove binding')}
                </Button>
              ) : null}
              <Button onClick={() => setPhoneDialogOpen(false)} type="button" variant="ghost">
                {t('Cancel')}
              </Button>
              <Button type="submit" variant="primary">
                {t('Save phone')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={wechatDialogOpen} onOpenChange={setWechatDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('WeChat binding')}</DialogTitle>
            <DialogDescription>
              {t('Bind WeChat for trusted sign-in confirmation and workspace notifications.')}
            </DialogDescription>
          </DialogHeader>
          <form className="grid gap-4" onSubmit={handleSubmitWeChat}>
            <SettingsField label={t('WeChat ID')} layout="vertical">
              <Input
                onChange={(event: ChangeEvent<HTMLInputElement>) =>
                  setWechatId(event.target.value)}
                placeholder={t('WeChat ID')}
                value={wechatId}
              />
            </SettingsField>

            <DialogFooter>
              {preferences.wechat_id ? (
                <Button onClick={handleRemoveWeChat} type="button" variant="secondary">
                  {t('Remove binding')}
                </Button>
              ) : null}
              <Button onClick={() => setWechatDialogOpen(false)} type="button" variant="ghost">
                {t('Cancel')}
              </Button>
              <Button type="submit" variant="primary">
                {t('Save WeChat')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="space-y-4" data-slot="portal-user-page">
        <div
          data-slot="portal-user-toolbar"
          className="flex flex-wrap items-start justify-between gap-3 rounded-[24px] border border-zinc-200/80 bg-zinc-50/85 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/45"
        >
          <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2 text-sm text-zinc-600 dark:text-zinc-300">
            <Badge variant={workspace?.user.active ? 'success' : 'warning'}>
              {userStatusLabel}
            </Badge>
            <span className="inline-flex items-center rounded-full border border-zinc-200 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300">
              {workspace?.project.name ?? t('Current workspace')}
            </span>
            <p className="min-w-[16rem] flex-1 leading-6 text-zinc-500 dark:text-zinc-400">
              {status}
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button onClick={() => onNavigate('gateway')} variant="secondary">
              {t('Return to command center')}
            </Button>
            <Button onClick={() => onNavigate('account')} variant="secondary">
              {t('Open account')}
            </Button>
            <Button onClick={() => setPasswordDialogOpen(true)} variant="primary">
              {t('Change password')}
            </Button>
          </div>
        </div>

        <div className="grid gap-4 xl:grid-cols-[minmax(0,1.15fr)_minmax(0,0.85fr)]">
          <Card
            className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
            data-slot="portal-user-identity-card"
          >
            <CardContent className="p-5">
              <div className="space-y-4">
                <div className="space-y-1">
                  <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {t('User details')}
                  </h2>
                  <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                    {t('Core identity, workspace ownership, and recovery posture stay visible before you change settings.')}
                  </p>
                </div>

                <div className="rounded-[20px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/60">
                  <div className="flex flex-wrap items-start justify-between gap-4">
                    <div className="flex min-w-0 items-start gap-4">
                      <div className="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl border border-zinc-200 bg-white text-lg font-semibold text-zinc-950 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-50">
                        {(workspace?.user.display_name ?? workspace?.user.email ?? t('Portal workspace'))
                          .slice(0, 2)
                          .toUpperCase()}
                      </div>
                      <div className="min-w-0 space-y-2">
                        <div className="text-xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                          {workspace?.user.display_name ?? t('Portal workspace')}
                        </div>
                        <div className="text-sm text-zinc-500 dark:text-zinc-400">
                          {workspace?.user.email ?? t('Awaiting workspace session')}
                        </div>
                        <div className="flex flex-wrap items-center gap-2">
                          <Badge variant={workspace?.user.active ? 'success' : 'warning'}>
                            {userStatusLabel}
                          </Badge>
                          <span className="inline-flex items-center rounded-full border border-zinc-200 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950 dark:text-zinc-300">
                            {workspace?.project.name ?? t('Current workspace')}
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="mt-5 grid gap-3 sm:grid-cols-2">
                    {workspaceSummary.map((item) => (
                      <div
                        className="rounded-2xl border border-zinc-200 bg-white px-4 py-4 dark:border-zinc-800 dark:bg-zinc-950"
                        key={item.id}
                      >
                        <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                          {item.title}
                        </span>
                        <strong className="mt-2 block text-base font-semibold text-zinc-950 dark:text-zinc-50">
                          {item.value}
                        </strong>
                        <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                          {item.detail}
                        </p>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          <div className="grid gap-3">
            {viewModel.summary_cards.map((item) => (
              <UserSummaryCard
                detail={item.detail}
                key={item.id}
                title={item.title}
                value={item.value}
              />
            ))}
          </div>
        </div>

        <Card
          className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
          data-slot="portal-user-center"
        >
          <CardContent className="p-5">
            <Tabs
              className="space-y-4"
              onValueChange={(value) =>
                startTransition(() => setActiveTab(value as UserCenterTab))}
              value={activeTab}
            >
              <div className="flex flex-col gap-3 border-b border-zinc-200 pb-4 dark:border-zinc-800 lg:flex-row lg:items-center lg:justify-between">
                <div className="space-y-1">
                  <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                    {t('User center')}
                  </h2>
                  <p className="text-sm text-zinc-500 dark:text-zinc-400">
                    {t('Protected identity and recovery controls stay available from one profile surface.')}
                  </p>
                </div>

                <TabsList
                  className="inline-flex w-auto justify-start"
                  data-slot="portal-user-center-tabs"
                >
                  <TabsTrigger value="profile">{t('Profile overview')}</TabsTrigger>
                  <TabsTrigger value="privacy">{t('Privacy preferences')}</TabsTrigger>
                  <TabsTrigger value="security">{t('Password and authentication')}</TabsTrigger>
                </TabsList>
              </div>

              <TabsContent className="pt-1" value="profile">
                <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,0.95fr)]">
                  <UserSectionCard
                    description={t('Core account identity and workspace ownership stay clear in one place.')}
                    title={t('Profile overview')}
                  >
                    <UserProfileFacts workspace={workspace} />
                  </UserSectionCard>

                  <UserSectionCard
                    description={t('Bind phone and WeChat so recovery and trusted sign-in stay available.')}
                    title={t('Connected methods')}
                  >
                    <div className="grid gap-3">
                      {viewModel.binding_items.map((item) => (
                        <UserDetailCard
                          badge={(
                            <Badge variant={item.connected ? 'success' : 'warning'}>
                              {item.connected ? t('Connected') : t('Not bound')}
                            </Badge>
                          )}
                          description={item.detail}
                          key={item.id}
                          title={item.title}
                        >
                          <div className="flex flex-wrap items-center justify-between gap-3">
                            <span className="text-sm font-medium text-zinc-950 dark:text-zinc-50">
                              {item.value}
                            </span>
                            <Button
                              onClick={() =>
                                item.id === 'phone'
                                  ? setPhoneDialogOpen(true)
                                  : setWechatDialogOpen(true)}
                              variant="secondary"
                            >
                              {item.action_label}
                            </Button>
                          </div>
                        </UserDetailCard>
                      ))}
                    </div>
                  </UserSectionCard>
                </div>
              </TabsContent>

              <TabsContent className="pt-1" value="privacy">
                <UserSectionCard
                  description={t('Control how your identity and activity appear across the workspace.')}
                  title={t('Privacy preferences')}
                >
                  <div className="grid gap-3">
                    {viewModel.privacy_preferences.map((item) => (
                      <label
                        className="flex cursor-pointer items-center gap-4 rounded-[20px] border border-zinc-200 bg-zinc-50/80 px-4 py-4 transition-colors hover:bg-zinc-100/80 dark:border-zinc-800 dark:bg-zinc-900/60 dark:hover:bg-zinc-900"
                        key={item.id}
                      >
                        <div className="min-w-0 flex-1">
                          <div className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                            {item.title}
                          </div>
                          <div className="mt-1 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                            {item.description}
                          </div>
                        </div>
                        <Checkbox
                          checked={item.enabled}
                          onCheckedChange={() => handleTogglePrivacyPreference(item.id)}
                        />
                      </label>
                    ))}
                  </div>
                </UserSectionCard>
              </TabsContent>

              <TabsContent className="pt-1" value="security">
                <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
                  <UserSectionCard
                    actions={(
                      <Button onClick={() => setPasswordDialogOpen(true)} variant="primary">
                        {t('Change password')}
                      </Button>
                    )}
                    description={t('Keep sign-in trust, recovery readiness, and linked channels under one security surface.')}
                    title={t('Password and authentication')}
                  >
                    <div className="grid gap-3">
                      {viewModel.security_items.map((item) => (
                        <UserDetailCard
                          badge={(
                            <Badge variant={item.tone}>
                              {item.status_label}
                            </Badge>
                          )}
                          description={item.detail}
                          key={item.id}
                          title={item.title}
                        />
                      ))}
                    </div>
                  </UserSectionCard>

                  <UserSectionCard
                    description={t('Password requirements stay visible before you submit a new credential.')}
                    title={t('Password policy')}
                  >
                    <div className="grid gap-3">
                      {viewModel.password_policy.map((item) => (
                        <UserDetailCard
                          badge={(
                            <Badge variant={item.met ? 'success' : 'warning'}>
                              {item.met ? t('Met') : t('Pending')}
                            </Badge>
                          )}
                          key={item.id}
                          title={item.label}
                        />
                      ))}
                    </div>
                  </UserSectionCard>
                </div>
              </TabsContent>
            </Tabs>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
