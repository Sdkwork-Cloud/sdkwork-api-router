import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';

import jiti from '../node_modules/.pnpm/jiti@2.6.1/node_modules/jiti/lib/jiti.mjs';

const appRoot = path.resolve(import.meta.dirname, '..');

function loadLedgerTimelineModule() {
  const load = jiti(import.meta.url, { moduleCache: false });
  return load(
    path.join(
      appRoot,
      'packages',
      'sdkwork-router-admin-commercial',
      'src',
      'ledgerTimeline.ts',
    ),
  );
}

test('admin commercial ledger timeline sorts capture and refund history while attaching settlement evidence', () => {
  const { buildCommercialLedgerTimelineRows, buildCommercialRefundTimelineRows } =
    loadLedgerTimelineModule();

  const ledgerHistory = [
    {
      entry: {
        ledger_entry_id: 8401,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        request_id: 6001,
        hold_id: 8101,
        entry_type: 'settlement_capture',
        benefit_type: null,
        quantity: 5,
        amount: 5,
        created_at_ms: 14,
      },
      allocations: [
        {
          ledger_allocation_id: 8501,
          tenant_id: 1001,
          organization_id: 2002,
          ledger_entry_id: 8401,
          lot_id: 8001,
          quantity_delta: -5,
          created_at_ms: 14,
        },
      ],
    },
    {
      entry: {
        ledger_entry_id: 8402,
        tenant_id: 1001,
        organization_id: 2002,
        account_id: 7001,
        user_id: 9001,
        request_id: 6001,
        hold_id: 8101,
        entry_type: 'refund',
        benefit_type: null,
        quantity: 2,
        amount: 2,
        created_at_ms: 15,
      },
      allocations: [
        {
          ledger_allocation_id: 8502,
          tenant_id: 1001,
          organization_id: 2002,
          ledger_entry_id: 8402,
          lot_id: 8001,
          quantity_delta: 2,
          created_at_ms: 15,
        },
      ],
    },
  ];

  const settlements = [
    {
      request_settlement_id: 8301,
      tenant_id: 1001,
      organization_id: 2002,
      request_id: 6001,
      account_id: 7001,
      user_id: 9001,
      hold_id: 8101,
      status: 'refunded',
      estimated_credit_hold: 5,
      released_credit_amount: 0,
      captured_credit_amount: 5,
      provider_cost_amount: 2.5,
      retail_charge_amount: 5,
      shortfall_amount: 0,
      refunded_amount: 2,
      settled_at_ms: 15,
      created_at_ms: 14,
      updated_at_ms: 15,
    },
  ];

  const rows = buildCommercialLedgerTimelineRows(ledgerHistory, settlements);
  const refundRows = buildCommercialRefundTimelineRows(rows);

  assert.equal(rows.length, 2);
  assert.equal(rows[0].ledger_entry_id, 8402);
  assert.equal(rows[0].entry_type, 'refund');
  assert.equal(rows[0].settlement_status, 'refunded');
  assert.equal(rows[0].refunded_amount, 2);
  assert.equal(rows[0].allocation_quantity_delta, 2);
  assert.equal(rows[1].ledger_entry_id, 8401);
  assert.equal(rows[1].entry_type, 'settlement_capture');
  assert.equal(rows[1].captured_credit_amount, 5);
  assert.equal(rows[1].allocation_quantity_delta, -5);

  assert.equal(refundRows.length, 1);
  assert.equal(refundRows[0].ledger_entry_id, 8402);
  assert.equal(refundRows[0].entry_type, 'refund');
});
