import {
  createPortalApiKey,
  createPortalApiKeyGroup,
  deletePortalApiKey,
  deletePortalApiKeyGroup,
  getPortalBillingEventSummary,
  listPortalApiKeys,
  listPortalApiKeyGroups,
  listPortalRoutingProfiles,
  updatePortalApiKeyStatus,
  updatePortalApiKeyGroup,
  updatePortalApiKeyGroupStatus,
} from 'sdkwork-router-portal-portal-api';
import type {
  ApiKeyGroupRecord,
  BillingEventSummary,
  CreatedGatewayApiKey,
  GatewayApiKeyRecord,
  RoutingProfileRecord,
} from 'sdkwork-router-portal-types';
import type { PortalApiKeyWorkbenchData } from '../types';

export function loadPortalApiKeys(): Promise<GatewayApiKeyRecord[]> {
  return listPortalApiKeys();
}

export function loadPortalApiKeyGroups(): Promise<ApiKeyGroupRecord[]> {
  return listPortalApiKeyGroups();
}

export function loadPortalRoutingProfiles(): Promise<RoutingProfileRecord[]> {
  return listPortalRoutingProfiles();
}

export async function loadPortalApiKeyWorkbenchData(): Promise<PortalApiKeyWorkbenchData> {
  const [api_keys, api_key_groups, billing_event_summary] = await Promise.all([
    listPortalApiKeys(),
    listPortalApiKeyGroups(),
    getPortalBillingEventSummary(),
  ]);

  return {
    api_keys,
    api_key_groups,
    billing_event_summary,
  };
}

export function issuePortalApiKey(input: {
  environment: string;
  label: string;
  api_key?: string | null;
  api_key_group_id?: string | null;
  notes?: string | null;
  expires_at_ms?: number | null;
}): Promise<CreatedGatewayApiKey> {
  return createPortalApiKey(input);
}

export function setPortalApiKeyActive(
  hashedKey: string,
  active: boolean,
): Promise<GatewayApiKeyRecord> {
  return updatePortalApiKeyStatus(hashedKey, active);
}

export function removePortalApiKey(hashedKey: string): Promise<void> {
  return deletePortalApiKey(hashedKey);
}

export function issuePortalApiKeyGroup(input: {
  environment: string;
  name: string;
  slug?: string | null;
  description?: string | null;
  color?: string | null;
  default_capability_scope?: string | null;
  default_accounting_mode?: string | null;
  default_routing_profile_id?: string | null;
}): Promise<ApiKeyGroupRecord> {
  return createPortalApiKeyGroup(input);
}

export function editPortalApiKeyGroup(
  groupId: string,
  input: {
    environment: string;
    name: string;
    slug?: string | null;
    description?: string | null;
    color?: string | null;
    default_capability_scope?: string | null;
    default_accounting_mode?: string | null;
    default_routing_profile_id?: string | null;
  },
): Promise<ApiKeyGroupRecord> {
  return updatePortalApiKeyGroup(groupId, input);
}

export function setPortalApiKeyGroupActive(
  groupId: string,
  active: boolean,
): Promise<ApiKeyGroupRecord> {
  return updatePortalApiKeyGroupStatus(groupId, active);
}

export function removePortalApiKeyGroup(groupId: string): Promise<void> {
  return deletePortalApiKeyGroup(groupId);
}
