import type { StatusBadgeVariant } from 'sdkwork-router-portal-commons/framework/display';

import type {
  PortalCommerceMembership,
  PortalDashboardSummary,
  PortalRouteKey,
  PortalRoutingDecisionLog,
  PortalRoutingSummary,
  UsageRecord,
} from 'sdkwork-router-portal-types';

export type DashboardStatusVariant = StatusBadgeVariant;

export interface DashboardInsight {
  id: string;
  title: string;
  detail: string;
  status_label: string;
  status_variant: DashboardStatusVariant;
  route?: PortalRouteKey;
  action_label?: string;
}

export interface DashboardMetric {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface DashboardMetricSummary {
  revenue: number;
  request_count: number;
  used_units: number;
  average_booked_spend: number;
}

export interface DashboardBalanceSummary {
  remaining_units: number | null;
  quota_limit_units: number | null;
  used_units: number;
  utilization_ratio: number | null;
}

export interface DashboardRoutingPosture {
  title: string;
  detail: string;
  strategy_label: string;
  selected_provider: string;
  preferred_region: string;
  evidence_count: string;
  latest_reason: string;
  status_label: string;
  status_variant: DashboardStatusVariant;
  route: PortalRouteKey;
  action_label: string;
}

export interface DashboardBreakdownItem {
  id: string;
  label: string;
  secondary_label: string;
  value_label: string;
  share: number;
}

export interface DashboardSeriesPoint {
  bucket: string;
  requests: number;
  amount: number;
}

export interface DashboardTrafficTrendPoint {
  label: string;
  bucket_key: string;
  request_count: number;
  amount: number;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
}

export interface DashboardSpendTrendPoint {
  label: string;
  bucket_key: string;
  amount: number;
  requests: number;
}

export interface DashboardDistributionPoint {
  name: string;
  value: number;
}

export interface DashboardDemandPoint {
  name: string;
  requests: number;
}

export interface DashboardActivityItem {
  id: string;
  title: string;
  detail: string;
  timestamp_label: string;
  status_label: string;
  status_variant: DashboardStatusVariant;
  route?: PortalRouteKey;
  action_label?: string;
}

export interface DashboardModuleItem {
  route: PortalRouteKey;
  title: string;
  status_label: string;
  detail: string;
  status_variant: DashboardStatusVariant;
  action_label: string;
}

export interface PortalDashboardSnapshotBundle {
  dashboard: PortalDashboardSummary;
  membership: PortalCommerceMembership | null;
  routing_summary: PortalRoutingSummary;
  routing_logs: PortalRoutingDecisionLog[];
  usage_records: UsageRecord[];
}

export interface PortalDashboardPageProps {
  onNavigate: (route: PortalRouteKey) => void;
  initialSnapshot?: PortalDashboardSummary | null;
}

export interface PortalDashboardPageViewModel {
  snapshot: PortalDashboardSummary;
  membership: PortalCommerceMembership | null;
  balance: DashboardBalanceSummary;
  totals: DashboardMetricSummary;
  today: DashboardMetricSummary;
  trailing_7d: DashboardMetricSummary;
  current_month: DashboardMetricSummary;
  insights: DashboardInsight[];
  metrics: DashboardMetric[];
  routing_posture: DashboardRoutingPosture | null;
  quick_actions: DashboardInsight[];
  provider_mix: DashboardBreakdownItem[];
  model_mix: DashboardBreakdownItem[];
  request_volume_series: DashboardSeriesPoint[];
  spend_series: DashboardSeriesPoint[];
  traffic_trend_points: DashboardTrafficTrendPoint[];
  spend_trend_points: DashboardSpendTrendPoint[];
  provider_share_series: DashboardDistributionPoint[];
  model_demand_series: DashboardDemandPoint[];
  activity_feed: DashboardActivityItem[];
  modules: DashboardModuleItem[];
}
