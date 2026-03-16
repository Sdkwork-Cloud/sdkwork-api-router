import { Pill } from 'sdkwork-router-portal-commons';

import type { AuthShellProps } from '../types';

export function AuthShell({
  eyebrow,
  title,
  detail,
  highlights,
  launchSteps,
  trustSignals,
  status,
  previewTitle,
  previewItems,
  children,
}: AuthShellProps) {
  return (
    <section className="portalx-auth-shell">
      <article className="portalx-auth-hero">
        <Pill tone="accent">SDKWork Router Portal</Pill>
        <p className="portalx-eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="portalx-auth-detail">{detail}</p>
        <div className="portalx-highlight-grid">
          {highlights.map((highlight) => (
            <article className="portalx-highlight-card" key={highlight.title}>
              <strong>{highlight.title}</strong>
              <p>{highlight.detail}</p>
            </article>
          ))}
        </div>
        <div className="portalx-auth-story-grid">
          <section className="portalx-auth-story-card">
            <div className="portalx-auth-section-heading">
              <p className="portalx-eyebrow">Start in four moves</p>
              <strong>Build enough confidence to launch on the first session.</strong>
            </div>
            <ol className="portalx-launch-list">
              {launchSteps.map((step, index) => (
                <li key={step.title}>
                  <span>{index + 1}</span>
                  <div>
                    <strong>{step.title}</strong>
                    <p>{step.detail}</p>
                  </div>
                </li>
              ))}
            </ol>
          </section>

          <section className="portalx-auth-story-card">
            <div className="portalx-auth-section-heading">
              <p className="portalx-eyebrow">Why teams trust this portal</p>
              <strong>Customer-facing self-service, without leaking operator controls.</strong>
            </div>
            <ul className="portalx-trust-list">
              {trustSignals.map((signal) => (
                <li key={signal}>{signal}</li>
              ))}
            </ul>
          </section>
        </div>
      </article>
      <article className="portalx-auth-card">
        <p className="portalx-status">{status}</p>
        {previewTitle && previewItems?.length ? (
          <section className="portalx-auth-preview">
            <div className="portalx-auth-section-heading">
              <p className="portalx-eyebrow">{previewTitle}</p>
              <strong>The first session should already feel production-grade.</strong>
            </div>
            <div className="portalx-preview-grid">
              {previewItems.map((item) => (
                <article className="portalx-preview-card" key={item.label}>
                  <span>{item.label}</span>
                  <strong>{item.value}</strong>
                  <p>{item.detail}</p>
                </article>
              ))}
            </div>
          </section>
        ) : null}
        {children}
      </article>
    </section>
  );
}
