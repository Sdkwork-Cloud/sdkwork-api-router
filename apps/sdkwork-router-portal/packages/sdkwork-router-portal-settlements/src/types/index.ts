import type {
  CommercialAccountId,
  CommercialAccountBalanceSnapshot,
  CommercialAccountBenefitLotRecord,
  CommercialAccountHoldRecord,
  CommercialAccountSummary,
  CommercialPricingPlanRecord,
  CommercialPricingRateRecord,
  CommercialRequestSettlementRecord,
  PortalRouteKey,
} from 'sdkwork-router-portal-types';

export interface PortalSettlementsPageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export interface PortalSettlementsWorkspaceData {
  commercialAccount: CommercialAccountSummary | null;
  accountBalance: CommercialAccountBalanceSnapshot | null;
  benefitLots: CommercialAccountBenefitLotRecord[];
  holds: CommercialAccountHoldRecord[];
  requestSettlements: CommercialRequestSettlementRecord[];
  pricingPlans: CommercialPricingPlanRecord[];
  pricingRates: CommercialPricingRateRecord[];
}

export interface BuildPortalSettlementsViewModelInput extends PortalSettlementsWorkspaceData {}

export interface PortalSettlementsViewModel {
  account_id: CommercialAccountId | null;
  account_status: string | null;
  available_balance: number;
  held_balance: number;
  grant_balance: number;
  active_benefit_lot_count: number;
  expired_benefit_lot_count: number;
  open_hold_count: number;
  settlement_count: number;
  captured_settlement_count: number;
  refunded_settlement_count: number;
  captured_credit_amount: number;
  refunded_credit_amount: number;
  primary_plan_display_name: string | null;
  primary_rate_metric_code: string | null;
  primary_rate_charge_unit: CommercialPricingRateRecord['charge_unit'] | null;
  primary_rate_pricing_method: CommercialPricingRateRecord['pricing_method'] | null;
  primary_rate_display_price_unit: string | null;
  priced_metric_count: number;
  latest_settlements: CommercialRequestSettlementRecord[];
  request_settlements: CommercialRequestSettlementRecord[];
  open_holds: CommercialAccountHoldRecord[];
  active_benefit_lots: CommercialAccountBenefitLotRecord[];
  pricing_plans: CommercialPricingPlanRecord[];
  pricing_rates: CommercialPricingRateRecord[];
}
