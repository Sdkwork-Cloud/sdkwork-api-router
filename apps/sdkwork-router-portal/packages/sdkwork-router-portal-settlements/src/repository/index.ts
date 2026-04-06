import {
  getPortalCommercialAccount,
  getPortalCommercialAccountBalance,
  listPortalCommercialBenefitLots,
  listPortalCommercialHolds,
  listPortalCommercialPricingPlans,
  listPortalCommercialPricingRates,
  listPortalCommercialRequestSettlements,
} from 'sdkwork-router-portal-portal-api';

import type { PortalSettlementsWorkspaceData } from '../types';

export async function loadPortalSettlementsWorkspace(): Promise<PortalSettlementsWorkspaceData> {
  const [
    commercialAccount,
    accountBalance,
    benefitLots,
    holds,
    requestSettlements,
    pricingPlans,
    pricingRates,
  ] = await Promise.all([
    getPortalCommercialAccount(),
    getPortalCommercialAccountBalance(),
    listPortalCommercialBenefitLots(),
    listPortalCommercialHolds(),
    listPortalCommercialRequestSettlements(),
    listPortalCommercialPricingPlans(),
    listPortalCommercialPricingRates(),
  ]);

  return {
    commercialAccount,
    accountBalance,
    benefitLots,
    holds,
    requestSettlements,
    pricingPlans,
    pricingRates,
  };
}

export {
  getPortalCommercialAccount,
  getPortalCommercialAccountBalance,
  listPortalCommercialBenefitLots,
  listPortalCommercialHolds,
  listPortalCommercialPricingPlans,
  listPortalCommercialPricingRates,
  listPortalCommercialRequestSettlements,
};
