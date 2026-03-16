import { useState } from 'react';
import type { FormEvent } from 'react';
import { InlineButton, Pill, SectionHero, Surface } from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import { AccountProfileFacts } from '../components';
import { changePortalPassword } from '../repository';
import { buildPortalAccountViewModel, passwordsMatch } from '../services';
import type { PortalAccountPageProps } from '../types';

export function PortalAccountPage({ workspace, onNavigate }: PortalAccountPageProps) {
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [status, setStatus] = useState(
    'Rotate your password without leaving the portal boundary.',
  );
  const [submitting, setSubmitting] = useState(false);
  const viewModel = buildPortalAccountViewModel(workspace, newPassword, confirmPassword);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!passwordsMatch(newPassword, confirmPassword)) {
      setStatus('New password confirmation does not match.');
      return;
    }
    if (!viewModel.can_submit_password) {
      setStatus('New password does not yet satisfy the visible portal security policy.');
      return;
    }

    setSubmitting(true);
    setStatus('Updating workspace password...');

    try {
      await changePortalPassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setStatus('Password updated. Use the new password the next time you sign in.');
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <>
      <SectionHero
        detail="Review your portal identity and rotate secrets without touching the admin control plane."
        eyebrow="Account"
        title={workspace?.user.display_name ?? 'Portal account'}
      />

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface detail="Current workspace identity and ownership boundary." title="Workspace trust center">
          <div className="portalx-checklist-grid">
            {viewModel.trust_center.map((item) => (
              <article className="portalx-checklist-card" key={item.id}>
                <strong>{item.title}</strong>
                <span>{item.value}</span>
                <p>{item.detail}</p>
              </article>
            ))}
          </div>
          <AccountProfileFacts workspace={workspace} />
        </Surface>

        <Surface detail={status} title="Rotate password">
          <form className="portalx-form portalx-form-card" onSubmit={handleSubmit}>
            <label className="portalx-field">
              <span>Current password</span>
              <input
                autoComplete="current-password"
                onChange={(event) => setCurrentPassword(event.target.value)}
                required
                type="password"
                value={currentPassword}
              />
            </label>
            <label className="portalx-field">
              <span>New password</span>
              <input
                autoComplete="new-password"
                onChange={(event) => setNewPassword(event.target.value)}
                required
                type="password"
                value={newPassword}
              />
            </label>
            <label className="portalx-field">
              <span>Confirm new password</span>
              <input
                autoComplete="new-password"
                onChange={(event) => setConfirmPassword(event.target.value)}
                required
                type="password"
                value={confirmPassword}
              />
            </label>
            <div className="portalx-form-actions">
              <InlineButton tone="primary" type="submit">
                {submitting ? 'Saving...' : 'Update password'}
              </InlineButton>
            </div>
            <div className="portalx-guardrail-list">
              {viewModel.password_policy.map((item) => (
                <article className="portalx-guardrail-card" key={item.id}>
                  <div className="portalx-status-row">
                    <strong>{item.label}</strong>
                    <Pill tone={item.met ? 'positive' : 'warning'}>
                      {item.met ? 'Met' : 'Pending'}
                    </Pill>
                  </div>
                </article>
              ))}
            </div>
          </form>
        </Surface>
      </div>

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail="A simple checklist that makes account posture visible before the next key, billing, or usage action."
          title="Security checklist"
        >
          <div className="portalx-checklist-grid">
            {viewModel.security_checklist.map((item) => (
              <article className="portalx-checklist-card" key={item.id}>
                <Pill tone={item.complete ? 'positive' : 'warning'}>
                  {item.complete ? 'Ready' : 'Needs action'}
                </Pill>
                <strong>{item.title}</strong>
                <p>{item.detail}</p>
              </article>
            ))}
          </div>
        </Surface>

        <Surface
          detail="The portal should help users recover safely from common account and access problems without relying on admin intervention."
          title="Recovery signals"
        >
          <div className="portalx-guardrail-list">
            {viewModel.recovery_signals.map((item) => (
              <article className="portalx-guardrail-card" key={item.id}>
                <strong>{item.title}</strong>
                <p>{item.detail}</p>
              </article>
            ))}
          </div>
        </Surface>
      </div>

      <Surface
        detail="Account work should fold back into the active workspace journey instead of becoming a dead-end settings page."
        title="Return to command center"
      >
        <div className="portalx-checklist-grid">
          <article className="portalx-checklist-card">
            <strong>Return to the live workspace pulse</strong>
            <p>After trust or password changes, go back to Dashboard to verify the workspace is still ready for the next action.</p>
            <InlineButton onClick={() => onNavigate('dashboard')} tone="primary">
              Open dashboard
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Audit credentials after a security change</strong>
            <p>If the account boundary changed because of risk or recovery, verify that API keys and environment ownership still look correct.</p>
            <InlineButton onClick={() => onNavigate('api-keys')} tone="secondary">
              Manage keys
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Review billing and credits before the next launch</strong>
            <p>Once identity and password posture are healthy, keep commercial runway aligned so the workspace can actually continue operating.</p>
            <div className="portalx-form-actions">
              <InlineButton onClick={() => onNavigate('credits')} tone="ghost">
                Open credits
              </InlineButton>
              <InlineButton onClick={() => onNavigate('billing')} tone="ghost">
                Review billing
              </InlineButton>
            </div>
          </article>
        </div>
      </Surface>
    </>
  );
}
