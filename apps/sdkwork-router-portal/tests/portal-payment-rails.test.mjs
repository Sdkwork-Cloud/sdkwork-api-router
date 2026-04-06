import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('portal billing surfaces structured payment rails and callback payload evidence', () => {
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');
  const portalTypes = read('packages/sdkwork-router-portal-types/src/index.ts');
  const zhMessages = read('packages/sdkwork-router-portal-commons/src/portalMessages.zh-CN.ts');

  assert.match(portalTypes, /export type PortalCommercePaymentProvider =/);
  assert.match(portalTypes, /'stripe'/);
  assert.match(portalTypes, /'alipay'/);
  assert.match(portalTypes, /'wechat_pay'/);
  assert.match(portalTypes, /export type PortalCommerceCheckoutMethodChannel =/);
  assert.match(portalTypes, /export type PortalCommerceCheckoutMethodSessionKind =/);
  assert.match(portalTypes, /'operator_action'/);
  assert.match(portalTypes, /'hosted_checkout'/);
  assert.match(portalTypes, /'qr_code'/);
  assert.match(portalTypes, /supports_webhook: boolean;/);
  assert.match(portalTypes, /recommended: boolean;/);
  assert.match(portalTypes, /session_kind: PortalCommerceCheckoutMethodSessionKind;/);
  assert.match(portalTypes, /session_reference: string;/);
  assert.match(portalTypes, /qr_code_payload\?: string \| null;/);
  assert.match(portalTypes, /webhook_verification: string;/);
  assert.match(portalTypes, /supports_refund: boolean;/);
  assert.match(portalTypes, /supports_partial_refund: boolean;/);
  assert.match(portalTypes, /checkout_method_id\?: string \| null;/);

  assert.match(billingPage, /function checkoutMethodProviderLabel\(/);
  assert.match(billingPage, /function checkoutMethodChannelLabel\(/);
  assert.match(billingPage, /function checkoutMethodSessionKindLabel\(/);
  assert.match(billingPage, /function buildProviderEventReplayId\(/);
  assert.match(billingPage, /function preferredProviderCallbackMethod\(/);
  assert.match(billingPage, /providerCallbackMethodId/);
  assert.match(billingPage, /provider:\s*method\.provider/);
  assert.match(billingPage, /provider_event_id:\s*buildProviderEventReplayId\(/);
  assert.match(billingPage, /checkout_method_id:\s*method\.id/);
  assert.match(billingPage, /method\.session_reference/);
  assert.match(billingPage, /method\.webhook_verification/);
  assert.match(billingPage, /method\.supports_refund/);
  assert.match(billingPage, /method\.supports_partial_refund/);
  assert.match(billingPage, /method\.qr_code_payload/);
  assert.match(billingPage, /Session reference/);
  assert.match(billingPage, /Webhook verification/);
  assert.match(billingPage, /Refund support/);
  assert.match(billingPage, /Partial refund/);
  assert.match(billingPage, /QR payload/);
  assert.match(billingPage, /Hosted checkout session/);
  assert.match(billingPage, /QR code session/);
  assert.match(billingPage, /Operator action/);
  assert.match(billingPage, /Stripe/);
  assert.match(billingPage, /Alipay/);
  assert.match(billingPage, /WeChat Pay/);
  assert.match(billingPage, /Hosted checkout/);
  assert.match(billingPage, /Scan QR/);
  assert.match(zhMessages, /'Manual lab':/);
  assert.match(zhMessages, /'WeChat Pay':/);
  assert.match(zhMessages, /'Hosted checkout':/);
  assert.match(zhMessages, /'Scan QR':/);
  assert.match(zhMessages, /'Session reference':/);
  assert.match(zhMessages, /'Webhook verification':/);
  assert.match(zhMessages, /'Refund support':/);
  assert.match(zhMessages, /'Partial refund':/);
  assert.match(zhMessages, /'QR payload':/);
  assert.match(zhMessages, /'Hosted checkout session':/);
  assert.match(zhMessages, /'QR code session':/);
  assert.match(zhMessages, /'Operator action':/);
});
