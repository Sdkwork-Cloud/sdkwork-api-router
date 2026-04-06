import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('admin commercial workspace wires canonical billing investigation into types, API, workbench, and gateway pages', () => {
  const adminTypes = read('packages/sdkwork-router-admin-types/src/index.ts');
  const adminApi = read('packages/sdkwork-router-admin-admin-api/src/index.ts');
  const workbench = read('packages/sdkwork-router-admin-core/src/workbench.tsx');
  const snapshot = read('packages/sdkwork-router-admin-core/src/workbenchSnapshot.ts');
  const accessPage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx');
  const usagePage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');

  assert.match(adminTypes, /export interface CommercialAccountRecord/);
  assert.match(adminTypes, /export interface CommercialAccountSummary/);
  assert.match(adminTypes, /export interface CommercialAccountBalanceSnapshot/);
  assert.match(adminTypes, /export interface CommercialAccountBenefitLotRecord/);
  assert.match(adminTypes, /export interface CommercialAccountHoldRecord/);
  assert.match(adminTypes, /export interface CommercialRequestSettlementRecord/);
  assert.match(adminTypes, /export type CommercialAccountLedgerEntryType =/);
  assert.match(adminTypes, /export interface CommercialAccountLedgerEntryRecord/);
  assert.match(adminTypes, /export interface CommercialAccountLedgerAllocationRecord/);
  assert.match(adminTypes, /export interface CommercialAccountLedgerHistoryEntry/);
  assert.match(adminTypes, /export interface CommerceOrderRecord/);
  assert.match(adminTypes, /export interface CommercePaymentEventRecord/);
  assert.match(adminTypes, /export interface CommerceOrderAuditRecord/);
  assert.match(adminTypes, /export interface CommercialPricingPlanRecord/);
  assert.match(adminTypes, /export interface CommercialPricingRateRecord/);
  assert.match(adminTypes, /commercialAccounts:/);
  assert.match(adminTypes, /commercialAccountHolds:/);
  assert.match(adminTypes, /commercialAccountLedger:/);
  assert.match(adminTypes, /commercialRequestSettlements:/);
  assert.match(adminTypes, /commerceOrders:/);
  assert.match(adminTypes, /commercePaymentEvents:/);
  assert.match(adminTypes, /commercialPricingPlans:/);
  assert.match(adminTypes, /commercialPricingRates:/);

  assert.match(adminApi, /listCommercialAccounts/);
  assert.match(adminApi, /getCommercialAccountBalance/);
  assert.match(adminApi, /listCommercialAccountBenefitLots/);
  assert.match(adminApi, /listCommercialAccountLedger/);
  assert.match(adminApi, /listCommercialAccountHolds/);
  assert.match(adminApi, /listCommercialRequestSettlements/);
  assert.match(adminApi, /listCommercialPricingPlans/);
  assert.match(adminApi, /listCommercialPricingRates/);
  assert.match(adminApi, /listRecentCommerceOrders/);
  assert.match(adminApi, /listCommercePaymentEvents/);
  assert.match(adminApi, /getCommerceOrderAudit/);

  assert.match(workbench, /listCommercialAccounts/);
  assert.match(workbench, /listCommercialAccountLedger/);
  assert.match(workbench, /listCommercialAccountHolds/);
  assert.match(workbench, /listCommercialRequestSettlements/);
  assert.match(workbench, /listCommercialPricingPlans/);
  assert.match(workbench, /listCommercialPricingRates/);
  assert.match(workbench, /listRecentCommerceOrders/);
  assert.match(workbench, /listCommercePaymentEvents/);
  assert.match(snapshot, /commercialAccounts:/);
  assert.match(snapshot, /commercialAccountHolds:/);
  assert.match(snapshot, /commercialAccountLedger:/);
  assert.match(snapshot, /commercialRequestSettlements:/);
  assert.match(snapshot, /commerceOrders:/);
  assert.match(snapshot, /commercePaymentEvents:/);
  assert.match(snapshot, /commercialPricingPlans:/);
  assert.match(snapshot, /commercialPricingRates:/);

  assert.match(accessPage, /Commercial governance/);
  assert.match(accessPage, /Commercial accounts/);
  assert.match(accessPage, /Pricing posture/);
  assert.match(usagePage, /Commercial accounts/);
  assert.match(usagePage, /Request settlements/);
  assert.match(usagePage, /Pricing posture/);

  const commercialPage = read('packages/sdkwork-router-admin-commercial/src/index.tsx');
  assert.match(commercialPage, /Settlement ledger/);
  assert.match(commercialPage, /Refund timeline/);
  assert.match(commercialPage, /Order payment audit/);
  assert.match(commercialPage, /Order refund audit/);
  assert.match(commercialPage, /Order audit detail/);
  assert.match(commercialPage, /View order audit/);
  assert.match(commercialPage, /Find order audit/);
  assert.match(commercialPage, /normalizeCommercialOrderAuditLookupValue/);
  assert.match(commercialPage, /buildCommercialLedgerTimelineRows/);
  assert.match(commercialPage, /buildCommercialOrderPaymentAuditRows/);
  assert.match(commercialPage, /commercialAccountLedger/);
  assert.match(commercialPage, /commercePaymentEvents/);
});
