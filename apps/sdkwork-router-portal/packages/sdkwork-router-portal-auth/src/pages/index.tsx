import { useState } from 'react';
import type { FormEvent } from 'react';
import { InlineButton } from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import { AuthShell } from '../components';
import { loginPortalUser, registerPortalUser } from '../repository';
import { persistPortalAuthSession } from '../services';
import type { PortalAuthPageProps } from '../types';

export function PortalRegisterPage({ onAuthenticated, onNavigate }: PortalAuthPageProps) {
  const [displayName, setDisplayName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [status, setStatus] = useState(
    'Create your developer workspace and receive a live portal session immediately.',
  );
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus('Provisioning workspace identity and self-service billing posture...');

    try {
      const session = await registerPortalUser({
        display_name: displayName,
        email,
        password,
      });
      persistPortalAuthSession(session);
      setStatus('Workspace created. Redirecting you to the portal dashboard...');
      onAuthenticated(session);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <AuthShell
      detail="Portal gives external developers a clear self-service boundary for onboarding, usage visibility, points posture, and API key lifecycle without exposing operator tooling."
      eyebrow="Developer Onboarding"
      highlights={[
        {
          title: 'Independent workspace boundary',
          detail: 'Every portal account lands inside its own tenant and project context, separate from the operator control plane.',
        },
        {
          title: 'Operational visibility from day one',
          detail: 'Usage, token units, quota posture, coupon motion, and billing guidance stay visible without opening admin tooling.',
        },
        {
          title: 'Launch-ready self-service',
          detail: 'The entry flow is designed to move a new developer from sign-up into first key issuance and first traffic quickly.',
        },
      ]}
      launchSteps={[
        {
          title: 'Create the workspace identity',
          detail: 'Provision the account owner, workspace tenant, and starter project in one move.',
        },
        {
          title: 'Open the dashboard command center',
          detail: 'Immediately inspect launch posture, quota state, and the next recommended action.',
        },
        {
          title: 'Issue environment keys',
          detail: 'Generate credentials for local, staging, and production traffic without operator involvement.',
        },
        {
          title: 'Watch usage and recharge posture',
          detail: 'Keep token-unit usage, coupon redemption, and subscription readiness visible as requests ramp up.',
        },
      ]}
      previewItems={[
        {
          label: 'Workspace pulse',
          value: 'Always visible',
          detail: 'Keys, traffic, and quota posture remain in view after sign-in.',
        },
        {
          label: 'First-launch checklist',
          value: '4 guided gates',
          detail: 'The dashboard shows exactly what is missing before traffic goes live.',
        },
        {
          label: 'Commerce posture',
          value: 'Credits + billing',
          detail: 'Coupon redemption, points runway, and subscription recommendations stay productized.',
        },
      ]}
      previewTitle="Preview the first launch path"
      trustSignals={[
        'Admin controls, provider operations, and runtime governance remain isolated in the admin product.',
        'Every request lands back in the portal as per-call token-unit telemetry and quota posture.',
        'Recharge, subscription, and coupon entry points sit behind repository seams so checkout can evolve without reworking the UI contract.',
      ]}
      status={status}
      title="Launch a managed workspace in minutes."
    >
      <h2>Create workspace</h2>
      <form className="portalx-form" onSubmit={handleSubmit}>
        <label className="portalx-field">
          <span>Display name</span>
          <input
            autoComplete="name"
            onChange={(event) => setDisplayName(event.target.value)}
            placeholder="Portal Workspace Owner"
            required
            value={displayName}
          />
        </label>
        <label className="portalx-field">
          <span>Email</span>
          <input
            autoComplete="email"
            onChange={(event) => setEmail(event.target.value)}
            placeholder="portal@example.com"
            required
            type="email"
            value={email}
          />
        </label>
        <label className="portalx-field">
          <span>Password</span>
          <input
            autoComplete="new-password"
            onChange={(event) => setPassword(event.target.value)}
            placeholder="At least 8 characters"
            required
            type="password"
            value={password}
          />
        </label>
        <div className="portalx-form-actions">
          <InlineButton tone="primary" type="submit">
            {submitting ? 'Creating workspace...' : 'Create workspace'}
          </InlineButton>
          <InlineButton onClick={() => onNavigate('login')} tone="ghost">
            Already have access
          </InlineButton>
        </div>
      </form>
    </AuthShell>
  );
}

export function PortalLoginPage({ onAuthenticated, onNavigate }: PortalAuthPageProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [status, setStatus] = useState(
    'Sign in to inspect usage, points posture, and production API keys.',
  );
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus('Restoring your self-service workspace session...');

    try {
      const session = await loginPortalUser({ email, password });
      persistPortalAuthSession(session);
      setStatus('Session restored. Opening your workspace...');
      onAuthenticated(session);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <AuthShell
      detail="This portal is designed for end-user self-service. Admin operations, provider controls, and platform runtime management remain isolated in the operator product."
      eyebrow="Workspace Access"
      highlights={[
        {
          title: 'Customer-facing operating surface',
          detail: 'Portal concentrates on the user journey: launch, observe, recharge, and keep production traffic healthy.',
        },
        {
          title: 'Live request accountability',
          detail: 'Every recent request can roll up into token-unit usage, total calls, and booked amount without context switching.',
        },
        {
          title: 'Commercial decisions in context',
          detail: 'Users can see runway, recommended bundles, and coupon posture next to actual workspace demand.',
        },
      ]}
      launchSteps={[
        {
          title: 'Re-open the dashboard pulse',
          detail: 'Pick up exactly where the workspace left off with live identity, usage, and quota posture.',
        },
        {
          title: 'Check the action queue',
          detail: 'See which production blockers matter now instead of reading raw telemetry alone.',
        },
        {
          title: 'Move into credits or billing only if needed',
          detail: 'Upgrade, recharge, or redeem coupons from the same product shell when runway gets tight.',
        },
        {
          title: 'Keep usage, keys, and account trust aligned',
          detail: 'The portal keeps the whole user-side operating loop together in one boundary.',
        },
      ]}
      previewItems={[
        {
          label: 'Command center',
          value: 'Action-led shell',
          detail: 'The active route is wrapped with a persistent workspace pulse and launch guidance.',
        },
        {
          label: 'Usage insight',
          value: 'Per-call visibility',
          detail: 'Recent requests list provider, model, token units, and booked amount for each call.',
        },
        {
          label: 'Growth support',
          value: 'Recommended next buy',
          detail: 'Billing surfaces point to the cleanest plan and pack path based on current demand.',
        },
      ]}
      previewTitle="Preview the first launch path"
      trustSignals={[
        'Self-service access stays focused on the user workspace and never exposes operator-only control surfaces.',
        'The portal keeps live workspace identity, API keys, usage, credits, billing, and account context in one shell.',
        'Coupon offers and recharge paths can evolve independently because the product boundary is already explicit.',
      ]}
      status={status}
      title="Open your developer control center."
    >
      <h2>Sign in</h2>
      <form className="portalx-form" onSubmit={handleSubmit}>
        <label className="portalx-field">
          <span>Email</span>
          <input
            autoComplete="email"
            onChange={(event) => setEmail(event.target.value)}
            placeholder="portal@example.com"
            required
            type="email"
            value={email}
          />
        </label>
        <label className="portalx-field">
          <span>Password</span>
          <input
            autoComplete="current-password"
            onChange={(event) => setPassword(event.target.value)}
            placeholder="Your portal password"
            required
            type="password"
            value={password}
          />
        </label>
        <div className="portalx-form-actions">
          <InlineButton tone="primary" type="submit">
            {submitting ? 'Opening workspace...' : 'Open workspace'}
          </InlineButton>
          <InlineButton onClick={() => onNavigate('register')} tone="ghost">
            Create account
          </InlineButton>
        </div>
        <div className="portalx-note">
          <strong>Local demo</strong>
          <span>`portal@sdkwork.local / ChangeMe123!`</span>
        </div>
      </form>
    </AuthShell>
  );
}
