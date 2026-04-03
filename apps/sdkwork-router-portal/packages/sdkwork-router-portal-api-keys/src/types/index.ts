import type {
  BillingEventSummary,
  PortalRouteKey,
} from 'sdkwork-router-portal-types';

export interface PortalApiKeysPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export type PortalApiKeyCreateMode = 'system-generated' | 'custom';

export interface PortalApiKeyEnvironmentOption {
  value: string;
  label: string;
  detail: string;
}

export interface PortalApiKeyFilterState {
  searchQuery: string;
  environment: string;
  groupId: string;
}

export interface PortalApiKeyCreateFormState {
  label: string;
  keyMode: PortalApiKeyCreateMode;
  customKey: string;
  environment: string;
  customEnvironment: string;
  apiKeyGroupId: string;
  expiresAt: string;
  notes: string;
}

export interface PortalApiKeyGroupOption {
  value: string;
  label: string;
  detail: string;
}

export interface PortalApiKeyUsagePreview {
  title: string;
  detail: string;
  curlExample: string | null;
  authorizationHeader: string | null;
}

export interface PortalApiKeyWorkbenchData {
  api_keys: import('sdkwork-router-portal-types').GatewayApiKeyRecord[];
  api_key_groups: import('sdkwork-router-portal-types').ApiKeyGroupRecord[];
  billing_event_summary: BillingEventSummary;
}

export interface PortalApiKeyGovernanceSummary {
  active_group_count: number;
  grouped_key_count: number;
  ungrouped_key_count: number;
  routing_profile_bound_group_count: number;
}

export interface PortalApiKeyChargebackLeader {
  api_key_group_id: string | null;
  group_name: string;
  total_customer_charge: number;
  total_upstream_cost: number;
  request_count: number;
  event_count: number;
  default_accounting_mode: string | null;
  default_routing_profile_id: string | null;
}

export interface PortalApiKeyGovernanceViewModel {
  summary: PortalApiKeyGovernanceSummary;
  leading_chargeback_group: PortalApiKeyChargebackLeader | null;
  dominant_default_accounting_mode: string | null;
}
