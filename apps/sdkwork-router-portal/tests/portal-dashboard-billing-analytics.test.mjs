import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('dashboard removes commercial highlights support across repository, types, services, and page', () => {
  const repository = read('packages/sdkwork-router-portal-dashboard/src/repository/index.ts');
  const dashboardTypes = read('packages/sdkwork-router-portal-dashboard/src/types/index.ts');
  const dashboardServices = read('packages/sdkwork-router-portal-dashboard/src/services/index.ts');
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.doesNotMatch(repository, /getPortalBillingEventSummary/);
  assert.doesNotMatch(repository, /billing_event_summary/);
  assert.doesNotMatch(dashboardTypes, /billing_event_summary: BillingEventSummary;/);
  assert.doesNotMatch(dashboardTypes, /DashboardCommercialHighlights/);
  assert.doesNotMatch(dashboardTypes, /commercial_highlights:/);
  assert.doesNotMatch(dashboardServices, /buildDashboardCommercialHighlights/);
  assert.doesNotMatch(dashboardServices, /BillingEventSummary/);
  assert.doesNotMatch(dashboardServices, /BillingEventAccountingModeSummary/);
  assert.doesNotMatch(dashboardServices, /BillingEventCapabilitySummary/);
  assert.doesNotMatch(dashboardPage, /Commercial highlights/);
  assert.doesNotMatch(dashboardPage, /Leading accounting mode/);
  assert.doesNotMatch(dashboardPage, /Leading capability/);
  assert.doesNotMatch(dashboardPage, /Multimodal demand/);
  assert.doesNotMatch(dashboardPage, /portal-dashboard-commercial-highlights/);
});
