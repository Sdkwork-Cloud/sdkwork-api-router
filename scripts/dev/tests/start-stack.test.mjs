import test from 'node:test';
import assert from 'node:assert/strict';
import {
  databaseDisplayValue,
  parseStackArgs,
  serviceEnv,
} from '../backend-launch-lib.mjs';

test('parseStackArgs keeps local sqlite default when no database override is provided', () => {
  const settings = parseStackArgs([]);

  assert.equal(settings.databaseUrl, null);
  assert.equal(settings.gatewayBind, '127.0.0.1:9980');
  assert.equal(settings.adminBind, '127.0.0.1:9981');
  assert.equal(settings.portalBind, '127.0.0.1:9982');
  assert.equal(databaseDisplayValue(settings), '(local default via config loader)');
});

test('serviceEnv omits SDKWORK_DATABASE_URL when local config defaults should apply', () => {
  const env = serviceEnv(
    {
      databaseUrl: null,
      gatewayBind: '127.0.0.1:9980',
      adminBind: '127.0.0.1:9981',
      portalBind: '127.0.0.1:9982',
    },
    {
      SDKWORK_DATABASE_URL: 'postgres://should-be-removed',
    },
  );

  assert.equal(env.SDKWORK_DATABASE_URL, undefined);
  assert.equal(env.SDKWORK_GATEWAY_BIND, '127.0.0.1:9980');
  assert.equal(env.SDKWORK_ADMIN_BIND, '127.0.0.1:9981');
  assert.equal(env.SDKWORK_PORTAL_BIND, '127.0.0.1:9982');
});
