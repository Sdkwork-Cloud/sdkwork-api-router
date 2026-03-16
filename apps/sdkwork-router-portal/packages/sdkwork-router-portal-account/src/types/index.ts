import type { PortalRouteKey, PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

export interface PortalAccountPageProps {
  workspace: PortalWorkspaceSummary | null;
  onNavigate: (route: PortalRouteKey) => void;
}

export interface AccountInsightItem {
  id: string;
  title: string;
  value: string;
  detail: string;
}

export interface AccountChecklistItem {
  id: string;
  title: string;
  detail: string;
  complete: boolean;
}

export interface AccountRecoverySignal {
  id: string;
  title: string;
  detail: string;
}

export interface PasswordPolicyItem {
  id: string;
  label: string;
  met: boolean;
}

export interface PortalAccountViewModel {
  trust_center: AccountInsightItem[];
  security_checklist: AccountChecklistItem[];
  recovery_signals: AccountRecoverySignal[];
  password_policy: PasswordPolicyItem[];
  can_submit_password: boolean;
}
