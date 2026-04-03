import {
  createPortalRoutingProfile,
  getPortalRoutingSummary,
  listPortalRoutingSnapshots,
  listPortalRoutingProfiles,
  listPortalRoutingDecisionLogs,
  previewPortalRouting,
  savePortalRoutingPreferences,
} from 'sdkwork-router-portal-portal-api';
import type {
  PortalCompiledRoutingSnapshotRecord,
  PortalRoutingDecision,
  PortalRoutingDecisionLog,
  PortalRoutingPreferences,
  RoutingProfileRecord,
  PortalRoutingSummary,
} from 'sdkwork-router-portal-types';

export function loadPortalRoutingSummary(): Promise<PortalRoutingSummary> {
  return getPortalRoutingSummary();
}

export function loadPortalRoutingDecisionLogs(): Promise<PortalRoutingDecisionLog[]> {
  return listPortalRoutingDecisionLogs();
}

export function loadPortalRoutingProfiles(): Promise<RoutingProfileRecord[]> {
  return listPortalRoutingProfiles();
}

export function loadPortalRoutingSnapshots(): Promise<PortalCompiledRoutingSnapshotRecord[]> {
  return listPortalRoutingSnapshots();
}

export function updatePortalRoutingPreferences(input: {
  preset_id: string;
  strategy: PortalRoutingPreferences['strategy'];
  ordered_provider_ids: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy: boolean;
  preferred_region?: string | null;
}): Promise<PortalRoutingPreferences> {
  return savePortalRoutingPreferences(input);
}

export function runPortalRoutingPreview(input: {
  capability: string;
  model: string;
  requested_region?: string | null;
  selection_seed?: number | null;
}): Promise<PortalRoutingDecision> {
  return previewPortalRouting(input);
}

export function issuePortalRoutingProfile(input: {
  name: string;
  slug?: string | null;
  description?: string | null;
  active?: boolean;
  strategy?: string;
  ordered_provider_ids?: string[];
  default_provider_id?: string | null;
  max_cost?: number | null;
  max_latency_ms?: number | null;
  require_healthy?: boolean;
  preferred_region?: string | null;
}): Promise<RoutingProfileRecord> {
  return createPortalRoutingProfile(input);
}
