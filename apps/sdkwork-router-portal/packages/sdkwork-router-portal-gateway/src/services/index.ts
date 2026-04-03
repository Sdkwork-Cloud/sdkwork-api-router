import type {
  PortalCommerceCatalog,
  PortalCommerceMembership,
  PortalDashboardSummary,
  PortalDesktopRuntimeSnapshot,
  PortalGatewayRateLimitSnapshot,
  PortalRuntimeHealthSnapshot,
  PortalRuntimeServiceHealth,
} from 'sdkwork-router-portal-types';
import { formatUnits } from 'sdkwork-router-portal-commons/format-core';
import { translatePortalText } from 'sdkwork-router-portal-commons/i18n-core';

import type { GatewayCommandCenterSnapshot } from '../types';

function joinUrl(baseUrl: string, path: string): string {
  const normalizedBase = baseUrl.replace(/\/+$/g, '');
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${normalizedBase}${normalizedPath}`;
}

function buildRuntimeCards(input: {
  desktopRuntime: PortalDesktopRuntimeSnapshot | null;
  gatewayBaseUrl: string;
}) {
  const { desktopRuntime, gatewayBaseUrl } = input;
  if (!desktopRuntime) {
    return [
      {
        id: 'runtime-context',
        label: translatePortalText('Launch context'),
        value: translatePortalText('Browser or hosted portal session'),
        detail: translatePortalText(
          'Desktop runtime evidence becomes live when the command center runs inside the portal desktop shell with the Tauri bridge available.',
        ),
      },
      {
        id: 'runtime-gateway-base',
        label: translatePortalText('Gateway base'),
        value: gatewayBaseUrl,
        detail: translatePortalText(
          'The current session can still verify the shared gateway surface even when local loopback binds are not exposed to the browser.',
        ),
      },
      {
        id: 'runtime-desktop-bridge',
        label: translatePortalText('Desktop bridge'),
        value: translatePortalText('Unavailable in this session'),
        detail: translatePortalText(
          'Use the desktop app when you need the exact local web, gateway, admin, and portal bind evidence owned by the integrated runtime.',
        ),
      },
    ];
  }

  return [
    {
      id: 'runtime-mode',
      label: translatePortalText('Launch mode'),
      value: translatePortalText('Desktop embedded runtime'),
      detail: translatePortalText(
        'The current portal session is backed by the local product runtime rather than a remote browser-only proxy path.',
      ),
    },
    {
      id: 'runtime-roles',
      label: translatePortalText('Roles online'),
      value: desktopRuntime.roles.join(', '),
      detail: translatePortalText(
        'Desktop mode keeps web, gateway, admin, and portal online together so onboarding and control-plane work stay on one machine.',
      ),
    },
    {
      id: 'runtime-public-bind',
      label: translatePortalText('Public web bind'),
      value: desktopRuntime.publicBindAddr ?? translatePortalText('Unavailable'),
      detail:
        desktopRuntime.publicBaseUrl
          ? translatePortalText('The embedded web host currently resolves to {baseUrl}.', {
              baseUrl: desktopRuntime.publicBaseUrl,
            })
          : translatePortalText('The public web host did not expose a resolved base URL.'),
    },
    {
      id: 'runtime-gateway-bind',
      label: translatePortalText('Gateway bind'),
      value: desktopRuntime.gatewayBindAddr ?? translatePortalText('Unavailable'),
      detail: translatePortalText(
        'Local gateway traffic remains inside the same product runtime that also fronts the admin and portal APIs.',
      ),
    },
    {
      id: 'runtime-admin-bind',
      label: translatePortalText('Admin bind'),
      value: desktopRuntime.adminBindAddr ?? translatePortalText('Unavailable'),
      detail: translatePortalText(
        'The admin control plane is started by the same desktop runtime and remains isolated to loopback in desktop mode.',
      ),
    },
    {
      id: 'runtime-portal-bind',
      label: translatePortalText('Portal bind'),
      value: desktopRuntime.portalBindAddr ?? translatePortalText('Unavailable'),
      detail: translatePortalText(
        'The portal API stays local, so authentication, commerce reads, and command-center posture are all sourced from the same workstation runtime.',
      ),
    },
  ];
}

function buildCommerceCatalogCards(
  catalog: PortalCommerceCatalog,
  membership: PortalCommerceMembership | null,
) {
  const liveCouponCount = catalog.coupons.filter((coupon) => coupon.source === 'live').length;
  const couponCodes = catalog.coupons
    .slice(0, 3)
    .map((coupon) => coupon.code)
    .join(', ');

  return [
    {
      id: 'catalog-plans',
      label: translatePortalText('Subscription plans'),
      value: translatePortalText('{count} plan(s)', {
        count: formatUnits(catalog.plans.length),
      }),
      detail: translatePortalText(
        'Commerce catalog membership and subscription posture is now exposed through the live commerce catalog instead of being limited to static launch copy.',
      ),
    },
    {
      id: 'catalog-packs',
      label: translatePortalText('Recharge packs'),
      value: translatePortalText('{count} pack(s)', {
        count: formatUnits(catalog.packs.length),
      }),
      detail: translatePortalText(
        'Top-up and recharge options are visible as live catalog entries, which keeps future checkout work anchored to a stable contract.',
      ),
    },
    {
      id: 'catalog-membership',
      label: translatePortalText('Active membership'),
      value: membership ? membership.plan_name : translatePortalText('No active membership'),
      detail: membership
        ? translatePortalText(
            '{planName} is active with {includedUnits} included units on a {cadence} cadence.',
            {
              planName: membership.plan_name,
              includedUnits: formatUnits(membership.included_units),
              cadence: membership.cadence,
            },
          )
        : translatePortalText(
            'Subscription purchases will promote the current workspace into an explicit active membership state instead of leaving plans as catalog-only entries.',
          ),
    },
    {
      id: 'catalog-coupons',
      label: translatePortalText('Coupon offers'),
      value: translatePortalText('{count} coupon(s)', {
        count: formatUnits(catalog.coupons.length),
      }),
      detail:
        liveCouponCount > 0
          ? translatePortalText('Active live campaigns currently visible: {couponCodes}.', {
              couponCodes: couponCodes || translatePortalText('available'),
            })
          : translatePortalText(
              'No live campaigns were found, so the catalog falls back to the seeded launch offers.',
            ),
    },
  ];
}

function buildRateLimitCards(snapshot: PortalGatewayRateLimitSnapshot) {
  return [
    {
      id: 'rate-limit-policies',
      label: translatePortalText('Policy roster'),
      value: translatePortalText('{count} policy(s)', {
        count: formatUnits(snapshot.policy_count),
      }),
      detail:
        snapshot.policy_count > 0
          ? translatePortalText(
              '{count} active policy(s) are currently shaping the gateway posture for project {projectId}.',
              {
                count: formatUnits(snapshot.active_policy_count),
                projectId: snapshot.project_id,
              },
            )
          : translatePortalText('No project-scoped rate-limit policies are configured yet.'),
    },
    {
      id: 'rate-limit-windows',
      label: translatePortalText('Live windows'),
      value: translatePortalText('{count} window(s)', {
        count: formatUnits(snapshot.window_count),
      }),
      detail:
        snapshot.window_count > 0
          ? translatePortalText(
              'Window snapshots are being tracked directly from the control plane so the portal can show live pressure instead of static policy text.',
            )
          : translatePortalText('No live window snapshots are currently available for this project.'),
    },
    {
      id: 'rate-limit-exceeded',
      label: translatePortalText('Over-limit windows'),
      value: translatePortalText('{count} flagged', {
        count: formatUnits(snapshot.exceeded_window_count),
      }),
      detail:
        snapshot.exceeded_window_count > 0
          ? translatePortalText(
              'At least one active window is currently over limit, so the gateway posture should be treated as under pressure.',
            )
          : translatePortalText('No active window is currently over limit.'),
    },
    {
      id: 'rate-limit-summary',
      label: translatePortalText('Operator headline'),
      value: snapshot.headline,
      detail: snapshot.detail,
    },
  ];
}

function buildDefaultRateLimitSnapshot(projectId: string): PortalGatewayRateLimitSnapshot {
  return {
    project_id: projectId,
    policy_count: 0,
    active_policy_count: 0,
    window_count: 0,
    exceeded_window_count: 0,
    headline: translatePortalText('No project-scoped rate-limit policy is configured yet.'),
    detail: translatePortalText(
      'The command center is falling back to an empty rate-limit posture because no policy snapshot is available for the current workspace yet.',
    ),
    generated_at_ms: Date.now(),
    policies: [],
    windows: [],
  };
}

function buildServiceHealthSummary(runtimeHealth: PortalRuntimeHealthSnapshot) {
  const healthyCount = runtimeHealth.services.filter((service) => service.status === 'healthy').length;
  const degradedCount = runtimeHealth.services.filter((service) => service.status === 'degraded').length;
  const unreachableCount = runtimeHealth.services.filter((service) => service.status === 'unreachable').length;
  const healthyLabel = translatePortalText('{healthyCount}/{totalCount} healthy', {
    healthyCount: formatUnits(healthyCount),
    totalCount: formatUnits(runtimeHealth.services.length),
  });

  if (unreachableCount > 0) {
    return {
      value: healthyLabel,
      detail: translatePortalText(
        '{count} role(s) are unreachable from the current session, so the command center is surfacing a real runtime gap instead of static launch copy.',
        {
          count: formatUnits(unreachableCount),
        },
      ),
    };
  }

  if (degradedCount > 0) {
    return {
      value: healthyLabel,
      detail: translatePortalText(
        '{count} role(s) returned degraded health responses, so the command center is flagging live service issues before launch traffic ramps up.',
        {
          count: formatUnits(degradedCount),
        },
      ),
    };
  }

  return {
    value: healthyLabel,
    detail: translatePortalText(
      'Every visible product role responded successfully to its live health route, turning the command center into an actual runtime status panel.',
    ),
  };
}

function summarizeServiceLabels(services: PortalRuntimeServiceHealth[]): string {
  return services.map((service) => service.label).join(', ');
}

function buildLaunchReadiness(input: {
  dashboard: PortalDashboardSummary;
  membership: PortalCommerceMembership | null;
  runtimeHealth: PortalRuntimeHealthSnapshot;
  rateLimitSnapshot: PortalGatewayRateLimitSnapshot;
}) {
  const { dashboard, membership, runtimeHealth, rateLimitSnapshot } = input;
  const blockers: string[] = [];
  const watchpoints: string[] = [];
  let score = 100;

  const unreachableServices = runtimeHealth.services.filter((service) => service.status === 'unreachable');
  const degradedServices = runtimeHealth.services.filter((service) => service.status === 'degraded');

  if (dashboard.api_key_count === 0) {
    blockers.push(
      translatePortalText(
        'No visible project API key is available yet, so Codex, Claude Code, Gemini CLI, and OpenClaw onboarding remain blocked.',
      ),
    );
    score -= 30;
  }

  if (dashboard.billing_summary.exhausted) {
    blockers.push(
      translatePortalText(
        'Runway exhausted, so launch traffic should wait until credits, a recharge pack, or a subscription order restores quota.',
      ),
    );
    score -= 30;
  } else if ((dashboard.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000) {
    watchpoints.push(
      translatePortalText(
        'Remaining units are below 5,000, so the current launch runway is thin and should be reinforced before traffic ramps up.',
      ),
    );
    score -= 10;
  }

  if (unreachableServices.length > 0) {
    blockers.push(
      translatePortalText(
        'Unreachable services: {services}. Recover them before production traffic is pointed at this gateway.',
        {
          services: summarizeServiceLabels(unreachableServices),
        },
      ),
    );
    score -= 25;
  } else if (degradedServices.length > 0) {
    watchpoints.push(
      translatePortalText(
        'Degraded services: {services}. Stabilize them before the next launch window.',
        {
          services: summarizeServiceLabels(degradedServices),
        },
      ),
    );
    score -= 15;
  }

  if (rateLimitSnapshot.policy_count === 0) {
    watchpoints.push(
      translatePortalText(
        'No project-scoped rate-limit policy is configured yet, so the gateway still lacks a visible request-frequency guardrail.',
      ),
    );
    score -= 10;
  } else if (rateLimitSnapshot.exceeded_window_count > 0) {
    blockers.push(
      translatePortalText(
        '{count} live window snapshot(s) are over limit, so the gateway is currently running hot and should be throttled or rebalanced before launch traffic expands.',
        {
          count: formatUnits(rateLimitSnapshot.exceeded_window_count),
        },
      ),
    );
    score -= 20;
  } else if (rateLimitSnapshot.window_count > 0) {
    watchpoints.push(
      translatePortalText(
        '{count} live window snapshot(s) are active and within limits, so the router has a readable request-frequency posture.',
        {
          count: formatUnits(rateLimitSnapshot.window_count),
        },
      ),
    );
    score -= 5;
  }

  if (!membership) {
    watchpoints.push(
      translatePortalText(
        'No active membership is recorded yet, so recurring entitlement posture has not been established for this workspace.',
      ),
    );
    score -= 10;
  }

  const status = blockers.length > 0 ? 'blocked' : watchpoints.length > 0 ? 'watch' : 'ready';
  const normalizedScore = Math.max(0, Math.min(100, score));

  return {
    score: normalizedScore,
    status,
    headline:
      status === 'blocked'
        ? translatePortalText('Launch readiness is currently blocked')
        : status === 'watch'
          ? translatePortalText('Launch readiness is viable with watchpoints')
          : translatePortalText('Launch readiness is ready'),
    detail:
      status === 'blocked'
        ? translatePortalText(
            'Critical blockers are still present across access, runtime health, or billing runway, so the command center is holding the launch posture in a blocked state.',
          )
        : status === 'watch'
          ? translatePortalText(
              'The workspace can move forward, but the command center is surfacing watchpoints that should be cleared before growth traffic expands.',
            )
          : translatePortalText(
              'Gateway access, runtime health, and commercial runway are aligned well enough for real traffic onboarding.',
            ),
    blockersHeading: translatePortalText('Critical blockers'),
    blockers,
    watchpointsHeading: translatePortalText('Watchpoints'),
    watchpoints,
  } as const;
}

function buildRuntimeControls(desktopRuntime: PortalDesktopRuntimeSnapshot | null) {
  if (!desktopRuntime) {
    return [
      {
        id: 'restart-desktop-runtime',
        title: translatePortalText('Restart desktop runtime'),
        detail: translatePortalText(
          'Desktop runtime controls are only available inside the portal desktop shell where the local app owns web, gateway, admin, and portal services directly.',
        ),
        cta: translatePortalText('Restart desktop runtime'),
        action: 'restart-desktop-runtime' as const,
        enabled: false,
        tone: 'ghost' as const,
      },
    ];
  }

  return [
    {
      id: 'restart-desktop-runtime',
      title: translatePortalText('Restart desktop runtime'),
      detail: translatePortalText(
        'Restart the embedded {roles} runtime without leaving the portal command center so service binds and local health can be recovered in place.',
        {
          roles: desktopRuntime.roles.join(', '),
        },
      ),
      cta: translatePortalText('Restart desktop runtime'),
      action: 'restart-desktop-runtime' as const,
      enabled: true,
      tone: 'secondary' as const,
    },
  ];
}

export function buildGatewayCommandCenterSnapshot(input: {
  commerceCatalog: PortalCommerceCatalog;
  membership: PortalCommerceMembership | null;
  dashboard: PortalDashboardSummary;
  desktopRuntime: PortalDesktopRuntimeSnapshot | null;
  gatewayBaseUrl: string;
  rateLimitSnapshot?: PortalGatewayRateLimitSnapshot | null;
  runtimeHealth: PortalRuntimeHealthSnapshot;
}): GatewayCommandCenterSnapshot {
  const {
    commerceCatalog,
    membership,
    dashboard,
    desktopRuntime,
    gatewayBaseUrl,
    rateLimitSnapshot: rawRateLimitSnapshot,
    runtimeHealth,
  } = input;
  const rateLimitSnapshot =
    rawRateLimitSnapshot
    ?? buildDefaultRateLimitSnapshot(dashboard.workspace.project.id);
  const remainingUnits =
    dashboard.billing_summary.remaining_units === null || dashboard.billing_summary.remaining_units === undefined
      ? translatePortalText('Unlimited')
      : formatUnits(dashboard.billing_summary.remaining_units);
  const launchReadiness = buildLaunchReadiness({
    dashboard,
    membership,
    runtimeHealth,
    rateLimitSnapshot,
  });
  const serviceHealthSummary = buildServiceHealthSummary(runtimeHealth);

  return {
    gatewayBaseUrl,
    postureCards: [
      {
        id: 'entrypoint',
        label: translatePortalText('Product entrypoint'),
        value: 'sdkwork-router-portal',
        detail: translatePortalText(
          'The portal now fronts the router product instead of hiding desktop mode and server mode behind separate engineering entrypoints.',
        ),
      },
      {
        id: 'compatibility',
        label: translatePortalText('Protocol families'),
        value: 'OpenAI + Anthropic + Gemini',
        detail: translatePortalText(
          'One gateway surface carries OpenAI-compatible execution plus translated Anthropic Messages and Gemini Generative Language routes.',
        ),
      },
      {
        id: 'desktop',
        label: translatePortalText('Desktop mode'),
        value: translatePortalText('Portal + Admin + Gateway + Web host'),
        detail: translatePortalText(
          'desktop mode starts the local product as one loopback-owned stack so onboarding, control-plane work, and gateway calls stay on the same machine.',
        ),
      },
      {
        id: 'gateway-traffic',
        label: translatePortalText('Visible traffic'),
        value: translatePortalText('{count} requests', {
          count: formatUnits(dashboard.usage_summary.total_requests),
        }),
        detail: translatePortalText(
          'Gateway posture is tied to the current workspace traffic instead of a blank launch story.',
        ),
      },
      {
        id: 'access-readiness',
        label: translatePortalText('Access readiness'),
        value: translatePortalText('{count} API keys', {
          count: formatUnits(dashboard.api_key_count),
        }),
        detail:
          dashboard.api_key_count > 0
            ? translatePortalText(
                'The workspace already has visible project keys and can move straight into tool onboarding or route validation.',
              )
            : translatePortalText(
                'No project key is visible yet, so the first launch step is still credential issuance.',
              ),
      },
      {
        id: 'commerce',
        label: translatePortalText('Commerce posture'),
        value: dashboard.billing_summary.exhausted
          ? translatePortalText('Runway exhausted')
          : translatePortalText('{units} units left', {
              units: remainingUnits,
            }),
        detail: translatePortalText(
          'API key issuance, routing posture, credits, billing, and account runway remain linked so growth and recovery decisions stay evidence-backed across {planCount} plan(s), {packCount} pack(s), and {couponCount} coupon offer(s).',
          {
            planCount: formatUnits(commerceCatalog.plans.length),
            packCount: formatUnits(commerceCatalog.packs.length),
            couponCount: formatUnits(commerceCatalog.coupons.length),
          },
        ),
      },
      {
        id: 'runtime-health',
        label: translatePortalText('Live service health'),
        value: serviceHealthSummary.value,
        detail: serviceHealthSummary.detail,
      },
    ],
    launchReadiness,
    runtimeCards: buildRuntimeCards({
      desktopRuntime,
      gatewayBaseUrl,
    }),
    runtimeHealth,
    serviceHealthChecks: runtimeHealth.services,
    runtimeControls: buildRuntimeControls(desktopRuntime),
    rateLimitCards: buildRateLimitCards(rateLimitSnapshot),
    rateLimitSnapshot,
    compatibilityRows: [
      {
        id: 'codex',
        tool: 'Codex',
        protocol: 'OpenAI / Responses',
        routeFamily: '/v1/responses',
        truth: 'OpenAI-compatible gateway surface',
        outcome: translatePortalText(
          'Use one workspace API key against the routed gateway without creating a second credential boundary.',
        ),
      },
      {
        id: 'claude-code',
        tool: 'Claude Code',
        protocol: 'Anthropic Messages',
        routeFamily: '/v1/messages',
        truth: 'translated compatibility route',
        outcome: translatePortalText(
          'Claude Code keeps Anthropic-style requests while the router preserves shared routing, quota, billing, usage recording, and upstream relay of anthropic-version plus anthropic-beta headers.',
        ),
      },
      {
        id: 'opencode',
        tool: 'OpenCode',
        protocol: 'OpenAI Chat / Responses',
        routeFamily: '/v1/chat/completions and /v1/responses',
        truth: 'OpenAI-compatible gateway surface',
        outcome: translatePortalText(
          'OpenCode can stay on OpenAI-shaped configuration while the router handles provider abstraction behind the same base URL.',
        ),
      },
      {
        id: 'gemini',
        tool: 'Gemini CLI and Gemini-compatible clients',
        protocol: 'Gemini Generative Language',
        routeFamily: '/v1beta/models/{model}:generateContent',
        truth: 'translated compatibility route',
        outcome: translatePortalText(
          'Gemini CLI can use the official GOOGLE_GEMINI_BASE_URL plus GEMINI_API_KEY_AUTH_MECHANISM=bearer path while the router keeps billing and routing policy in the shared gateway flow.',
        ),
      },
      {
        id: 'openclaw',
        tool: 'OpenClaw',
        protocol: 'OpenAI provider manifest',
        routeFamily: 'desktop-assisted provider install',
        truth: 'desktop-assisted setup flow',
        outcome: translatePortalText(
          'OpenClaw instances can be pointed at the routed gateway from the portal desktop shell instead of being configured manually per instance.',
        ),
      },
    ],
    modeCards: [
      {
        id: 'desktop-mode',
        title: translatePortalText('Desktop mode'),
        command: 'pnpm product:start',
        summary: translatePortalText(
          'Start the full router product locally with the portal desktop shell as the operator-facing entrypoint.',
        ),
        notes: [
          translatePortalText('Starts admin, gateway, portal, and the public web host together.'),
          translatePortalText(
            'Uses loopback-owned binds and desktop-assisted runtime base URL discovery.',
          ),
          translatePortalText(
            'Best fit for local labs, private workstation gateways, and OpenClaw-assisted onboarding.',
          ),
        ],
      },
      {
        id: 'server-mode',
        title: translatePortalText('Server mode'),
        command: 'pnpm product:start -- server',
        summary: translatePortalText(
          'Run the same product as a server entrypoint so the portal, admin, and gateway can be served to remote users.',
        ),
        notes: [
          translatePortalText('Serves /portal/*, /admin/*, and /api/v1/* from the shared router product.'),
          translatePortalText(
            'Keeps admin, portal, and gateway under one product plan instead of three unrelated startup paths.',
          ),
          translatePortalText('Best fit for hosted teams, private clusters, and shared gateway operations.'),
        ],
      },
      {
        id: 'role-sliced',
        title: translatePortalText('Role-sliced topology'),
        command: 'pnpm server:start -- --roles web,gateway,admin,portal',
        summary: translatePortalText(
          'Split the product across edge, control-plane, and data-plane nodes when a single process is no longer the right deployment shape.',
        ),
        notes: [
          translatePortalText('The canonical role set is web, gateway, admin, portal.'),
          translatePortalText(
            'Supports single-node all-in-one and split-role deployments from the same product runtime.',
          ),
          translatePortalText('Pairs cleanly with dry-run planning before rollout.'),
        ],
      },
    ],
    topologyPlaybooks: [
      {
        id: 'single-node-local',
        title: translatePortalText('Single-node local product'),
        command: 'pnpm product:start',
        topology: translatePortalText('Desktop shell plus local gateway stack'),
        detail: translatePortalText(
          'Use this when the user wants a private API router on their own machine with admin, portal, and gateway started together.',
        ),
      },
      {
        id: 'single-node-server',
        title: translatePortalText('Single-node server'),
        command: 'pnpm product:start -- server',
        topology: translatePortalText('One process owns web, gateway, admin, and portal'),
        detail: translatePortalText(
          'Use this when the product should be hosted as one deployable router service without immediately splitting the topology.',
        ),
      },
      {
        id: 'edge-only',
        title: translatePortalText('Edge-only web node'),
        command:
          'pnpm server:start -- --dry-run --roles web --gateway-upstream 10.0.0.21:8080 --admin-upstream 10.0.0.22:8081 --portal-upstream 10.0.0.23:8082',
        topology: translatePortalText('Web edge proxies to dedicated API nodes'),
        detail: translatePortalText(
          'Use this when traffic termination and public site serving should stay separate from control-plane and gateway execution.',
        ),
      },
      {
        id: 'split-plane',
        title: translatePortalText('Split control-plane and data-plane'),
        command: 'pnpm server:start -- --roles gateway or admin,portal',
        topology: translatePortalText('Independent control-plane and data-plane services'),
        detail: translatePortalText(
          'Use this when operator traffic, public portal traffic, and gateway execution need different scaling or trust boundaries.',
        ),
      },
    ],
    verificationSnippets: [
      {
        id: 'openai-models',
        title: translatePortalText('OpenAI-compatible route check'),
        routeFamily: '/api/v1/models',
        command: [
          `curl ${joinUrl(gatewayBaseUrl, '/v1/models')} \\`,
          '  -H "Authorization: Bearer <project-api-key>"',
        ].join('\n'),
      },
      {
        id: 'anthropic-messages',
        title: translatePortalText('Anthropic Messages route check'),
        routeFamily: '/v1/messages',
        command: [
          `curl ${joinUrl(gatewayBaseUrl, '/v1/messages')} \\`,
          '  -H "x-api-key: <project-api-key>" \\',
          '  -H "anthropic-version: 2023-06-01" \\',
          '  -H "anthropic-beta: tools-2024-04-04" \\',
          '  -H "content-type: application/json" \\',
          '  -d \'{"model":"claude-sonnet-4","max_tokens":64,"messages":[{"role":"user","content":"Ping SDKWork Router"}]}\'',
        ].join('\n'),
      },
      {
        id: 'gemini-generate-content',
        title: translatePortalText('Gemini generateContent route check'),
        routeFamily: '/v1beta/models/{model}:generateContent',
        command: [
          `curl "${joinUrl(gatewayBaseUrl, '/v1beta/models/gemini-2.5-pro:generateContent')}?key=<project-api-key>" \\`,
          '  -H "content-type: application/json" \\',
          '  -d \'{"contents":[{"role":"user","parts":[{"text":"Ping SDKWork Router"}]}]}\'',
        ].join('\n'),
      },
    ],
    commerceCatalogCards: buildCommerceCatalogCards(commerceCatalog, membership),
    readinessActions: [
      {
        id: 'api-keys',
        title: translatePortalText('Access and onboarding'),
        detail: translatePortalText(
          'Issue or rotate the workspace key, then use the quick-setup flow for Codex, Claude Code, OpenCode, Gemini CLI, or OpenClaw.',
        ),
        cta: translatePortalText('Open API Keys'),
        route: 'api-keys',
        tone: 'primary',
      },
      {
        id: 'routing',
        title: translatePortalText('Routing guardrails'),
        detail: translatePortalText(
          'Confirm provider order, reliability guardrails, and preview evidence before real traffic is allowed to fan out across upstreams.',
        ),
        cta: translatePortalText('Open Routing'),
        route: 'routing',
        tone: 'secondary',
      },
      {
        id: 'billing',
        title: translatePortalText('Runway and recovery'),
        detail: translatePortalText(
          'Connect launch posture with credits, billing, coupons, recharge packs, and account runway so traffic growth is commercially safe.',
        ),
        cta: translatePortalText('Open Billing'),
        route: 'billing',
        tone: 'ghost',
      },
    ],
  };
}
