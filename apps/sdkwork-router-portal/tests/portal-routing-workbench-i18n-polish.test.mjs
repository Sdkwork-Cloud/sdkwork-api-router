import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('routing workbench replaces technical placeholders with localized product guidance', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.doesNotMatch(routingPage, /placeholder="predictable"/);
  assert.doesNotMatch(routingPage, /placeholder="0\.30"/);
  assert.doesNotMatch(routingPage, /placeholder="250"/);
  assert.doesNotMatch(routingPage, /placeholder="chat_completion"/);

  assert.match(routingPage, /placeholder=\{t\('Example: Balanced production posture'\)\}/);
  assert.match(routingPage, /placeholder=\{t\('Example: 0\.30 USD ceiling'\)\}/);
  assert.match(routingPage, /placeholder=\{t\('Example: 250 ms target'\)\}/);
  assert.match(routingPage, /placeholder=\{t\('Example: Chat completions'\)\}/);

  assert.match(commons, /'Example: Balanced production posture'/);
  assert.match(commons, /'Example: 0\.30 USD ceiling'/);
  assert.match(commons, /'Example: 250 ms target'/);
  assert.match(commons, /'Example: Chat completions'/);
});

test('routing evidence rows render product labels for capability and decision source values', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const routingServices = read('packages/sdkwork-router-portal-routing/src/services/index.ts');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.match(routingPage, /buildRoutingCapabilityLabel\(log\.capability\)/);
  assert.match(routingPage, /buildRoutingDecisionSourceLabel\(log\.decision_source\)/);

  assert.match(routingServices, /export function buildRoutingCapabilityLabel/);
  assert.match(routingServices, /export function buildRoutingDecisionSourceLabel/);
  assert.match(routingServices, /translatePortalText\('Chat completions'\)/);
  assert.match(routingServices, /translatePortalText\('Music generation'\)/);
  assert.match(routingServices, /translatePortalText\('Live traffic'\)/);
  assert.match(routingServices, /translatePortalText\('Preview request'\)/);

  assert.match(commons, /'Chat completions'/);
  assert.match(commons, /'Music generation'/);
  assert.match(commons, /'Live traffic'/);
  assert.match(commons, /'Preview request'/);
});

test('routing preview assessments localize health and metric labels instead of rendering raw template strings', () => {
  const routingPage = read('packages/sdkwork-router-portal-routing/src/pages/index.tsx');
  const routingServices = read('packages/sdkwork-router-portal-routing/src/services/index.ts');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.doesNotMatch(routingPage, /t\(assessment\.health\)/);
  assert.doesNotMatch(routingPage, /`Rank \$\{assessment\.policy_rank\}`/);
  assert.doesNotMatch(routingPage, /`Latency \$\{/);
  assert.doesNotMatch(routingPage, /`Cost \$\{/);

  assert.match(routingPage, /buildRoutingAssessmentHealthLabel\(assessment\.health\)/);
  assert.match(routingPage, /t\('Rank \{rank\}', \{ rank: assessment\.policy_rank \}\)/);
  assert.match(routingPage, /t\('Latency \{latency\}', \{/);
  assert.match(routingPage, /t\('Cost \{cost\}', \{/);
  assert.match(routingPage, /t\('No sample'\)/);

  assert.match(routingServices, /export function buildRoutingAssessmentHealthLabel/);
  assert.match(routingServices, /translatePortalText\('Healthy'\)/);
  assert.match(routingServices, /translatePortalText\('Unhealthy'\)/);
  assert.match(routingServices, /translatePortalText\('Unknown'\)/);

  assert.match(commons, /'Rank \{rank\}'/);
  assert.match(commons, /'Latency \{latency\}'/);
  assert.match(commons, /'Cost \{cost\}'/);
  assert.match(commons, /'No sample'/);
  assert.match(commons, /'Unhealthy'/);
  assert.match(commons, /'Unknown'/);
});
