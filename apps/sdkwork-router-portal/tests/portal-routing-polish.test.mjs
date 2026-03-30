import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function loadRoutingServices() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-portal-routing',
      'src',
      'services',
      'index.ts',
    ),
  );
}

test('portal API SDK exposes routing summary, preferences, preview, and evidence calls', () => {
  const portalApi = read('packages/sdkwork-router-portal-portal-api/src/index.ts');

  assert.match(portalApi, /getPortalRoutingSummary/);
  assert.match(portalApi, /getPortalRoutingPreferences/);
  assert.match(portalApi, /savePortalRoutingPreferences/);
  assert.match(portalApi, /previewPortalRouting/);
  assert.match(portalApi, /listPortalRoutingDecisionLogs/);
});

test('portal shared types expose routing contracts and expanded route keys', () => {
  const types = read('packages/sdkwork-router-portal-types/src/index.ts');

  assert.match(types, /'routing'/);
  assert.match(types, /'user'/);
  assert.match(types, /interface PortalRoutingSummary/);
  assert.match(types, /interface PortalRoutingPreferences/);
  assert.match(types, /interface PortalRoutingDecision/);
  assert.match(types, /interface PortalRoutingDecisionLog/);
});

test('routing module speaks in user-facing routing posture language', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const routingServices = read('packages/sdkwork-router-portal-routing/src/services/index.ts');

  assert.match(routingPage, /Routing workbench/);
  assert.match(routingPage, /Preset catalog/);
  assert.match(routingPage, /Provider roster/);
  assert.match(routingPage, /Evidence stream/);
  assert.match(routingPage, /Edit routing posture/);
  assert.match(routingPage, /Preview route/);
  assert.match(routingPage, /Routing profile label/);
  assert.match(routingPage, /Capability/);
  assert.match(routingPage, /Requested model/);
  assert.match(routingPage, /Selection seed/);
  assert.match(routingPage, /Search routing evidence/);
  assert.match(routingPage, /Save posture/);
  assert.doesNotMatch(routingPage, /<Tabs/);
  assert.doesNotMatch(routingPage, /Policy editor/);
  assert.doesNotMatch(routingPage, /Recent routing evidence/);
  assert.match(routingServices, /first healthy available provider in your ordered list wins/);
});

test('routing view model tolerates missing preview assessments from live payloads', () => {
  const { buildPortalRoutingViewModel } = loadRoutingServices();
  const now = Date.now();

  const viewModel = buildPortalRoutingViewModel(
    {
      project_id: 'project-demo',
      preferences: {
        project_id: 'project-demo',
        preset_id: 'balanced',
        strategy: 'geo_affinity',
        ordered_provider_ids: ['provider-a'],
        default_provider_id: 'provider-a',
        max_cost: null,
        max_latency_ms: null,
        require_healthy: true,
        preferred_region: 'us-east',
        updated_at_ms: now,
      },
      latest_model_hint: 'gpt-4o-mini',
      preview: {
        selected_provider_id: 'provider-a',
        candidate_ids: ['provider-a'],
        matched_policy_id: null,
        strategy: 'geo_affinity',
        selection_seed: 7,
        selection_reason: 'provider-a matched the region',
        requested_region: 'us-east',
        slo_applied: false,
        slo_degraded: false,
      },
      provider_options: [
        {
          provider_id: 'provider-a',
          display_name: 'Provider A',
          channel_id: 'openai',
          preferred: true,
          default_provider: true,
        },
      ],
    },
    [
      {
        decision_id: 'decision-1',
        decision_source: 'preview',
        capability: 'chat',
        route_key: 'chat',
        selected_provider_id: 'provider-a',
        strategy: 'geo_affinity',
        selection_seed: 7,
        selection_reason: 'provider-a matched the region',
        requested_region: 'us-east',
        slo_applied: false,
        slo_degraded: false,
        created_at_ms: now,
      },
    ],
  );

  assert.deepEqual(viewModel.preview.assessments, []);
  assert.deepEqual(viewModel.logs[0].assessments, []);
});

test('routing view model tolerates missing decision log collections', () => {
  const { buildPortalRoutingViewModel } = loadRoutingServices();
  const now = Date.now();

  const viewModel = buildPortalRoutingViewModel(
    {
      project_id: 'project-demo',
      preferences: {
        project_id: 'project-demo',
        preset_id: 'balanced',
        strategy: 'deterministic_priority',
        ordered_provider_ids: [],
        default_provider_id: null,
        max_cost: null,
        max_latency_ms: null,
        require_healthy: true,
        preferred_region: null,
        updated_at_ms: now,
      },
      latest_model_hint: 'gpt-4o-mini',
      preview: {
        selected_provider_id: 'provider-a',
        candidate_ids: ['provider-a'],
        matched_policy_id: null,
        strategy: 'deterministic_priority',
        selection_seed: null,
        selection_reason: 'provider-a is the default provider',
        requested_region: null,
        slo_applied: false,
        slo_degraded: false,
      },
      provider_options: [],
    },
    undefined,
  );

  assert.deepEqual(viewModel.logs, []);
  assert.deepEqual(viewModel.evidence, []);
});
