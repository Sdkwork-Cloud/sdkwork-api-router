export type PortalAnonymousRouteKey = 'login' | 'register';
export type PortalRouteKey = 'dashboard' | 'api-keys' | 'usage' | 'credits' | 'billing' | 'account';
export type PortalDataSource = 'live' | 'workspace_seed';

export interface PortalRouteDefinition {
  key: PortalRouteKey;
  label: string;
  eyebrow: string;
  detail: string;
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

export interface PortalUserProfile {
  id: string;
  email: string;
  display_name: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
  created_at_ms: number;
}

export interface PortalAuthSession {
  token: string;
  user: PortalUserProfile;
  workspace: {
    tenant_id: string;
    project_id: string;
  };
}

export interface PortalWorkspaceSummary {
  user: PortalUserProfile;
  tenant: TenantRecord;
  project: ProjectRecord;
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

export interface UsageRecord {
  project_id: string;
  model: string;
  provider: string;
  units: number;
  amount: number;
  created_at_ms: number;
}

export interface UsageSummary {
  total_requests: number;
  project_count: number;
  model_count: number;
  provider_count: number;
  projects: Array<{ project_id: string; request_count: number }>;
  providers: Array<{ provider: string; request_count: number; project_count: number }>;
  models: Array<{ model: string; request_count: number; provider_count: number }>;
}

export interface ProjectBillingSummary {
  project_id: string;
  entry_count: number;
  used_units: number;
  booked_amount: number;
  quota_policy_id?: string | null;
  quota_limit_units?: number | null;
  remaining_units?: number | null;
  exhausted: boolean;
}

export interface LedgerEntry {
  project_id: string;
  units: number;
  amount: number;
}

export interface PortalDashboardSummary {
  workspace: PortalWorkspaceSummary;
  usage_summary: UsageSummary;
  billing_summary: ProjectBillingSummary;
  recent_requests: UsageRecord[];
  api_key_count: number;
}

export interface SubscriptionPlan {
  id: string;
  name: string;
  price_label: string;
  cadence: string;
  included_units: number;
  highlight: string;
  features: string[];
  cta: string;
  source: PortalDataSource;
}

export interface RechargePack {
  id: string;
  label: string;
  points: number;
  price_label: string;
  note: string;
  source: PortalDataSource;
}

export interface CouponOffer {
  code: string;
  title: string;
  benefit: string;
  description: string;
  bonus_units: number;
  source: PortalDataSource;
}
