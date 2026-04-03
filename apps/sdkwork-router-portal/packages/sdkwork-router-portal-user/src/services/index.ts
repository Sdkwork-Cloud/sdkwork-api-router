import { formatDateTime } from 'sdkwork-router-portal-commons/format-core';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import type {
  PasswordPolicyItem,
  PortalUserPreferenceState,
  PortalUserViewModel,
  UserBindingItem,
  UserFactItem,
  UserPrivacyPreferenceItem,
  UserSecurityItem,
} from '../types';

const PORTAL_USER_CENTER_STORAGE_KEY = 'sdkwork-router-portal.user-center.v1';

export function passwordsMatch(left: string, right: string): boolean {
  return left === right;
}

function hasUppercase(value: string): boolean {
  return /[A-Z]/.test(value);
}

function hasLowercase(value: string): boolean {
  return /[a-z]/.test(value);
}

function hasNumber(value: string): boolean {
  return /\d/.test(value);
}

function storage(): Storage | null {
  if (typeof globalThis.localStorage !== 'undefined') {
    return globalThis.localStorage;
  }

  if (typeof window !== 'undefined' && window.localStorage) {
    return window.localStorage;
  }

  return null;
}

function buildPreferenceScope(workspace: PortalWorkspaceSummary | null): string {
  if (!workspace) {
    return 'anonymous';
  }

  return [
    workspace.user.id,
    workspace.tenant.id,
    workspace.project.id,
  ].join(':');
}

export function createDefaultPortalUserPreferenceState(): PortalUserPreferenceState {
  return {
    phone_number: '',
    wechat_id: '',
    privacy_preferences: {
      'workspace-profile': true,
      'invite-attribution': true,
      'usage-insights': false,
    },
  };
}

function normalizePreferenceState(
  value: Partial<PortalUserPreferenceState> | null | undefined,
): PortalUserPreferenceState {
  const defaults = createDefaultPortalUserPreferenceState();

  return {
    phone_number: value?.phone_number?.trim() ?? defaults.phone_number,
    wechat_id: value?.wechat_id?.trim() ?? defaults.wechat_id,
    privacy_preferences: {
      'workspace-profile':
        value?.privacy_preferences?.['workspace-profile']
        ?? defaults.privacy_preferences['workspace-profile'],
      'invite-attribution':
        value?.privacy_preferences?.['invite-attribution']
        ?? defaults.privacy_preferences['invite-attribution'],
      'usage-insights':
        value?.privacy_preferences?.['usage-insights']
        ?? defaults.privacy_preferences['usage-insights'],
    },
  };
}

function readPreferenceCache(): Record<string, PortalUserPreferenceState> {
  const currentStorage = storage();

  if (!currentStorage) {
    return {};
  }

  const rawValue = currentStorage.getItem(PORTAL_USER_CENTER_STORAGE_KEY);
  if (!rawValue) {
    return {};
  }

  try {
    return JSON.parse(rawValue) as Record<string, PortalUserPreferenceState>;
  } catch {
    return {};
  }
}

function writePreferenceCache(value: Record<string, PortalUserPreferenceState>): void {
  const currentStorage = storage();

  if (!currentStorage) {
    return;
  }

  currentStorage.setItem(PORTAL_USER_CENTER_STORAGE_KEY, JSON.stringify(value));
}

export function readPortalUserPreferenceState(
  workspace: PortalWorkspaceSummary | null,
): PortalUserPreferenceState {
  const cache = readPreferenceCache();
  return normalizePreferenceState(cache[buildPreferenceScope(workspace)]);
}

export function writePortalUserPreferenceState(
  workspace: PortalWorkspaceSummary | null,
  value: PortalUserPreferenceState,
): void {
  const cache = readPreferenceCache();
  cache[buildPreferenceScope(workspace)] = normalizePreferenceState(value);
  writePreferenceCache(cache);
}

export function sanitizePhoneNumber(value: string): string {
  return value.trim();
}

export function sanitizeWeChatId(value: string): string {
  return value.trim();
}

