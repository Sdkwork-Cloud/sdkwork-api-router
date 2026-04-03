import type { ProxyProviderRecord } from 'sdkwork-router-admin-types';

import type { GatewayRouteInventoryRow } from '../../services/gatewayViewService';

export type ProviderDraft = {
  id: string;
  display_name: string;
  adapter_kind: string;
  base_url: string;
  extension_id: string;
  primary_channel_id: string;
  bound_channel_ids: string[];
};

export type HealthFilter = 'all' | 'healthy' | 'degraded';

function collectProviderChannelIds(provider: ProxyProviderRecord): string[] {
  const ids = new Set<string>([provider.channel_id]);

  for (const binding of provider.channel_bindings) {
    ids.add(binding.channel_id);
  }

  return Array.from(ids);
}

export function emptyProviderDraft(defaultChannelId: string): ProviderDraft {
  return {
    id: '',
    display_name: '',
    adapter_kind: 'openai',
    base_url: '',
    extension_id: '',
    primary_channel_id: defaultChannelId,
    bound_channel_ids: defaultChannelId ? [defaultChannelId] : [],
  };
}

export function providerDraftFromRecord(
  provider: ProxyProviderRecord,
): ProviderDraft {
  return {
    id: provider.id,
    display_name: provider.display_name,
    adapter_kind: provider.adapter_kind,
    base_url: provider.base_url,
    extension_id: provider.extension_id ?? '',
    primary_channel_id: provider.channel_id,
    bound_channel_ids: collectProviderChannelIds(provider),
  };
}

export function formatChannels(row: GatewayRouteInventoryRow): string {
  return (
    row.channels.map((channel) => channel.name).join(', ')
    || row.primary_channel_name
  );
}

export function statusVariant(row: GatewayRouteInventoryRow) {
  return row.healthy ? 'success' : 'danger';
}
