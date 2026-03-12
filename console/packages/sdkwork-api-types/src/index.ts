export type RuntimeMode = 'server' | 'embedded';

export interface ConsoleSection {
  id: string;
  title: string;
}

export interface TenantRecord {
  id: string;
  name: string;
}

export interface ProjectRecord {
  tenant_id: string;
  id: string;
  name: string;
}

export interface GatewayApiKeyRecord {
  tenant_id: string;
  project_id: string;
  environment: string;
  hashed_key: string;
  active: boolean;
}

export interface CreatedGatewayApiKey {
  plaintext: string;
  hashed: string;
  tenant_id: string;
  project_id: string;
  environment: string;
}

export interface ChannelRecord {
  id: string;
  name: string;
}

export interface ProxyProviderRecord {
  id: string;
  channel_id: string;
  display_name: string;
}

export interface ModelCatalogRecord {
  external_name: string;
  provider_id: string;
}

export interface RoutingSimulationResult {
  selected_provider_id: string;
  candidate_ids: string[];
}

export interface UsageRecord {
  project_id: string;
  model: string;
  provider: string;
}

export interface LedgerEntry {
  project_id: string;
  units: number;
  amount: number;
}