export function buildPasswordPolicy(
  nextPassword: string,
  confirmPassword: string,
): PasswordPolicyItem[] {
  return [
    {
      id: 'length',
      label: translatePortalText('At least 12 characters'),
      met: nextPassword.length >= 12,
    },
    {
      id: 'mixed-case',
      label: translatePortalText('Include uppercase and lowercase letters'),
      met: hasUppercase(nextPassword) && hasLowercase(nextPassword),
    },
    {
      id: 'number',
      label: translatePortalText('Include at least one number'),
      met: hasNumber(nextPassword),
    },
    {
      id: 'confirm',
      label: translatePortalText('Confirmation matches the new password'),
      met: Boolean(nextPassword) && passwordsMatch(nextPassword, confirmPassword),
    },
  ];
}

function maskPhoneNumber(value: string): string {
  const digits = value.replace(/\D+/g, '');

  if (!digits) {
    return translatePortalText('Not bound');
  }

  if (digits.length <= 7) {
    return digits;
  }

  return `${digits.slice(0, 3)}****${digits.slice(-4)}`;
}

function maskWeChatId(value: string): string {
  const normalized = value.trim();

  if (!normalized) {
    return translatePortalText('Not bound');
  }

  if (normalized.length <= 4) {
    return normalized;
  }

  return `${normalized.slice(0, 2)}***${normalized.slice(-2)}`;
}

function buildSummaryCards(
  workspace: PortalWorkspaceSummary | null,
  preferences: PortalUserPreferenceState,
): UserFactItem[] {
  if (!workspace) {
    return [
      {
        id: 'primary-email',
        title: translatePortalText('Primary email'),
        value: translatePortalText('Loading...'),
        detail: translatePortalText(
          'User details will appear after the workspace session is restored.',
        ),
      },
      {
        id: 'phone',
        title: translatePortalText('Phone'),
        value: translatePortalText('Not bound'),
        detail: translatePortalText('Add a recovery phone for password resets and important notices.'),
      },
      {
        id: 'wechat',
        title: translatePortalText('WeChat'),
        value: translatePortalText('Not bound'),
        detail: translatePortalText('Connect WeChat for faster sign-in confirmation and workspace notifications.'),
      },
    ];
  }

  return [
    {
      id: 'primary-email',
      title: translatePortalText('Primary email'),
      value: workspace.user.email,
      detail: translatePortalText(
        'Email remains the primary sign-in identity for recovery and password rotation.',
      ),
    },
    {
      id: 'phone',
      title: translatePortalText('Phone'),
      value: maskPhoneNumber(preferences.phone_number),
      detail: preferences.phone_number
        ? translatePortalText(
            'Recovery phone is connected for important notices and access verification.',
          )
        : translatePortalText('Add a recovery phone for password resets and important notices.'),
    },
    {
      id: 'wechat',
      title: translatePortalText('WeChat'),
      value: maskWeChatId(preferences.wechat_id),
      detail: preferences.wechat_id
        ? translatePortalText(
            'WeChat is connected for faster sign-in confirmation and workspace notifications.',
          )
        : translatePortalText(
            'Connect WeChat for faster sign-in confirmation and workspace notifications.',
          ),
    },
  ];
}

function buildBindingItems(
  preferences: PortalUserPreferenceState,
): UserBindingItem[] {
  return [
    {
      id: 'phone',
      title: translatePortalText('Phone binding'),
      value: maskPhoneNumber(preferences.phone_number),
      detail: preferences.phone_number
        ? translatePortalText(
            'Recovery phone is connected and can support important access notifications.',
          )
        : translatePortalText(
            'Bind a phone number to improve password recovery and important security notices.',
          ),
      action_label: preferences.phone_number
        ? translatePortalText('Update phone')
        : translatePortalText('Bind phone'),
      connected: Boolean(preferences.phone_number),
    },
    {
      id: 'wechat',
      title: translatePortalText('WeChat binding'),
      value: maskWeChatId(preferences.wechat_id),
      detail: preferences.wechat_id
        ? translatePortalText(
            'WeChat is connected and ready for trusted sign-in confirmation and notifications.',
          )
        : translatePortalText(
            'Bind WeChat for trusted sign-in confirmation and workspace notifications.',
          ),
      action_label: preferences.wechat_id
        ? translatePortalText('Update WeChat')
        : translatePortalText('Bind WeChat'),
      connected: Boolean(preferences.wechat_id),
    },
  ];
}

