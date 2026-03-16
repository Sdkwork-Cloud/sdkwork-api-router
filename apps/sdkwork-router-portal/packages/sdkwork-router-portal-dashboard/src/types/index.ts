import type { PortalDashboardSummary, PortalRouteKey } from 'sdkwork-router-portal-types';

export interface DashboardInsight {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
  route?: PortalRouteKey;
  action_label?: string;
}

export interface DashboardReadinessItem {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface DashboardActionItem {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
  priority_label: string;
  route: PortalRouteKey;
  action_label: string;
}

export interface DashboardChecklistItem {
  id: string;
  title: string;
  detail: string;
  complete: boolean;
  route: PortalRouteKey;
  action_label: string;
}

export interface DashboardProductionReadiness {
  score: number;
  title: string;
  detail: string;
  blockers: string[];
  strengths: string[];
}

export interface DashboardJourneyGuide {
  completed_count: number;
  total_count: number;
  progress_label: string;
  current_blocker: string;
  next_milestone_title: string;
  next_milestone_detail: string;
  next_route: PortalRouteKey;
  next_action_label: string;
}

export interface DashboardEvidenceItem {
  id: string;
  title: string;
  detail: string;
  timestamp_label: string;
}

export interface DashboardConfidenceSignal {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface DashboardModeNarrative {
  title: string;
  detail: string;
  why_now: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface DashboardDecisionPathItem {
  id: string;
  title: string;
  detail: string;
  route: PortalRouteKey;
  action_label: string;
}

export interface DashboardRouteSignal {
  route: PortalRouteKey;
  title: string;
  status_label: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface DashboardReviewCadenceItem {
  id: string;
  title: string;
  detail: string;
  route: PortalRouteKey;
  action_label: string;
}

export interface DashboardPlaybookLaneItem {
  id: string;
  title: string;
  detail: string;
  route: PortalRouteKey;
  action_label: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface DashboardFocusBoardItem {
  id: string;
  title: string;
  detail: string;
  priority_label: string;
  route: PortalRouteKey;
  action_label: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface DashboardRiskWatchItem {
  id: string;
  title: string;
  detail: string;
  status_label: string;
  route: PortalRouteKey;
  action_label: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface DashboardDailyBrief {
  title: string;
  detail: string;
  top_focus: DashboardFocusBoardItem;
  risk_watch: DashboardRiskWatchItem;
}

export interface PortalDashboardPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface PortalDashboardPageViewModel {
  snapshot: PortalDashboardSummary;
  insights: DashboardInsight[];
  readiness: DashboardReadinessItem[];
  action_queue: DashboardActionItem[];
  launch_checklist: DashboardChecklistItem[];
  production_readiness: DashboardProductionReadiness;
  journey: DashboardJourneyGuide;
  evidence_timeline: DashboardEvidenceItem[];
  confidence_signals: DashboardConfidenceSignal[];
  mode: DashboardModeNarrative;
  decision_path: DashboardDecisionPathItem[];
  route_signals: DashboardRouteSignal[];
  review_cadence: DashboardReviewCadenceItem[];
  playbook_lane: DashboardPlaybookLaneItem[];
  focus_board: DashboardFocusBoardItem[];
  risk_watchlist: DashboardRiskWatchItem[];
  daily_brief: DashboardDailyBrief;
}
