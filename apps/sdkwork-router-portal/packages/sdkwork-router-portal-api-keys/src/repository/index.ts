import {
  createPortalApiKey,
  listPortalApiKeys,
} from 'sdkwork-router-portal-portal-api';
import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

export function loadPortalApiKeys(): Promise<GatewayApiKeyRecord[]> {
  return listPortalApiKeys();
}

export function issuePortalApiKey(environment: string): Promise<CreatedGatewayApiKey> {
  return createPortalApiKey(environment);
}
