import type { CreatedGatewayApiKey, GatewayApiKeyRecord, PortalRouteKey } from 'sdkwork-router-portal-types';

export interface PortalApiKeysPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface ApiKeyEnvironmentSummary {
  environment: string;
  total: number;
  active: number;
}

export interface ApiKeyEnvironmentStrategyItem {
  environment: string;
  status: string;
  detail: string;
  recommended: boolean;
}

export interface ApiKeyRotationStep {
  id: string;
  title: string;
  detail: string;
}

export interface ApiKeyGuardrail {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface PortalApiKeysPageViewModel {
  keys: GatewayApiKeyRecord[];
  environment_summaries: ApiKeyEnvironmentSummary[];
  environment_strategy: ApiKeyEnvironmentStrategyItem[];
  rotation_checklist: ApiKeyRotationStep[];
  guardrails: ApiKeyGuardrail[];
  created_key: CreatedGatewayApiKey | null;
  quickstart_snippet: string | null;
}
