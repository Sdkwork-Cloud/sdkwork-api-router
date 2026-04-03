import type { PortalRouteKey, PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

export interface PortalUserPageProps {
  workspace: PortalWorkspaceSummary | null;
  onNavigate: (route: PortalRouteKey) => void;
}

export type PortalUserPrivacyPreferenceId =
  | 'workspace-profile'
  | 'invite-attribution'
  | 'usage-insights';

export interface UserFactItem {
  id: string;
  title: string;
  value: string;
  detail: string;
}

export interface UserBindingItem {
  id: string;
  title: string;
  value: string;
  detail: string;
  action_label: string;
  connected: boolean;
}

export interface UserPrivacyPreferenceItem {
  id: PortalUserPrivacyPreferenceId;
  title: string;
  description: string;
  enabled: boolean;
}

export interface UserSecurityItem {
  id: string;
  title: string;
  detail: string;
  status_label: string;
  tone: 'success' | 'warning';
}

export interface PasswordPolicyItem {
  id: string;
  label: string;
  met: boolean;
}

export interface PortalUserPreferenceState {
  phone_number: string;
  wechat_id: string;
  privacy_preferences: Record<PortalUserPrivacyPreferenceId, boolean>;
}

export interface PortalUserViewModel {
  summary_cards: UserFactItem[];
  binding_items: UserBindingItem[];
  privacy_preferences: UserPrivacyPreferenceItem[];
  security_items: UserSecurityItem[];
  password_policy: PasswordPolicyItem[];
  can_submit_password: boolean;
}
