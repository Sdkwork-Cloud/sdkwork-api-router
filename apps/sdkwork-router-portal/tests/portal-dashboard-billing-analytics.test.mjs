import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadDashboardServices() {
  const load = jiti(import.meta.url, {
    moduleCache: false,
    alias: {
      'sdkwork-router-portal-commons/format-core': path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-commons',
        'src',
        'format-core.ts',
      ),
      'sdkwork-router-portal-commons/i18n-core': path.join(
        appRoot,
        'packages',
        'sdkwork-router-portal-commons',
        'src',
        'i18n-core.ts',
      ),
    },
  });

  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-dashboard',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('dashboard workspace consumes billing event summary across repository, types, services, and page', () => {
  const repository = read('packages/sdkwork-router-portal-dashboard/src/repository/index.ts');
  const dashboardTypes = read('packages/sdkwork-router-portal-dashboard/src/types/index.ts');
  const dashboardServices = read('packages/sdkwork-router-portal-dashboard/src/services/index.ts');
  const dashboardPage = read('packages/sdkwork-router-portal-dashboard/src/pages/index.tsx');

  assert.match(repository, /getPortalBillingEventSummary/);
  assert.match(dashboardTypes, /billing_event_summary: BillingEventSummary;/);
  assert.match(dashboardTypes, /DashboardCommercialHighlights/);
  assert.match(dashboardServices, /buildDashboardCommercialHighlights/);
  assert.match(dashboardPage, /Commercial highlights/);
  assert.match(dashboardPage, /Leading accounting mode/);
  assert.match(dashboardPage, /Leading capability/);
  assert.match(dashboardPage, /Multimodal demand/);
  assert.match(dashboardPage, /portal-dashboard-commercial-highlights/);
});

test('dashboard services derive commercial highlights from billing event evidence', () => {
  const { buildDashboardCommercialHighlights } = loadDashboardServices();

  const highlights = buildDashboardCommercialHighlights({
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
  });

  assert.equal(highlights.total_customer_charge, 12.4);
  assert.equal(highlights.leading_accounting_mode?.accounting_mode, 'byok');
  assert.equal(highlights.leading_capability?.capability, 'images');
  assert.deepEqual(highlights.multimodal_totals, {
    image_count: 6,
    audio_seconds: 92,
    video_seconds: 48,
    music_seconds: 25,
  });
});
