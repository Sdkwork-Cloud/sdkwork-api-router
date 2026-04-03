import type {
  AdminWorkspaceSnapshot,
  ApiKeyGroupRecord,
  CompiledRoutingSnapshotRecord,
  RoutingProfileRecord,
} from 'sdkwork-router-admin-types';

export interface RoutingSnapshotEvidenceRow {
  snapshot: CompiledRoutingSnapshotRecord;
  routingProfile: RoutingProfileRecord | null;
  apiKeyGroup: ApiKeyGroupRecord | null;
}

export interface RoutingSnapshotProfileImpact {
  routingProfile: RoutingProfileRecord;
  boundGroups: ApiKeyGroupRecord[];
  compiledSnapshots: CompiledRoutingSnapshotRecord[];
  routeKeyCount: number;
  capabilityCount: number;
  latestUpdatedAtMs: number | null;
}

export interface RoutingSnapshotAnalytics {
  totalCompiledSnapshots: number;
  profileBackedSnapshotCount: number;
  boundGroupCount: number;
  activeProfileCount: number;
  uniqueRouteKeyCount: number;
  evidenceRows: RoutingSnapshotEvidenceRow[];
  topProfiles: RoutingSnapshotProfileImpact[];
}

export interface ProviderRoutingImpact {
  providerId: string;
  routingProfileCount: number;
  boundGroupCount: number;
  compiledSnapshotCount: number;
  defaultSnapshotCount: number;
  topProfiles: RoutingSnapshotProfileImpact[];
  recentSnapshots: RoutingSnapshotEvidenceRow[];
}

function compareProfileImpact(
  left: RoutingSnapshotProfileImpact,
  right: RoutingSnapshotProfileImpact,
) {
  return (
    right.compiledSnapshots.length - left.compiledSnapshots.length
    || right.boundGroups.length - left.boundGroups.length
    || right.routeKeyCount - left.routeKeyCount
    || (right.latestUpdatedAtMs ?? 0) - (left.latestUpdatedAtMs ?? 0)
  );
}

function compareEvidenceRow(
  left: RoutingSnapshotEvidenceRow,
  right: RoutingSnapshotEvidenceRow,
) {
  return (
    right.snapshot.updated_at_ms - left.snapshot.updated_at_ms
    || right.snapshot.created_at_ms - left.snapshot.created_at_ms
  );
}

export function buildRoutingSnapshotAnalytics(
  snapshot: AdminWorkspaceSnapshot,
): RoutingSnapshotAnalytics {
  const routingProfileById = new Map(
    snapshot.routingProfiles.map((profile) => [profile.profile_id, profile]),
  );
  const apiKeyGroupById = new Map(
    snapshot.apiKeyGroups.map((group) => [group.group_id, group]),
  );
  const groupsByProfileId = new Map<string, ApiKeyGroupRecord[]>();

  for (const group of snapshot.apiKeyGroups) {
    if (!group.default_routing_profile_id) {
      continue;
    }

    const existingGroups = groupsByProfileId.get(group.default_routing_profile_id) ?? [];
    existingGroups.push(group);
    groupsByProfileId.set(group.default_routing_profile_id, existingGroups);
  }

  const snapshotsByProfileId = new Map<string, CompiledRoutingSnapshotRecord[]>();
  const evidenceRows = snapshot.compiledRoutingSnapshots
    .map<RoutingSnapshotEvidenceRow>((compiledRoutingSnapshot) => ({
      snapshot: compiledRoutingSnapshot,
      routingProfile: compiledRoutingSnapshot.applied_routing_profile_id
        ? routingProfileById.get(compiledRoutingSnapshot.applied_routing_profile_id) ?? null
        : null,
      apiKeyGroup: compiledRoutingSnapshot.api_key_group_id
        ? apiKeyGroupById.get(compiledRoutingSnapshot.api_key_group_id) ?? null
        : null,
    }))
    .sort(compareEvidenceRow);

  for (const compiledRoutingSnapshot of snapshot.compiledRoutingSnapshots) {
    if (!compiledRoutingSnapshot.applied_routing_profile_id) {
      continue;
    }

    const existingSnapshots =
      snapshotsByProfileId.get(compiledRoutingSnapshot.applied_routing_profile_id) ?? [];
    existingSnapshots.push(compiledRoutingSnapshot);
    snapshotsByProfileId.set(
      compiledRoutingSnapshot.applied_routing_profile_id,
      existingSnapshots,
    );
  }

  const topProfiles = snapshot.routingProfiles
    .map<RoutingSnapshotProfileImpact>((routingProfile) => {
      const boundGroups = groupsByProfileId.get(routingProfile.profile_id) ?? [];
      const compiledSnapshots = snapshotsByProfileId.get(routingProfile.profile_id) ?? [];

      return {
        routingProfile,
        boundGroups,
        compiledSnapshots,
        routeKeyCount: new Set(compiledSnapshots.map((item) => item.route_key)).size,
        capabilityCount: new Set(compiledSnapshots.map((item) => item.capability)).size,
        latestUpdatedAtMs: compiledSnapshots.reduce<number | null>(
          (latest, item) =>
            latest == null || item.updated_at_ms > latest ? item.updated_at_ms : latest,
          null,
        ),
      };
    })
    .filter((item) => item.boundGroups.length > 0 || item.compiledSnapshots.length > 0)
    .sort(compareProfileImpact);

  return {
    totalCompiledSnapshots: snapshot.compiledRoutingSnapshots.length,
    profileBackedSnapshotCount: snapshot.compiledRoutingSnapshots.filter(
      (compiledRoutingSnapshot) => compiledRoutingSnapshot.applied_routing_profile_id,
    ).length,
    boundGroupCount: snapshot.apiKeyGroups.filter(
      (group) => group.default_routing_profile_id,
    ).length,
    activeProfileCount: snapshot.routingProfiles.filter((profile) => profile.active).length,
    uniqueRouteKeyCount: new Set(
      snapshot.compiledRoutingSnapshots.map((compiledRoutingSnapshot) => compiledRoutingSnapshot.route_key),
    ).size,
    evidenceRows,
    topProfiles,
  };
}

export function buildProviderRoutingImpact(
  providerId: string,
  analytics: RoutingSnapshotAnalytics,
): ProviderRoutingImpact {
  const topProfiles = analytics.topProfiles
    .filter(
      (profileImpact) =>
        profileImpact.routingProfile.default_provider_id === providerId
        || profileImpact.routingProfile.ordered_provider_ids.includes(providerId),
    )
    .sort(compareProfileImpact);
  const recentSnapshots = analytics.evidenceRows
    .filter(
      (evidenceRow) =>
        evidenceRow.snapshot.default_provider_id === providerId
        || evidenceRow.snapshot.ordered_provider_ids.includes(providerId),
    )
    .sort(compareEvidenceRow);

  return {
    providerId,
    routingProfileCount: topProfiles.length,
    boundGroupCount: topProfiles.reduce(
      (sum, profileImpact) => sum + profileImpact.boundGroups.length,
      0,
    ),
    compiledSnapshotCount: recentSnapshots.length,
    defaultSnapshotCount: recentSnapshots.filter(
      (evidenceRow) => evidenceRow.snapshot.default_provider_id === providerId,
    ).length,
    topProfiles,
    recentSnapshots: recentSnapshots.slice(0, 5),
  };
}
