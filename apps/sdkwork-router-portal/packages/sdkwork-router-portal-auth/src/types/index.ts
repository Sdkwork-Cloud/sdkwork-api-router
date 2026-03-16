import type { ReactNode } from 'react';
import type { PortalAnonymousRouteKey, PortalAuthSession } from 'sdkwork-router-portal-types';

export interface AuthShellStoryItem {
  title: string;
  detail: string;
}

export interface AuthShellPreviewItem {
  label: string;
  value: string;
  detail: string;
}

export interface PortalAuthPageProps {
  onAuthenticated: (session: PortalAuthSession) => void;
  onNavigate: (route: PortalAnonymousRouteKey) => void;
}

export interface AuthShellProps {
  eyebrow: string;
  title: string;
  detail: string;
  highlights: AuthShellStoryItem[];
  launchSteps: AuthShellStoryItem[];
  trustSignals: string[];
  status: string;
  previewTitle?: string;
  previewItems?: AuthShellPreviewItem[];
  children: ReactNode;
}
