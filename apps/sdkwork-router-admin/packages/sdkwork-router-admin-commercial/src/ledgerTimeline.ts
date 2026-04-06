import type {
  CommercialAccountLedgerHistoryEntry,
  CommercialAccountLedgerEntryType,
  CommercialRequestSettlementRecord,
  CommercialRequestSettlementStatus,
} from 'sdkwork-router-admin-types';

export interface CommercialLedgerTimelineRow {
  id: string;
  ledger_entry_id: number;
  account_id: number;
  request_id?: number | null;
  hold_id?: number | null;
  entry_type: CommercialAccountLedgerEntryType;
  benefit_type?: string | null;
  quantity: number;
  amount: number;
  allocation_quantity_delta: number;
  allocation_lot_count: number;
  created_at_ms: number;
  request_settlement_id?: number | null;
  settlement_status?: CommercialRequestSettlementStatus | null;
  captured_credit_amount: number;
  released_credit_amount: number;
  retail_charge_amount: number;
  provider_cost_amount: number;
  refunded_amount: number;
}

function settlementLookupKey(input: {
  account_id: number;
  request_id?: number | null;
  hold_id?: number | null;
}): string {
  return `${input.account_id}:${input.request_id ?? 'na'}:${input.hold_id ?? 'na'}`;
}

function buildSettlementLookup(
  settlements: CommercialRequestSettlementRecord[],
): {
  exact: Map<string, CommercialRequestSettlementRecord>;
  requestOnly: Map<string, CommercialRequestSettlementRecord>;
} {
  const exact = new Map<string, CommercialRequestSettlementRecord>();
  const requestOnly = new Map<string, CommercialRequestSettlementRecord>();

  const orderedSettlements = [...settlements].sort((left, right) =>
    right.updated_at_ms - left.updated_at_ms
    || right.request_settlement_id - left.request_settlement_id,
  );

  for (const settlement of orderedSettlements) {
    const exactKey = settlementLookupKey({
      account_id: settlement.account_id,
      request_id: settlement.request_id,
      hold_id: settlement.hold_id ?? null,
    });
    if (!exact.has(exactKey)) {
      exact.set(exactKey, settlement);
    }

    const requestOnlyKey = settlementLookupKey({
      account_id: settlement.account_id,
      request_id: settlement.request_id,
      hold_id: null,
    });
    if (!requestOnly.has(requestOnlyKey)) {
      requestOnly.set(requestOnlyKey, settlement);
    }
  }

  return { exact, requestOnly };
}

function findMatchingSettlement(
  history: CommercialAccountLedgerHistoryEntry,
  lookup: ReturnType<typeof buildSettlementLookup>,
): CommercialRequestSettlementRecord | null {
  const exactKey = settlementLookupKey({
    account_id: history.entry.account_id,
    request_id: history.entry.request_id ?? null,
    hold_id: history.entry.hold_id ?? null,
  });
  const requestOnlyKey = settlementLookupKey({
    account_id: history.entry.account_id,
    request_id: history.entry.request_id ?? null,
    hold_id: null,
  });

  return lookup.exact.get(exactKey) ?? lookup.requestOnly.get(requestOnlyKey) ?? null;
}

function compareTimelineRows(
  left: CommercialLedgerTimelineRow,
  right: CommercialLedgerTimelineRow,
): number {
  return right.created_at_ms - left.created_at_ms
    || right.ledger_entry_id - left.ledger_entry_id;
}

export function buildCommercialLedgerTimelineRows(
  commercialAccountLedger: CommercialAccountLedgerHistoryEntry[],
  commercialRequestSettlements: CommercialRequestSettlementRecord[],
): CommercialLedgerTimelineRow[] {
  const lookup = buildSettlementLookup(commercialRequestSettlements);

  return commercialAccountLedger
    .map((history) => {
      const settlement = findMatchingSettlement(history, lookup);
      const allocationQuantityDelta = history.allocations.reduce(
        (sum, allocation) => sum + allocation.quantity_delta,
        0,
      );

      return {
        id: `${history.entry.account_id}:${history.entry.ledger_entry_id}`,
        ledger_entry_id: history.entry.ledger_entry_id,
        account_id: history.entry.account_id,
        request_id: history.entry.request_id ?? null,
        hold_id: history.entry.hold_id ?? null,
        entry_type: history.entry.entry_type,
        benefit_type: history.entry.benefit_type ?? null,
        quantity: history.entry.quantity,
        amount: history.entry.amount,
        allocation_quantity_delta: allocationQuantityDelta,
        allocation_lot_count: history.allocations.length,
        created_at_ms: history.entry.created_at_ms,
        request_settlement_id: settlement?.request_settlement_id ?? null,
        settlement_status: settlement?.status ?? null,
        captured_credit_amount: settlement?.captured_credit_amount ?? 0,
        released_credit_amount: settlement?.released_credit_amount ?? 0,
        retail_charge_amount: settlement?.retail_charge_amount ?? 0,
        provider_cost_amount: settlement?.provider_cost_amount ?? 0,
        refunded_amount: settlement?.refunded_amount ?? 0,
      };
    })
    .sort(compareTimelineRows);
}

export function buildCommercialRefundTimelineRows(
  rows: CommercialLedgerTimelineRow[],
): CommercialLedgerTimelineRow[] {
  return rows.filter((row) => row.entry_type === 'refund');
}
