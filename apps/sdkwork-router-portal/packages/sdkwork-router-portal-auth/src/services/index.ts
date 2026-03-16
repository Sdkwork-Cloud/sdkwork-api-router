import { persistPortalSessionToken } from 'sdkwork-router-portal-portal-api';
import type { PortalAuthSession } from 'sdkwork-router-portal-types';

export function persistPortalAuthSession(session: PortalAuthSession): void {
  persistPortalSessionToken(session.token);
}
