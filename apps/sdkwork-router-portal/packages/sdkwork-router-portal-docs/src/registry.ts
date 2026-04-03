export interface PortalDocRegistryAction {
  label: string;
  href: string;
}

export interface PortalDocRegistryGroup {
  id: string;
  title: string;
  description: string;
  primaryAction: PortalDocRegistryAction;
  secondaryAction: PortalDocRegistryAction;
  entries: Array<{
    id: string;
    title: string;
    detail: string;
  }>;
}

export const portalDocsRegistry: PortalDocRegistryGroup[] = [
  {
    id: 'quickstart',
    title: 'Quickstart',
    description: 'Start the gateway, desktop runtime, and workspace access flows.',
    primaryAction: {
      label: 'Open quickstart',
      href: '/downloads',
    },
    secondaryAction: {
      label: 'Open console',
      href: '/console/dashboard',
    },
    entries: [
      {
        id: 'first-request',
        title: 'First request',
        detail: 'Bring up the router, create an API key, and verify a request against the shared gateway.',
      },
      {
        id: 'desktop-setup',
        title: 'Desktop setup',
        detail: 'Install the desktop runtime, sign in, and verify local product health before launch.',
      },
    ],
  },
  {
    id: 'integrations',
    title: 'SDKs & Integration',
    description: 'Connect SDKs, routing policies, providers, and workspace credentials.',
    primaryAction: {
      label: 'Explore models',
      href: '/models',
    },
    secondaryAction: {
      label: 'Open console',
      href: '/console/dashboard',
    },
    entries: [
      {
        id: 'provider-routing',
        title: 'Model routing',
        detail: 'Map providers, fallback posture, and workspace routing preferences into a single policy flow.',
      },
      {
        id: 'sdk-clients',
        title: 'SDK clients',
        detail: 'Use compatible Claude, Gemini, and OpenAI-style clients against the unified router endpoints.',
      },
    ],
  },
  {
    id: 'reference',
    title: 'Reference',
    description: 'API surface details, field definitions, and request lifecycle facts.',
    primaryAction: {
      label: 'Explore models',
      href: '/models',
    },
    secondaryAction: {
      label: 'Software Downloads',
      href: '/downloads',
    },
    entries: [
      {
        id: 'api-reference',
        title: 'API Reference',
        detail: 'Understand the gateway, portal, auth, usage, and billing API surfaces in one place.',
      },
      {
        id: 'model-catalog',
        title: 'Model matrix',
        detail: 'Cross-check provider coverage, modality support, and pricing posture before rollout.',
      },
    ],
  },
  {
    id: 'operations',
    title: 'Operations',
    description: 'Runbooks for telemetry review, quotas, billing posture, and software rollout.',
    primaryAction: {
      label: 'Open console',
      href: '/console/dashboard',
    },
    secondaryAction: {
      label: 'Open install guide',
      href: '/downloads',
    },
    entries: [
      {
        id: 'usage-governance',
        title: 'Usage governance',
        detail: 'Review usage slices, detect cost spikes, and align API keys with project boundaries.',
      },
      {
        id: 'billing-runway',
        title: 'Billing runway',
        detail: 'Track credits, checkout posture, recharge actions, and financial account visibility.',
      },
    ],
  },
];
