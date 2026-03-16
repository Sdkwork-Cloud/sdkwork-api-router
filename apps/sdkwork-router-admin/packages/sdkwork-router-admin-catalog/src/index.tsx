import { useState } from 'react';
import type { FormEvent } from 'react';

import {
  DataTable,
  InlineButton,
  Pill,
  SectionHero,
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, CredentialRecord } from 'sdkwork-router-admin-types';

function credentialStorageLabel(credential: CredentialRecord): string {
  if (credential.secret_backend === 'local_encrypted_file') {
    return credential.secret_local_file ?? 'local encrypted file';
  }

  if (credential.secret_backend === 'os_keyring') {
    return credential.secret_keyring_service ?? 'os keyring';
  }

  return 'database envelope';
}

export function CatalogPage({
  snapshot,
  onSaveChannel,
  onSaveProvider,
  onSaveCredential,
  onSaveModel,
  onDeleteChannel,
  onDeleteProvider,
  onDeleteCredential,
  onDeleteModel,
}: AdminPageProps & {
  onSaveChannel: (input: { id: string; name: string }) => Promise<void>;
  onSaveProvider: (input: {
    id: string;
    channel_id: string;
    extension_id?: string;
    adapter_kind: string;
    base_url: string;
    display_name: string;
    channel_bindings: Array<{ channel_id: string; is_primary: boolean }>;
  }) => Promise<void>;
  onSaveCredential: (input: {
    tenant_id: string;
    provider_id: string;
    key_reference: string;
    secret_value: string;
  }) => Promise<void>;
  onSaveModel: (input: {
    external_name: string;
    provider_id: string;
    capabilities: string[];
    streaming: boolean;
    context_window?: number;
  }) => Promise<void>;
  onDeleteChannel: (channelId: string) => Promise<void>;
  onDeleteProvider: (providerId: string) => Promise<void>;
  onDeleteCredential: (
    tenantId: string,
    providerId: string,
    keyReference: string,
  ) => Promise<void>;
  onDeleteModel: (externalName: string, providerId: string) => Promise<void>;
}) {
  const [channelDraft, setChannelDraft] = useState({
    id: snapshot.channels[0]?.id ?? 'openai',
    name: snapshot.channels[0]?.name ?? 'OpenAI',
  });
  const [providerDraft, setProviderDraft] = useState({
    id: snapshot.providers[0]?.id ?? 'provider-openai-official',
    channel_id: snapshot.providers[0]?.channel_id ?? snapshot.channels[0]?.id ?? 'openai',
    display_name: snapshot.providers[0]?.display_name ?? 'OpenAI Official',
    adapter_kind: snapshot.providers[0]?.adapter_kind ?? 'openai',
    base_url: snapshot.providers[0]?.base_url ?? 'https://api.openai.com',
    extension_id: snapshot.providers[0]?.extension_id ?? 'sdkwork.provider.openai.official',
  });
  const [credentialDraft, setCredentialDraft] = useState({
    tenant_id: snapshot.credentials[0]?.tenant_id ?? snapshot.tenants[0]?.id ?? 'tenant-local',
    provider_id:
      snapshot.credentials[0]?.provider_id
      ?? snapshot.providers[0]?.id
      ?? 'provider-openai-official',
    key_reference: snapshot.credentials[0]?.key_reference ?? 'cred-openai-primary',
    secret_value: '',
  });
  const [modelDraft, setModelDraft] = useState({
    external_name: snapshot.models[0]?.external_name ?? 'gpt-4.1',
    provider_id:
      snapshot.models[0]?.provider_id ?? snapshot.providers[0]?.id ?? 'provider-openai-official',
    capabilities: snapshot.models[0]?.capabilities.join(', ') ?? 'responses, chat_completions',
    streaming: snapshot.models[0]?.streaming ?? true,
    context_window: String(snapshot.models[0]?.context_window ?? 128000),
  });

  const selectedProvider = snapshot.providers.find((provider) => provider.id === modelDraft.provider_id);
  const selectedCredential = snapshot.credentials.find(
    (credential) => (
      credential.tenant_id === credentialDraft.tenant_id
      && credential.provider_id === credentialDraft.provider_id
      && credential.key_reference === credentialDraft.key_reference
    ),
  );
  const channelsWithProviders = new Set(
    snapshot.providers.flatMap((provider) => [
      provider.channel_id,
      ...provider.channel_bindings.map((binding) => binding.channel_id),
    ]),
  );
  const providersWithModels = new Set(snapshot.models.map((model) => model.provider_id));
  const providersWithCredentials = new Set(
    snapshot.credentials.map((credential) => credential.provider_id),
  );
  const providersWithoutCredentials = snapshot.providers.filter(
    (provider) => !providersWithCredentials.has(provider.id),
  );
  const orphanCredentials = snapshot.credentials.filter(
    (credential) => !snapshot.providers.some((provider) => provider.id === credential.provider_id),
  );
  const credentialCounts = new Map<string, number>();
  for (const credential of snapshot.credentials) {
    credentialCounts.set(
      credential.provider_id,
      (credentialCounts.get(credential.provider_id) ?? 0) + 1,
    );
  }

  async function handleChannel(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveChannel(channelDraft);
  }

  async function handleProvider(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveProvider({
      id: providerDraft.id,
      channel_id: providerDraft.channel_id,
      extension_id: providerDraft.extension_id || undefined,
      adapter_kind: providerDraft.adapter_kind,
      base_url: providerDraft.base_url,
      display_name: providerDraft.display_name,
      channel_bindings: [{ channel_id: providerDraft.channel_id, is_primary: true }],
    });
  }

  async function handleCredential(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveCredential({
      tenant_id: credentialDraft.tenant_id,
      provider_id: credentialDraft.provider_id,
      key_reference: credentialDraft.key_reference,
      secret_value: credentialDraft.secret_value,
    });
    setCredentialDraft((current) => ({ ...current, secret_value: '' }));
  }

  async function handleModel(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveModel({
      external_name: modelDraft.external_name,
      provider_id: modelDraft.provider_id,
      capabilities: modelDraft.capabilities
        .split(',')
        .map((value) => value.trim())
        .filter(Boolean),
      streaming: modelDraft.streaming,
      context_window: Number(modelDraft.context_window),
    });
  }

  return (
    <div className="adminx-page-grid">
      <SectionHero
        eyebrow="Routing Mesh"
        title="Shape routing inventory, provider secrets, and model exposure."
        detail="Catalog is now the control surface for channel, provider, credential, and model lifecycle management. Operators can curate exposure, rotate upstream secrets, and inspect credential coverage without leaving the standalone admin workspace."
      />

      <section className="adminx-stat-grid">
        <StatCard
          label="Channels"
          value={String(snapshot.channels.length)}
          detail="Protocol or adapter surfaces exposed to the router."
        />
        <StatCard
          label="Providers"
          value={String(snapshot.providers.length)}
          detail="Proxy provider records and base URL definitions."
        />
        <StatCard
          label="Credentials"
          value={String(snapshot.credentials.length)}
          detail="Upstream provider credentials tracked by the control plane."
        />
        <StatCard
          label="Models"
          value={String(snapshot.models.length)}
          detail="Model entries routed through the provider catalog."
        />
      </section>

      <div className="adminx-users-grid">
        <Surface title="Channel maintenance" detail="Create or update channel definitions.">
          <form className="adminx-form-grid" onSubmit={(event) => void handleChannel(event)}>
            <label className="adminx-field">
              <span>Channel id</span>
              <input
                value={channelDraft.id}
                onChange={(event) => setChannelDraft((current) => ({ ...current, id: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Channel name</span>
              <input
                value={channelDraft.name}
                onChange={(event) => setChannelDraft((current) => ({ ...current, name: event.target.value }))}
                required
              />
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                Save channel
              </InlineButton>
            </div>
          </form>
        </Surface>

        <Surface
          title="Provider maintenance"
          detail="Maintain provider records and their primary channel bindings."
        >
          <form className="adminx-form-grid" onSubmit={(event) => void handleProvider(event)}>
            <label className="adminx-field">
              <span>Provider id</span>
              <input
                value={providerDraft.id}
                onChange={(event) => setProviderDraft((current) => ({ ...current, id: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Channel id</span>
              {snapshot.channels.length ? (
                <select
                  value={providerDraft.channel_id}
                  onChange={(event) => setProviderDraft((current) => ({ ...current, channel_id: event.target.value }))}
                >
                  {snapshot.channels.map((channel) => (
                    <option key={channel.id} value={channel.id}>
                      {channel.name} ({channel.id})
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  value={providerDraft.channel_id}
                  onChange={(event) => setProviderDraft((current) => ({ ...current, channel_id: event.target.value }))}
                  required
                />
              )}
            </label>
            <label className="adminx-field">
              <span>Display name</span>
              <input
                value={providerDraft.display_name}
                onChange={(event) => setProviderDraft((current) => ({ ...current, display_name: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Adapter kind</span>
              <input
                value={providerDraft.adapter_kind}
                onChange={(event) => setProviderDraft((current) => ({ ...current, adapter_kind: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Base URL</span>
              <input
                value={providerDraft.base_url}
                onChange={(event) => setProviderDraft((current) => ({ ...current, base_url: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Extension id</span>
              <input
                value={providerDraft.extension_id}
                onChange={(event) => setProviderDraft((current) => ({ ...current, extension_id: event.target.value }))}
              />
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                Save provider
              </InlineButton>
            </div>
          </form>
        </Surface>
      </div>

      <div className="adminx-users-grid">
        <Surface
          title="Credential maintenance"
          detail="Create a provider credential or rotate an existing secret. Secrets remain write-only and are never returned by the admin API."
        >
          <form className="adminx-form-grid" onSubmit={(event) => void handleCredential(event)}>
            <label className="adminx-field">
              <span>Tenant id</span>
              {snapshot.tenants.length ? (
                <select
                  value={credentialDraft.tenant_id}
                  onChange={(event) => setCredentialDraft((current) => ({ ...current, tenant_id: event.target.value }))}
                >
                  {snapshot.tenants.map((tenant) => (
                    <option key={tenant.id} value={tenant.id}>
                      {tenant.name} ({tenant.id})
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  value={credentialDraft.tenant_id}
                  onChange={(event) => setCredentialDraft((current) => ({ ...current, tenant_id: event.target.value }))}
                  required
                />
              )}
            </label>
            <label className="adminx-field">
              <span>Provider id</span>
              {snapshot.providers.length ? (
                <select
                  value={credentialDraft.provider_id}
                  onChange={(event) => setCredentialDraft((current) => ({ ...current, provider_id: event.target.value }))}
                >
                  {snapshot.providers.map((provider) => (
                    <option key={provider.id} value={provider.id}>
                      {provider.display_name} ({provider.id})
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  value={credentialDraft.provider_id}
                  onChange={(event) => setCredentialDraft((current) => ({ ...current, provider_id: event.target.value }))}
                  required
                />
              )}
            </label>
            <label className="adminx-field">
              <span>Key reference</span>
              <input
                value={credentialDraft.key_reference}
                onChange={(event) => setCredentialDraft((current) => ({ ...current, key_reference: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Secret value</span>
              <input
                value={credentialDraft.secret_value}
                onChange={(event) => setCredentialDraft((current) => ({ ...current, secret_value: event.target.value }))}
                placeholder="paste upstream key or secret"
                type="password"
                required
              />
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                Save credential
              </InlineButton>
              <InlineButton
                onClick={() => setCredentialDraft((current) => ({ ...current, secret_value: '' }))}
              >
                Clear secret
              </InlineButton>
            </div>
          </form>

          <div className="adminx-note">
            <strong>Credential posture</strong>
            <p>
              Backend: {selectedCredential?.secret_backend ?? 'server-managed default'}
              {' | '}
              Storage: {selectedCredential ? credentialStorageLabel(selectedCredential) : 'will be selected by the control plane'}
            </p>
          </div>
        </Surface>

        <Surface
          title="Model maintenance"
          detail="Maintain model exposure by provider, capability set, and context limits."
        >
          <form className="adminx-form-grid" onSubmit={(event) => void handleModel(event)}>
            <label className="adminx-field">
              <span>Model name</span>
              <input
                value={modelDraft.external_name}
                onChange={(event) => setModelDraft((current) => ({ ...current, external_name: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Provider id</span>
              {snapshot.providers.length ? (
                <select
                  value={modelDraft.provider_id}
                  onChange={(event) => setModelDraft((current) => ({ ...current, provider_id: event.target.value }))}
                >
                  {snapshot.providers.map((provider) => (
                    <option key={provider.id} value={provider.id}>
                      {provider.display_name} ({provider.id})
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  value={modelDraft.provider_id}
                  onChange={(event) => setModelDraft((current) => ({ ...current, provider_id: event.target.value }))}
                  required
                />
              )}
            </label>
            <label className="adminx-field">
              <span>Capabilities</span>
              <input
                value={modelDraft.capabilities}
                onChange={(event) => setModelDraft((current) => ({ ...current, capabilities: event.target.value }))}
                required
              />
            </label>
            <label className="adminx-field">
              <span>Context window</span>
              <input
                value={modelDraft.context_window}
                onChange={(event) => setModelDraft((current) => ({ ...current, context_window: event.target.value }))}
                type="number"
              />
            </label>
            <label className="adminx-checkbox">
              <input
                checked={modelDraft.streaming}
                onChange={(event) => setModelDraft((current) => ({ ...current, streaming: event.target.checked }))}
                type="checkbox"
              />
              <span>Streaming enabled</span>
            </label>
            <div className="adminx-form-actions">
              <InlineButton tone="primary" type="submit">
                Save model
              </InlineButton>
            </div>
          </form>

          <div className="adminx-note">
            <strong>Selected provider posture</strong>
            <p>
              Channel: {selectedProvider?.channel_id ?? '-'}
              {' | '}
              Base URL: {selectedProvider?.base_url ?? '-'}
            </p>
          </div>
        </Surface>
      </div>

      <Surface
        title="Credential coverage"
        detail="Inspect which providers are ready for live upstream traffic and which still need secret coverage."
      >
        <div className="adminx-card-grid">
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Covered providers</strong>
              <Pill tone="live">{providersWithCredentials.size}</Pill>
            </div>
            <p>Providers with at least one credential record.</p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Missing coverage</strong>
              <Pill tone={providersWithoutCredentials.length ? 'danger' : 'default'}>
                {providersWithoutCredentials.length}
              </Pill>
            </div>
            <p>
              {providersWithoutCredentials.length
                ? providersWithoutCredentials.map((provider) => provider.display_name).join(', ')
                : 'All providers currently have credential coverage.'}
            </p>
          </article>
          <article className="adminx-mini-card">
            <div className="adminx-row">
              <strong>Orphan credentials</strong>
              <Pill tone={orphanCredentials.length ? 'danger' : 'default'}>
                {orphanCredentials.length}
              </Pill>
            </div>
            <p>
              {orphanCredentials.length
                ? 'Credential rows exist for providers that are no longer present in the registry.'
                : 'No orphaned credential rows detected.'}
            </p>
          </article>
        </div>
      </Surface>

      <Surface
        title="Deletion posture"
        detail="Catalog deletes are live and destructive. Channels cannot be retired while providers still depend on them, and providers should be cleared only after their model variants are removed."
      >
        <div className="adminx-note">
          <strong>Safe order</strong>
          <p>Delete model variants first, then providers, then their channels. Provider and tenant retirement now cascades credential cleanup through the active secret backend so upstream secrets do not linger.</p>
        </div>
      </Surface>

      <Surface title="Channel registry" detail="Live channel catalog from the admin API.">
        <DataTable
          columns={[
            { key: 'id', label: 'Channel id', render: (channel) => <strong>{channel.id}</strong> },
            { key: 'name', label: 'Name', render: (channel) => channel.name },
            {
              key: 'actions',
              label: 'Actions',
              render: (channel) => (
                <div className="adminx-row">
                  <InlineButton onClick={() => setChannelDraft({ id: channel.id, name: channel.name })}>
                    Edit
                  </InlineButton>
                  <InlineButton
                    disabled={channelsWithProviders.has(channel.id)}
                    onClick={() => void onDeleteChannel(channel.id)}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.channels}
          empty="No channels available."
          getKey={(channel) => channel.id}
        />
      </Surface>

      <Surface title="Provider registry" detail="Provider records, bound channels, and credential coverage.">
        <DataTable
          columns={[
            { key: 'id', label: 'Provider id', render: (provider) => <strong>{provider.id}</strong> },
            { key: 'channel', label: 'Channel', render: (provider) => provider.channel_id },
            { key: 'display', label: 'Display', render: (provider) => provider.display_name },
            { key: 'base', label: 'Base URL', render: (provider) => provider.base_url },
            {
              key: 'credentials',
              label: 'Credentials',
              render: (provider) => (
                <Pill tone={providersWithCredentials.has(provider.id) ? 'live' : 'danger'}>
                  {credentialCounts.get(provider.id) ?? 0}
                </Pill>
              ),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (provider) => (
                <div className="adminx-row">
                  <InlineButton
                    onClick={() => setProviderDraft({
                      id: provider.id,
                      channel_id: provider.channel_id,
                      display_name: provider.display_name,
                      adapter_kind: provider.adapter_kind,
                      base_url: provider.base_url,
                      extension_id: provider.extension_id ?? '',
                    })}
                  >
                    Edit
                  </InlineButton>
                  <InlineButton
                    onClick={() => setCredentialDraft((current) => ({
                      ...current,
                      provider_id: provider.id,
                      tenant_id: snapshot.credentials.find((credential) => credential.provider_id === provider.id)?.tenant_id
                        ?? snapshot.tenants[0]?.id
                        ?? current.tenant_id,
                      key_reference: snapshot.credentials.find((credential) => credential.provider_id === provider.id)?.key_reference
                        ?? current.key_reference,
                      secret_value: '',
                    }))}
                  >
                    Rotate secret
                  </InlineButton>
                  <InlineButton
                    disabled={providersWithModels.has(provider.id)}
                    onClick={() => void onDeleteProvider(provider.id)}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.providers}
          empty="No providers available."
          getKey={(provider) => provider.id}
        />
      </Surface>

      <Surface title="Credential inventory" detail="Write-only upstream secret inventory and backend placement metadata.">
        <DataTable
          columns={[
            { key: 'tenant', label: 'Tenant', render: (credential) => <strong>{credential.tenant_id}</strong> },
            {
              key: 'provider',
              label: 'Provider',
              render: (credential) => {
                const provider = snapshot.providers.find((item) => item.id === credential.provider_id);
                return (
                  <div>
                    <strong>{provider?.display_name ?? credential.provider_id}</strong>
                    <div>{credential.provider_id}</div>
                  </div>
                );
              },
            },
            { key: 'reference', label: 'Key reference', render: (credential) => credential.key_reference },
            {
              key: 'backend',
              label: 'Backend',
              render: (credential) => (
                <Pill tone={credential.secret_backend === 'database_encrypted' ? 'live' : 'default'}>
                  {credential.secret_backend}
                </Pill>
              ),
            },
            {
              key: 'storage',
              label: 'Storage',
              render: (credential) => credentialStorageLabel(credential),
            },
            {
              key: 'actions',
              label: 'Actions',
              render: (credential) => (
                <div className="adminx-row">
                  <InlineButton
                    onClick={() => setCredentialDraft({
                      tenant_id: credential.tenant_id,
                      provider_id: credential.provider_id,
                      key_reference: credential.key_reference,
                      secret_value: '',
                    })}
                  >
                    Rotate secret
                  </InlineButton>
                  <InlineButton
                    onClick={() => void onDeleteCredential(
                      credential.tenant_id,
                      credential.provider_id,
                      credential.key_reference,
                    )}
                  >
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.credentials}
          empty="No provider credentials available."
          getKey={(credential) => `${credential.tenant_id}:${credential.provider_id}:${credential.key_reference}`}
        />
      </Surface>

      <Surface title="Model registry" detail="Live model catalog used by the routing layer.">
        <DataTable
          columns={[
            { key: 'name', label: 'Model', render: (model) => <strong>{model.external_name}</strong> },
            { key: 'provider', label: 'Provider', render: (model) => model.provider_id },
            {
              key: 'caps',
              label: 'Capabilities',
              render: (model) => model.capabilities.join(', ') || '-',
            },
            { key: 'streaming', label: 'Streaming', render: (model) => String(model.streaming) },
            {
              key: 'actions',
              label: 'Actions',
              render: (model) => (
                <div className="adminx-row">
                  <InlineButton
                    onClick={() => setModelDraft({
                      external_name: model.external_name,
                      provider_id: model.provider_id,
                      capabilities: model.capabilities.join(', '),
                      streaming: model.streaming,
                      context_window: String(model.context_window ?? ''),
                    })}
                  >
                    Edit
                  </InlineButton>
                  <InlineButton onClick={() => void onDeleteModel(model.external_name, model.provider_id)}>
                    Delete
                  </InlineButton>
                </div>
              ),
            },
          ]}
          rows={snapshot.models}
          empty="No models available."
          getKey={(model) => `${model.external_name}:${model.provider_id}`}
        />
      </Surface>
    </div>
  );
}
