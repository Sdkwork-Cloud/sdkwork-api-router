import { formatDateTime } from 'sdkwork-router-portal-commons';
import type { PortalWorkspaceSummary } from 'sdkwork-router-portal-types';

import type {
  AccountChecklistItem,
  AccountInsightItem,
  AccountRecoverySignal,
  PasswordPolicyItem,
  PortalAccountViewModel,
} from '../types';

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

export function buildPasswordPolicy(
  nextPassword: string,
  confirmPassword: string,
): PasswordPolicyItem[] {
  return [
    {
      id: 'length',
      label: 'At least 12 characters',
      met: nextPassword.length >= 12,
    },
    {
      id: 'mixed-case',
      label: 'Include uppercase and lowercase letters',
      met: hasUppercase(nextPassword) && hasLowercase(nextPassword),
    },
    {
      id: 'number',
      label: 'Include at least one number',
      met: hasNumber(nextPassword),
    },
    {
      id: 'confirm',
      label: 'Confirmation matches the new password',
      met: Boolean(nextPassword) && passwordsMatch(nextPassword, confirmPassword),
    },
  ];
}

function buildTrustCenter(workspace: PortalWorkspaceSummary | null): AccountInsightItem[] {
  if (!workspace) {
    return [
      {
        id: 'pending',
        title: 'Workspace boundary',
        value: 'Loading',
        detail: 'Trust details will appear once workspace identity finishes loading.',
      },
    ];
  }

  return [
    {
      id: 'identity',
      title: 'Signed-in identity',
      value: workspace.user.email,
      detail: 'This portal session is scoped to the visible workspace owner and remains separate from the admin control plane.',
    },
    {
      id: 'workspace',
      title: 'Workspace boundary',
      value: `${workspace.tenant.name} / ${workspace.project.name}`,
      detail: 'Tenant and project context define the self-service boundary for keys, usage, billing, and account actions.',
    },
    {
      id: 'account-state',
      title: 'Account state',
      value: workspace.user.active ? 'Active' : 'Inactive',
      detail: `Portal access was created ${formatDateTime(workspace.user.created_at_ms)} and is currently ${workspace.user.active ? 'available' : 'restricted'}.`,
    },
  ];
}

function buildSecurityChecklist(
  workspace: PortalWorkspaceSummary | null,
  passwordPolicy: PasswordPolicyItem[],
): AccountChecklistItem[] {
  return [
    {
      id: 'workspace-active',
      title: 'Workspace identity is active',
      detail: workspace?.user.active
        ? 'This account can currently authenticate and manage the user-side workspace boundary.'
        : 'If the account is inactive, restore access before attempting further key or billing actions.',
      complete: Boolean(workspace?.user.active),
    },
    {
      id: 'policy',
      title: 'New password meets portal policy',
      detail: 'Use length, mixed case, and numeric entropy so password rotation materially improves account posture.',
      complete: passwordPolicy.every((item) => item.met),
    },
    {
      id: 'boundary',
      title: 'Admin and portal access stay separated',
      detail: 'User-side password rotation should never be treated as operator credential rotation. Keep the trust boundary explicit.',
      complete: true,
    },
  ];
}

function buildRecoverySignals(workspace: PortalWorkspaceSummary | null): AccountRecoverySignal[] {
  return [
    {
      id: 'password-loss',
      title: 'Password loss should not block workspace continuity',
      detail: 'Use the portal account boundary for sign-in recovery, and keep API keys stored in a secret manager so application traffic is not coupled to browser access.',
    },
    {
      id: 'key-leak',
      title: 'A suspected key leak should trigger credential rotation first',
      detail: 'If a client secret is exposed, issue a replacement key, update integrations, and only then retire the prior credential from deployment pipelines.',
    },
    {
      id: 'billing-block',
      title: workspace?.user.active
        ? 'Quota or billing issues can be resolved without operator UI access'
        : 'Resolve account availability before expecting self-service recovery to succeed',
      detail: 'Credits and billing remain inside the portal boundary, so users can recover runway without crossing into admin tooling.',
    },
  ];
}

export function buildPortalAccountViewModel(
  workspace: PortalWorkspaceSummary | null,
  nextPassword: string,
  confirmPassword: string,
): PortalAccountViewModel {
  const password_policy = buildPasswordPolicy(nextPassword, confirmPassword);

  return {
    trust_center: buildTrustCenter(workspace),
    security_checklist: buildSecurityChecklist(workspace, password_policy),
    recovery_signals: buildRecoverySignals(workspace),
    password_policy,
    can_submit_password: password_policy.every((item) => item.met),
  };
}
