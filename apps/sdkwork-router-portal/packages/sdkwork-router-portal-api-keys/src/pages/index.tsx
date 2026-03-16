import { useEffect, useMemo, useState } from 'react';
import type { FormEvent } from 'react';
import {
  copyText,
  DataTable,
  EmptyState,
  InlineButton,
  SectionHero,
  Surface,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

import { ApiKeyEnvironmentSummaryGrid } from '../components';
import { issuePortalApiKey, loadPortalApiKeys } from '../repository';
import { buildPortalApiKeysViewModel } from '../services';
import type { PortalApiKeysPageProps } from '../types';

export function PortalApiKeysPage({ onNavigate }: PortalApiKeysPageProps) {
  const [apiKeys, setApiKeys] = useState<GatewayApiKeyRecord[]>([]);
  const [environment, setEnvironment] = useState('live');
  const [createdKey, setCreatedKey] = useState<CreatedGatewayApiKey | null>(null);
  const [status, setStatus] = useState('Loading issued keys...');
  const [submitting, setSubmitting] = useState(false);
  const [copyStatus, setCopyStatus] = useState('Plaintext keys are only shown once at creation time.');

  async function refresh() {
    const keys = await loadPortalApiKeys();
    setApiKeys(keys);
  }

  useEffect(() => {
    let cancelled = false;

    void refresh()
      .then(() => {
        if (!cancelled) {
          setStatus('Plaintext keys are only shown once at creation time.');
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setStatus(`Issuing a ${environment} key for this workspace...`);

    try {
      const nextKey = await issuePortalApiKey(environment);
      await refresh();
      setCreatedKey(nextKey);
      setCopyStatus('Copy the plaintext secret now. It will not be shown again after you leave this screen.');
      setStatus(`Key issued for ${environment}. Copy the plaintext secret before leaving this page.`);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleCopyPlaintext() {
    if (!createdKey) {
      return;
    }

    const copied = await copyText(createdKey.plaintext);
    setCopyStatus(copied ? 'Plaintext key copied to clipboard.' : 'Clipboard copy is unavailable in this browser context.');
  }

  const viewModel = useMemo(
    () => buildPortalApiKeysViewModel(apiKeys, createdKey),
    [apiKeys, createdKey],
  );

  return (
    <>
      <SectionHero
        detail="Create environment-scoped gateway credentials, copy plaintext once, and keep the API key lifecycle inside the portal boundary."
        eyebrow="Credentials"
        title="API key management"
      />

      {viewModel.environment_summaries.length ? (
        <Surface detail="A quick audit of key posture across environments." title="Environment coverage">
          <ApiKeyEnvironmentSummaryGrid summaries={viewModel.environment_summaries} />
        </Surface>
      ) : null}

      <div className="portalx-split-grid portalx-split-grid-wide">
        <Surface
          detail="A recommended rollout order for test, staging, and live credentials based on the current workspace posture."
          title="Environment strategy"
        >
          <div className="portalx-checklist-grid">
            {viewModel.environment_strategy.map((item) => (
              <article className="portalx-checklist-card" key={item.environment}>
                <div className="portalx-status-row">
                  <strong>{item.environment}</strong>
                  <span>{item.status}</span>
                </div>
                <p>{item.detail}</p>
                {item.recommended ? <span className="portalx-status">Recommended next environment</span> : null}
              </article>
            ))}
          </div>
        </Surface>

        <Surface
          detail="The portal keeps secret lifecycle guidance visible so new keys turn into safe operational habits."
          title="Key handling guardrails"
        >
          <div className="portalx-guardrail-list">
            {viewModel.guardrails.map((guardrail) => (
              <article className="portalx-guardrail-card" key={guardrail.id}>
                <strong>{guardrail.title}</strong>
                <p>{guardrail.detail}</p>
              </article>
            ))}
          </div>
        </Surface>
      </div>

      <Surface detail={status} title="Issue a new key">
        <div className="portalx-split-grid">
          <form className="portalx-form portalx-form-card" onSubmit={handleSubmit}>
            <label className="portalx-field">
              <span>Environment</span>
              <select value={environment} onChange={(event) => setEnvironment(event.target.value)}>
                <option value="live">live</option>
                <option value="staging">staging</option>
                <option value="test">test</option>
              </select>
            </label>
            <div className="portalx-form-actions">
              <InlineButton tone="primary" type="submit">
                {submitting ? 'Issuing key...' : 'Create key'}
              </InlineButton>
              <InlineButton disabled={!createdKey} onClick={handleCopyPlaintext} tone="secondary">
                Copy plaintext
              </InlineButton>
            </div>
            {viewModel.created_key ? (
              <div className="portalx-note">
                <strong>Plaintext key</strong>
                <code>{viewModel.created_key.plaintext}</code>
                <span>{copyStatus}</span>
              </div>
            ) : null}
          </form>

          <div className="portalx-note portalx-note-strong">
            <strong>Integration posture</strong>
            <span>Use `Authorization: Bearer &lt;key&gt;` against your `/v1/*` gateway endpoints.</span>
            <span>Issue separate keys for production, staging, and local automation.</span>
          </div>
        </div>
      </Surface>

      <Surface
        detail="Use this flow after each new key issuance so environment cutovers stay deliberate and reversible."
        title="Rotation checklist"
      >
        <ol className="portalx-queue-list">
          {viewModel.rotation_checklist.map((item) => (
            <li className="portalx-queue-card" key={item.id}>
              <strong>{item.title}</strong>
              <p>{item.detail}</p>
            </li>
          ))}
        </ol>
      </Surface>

      <Surface
        detail="The best next action depends on whether you want to validate traffic, review global posture, or protect quota before launch."
        title="Recommended next move"
      >
        <div className="portalx-checklist-grid">
          <article className="portalx-checklist-card">
            <strong>Validate the new credential with live telemetry</strong>
            <p>After issuing a key, send a small authenticated request so Usage can confirm model, provider, and token-unit visibility.</p>
            <InlineButton onClick={() => onNavigate('usage')} tone="primary">
              Open usage
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Return to workspace posture</strong>
            <p>Go back to Dashboard to confirm keys, traffic, and launch readiness stay aligned.</p>
            <InlineButton onClick={() => onNavigate('dashboard')} tone="secondary">
              Open dashboard
            </InlineButton>
          </article>
          <article className="portalx-checklist-card">
            <strong>Protect runway before production rollout</strong>
            <p>If key issuance is happening right before a launch window, confirm credits posture before promoting traffic.</p>
            <InlineButton onClick={() => onNavigate('credits')} tone="ghost">
              Review credits
            </InlineButton>
          </article>
        </div>
      </Surface>

      {viewModel.quickstart_snippet ? (
        <Surface detail="A first authenticated call using the freshly created key." title="Quickstart snippet">
          <div className="portalx-code-block">
            <code>{viewModel.quickstart_snippet}</code>
          </div>
        </Surface>
      ) : null}

      <Surface detail="Write-only key secrets never reappear in list responses." title="Issued keys">
        {viewModel.keys.length ? (
          <DataTable
            columns={[
              {
                key: 'environment',
                label: 'Environment',
                render: (row) => row.environment,
              },
              {
                key: 'hashed',
                label: 'Hashed Key',
                render: (row) => row.hashed_key,
              },
              {
                key: 'status',
                label: 'Status',
                render: (row) => (row.active ? 'Active' : 'Inactive'),
              },
            ]}
            empty="No gateway keys issued yet."
            getKey={(row) => row.hashed_key}
            rows={viewModel.keys}
          />
        ) : (
          <EmptyState
            detail="Create your first key to connect a client or service to the SDKWork Router gateway."
            title="No API keys yet"
          />
        )}
      </Surface>
    </>
  );
}
