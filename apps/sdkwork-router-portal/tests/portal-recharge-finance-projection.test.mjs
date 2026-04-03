import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadRechargeServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-recharge',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('recharge workspace consumes membership and billing event summary across repository, types, services, and page', () => {
  const repository = read('packages/sdkwork-router-portal-recharge/src/repository/index.ts');
  const pageTypes = read('packages/sdkwork-router-portal-recharge/src/types/index.ts');
  const services = read('packages/sdkwork-router-portal-recharge/src/services/index.ts');
  const page = read('packages/sdkwork-router-portal-recharge/src/pages/index.tsx');

  assert.match(repository, /getPortalCommerceMembership/);
  assert.match(repository, /getPortalBillingEventSummary/);
  assert.match(pageTypes, /membership: PortalCommerceMembership \| null;/);
  assert.match(pageTypes, /billing_event_summary: BillingEventSummary;/);
  assert.match(pageTypes, /PortalRechargeFinanceProjection/);
  assert.match(services, /buildPortalRechargeFinanceProjection/);
  assert.match(page, /Recharge decision support/);
  assert.match(page, /Leading accounting mode/);
  assert.match(page, /Leading capability/);
  assert.match(page, /Multimodal demand/);
  assert.match(page, /portal-recharge-decision-support/);
  assert.match(page, /portal-recharge-multimodal-demand/);
});

test('recharge services derive finance projection from membership and billing event evidence', () => {
  const { buildPortalRechargeFinanceProjection } = loadRechargeServices();

  const projection = buildPortalRechargeFinanceProjection({
    membership: {
      membership_id: 'member-growth',
      project_id: 'project-demo',
      user_id: 'user-1',
      plan_id: 'growth',
      plan_name: 'Growth',
      price_cents: 19900,
      price_label: '$199 / month',
      cadence: 'monthly',
      included_units: 12000,
      status: 'active',
      source: 'workspace_seed',
      activated_at_ms: 100,
      updated_at_ms: 200,
    },
    billingEventSummary: {
      total_events: 4,
      project_count: 1,
      group_count: 2,
      capability_count: 3,
      total_request_count: 7,
      total_units: 480,
      total_input_tokens: 160,
      total_output_tokens: 120,
      total_tokens: 280,
      total_image_count: 6,
      total_audio_seconds: 92,
      total_video_seconds: 48,
      total_music_seconds: 25,
      total_upstream_cost: 9.1,
      total_customer_charge: 12.4,
      projects: [],
      groups: [],
      capabilities: [
        {
          capability: 'audio',
          event_count: 1,
          request_count: 2,
          total_tokens: 0,
          image_count: 0,
          audio_seconds: 92,
          video_seconds: 48,
          music_seconds: 25,
          total_upstream_cost: 1.8,
          total_customer_charge: 1.4,
        },
        {
          capability: 'responses',
          event_count: 2,
          request_count: 4,
          total_tokens: 280,
          image_count: 0,
          audio_seconds: 0,
          video_seconds: 0,
          music_seconds: 0,
          total_upstream_cost: 3.1,
          total_customer_charge: 4.1,
        },
        {
          capability: 'images',
          event_count: 1,
          request_count: 1,
          total_tokens: 0,
          image_count: 6,
          audio_seconds: 0,
          video_seconds: 0,
          music_seconds: 0,
          total_upstream_cost: 4.2,
          total_customer_charge: 6.9,
        },
      ],
      accounting_modes: [
        {
          accounting_mode: 'passthrough',
          event_count: 1,
          request_count: 2,
          total_upstream_cost: 1.8,
          total_customer_charge: 1.4,
        },
        {
          accounting_mode: 'platform_credit',
          event_count: 2,
          request_count: 4,
          total_upstream_cost: 3.1,
          total_customer_charge: 4.1,
        },
        {
          accounting_mode: 'byok',
          event_count: 1,
          request_count: 1,
          total_upstream_cost: 4.2,
          total_customer_charge: 6.9,
        },
      ],
    },
  });

  assert.equal(projection.membership?.plan_name, 'Growth');
  assert.equal(projection.leading_accounting_mode?.accounting_mode, 'byok');
  assert.equal(projection.leading_capability?.capability, 'images');
  assert.deepEqual(projection.multimodal_totals, {
    image_count: 6,
    audio_seconds: 92,
    video_seconds: 48,
    music_seconds: 25,
  });
});
