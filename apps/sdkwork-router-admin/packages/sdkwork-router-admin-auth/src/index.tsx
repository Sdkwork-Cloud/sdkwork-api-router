import { useState } from 'react';
import type { FormEvent } from 'react';

import { InlineButton, Pill } from 'sdkwork-router-admin-commons';

export function AdminLoginPage({
  status,
  loading,
  onLogin,
}: {
  status: string;
  loading: boolean;
  onLogin: (input: { email: string; password: string }) => Promise<void>;
}) {
  const [email, setEmail] = useState('admin@sdkwork.local');
  const [password, setPassword] = useState('ChangeMe123!');

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onLogin({ email, password });
  }

  return (
    <section className="adminx-auth-shell">
      <article className="adminx-auth-story">
        <p className="adminx-eyebrow">SDKWork Router Admin</p>
        <h1>Super admin control center for daily router operations.</h1>
        <p className="adminx-auth-detail">
          This independent engineering project is built for operators managing identities,
          workspaces, coupon campaigns, routing mesh, and runtime health from one industrial-grade
          surface.
        </p>
        <div className="adminx-auth-pills">
          <Pill tone="live">Independent React workspace</Pill>
          <Pill tone="default">Operational CRUD flows</Pill>
          <Pill tone="live">Live coupon campaigns</Pill>
        </div>
      </article>

      <article className="adminx-auth-card">
        <p className="adminx-eyebrow">Operator Login</p>
        <h2>Authenticate the super-admin project</h2>
        <p className="adminx-status">{status}</p>
        <form className="adminx-form" onSubmit={handleSubmit}>
          <label className="adminx-field">
            <span>Email</span>
            <input
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              type="email"
              autoComplete="email"
              required
            />
          </label>
          <label className="adminx-field">
            <span>Password</span>
            <input
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              type="password"
              autoComplete="current-password"
              required
            />
          </label>
          <InlineButton tone="primary" type="submit">
            {loading ? 'Signing in...' : 'Open super admin'}
          </InlineButton>
        </form>
        <div className="adminx-note">
          <span>Local bootstrap account</span>
          <code>admin@sdkwork.local / ChangeMe123!</code>
        </div>
      </article>
    </section>
  );
}
