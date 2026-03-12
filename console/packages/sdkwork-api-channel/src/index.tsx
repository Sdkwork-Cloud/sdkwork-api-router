import { useEffect, useState } from 'react';
import { listChannels, listModels, listProviders } from 'sdkwork-api-admin-sdk';
import type { ChannelRecord, ModelCatalogRecord, ProxyProviderRecord } from 'sdkwork-api-types';

interface ChannelSnapshot {
  channels: ChannelRecord[];
  providers: ProxyProviderRecord[];
  models: ModelCatalogRecord[];
}

const emptySnapshot: ChannelSnapshot = {
  channels: [],
  providers: [],
  models: [],
};

export function ChannelRegistryPage() {
  const [snapshot, setSnapshot] = useState<ChannelSnapshot>(emptySnapshot);
  const [status, setStatus] = useState('Loading channel registry...');

  useEffect(() => {
    let cancelled = false;

    void Promise.all([listChannels(), listProviders(), listModels()])
      .then(([channels, providers, models]) => {
        if (cancelled) {
          return;
        }

        setSnapshot({ channels, providers, models });
        setStatus('Catalog registry synchronized from admin API.');
      })
      .catch(() => {
        if (!cancelled) {
          setStatus('Admin API unavailable. Channel registry is operating in offline view.');
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Channel Mesh</p>
          <h2>Channels, providers, and exposed models</h2>
        </div>
        <p className="status">{status}</p>
      </div>

      <div className="metric-grid">
        <article className="metric-card">
          <span className="metric-label">Channels</span>
          <strong>{snapshot.channels.length}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Proxy Providers</span>
          <strong>{snapshot.providers.length}</strong>
        </article>
        <article className="metric-card">
          <span className="metric-label">Catalog Models</span>
          <strong>{snapshot.models.length}</strong>
        </article>
      </div>

      <div className="detail-grid">
        <article className="detail-card">
          <h3>Channels</h3>
          <ul className="compact-list">
            {snapshot.channels.map((channel) => (
              <li key={channel.id}>
                <strong>{channel.name}</strong>
                <span>{channel.id}</span>
              </li>
            ))}
            {!snapshot.channels.length && <li className="empty">No channels registered yet.</li>}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Providers</h3>
          <ul className="compact-list">
            {snapshot.providers.map((provider) => (
              <li key={provider.id}>
                <strong>{provider.display_name}</strong>
                <span>{provider.channel_id}</span>
              </li>
            ))}
            {!snapshot.providers.length && (
              <li className="empty">No proxy providers connected yet.</li>
            )}
          </ul>
        </article>

        <article className="detail-card">
          <h3>Models</h3>
          <ul className="compact-list">
            {snapshot.models.map((model) => (
              <li key={`${model.external_name}:${model.provider_id}`}>
                <strong>{model.external_name}</strong>
                <span>{model.provider_id}</span>
              </li>
            ))}
            {!snapshot.models.length && <li className="empty">No models published yet.</li>}
          </ul>
        </article>
      </div>
    </section>
  );
}
