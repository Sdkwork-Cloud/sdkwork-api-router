import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('gateway command workbench replaces raw meter templates with localized helper labels', () => {
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');

  assert.doesNotMatch(gatewayPage, /HTTP \$\{check\.httpStatus/);
  assert.doesNotMatch(gatewayPage, /remaining`/);
  assert.doesNotMatch(gatewayPage, /req \/ \$\{policy\.window_seconds\}s/);
  assert.doesNotMatch(gatewayPage, /路/);

  assert.match(gatewayPage, /buildGatewayServiceHealthMeter\(check\)/);
  assert.match(gatewayPage, /buildGatewayRateLimitPolicyMeter\(policy\)/);
  assert.match(gatewayPage, /buildGatewayRateLimitWindowMeter\(window\)/);
  assert.match(gatewayPage, /buildGatewayRateLimitScopeLabel\(/);

  assert.match(commons, /'HTTP \{httpStatus\} · \{latency\}'/);
  assert.match(commons, /'\{limitRequests\} req \/ \{windowSeconds\}s · burst \{burstRequests\}'/);
  assert.match(commons, /'\{requestCount\} \/ \{limitRequests\} requests · \{remainingRequests\} remaining'/);
  assert.match(commons, /'API key \{keyHash\}'/);
  assert.match(commons, /'Route \{routeKey\}'/);
  assert.match(commons, /'Model \{modelName\}'/);
  assert.match(commons, /'No HTTP status'/);
});

test('gateway verification protocol badges render through shared i18n instead of raw literals', () => {
  const gatewayPage = read('packages/sdkwork-router-portal-gateway/src/pages/index.tsx');

  assert.doesNotMatch(gatewayPage, /\? 'OpenAI-compatible'/);
  assert.doesNotMatch(gatewayPage, /\? 'Anthropic Messages'/);
  assert.doesNotMatch(gatewayPage, /: 'Gemini'/);

  assert.match(gatewayPage, /buildGatewayVerificationProtocolLabel\(focus\)/);
  assert.match(gatewayPage, /translatePortalText\('OpenAI-compatible'\)/);
  assert.match(gatewayPage, /translatePortalText\('Anthropic Messages'\)/);
  assert.match(gatewayPage, /translatePortalText\('Gemini'\)/);
});
