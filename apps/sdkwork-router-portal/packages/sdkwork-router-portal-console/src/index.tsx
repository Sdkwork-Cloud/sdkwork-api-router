import { PortalAccountPage } from 'sdkwork-router-portal-account';
import { PortalApiKeysPage } from 'sdkwork-router-portal-api-keys';
import { PortalBillingPage } from 'sdkwork-router-portal-billing';
import { PortalCreditsPage } from 'sdkwork-router-portal-credits';
import { PortalDashboardPage } from 'sdkwork-router-portal-dashboard';
import { PortalGatewayPage } from 'sdkwork-router-portal-gateway';
import { PortalRechargePage } from 'sdkwork-router-portal-recharge';
import { PortalRoutingPage } from 'sdkwork-router-portal-routing';
import type {
  PortalDashboardSummary,
  PortalRouteKey,
  PortalWorkspaceSummary,
} from 'sdkwork-router-portal-types';
import { PortalUserPage } from 'sdkwork-router-portal-user';
import { PortalUsagePage } from 'sdkwork-router-portal-usage';

export function PortalConsoleRoute({
  dashboardSnapshot,
  onNavigate,
  routeKey,
  workspace,
}: {
  dashboardSnapshot: PortalDashboardSummary | null;
  onNavigate: (route: PortalRouteKey) => void;
  routeKey: PortalRouteKey;
  workspace: PortalWorkspaceSummary | null;
}) {
  switch (routeKey) {
    case 'gateway':
      return <PortalGatewayPage onNavigate={onNavigate} />;
    case 'dashboard':
      return (
        <PortalDashboardPage
          initialSnapshot={dashboardSnapshot}
          onNavigate={onNavigate}
        />
      );
    case 'routing':
      return <PortalRoutingPage onNavigate={onNavigate} />;
    case 'api-keys':
      return <PortalApiKeysPage onNavigate={onNavigate} />;
    case 'usage':
      return <PortalUsagePage onNavigate={onNavigate} />;
    case 'user':
      return <PortalUserPage onNavigate={onNavigate} workspace={workspace} />;
    case 'credits':
      return <PortalCreditsPage onNavigate={onNavigate} workspace={workspace} />;
    case 'recharge':
      return <PortalRechargePage onNavigate={onNavigate} />;
    case 'billing':
      return <PortalBillingPage onNavigate={onNavigate} />;
    case 'account':
      return <PortalAccountPage onNavigate={onNavigate} workspace={workspace} />;
    default:
      return null;
  }
}