function buildPrivacyPreferences(
  preferences: PortalUserPreferenceState,
): UserPrivacyPreferenceItem[] {
  return [
    {
      id: 'workspace-profile',
      title: translatePortalText('Show full profile inside workspace'),
      description: translatePortalText(
        'Allow teammates to see your full profile details inside shared workspace views.',
      ),
      enabled: preferences.privacy_preferences['workspace-profile'],
    },
    {
      id: 'invite-attribution',
      title: translatePortalText('Allow invite attribution'),
      description: translatePortalText(
        'Keep your profile visible when invite rewards and team activations are recorded.',
      ),
      enabled: preferences.privacy_preferences['invite-attribution'],
    },
    {
      id: 'usage-insights',
      title: translatePortalText('Personalized usage insights'),
      description: translatePortalText(
        'Allow the portal to adapt usage insights and shortcuts to your recent activity.',
      ),
      enabled: preferences.privacy_preferences['usage-insights'],
    },
  ];
}

function buildSecurityItems(
  workspace: PortalWorkspaceSummary | null,
  preferences: PortalUserPreferenceState,
  passwordPolicy: PasswordPolicyItem[],
): UserSecurityItem[] {
  const canRecover = Boolean(workspace?.user.email || preferences.phone_number);

  return [
    {
      id: 'access',
      title: translatePortalText('Sign-in access'),
      detail: workspace?.user.active
        ? translatePortalText(
            'This account can currently sign in and use protected workspace actions.',
          )
        : translatePortalText(
            'Restore account access before expecting password rotation or protected actions to succeed.',
          ),
      status_label: workspace?.user.active
        ? translatePortalText('Active')
        : translatePortalText('Review'),
      tone: workspace?.user.active ? 'success' : 'warning',
    },
    {
      id: 'password',
      title: translatePortalText('Password readiness'),
      detail: translatePortalText('Password requirements are visible before rotation is submitted.'),
      status_label: passwordPolicy.every((item) => item.met)
        ? translatePortalText('Ready')
        : translatePortalText('Needs action'),
      tone: passwordPolicy.every((item) => item.met) ? 'success' : 'warning',
    },
    {
      id: 'recovery',
      title: translatePortalText('Recovery channel'),
      detail: canRecover
        ? translatePortalText('A verified email or phone keeps account recovery available.')
        : translatePortalText('Add a recovery phone so account access can be restored quickly.'),
      status_label: canRecover
        ? translatePortalText('Protected')
        : translatePortalText('Needs setup'),
      tone: canRecover ? 'success' : 'warning',
    },
    {
      id: 'wechat',
      title: translatePortalText('WeChat confirmation'),
      detail: preferences.wechat_id
        ? translatePortalText('WeChat is connected for an additional trust and notification channel.')
        : translatePortalText('Add WeChat to broaden trusted sign-in and service notifications.'),
      status_label: preferences.wechat_id
        ? translatePortalText('Connected')
        : translatePortalText('Needs setup'),
      tone: preferences.wechat_id ? 'success' : 'warning',
    },
  ];
}

export function buildPortalUserViewModel(
  workspace: PortalWorkspaceSummary | null,
  preferences: PortalUserPreferenceState,
  nextPassword: string,
  confirmPassword: string,
): PortalUserViewModel {
  const password_policy = buildPasswordPolicy(nextPassword, confirmPassword);

  return {
    summary_cards: buildSummaryCards(workspace, preferences),
    binding_items: buildBindingItems(preferences),
    privacy_preferences: buildPrivacyPreferences(preferences),
    security_items: buildSecurityItems(workspace, preferences, password_policy),
    password_policy,
    can_submit_password: password_policy.every((item) => item.met),
  };
}

export function buildUserWorkspaceSummary(workspace: PortalWorkspaceSummary | null): UserFactItem[] {
  if (!workspace) {
    return [
      {
        id: 'workspace',
        title: translatePortalText('Current workspace'),
        value: translatePortalText('Loading...'),
        detail: translatePortalText('Workspace identity becomes available after the session is restored.'),
      },
      {
        id: 'joined',
        title: translatePortalText('User since'),
        value: translatePortalText('Loading...'),
        detail: translatePortalText('Joined date becomes available after the session is restored.'),
      },
    ];
  }

  return [
    {
      id: 'workspace',
      title: translatePortalText('Current workspace'),
      value: `${workspace.tenant.name} / ${workspace.project.name}`,
      detail: translatePortalText('See which tenant and project this user currently operates.'),
    },
    {
      id: 'joined',
      title: translatePortalText('User since'),
      value: formatDateTime(workspace.user.created_at_ms),
      detail: translatePortalText('Joined date keeps workspace access age visible during security review.'),
    },
  ];
}
