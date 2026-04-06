use anyhow::{ensure, Result};
use async_trait::async_trait;
use sdkwork_api_app_identity::{
    gateway_auth_subject_from_request_context,
    GatewayRequestContext as IdentityGatewayRequestContext,
};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountCommerceReconciliationStateRecord, AccountHoldAllocationRecord, AccountHoldRecord,
    AccountHoldStatus, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountLedgerEntryType, AccountRecord, AccountStatus, AccountType,
    BillingEventAccountingModeSummary, BillingEventCapabilitySummary, BillingEventGroupSummary,
    BillingEventProjectSummary, BillingEventRecord, BillingEventSummary, BillingSummary,
    LedgerEntry, PricingPlanRecord, PricingRateRecord, ProjectBillingSummary,
    RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_domain_identity::GatewayAuthSubject;
use sdkwork_api_policy_quota::{
    builtin_quota_policy_registry, QuotaPolicyExecutionInput, STRICTEST_LIMIT_QUOTA_POLICY_ID,
};
use sdkwork_api_storage_core::{
    AccountKernelStore, AccountKernelTransaction, AccountKernelTransactionExecutor, AdminStore,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use utoipa::ToSchema;

pub use sdkwork_api_domain_billing::{BillingAccountingMode, QuotaCheckResult, QuotaPolicy};

pub fn service_name() -> &'static str {
    "billing-service"
}

pub struct CreateBillingEventInput<'a> {
    pub event_id: &'a str,
    pub tenant_id: &'a str,
    pub project_id: &'a str,
    pub api_key_group_id: Option<&'a str>,
    pub capability: &'a str,
    pub route_key: &'a str,
    pub usage_model: &'a str,
    pub provider_id: &'a str,
    pub accounting_mode: BillingAccountingMode,
    pub operation_kind: &'a str,
    pub modality: &'a str,
    pub api_key_hash: Option<&'a str>,
    pub channel_id: Option<&'a str>,
    pub reference_id: Option<&'a str>,
    pub latency_ms: Option<u64>,
    pub units: u64,
    pub request_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub image_count: u64,
    pub audio_seconds: f64,
    pub video_seconds: f64,
    pub music_seconds: f64,
    pub upstream_cost: f64,
    pub customer_charge: f64,
    pub applied_routing_profile_id: Option<&'a str>,
    pub compiled_routing_snapshot_id: Option<&'a str>,
    pub fallback_reason: Option<&'a str>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountLotBalanceSnapshot {
    pub lot_id: u64,
    pub benefit_type: AccountBenefitType,
    pub scope_json: Option<String>,
    pub expires_at_ms: Option<u64>,
    pub original_quantity: f64,
    pub remaining_quantity: f64,
    pub held_quantity: f64,
    pub available_quantity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountBalanceSnapshot {
    pub account_id: u64,
    pub available_balance: f64,
    pub held_balance: f64,
    pub consumed_balance: f64,
    pub grant_balance: f64,
    pub active_lot_count: u64,
    pub lots: Vec<AccountLotBalanceSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountLedgerHistoryEntry {
    pub entry: AccountLedgerEntryRecord,
    pub allocations: Vec<AccountLedgerAllocationRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedHoldAllocation {
    pub lot_id: u64,
    pub quantity: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountHoldPlan {
    pub account_id: u64,
    pub requested_quantity: f64,
    pub covered_quantity: f64,
    pub shortfall_quantity: f64,
    pub sufficient_balance: bool,
    pub allocations: Vec<PlannedHoldAllocation>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreateAccountHoldInput {
    pub hold_id: u64,
    pub hold_allocation_start_id: u64,
    pub request_id: u64,
    pub account_id: u64,
    pub requested_quantity: f64,
    pub expires_at_ms: u64,
    pub now_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReleaseAccountHoldInput {
    pub request_id: u64,
    pub released_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaptureAccountHoldInput {
    pub request_settlement_id: u64,
    pub request_id: u64,
    pub captured_quantity: f64,
    pub provider_cost_amount: f64,
    pub retail_charge_amount: f64,
    pub settled_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountHoldMutationResult {
    pub idempotent_replay: bool,
    pub hold: AccountHoldRecord,
    pub allocations: Vec<AccountHoldAllocationRecord>,
    pub updated_lots: Vec<AccountBenefitLotRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureAccountHoldResult {
    pub idempotent_replay: bool,
    pub hold: AccountHoldRecord,
    pub allocations: Vec<AccountHoldAllocationRecord>,
    pub updated_lots: Vec<AccountBenefitLotRecord>,
    pub settlement: RequestSettlementRecord,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RefundAccountSettlementInput {
    pub request_settlement_id: u64,
    pub refund_ledger_entry_id: u64,
    pub refund_ledger_allocation_start_id: u64,
    pub refunded_amount: f64,
    pub refunded_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefundAccountSettlementResult {
    pub idempotent_replay: bool,
    pub settlement: RequestSettlementRecord,
    pub updated_lots: Vec<AccountBenefitLotRecord>,
    pub ledger_entry: AccountLedgerEntryRecord,
    pub ledger_allocations: Vec<AccountLedgerAllocationRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IssueCommerceOrderCreditsInput<'a> {
    pub account_id: u64,
    pub order_id: &'a str,
    pub project_id: &'a str,
    pub target_kind: &'a str,
    pub granted_quantity: f64,
    pub payable_amount: f64,
    pub issued_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IssueCommerceOrderCreditsResult {
    pub idempotent_replay: bool,
    pub lot: AccountBenefitLotRecord,
    pub ledger_entry: AccountLedgerEntryRecord,
    pub ledger_allocations: Vec<AccountLedgerAllocationRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RefundCommerceOrderCreditsInput<'a> {
    pub account_id: u64,
    pub order_id: &'a str,
    pub refunded_quantity: f64,
    pub refunded_amount: f64,
    pub refunded_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefundCommerceOrderCreditsResult {
    pub idempotent_replay: bool,
    pub lot: AccountBenefitLotRecord,
    pub ledger_entry: AccountLedgerEntryRecord,
    pub ledger_allocations: Vec<AccountLedgerAllocationRecord>,
}

pub fn create_quota_policy(
    policy_id: &str,
    project_id: &str,
    max_units: u64,
    enabled: bool,
) -> Result<QuotaPolicy> {
    ensure!(!policy_id.trim().is_empty(), "policy_id must not be empty");
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(max_units > 0, "max_units must be greater than 0");

    Ok(QuotaPolicy::new(policy_id, project_id, max_units).with_enabled(enabled))
}

pub fn book_usage_cost(project_id: &str, units: u64, amount: f64) -> Result<LedgerEntry> {
    Ok(LedgerEntry::new(project_id, units, amount))
}

pub fn create_billing_event(input: CreateBillingEventInput<'_>) -> Result<BillingEventRecord> {
    ensure!(
        !input.event_id.trim().is_empty(),
        "event_id must not be empty"
    );
    ensure!(
        !input.tenant_id.trim().is_empty(),
        "tenant_id must not be empty"
    );
    ensure!(
        !input.project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(
        !input.capability.trim().is_empty(),
        "capability must not be empty"
    );
    ensure!(
        !input.usage_model.trim().is_empty(),
        "usage_model must not be empty"
    );
    ensure!(
        !input.provider_id.trim().is_empty(),
        "provider_id must not be empty"
    );
    ensure!(
        !input.operation_kind.trim().is_empty(),
        "operation_kind must not be empty"
    );
    ensure!(
        !input.modality.trim().is_empty(),
        "modality must not be empty"
    );
    ensure!(
        input.upstream_cost >= 0.0,
        "upstream_cost must not be negative"
    );
    ensure!(
        input.customer_charge >= 0.0,
        "customer_charge must not be negative"
    );

    let route_key = if input.route_key.trim().is_empty() {
        input.usage_model.trim()
    } else {
        input.route_key.trim()
    };
    let request_count = input.request_count.max(1);
    let total_tokens = if input.total_tokens == 0 {
        input.input_tokens.saturating_add(input.output_tokens)
    } else {
        input.total_tokens
    };

    let mut event = BillingEventRecord::new(
        input.event_id.trim(),
        input.tenant_id.trim(),
        input.project_id.trim(),
        input.capability.trim(),
        route_key,
        input.usage_model.trim(),
        input.provider_id.trim(),
        input.accounting_mode,
        input.created_at_ms,
    )
    .with_operation(input.operation_kind.trim(), input.modality.trim())
    .with_request_facts(
        input.api_key_hash.map(str::trim),
        input.channel_id.map(str::trim),
        input.reference_id.map(str::trim),
        input.latency_ms,
    )
    .with_units(input.units)
    .with_request_count(request_count)
    .with_token_usage(input.input_tokens, input.output_tokens, total_tokens)
    .with_cache_token_usage(input.cache_read_tokens, input.cache_write_tokens)
    .with_media_usage(
        input.image_count,
        input.audio_seconds,
        input.video_seconds,
        input.music_seconds,
    )
    .with_financials(input.upstream_cost, input.customer_charge)
    .with_routing_evidence(
        input.applied_routing_profile_id.map(str::trim),
        input.compiled_routing_snapshot_id.map(str::trim),
        input.fallback_reason.map(str::trim),
    );

    if let Some(api_key_group_id) = input.api_key_group_id.map(str::trim) {
        if !api_key_group_id.is_empty() {
            event = event.with_api_key_group_id(api_key_group_id);
        }
    }

    Ok(event)
}

pub async fn summarize_account_balance<S>(
    store: &S,
    account_id: u64,
    now_ms: u64,
) -> Result<AccountBalanceSnapshot>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        store.find_account_record(account_id).await?.is_some(),
        "account {account_id} does not exist"
    );

    let account_lots = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == account_id)
        .collect::<Vec<_>>();
    let active_lots = eligible_lots_for_hold(&account_lots, now_ms);

    let available_balance = active_lots.iter().map(|lot| free_quantity(lot)).sum();
    let held_balance = account_lots.iter().map(|lot| lot.held_quantity).sum();
    let consumed_balance = account_lots
        .iter()
        .map(|lot| (lot.original_quantity - lot.remaining_quantity).max(0.0))
        .sum();
    let grant_balance = account_lots.iter().map(|lot| lot.original_quantity).sum();
    let lots = active_lots
        .into_iter()
        .map(|lot| AccountLotBalanceSnapshot {
            lot_id: lot.lot_id,
            benefit_type: lot.benefit_type,
            scope_json: lot.scope_json.clone(),
            expires_at_ms: lot.expires_at_ms,
            original_quantity: lot.original_quantity,
            remaining_quantity: lot.remaining_quantity,
            held_quantity: lot.held_quantity,
            available_quantity: free_quantity(lot),
        })
        .collect::<Vec<_>>();

    Ok(AccountBalanceSnapshot {
        account_id,
        available_balance,
        held_balance,
        consumed_balance,
        grant_balance,
        active_lot_count: lots.len() as u64,
        lots,
    })
}

pub async fn list_account_ledger_history<S>(
    store: &S,
    account_id: u64,
) -> Result<Vec<AccountLedgerHistoryEntry>>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        store.find_account_record(account_id).await?.is_some(),
        "account {account_id} does not exist"
    );

    let mut allocations_by_entry_id = BTreeMap::<u64, Vec<AccountLedgerAllocationRecord>>::new();
    for allocation in store.list_account_ledger_allocations().await? {
        allocations_by_entry_id
            .entry(allocation.ledger_entry_id)
            .or_default()
            .push(allocation);
    }
    for allocations in allocations_by_entry_id.values_mut() {
        allocations.sort_by_key(|allocation| allocation.ledger_allocation_id);
    }

    let mut history = store
        .list_account_ledger_entry_records()
        .await?
        .into_iter()
        .filter(|entry| entry.account_id == account_id)
        .map(|entry| AccountLedgerHistoryEntry {
            allocations: allocations_by_entry_id
                .remove(&entry.ledger_entry_id)
                .unwrap_or_default(),
            entry,
        })
        .collect::<Vec<_>>();
    history.sort_by(|left, right| {
        right
            .entry
            .created_at_ms
            .cmp(&left.entry.created_at_ms)
            .then_with(|| right.entry.ledger_entry_id.cmp(&left.entry.ledger_entry_id))
    });
    Ok(history)
}

pub async fn plan_account_hold<S>(
    store: &S,
    account_id: u64,
    requested_quantity: f64,
    now_ms: u64,
) -> Result<AccountHoldPlan>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        requested_quantity > 0.0,
        "requested_quantity must be positive"
    );

    ensure!(
        store.find_account_record(account_id).await?.is_some(),
        "account {account_id} does not exist"
    );

    let lots = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == account_id)
        .collect::<Vec<_>>();
    let eligible_lots = eligible_lots_for_hold(&lots, now_ms);
    let mut remaining = requested_quantity;
    let mut allocations = Vec::new();

    for lot in eligible_lots {
        if remaining <= 0.0 {
            break;
        }
        let quantity = free_quantity(lot).min(remaining);
        if quantity <= 0.0 {
            continue;
        }
        allocations.push(PlannedHoldAllocation {
            lot_id: lot.lot_id,
            quantity,
        });
        remaining -= quantity;
    }

    let covered_quantity = requested_quantity - remaining.max(0.0);
    let shortfall_quantity = remaining.max(0.0);

    Ok(AccountHoldPlan {
        account_id,
        requested_quantity,
        covered_quantity,
        shortfall_quantity,
        sufficient_balance: shortfall_quantity <= f64::EPSILON,
        allocations,
    })
}

pub async fn resolve_payable_account_for_gateway_subject<S>(
    store: &S,
    subject: &GatewayAuthSubject,
) -> Result<Option<AccountRecord>>
where
    S: AccountKernelStore + ?Sized,
{
    let Some(account) = store
        .find_account_record_by_owner(
            subject.tenant_id,
            subject.organization_id,
            subject.user_id,
            AccountType::Primary,
        )
        .await?
    else {
        return Ok(None);
    };

    ensure!(
        account.status == AccountStatus::Active,
        "primary account {} is not active",
        account.account_id
    );

    Ok(Some(account))
}

pub async fn resolve_payable_account_for_gateway_request_context<S>(
    store: &S,
    context: &IdentityGatewayRequestContext,
) -> Result<Option<AccountRecord>>
where
    S: AccountKernelStore + ?Sized,
{
    let subject = gateway_auth_subject_from_request_context(context);
    resolve_payable_account_for_gateway_subject(store, &subject).await
}

pub async fn create_account_hold<S>(
    store: &S,
    input: CreateAccountHoldInput,
) -> Result<AccountHoldMutationResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    ensure!(
        input.requested_quantity > 0.0,
        "requested_quantity must be positive"
    );

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                if let Some(existing_hold) =
                    tx.find_account_hold_by_request_id(input.request_id).await?
                {
                    let (allocations, updated_lots) =
                        load_hold_allocations_and_lots(tx, existing_hold.hold_id).await?;
                    return Ok(AccountHoldMutationResult {
                        idempotent_replay: true,
                        hold: existing_hold,
                        allocations,
                        updated_lots,
                    });
                }

                let account = tx
                    .find_account_record(input.account_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("account {} does not exist", input.account_id)
                    })?;
                let lots = tx
                    .list_account_benefit_lots_for_account(input.account_id)
                    .await?;
                let eligible_lots = eligible_lots_for_hold(&lots, input.now_ms);
                let mut remaining = input.requested_quantity;
                let mut allocations = Vec::new();
                let mut updated_lots = Vec::new();

                for (index, lot) in eligible_lots.into_iter().enumerate() {
                    if remaining <= f64::EPSILON {
                        break;
                    }
                    let quantity = free_quantity(lot).min(remaining);
                    if quantity <= f64::EPSILON {
                        continue;
                    }

                    let updated_lot = lot
                        .clone()
                        .with_held_quantity(lot.held_quantity + quantity)
                        .with_updated_at_ms(input.now_ms);
                    tx.upsert_account_benefit_lot(&updated_lot).await?;
                    updated_lots.push(updated_lot.clone());

                    let allocation = AccountHoldAllocationRecord::new(
                        input.hold_allocation_start_id + index as u64,
                        account.tenant_id,
                        account.organization_id,
                        input.hold_id,
                        lot.lot_id,
                    )
                    .with_allocated_quantity(quantity)
                    .with_created_at_ms(input.now_ms)
                    .with_updated_at_ms(input.now_ms);
                    tx.upsert_account_hold_allocation(&allocation).await?;
                    allocations.push(allocation);

                    remaining -= quantity;
                }

                ensure!(
                    remaining <= f64::EPSILON,
                    "account {} has insufficient available balance for request {}",
                    input.account_id,
                    input.request_id
                );

                let hold = AccountHoldRecord::new(
                    input.hold_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    input.request_id,
                )
                .with_estimated_quantity(input.requested_quantity)
                .with_expires_at_ms(input.expires_at_ms)
                .with_created_at_ms(input.now_ms)
                .with_updated_at_ms(input.now_ms);
                tx.upsert_account_hold(&hold).await?;

                let hold_ledger_entry = AccountLedgerEntryRecord::new(
                    account_ledger_entry_id(hold.hold_id, HOLD_CREATE_LEDGER_SUFFIX),
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountLedgerEntryType::HoldCreate,
                )
                .with_request_id(Some(input.request_id))
                .with_hold_id(Some(hold.hold_id))
                .with_quantity(input.requested_quantity)
                .with_created_at_ms(input.now_ms);
                let hold_ledger_allocations = allocations
                    .iter()
                    .map(|allocation| {
                        (
                            account_ledger_allocation_id(
                                allocation.hold_allocation_id,
                                HOLD_CREATE_LEDGER_SUFFIX,
                            ),
                            allocation.lot_id,
                            -allocation.allocated_quantity,
                        )
                    })
                    .collect::<Vec<_>>();
                write_account_ledger_entry(
                    tx,
                    &hold_ledger_entry,
                    &hold_ledger_allocations,
                    input.now_ms,
                )
                .await?;

                Ok(AccountHoldMutationResult {
                    idempotent_replay: false,
                    hold,
                    allocations,
                    updated_lots,
                })
            })
        })
        .await
}

pub async fn release_account_hold<S>(
    store: &S,
    input: ReleaseAccountHoldInput,
) -> Result<AccountHoldMutationResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let hold = tx
                    .find_account_hold_by_request_id(input.request_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("hold for request {} does not exist", input.request_id)
                    })?;
                if hold.status == AccountHoldStatus::Released {
                    let (allocations, updated_lots) =
                        load_hold_allocations_and_lots(tx, hold.hold_id).await?;
                    return Ok(AccountHoldMutationResult {
                        idempotent_replay: true,
                        hold,
                        allocations,
                        updated_lots,
                    });
                }

                let existing_allocations = tx
                    .list_account_hold_allocations_for_hold(hold.hold_id)
                    .await?;
                let mut allocations = Vec::with_capacity(existing_allocations.len());
                let mut updated_lots = Vec::with_capacity(existing_allocations.len());
                let mut released_quantity = hold.released_quantity;
                let mut released_now_total = 0.0;
                let mut released_ledger_allocations = Vec::new();

                for allocation in existing_allocations {
                    let releasable = (allocation.allocated_quantity
                        - allocation.captured_quantity
                        - allocation.released_quantity)
                        .max(0.0);
                    let lot = tx
                        .find_account_benefit_lot(allocation.lot_id)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!("lot {} does not exist", allocation.lot_id)
                        })?;

                    let updated_lot = lot
                        .clone()
                        .with_held_quantity((lot.held_quantity - releasable).max(0.0))
                        .with_updated_at_ms(input.released_at_ms);
                    tx.upsert_account_benefit_lot(&updated_lot).await?;
                    updated_lots.push(updated_lot);

                    let updated_allocation = allocation
                        .clone()
                        .with_released_quantity(allocation.released_quantity + releasable)
                        .with_updated_at_ms(input.released_at_ms);
                    tx.upsert_account_hold_allocation(&updated_allocation)
                        .await?;
                    allocations.push(updated_allocation);

                    released_quantity += releasable;
                    released_now_total += releasable;
                    if releasable > f64::EPSILON {
                        released_ledger_allocations.push((
                            account_ledger_allocation_id(
                                allocation.hold_allocation_id,
                                HOLD_RELEASE_LEDGER_SUFFIX,
                            ),
                            allocation.lot_id,
                            releasable,
                        ));
                    }
                }

                let partially_captured = hold.captured_quantity > 0.0;
                let updated_hold = hold
                    .with_status(if partially_captured {
                        AccountHoldStatus::PartiallyReleased
                    } else {
                        AccountHoldStatus::Released
                    })
                    .with_released_quantity(released_quantity)
                    .with_updated_at_ms(input.released_at_ms);
                tx.upsert_account_hold(&updated_hold).await?;

                if released_now_total > f64::EPSILON {
                    let release_ledger_entry = AccountLedgerEntryRecord::new(
                        account_ledger_entry_id(updated_hold.hold_id, HOLD_RELEASE_LEDGER_SUFFIX),
                        updated_hold.tenant_id,
                        updated_hold.organization_id,
                        updated_hold.account_id,
                        updated_hold.user_id,
                        AccountLedgerEntryType::HoldRelease,
                    )
                    .with_request_id(Some(updated_hold.request_id))
                    .with_hold_id(Some(updated_hold.hold_id))
                    .with_quantity(released_now_total)
                    .with_created_at_ms(input.released_at_ms);
                    write_account_ledger_entry(
                        tx,
                        &release_ledger_entry,
                        &released_ledger_allocations,
                        input.released_at_ms,
                    )
                    .await?;
                }

                Ok(AccountHoldMutationResult {
                    idempotent_replay: false,
                    hold: updated_hold,
                    allocations,
                    updated_lots,
                })
            })
        })
        .await
}

pub async fn capture_account_hold<S>(
    store: &S,
    input: CaptureAccountHoldInput,
) -> Result<CaptureAccountHoldResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    ensure!(
        input.captured_quantity > 0.0,
        "captured_quantity must be positive"
    );
    ensure!(
        input.provider_cost_amount >= 0.0,
        "provider_cost_amount must not be negative"
    );
    ensure!(
        input.retail_charge_amount >= 0.0,
        "retail_charge_amount must not be negative"
    );

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let hold = tx
                    .find_account_hold_by_request_id(input.request_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("hold for request {} does not exist", input.request_id)
                    })?;

                if let Some(existing_settlement) = tx
                    .find_request_settlement_by_request_id(input.request_id)
                    .await?
                {
                    let (allocations, updated_lots) =
                        load_hold_allocations_and_lots(tx, hold.hold_id).await?;
                    return Ok(CaptureAccountHoldResult {
                        idempotent_replay: true,
                        hold,
                        allocations,
                        updated_lots,
                        settlement: existing_settlement,
                    });
                }

                let existing_allocations = tx
                    .list_account_hold_allocations_for_hold(hold.hold_id)
                    .await?;
                let mut allocations = Vec::with_capacity(existing_allocations.len());
                let mut updated_lots = Vec::with_capacity(existing_allocations.len());
                let mut remaining_capture = input.captured_quantity;
                let mut captured_quantity = hold.captured_quantity;
                let mut released_quantity = hold.released_quantity;
                let mut captured_now_total = 0.0;
                let mut released_now_total = 0.0;
                let mut captured_ledger_allocations = Vec::new();
                let mut released_ledger_allocations = Vec::new();

                for allocation in existing_allocations {
                    let available = (allocation.allocated_quantity
                        - allocation.captured_quantity
                        - allocation.released_quantity)
                        .max(0.0);
                    let captured_now = available.min(remaining_capture);
                    remaining_capture -= captured_now;
                    let released_now = available - captured_now;

                    let lot = tx
                        .find_account_benefit_lot(allocation.lot_id)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!("lot {} does not exist", allocation.lot_id)
                        })?;
                    let updated_lot = lot
                        .clone()
                        .with_remaining_quantity((lot.remaining_quantity - captured_now).max(0.0))
                        .with_held_quantity((lot.held_quantity - available).max(0.0))
                        .with_updated_at_ms(input.settled_at_ms);
                    tx.upsert_account_benefit_lot(&updated_lot).await?;
                    updated_lots.push(updated_lot);

                    let updated_allocation = allocation
                        .clone()
                        .with_captured_quantity(allocation.captured_quantity + captured_now)
                        .with_released_quantity(allocation.released_quantity + released_now)
                        .with_updated_at_ms(input.settled_at_ms);
                    tx.upsert_account_hold_allocation(&updated_allocation)
                        .await?;
                    allocations.push(updated_allocation);

                    captured_quantity += captured_now;
                    released_quantity += released_now;
                    captured_now_total += captured_now;
                    released_now_total += released_now;
                    if captured_now > f64::EPSILON {
                        captured_ledger_allocations.push((
                            account_ledger_allocation_id(
                                allocation.hold_allocation_id,
                                SETTLEMENT_CAPTURE_LEDGER_SUFFIX,
                            ),
                            allocation.lot_id,
                            -captured_now,
                        ));
                    }
                    if released_now > f64::EPSILON {
                        released_ledger_allocations.push((
                            account_ledger_allocation_id(
                                allocation.hold_allocation_id,
                                SETTLEMENT_RELEASE_LEDGER_SUFFIX,
                            ),
                            allocation.lot_id,
                            released_now,
                        ));
                    }
                }

                ensure!(
                    remaining_capture <= f64::EPSILON,
                    "capture quantity {} exceeds held quantity for request {}",
                    input.captured_quantity,
                    input.request_id
                );

                let status = if released_quantity > f64::EPSILON {
                    AccountHoldStatus::PartiallyReleased
                } else {
                    AccountHoldStatus::Captured
                };
                let updated_hold = hold
                    .with_status(status)
                    .with_captured_quantity(captured_quantity)
                    .with_released_quantity(released_quantity)
                    .with_updated_at_ms(input.settled_at_ms);
                tx.upsert_account_hold(&updated_hold).await?;

                let settlement = RequestSettlementRecord::new(
                    input.request_settlement_id,
                    updated_hold.tenant_id,
                    updated_hold.organization_id,
                    input.request_id,
                    updated_hold.account_id,
                    updated_hold.user_id,
                )
                .with_hold_id(Some(updated_hold.hold_id))
                .with_status(if released_quantity > f64::EPSILON {
                    RequestSettlementStatus::PartiallyReleased
                } else {
                    RequestSettlementStatus::Captured
                })
                .with_estimated_credit_hold(updated_hold.estimated_quantity)
                .with_released_credit_amount(released_quantity)
                .with_captured_credit_amount(captured_quantity)
                .with_provider_cost_amount(input.provider_cost_amount)
                .with_retail_charge_amount(input.retail_charge_amount)
                .with_shortfall_amount(
                    (input.retail_charge_amount - input.captured_quantity).max(0.0),
                )
                .with_settled_at_ms(input.settled_at_ms)
                .with_created_at_ms(input.settled_at_ms)
                .with_updated_at_ms(input.settled_at_ms);
                tx.upsert_request_settlement_record(&settlement).await?;

                if captured_now_total > f64::EPSILON {
                    let capture_ledger_entry = AccountLedgerEntryRecord::new(
                        account_ledger_entry_id(
                            input.request_settlement_id,
                            SETTLEMENT_CAPTURE_LEDGER_SUFFIX,
                        ),
                        updated_hold.tenant_id,
                        updated_hold.organization_id,
                        updated_hold.account_id,
                        updated_hold.user_id,
                        AccountLedgerEntryType::SettlementCapture,
                    )
                    .with_request_id(Some(input.request_id))
                    .with_hold_id(Some(updated_hold.hold_id))
                    .with_quantity(captured_now_total)
                    .with_amount(input.retail_charge_amount)
                    .with_created_at_ms(input.settled_at_ms);
                    write_account_ledger_entry(
                        tx,
                        &capture_ledger_entry,
                        &captured_ledger_allocations,
                        input.settled_at_ms,
                    )
                    .await?;
                }

                if released_now_total > f64::EPSILON {
                    let release_ledger_entry = AccountLedgerEntryRecord::new(
                        account_ledger_entry_id(
                            input.request_settlement_id,
                            SETTLEMENT_RELEASE_LEDGER_SUFFIX,
                        ),
                        updated_hold.tenant_id,
                        updated_hold.organization_id,
                        updated_hold.account_id,
                        updated_hold.user_id,
                        AccountLedgerEntryType::HoldRelease,
                    )
                    .with_request_id(Some(input.request_id))
                    .with_hold_id(Some(updated_hold.hold_id))
                    .with_quantity(released_now_total)
                    .with_created_at_ms(input.settled_at_ms);
                    write_account_ledger_entry(
                        tx,
                        &release_ledger_entry,
                        &released_ledger_allocations,
                        input.settled_at_ms,
                    )
                    .await?;
                }

                Ok(CaptureAccountHoldResult {
                    idempotent_replay: false,
                    hold: updated_hold,
                    allocations,
                    updated_lots,
                    settlement,
                })
            })
        })
        .await
}

pub async fn refund_account_settlement<S>(
    store: &S,
    input: RefundAccountSettlementInput,
) -> Result<RefundAccountSettlementResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    ensure!(
        input.refunded_amount > 0.0,
        "refunded_amount must be positive"
    );

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let settlement = tx
                    .find_request_settlement_record(input.request_settlement_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "request settlement {} does not exist",
                            input.request_settlement_id
                        )
                    })?;

                if let Some(existing_ledger_entry) = tx
                    .find_account_ledger_entry_record(input.refund_ledger_entry_id)
                    .await?
                {
                    ensure!(
                        existing_ledger_entry.entry_type == AccountLedgerEntryType::Refund,
                        "ledger entry {} is not a refund entry",
                        input.refund_ledger_entry_id
                    );
                    let (ledger_allocations, updated_lots) =
                        load_account_ledger_allocations_and_lots(
                            tx,
                            existing_ledger_entry.ledger_entry_id,
                        )
                        .await?;
                    return Ok(RefundAccountSettlementResult {
                        idempotent_replay: true,
                        settlement,
                        updated_lots,
                        ledger_entry: existing_ledger_entry,
                        ledger_allocations,
                    });
                }

                ensure!(
                    matches!(
                        settlement.status,
                        RequestSettlementStatus::Captured
                            | RequestSettlementStatus::PartiallyReleased
                            | RequestSettlementStatus::Refunded
                    ),
                    "request settlement {} is not refundable",
                    input.request_settlement_id
                );

                let remaining_refundable =
                    (settlement.captured_credit_amount - settlement.refunded_amount).max(0.0);
                ensure!(
                    remaining_refundable >= input.refunded_amount,
                    "refund amount {} exceeds refundable captured balance {} for settlement {}",
                    input.refunded_amount,
                    remaining_refundable,
                    input.request_settlement_id
                );

                let hold_id = settlement.hold_id.ok_or_else(|| {
                    anyhow::anyhow!(
                        "request settlement {} does not reference an account hold",
                        input.request_settlement_id
                    )
                })?;
                let existing_allocations =
                    tx.list_account_hold_allocations_for_hold(hold_id).await?;
                let mut already_refunded_remaining = settlement.refunded_amount;
                let mut refund_remaining = input.refunded_amount;
                let mut updated_lots = Vec::new();
                let mut refund_ledger_allocations = Vec::new();

                for allocation in existing_allocations {
                    let mut refundable_from_allocation = allocation.captured_quantity.max(0.0);
                    if already_refunded_remaining > f64::EPSILON {
                        let previously_refunded =
                            refundable_from_allocation.min(already_refunded_remaining);
                        already_refunded_remaining -= previously_refunded;
                        refundable_from_allocation -= previously_refunded;
                    }

                    if refund_remaining <= f64::EPSILON {
                        break;
                    }

                    let refunded_now = refundable_from_allocation.min(refund_remaining);
                    if refunded_now <= f64::EPSILON {
                        continue;
                    }

                    let lot = tx
                        .find_account_benefit_lot(allocation.lot_id)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!("lot {} does not exist", allocation.lot_id)
                        })?;
                    let updated_lot = lot
                        .clone()
                        .with_remaining_quantity(lot.remaining_quantity + refunded_now)
                        .with_updated_at_ms(input.refunded_at_ms);
                    tx.upsert_account_benefit_lot(&updated_lot).await?;
                    updated_lots.push(updated_lot);

                    refund_ledger_allocations.push((
                        input.refund_ledger_allocation_start_id
                            + refund_ledger_allocations.len() as u64,
                        allocation.lot_id,
                        refunded_now,
                    ));
                    refund_remaining -= refunded_now;
                }

                ensure!(
                    refund_remaining <= f64::EPSILON,
                    "refund amount {} exceeds captured allocations for settlement {}",
                    input.refunded_amount,
                    input.request_settlement_id
                );

                let updated_settlement = settlement
                    .clone()
                    .with_status(RequestSettlementStatus::Refunded)
                    .with_refunded_amount(settlement.refunded_amount + input.refunded_amount)
                    .with_settled_at_ms(input.refunded_at_ms)
                    .with_updated_at_ms(input.refunded_at_ms);
                tx.upsert_request_settlement_record(&updated_settlement)
                    .await?;

                let refund_ledger_entry = AccountLedgerEntryRecord::new(
                    input.refund_ledger_entry_id,
                    settlement.tenant_id,
                    settlement.organization_id,
                    settlement.account_id,
                    settlement.user_id,
                    AccountLedgerEntryType::Refund,
                )
                .with_request_id(Some(settlement.request_id))
                .with_hold_id(settlement.hold_id)
                .with_quantity(input.refunded_amount)
                .with_amount(input.refunded_amount)
                .with_created_at_ms(input.refunded_at_ms);
                let ledger_allocations = write_account_ledger_entry(
                    tx,
                    &refund_ledger_entry,
                    &refund_ledger_allocations,
                    input.refunded_at_ms,
                )
                .await?;

                Ok(RefundAccountSettlementResult {
                    idempotent_replay: false,
                    settlement: updated_settlement,
                    updated_lots,
                    ledger_entry: refund_ledger_entry,
                    ledger_allocations,
                })
            })
        })
        .await
}

pub async fn issue_commerce_order_credits<S>(
    store: &S,
    input: IssueCommerceOrderCreditsInput<'_>,
) -> Result<IssueCommerceOrderCreditsResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    let account_id = input.account_id;
    let order_id = input.order_id.trim().to_owned();
    let project_id = input.project_id.trim().to_owned();
    let target_kind = input.target_kind.trim().to_owned();
    let granted_quantity = input.granted_quantity;
    let payable_amount = input.payable_amount;
    let issued_at_ms = input.issued_at_ms;
    ensure!(!order_id.is_empty(), "order_id is required");
    ensure!(!project_id.is_empty(), "project_id is required");
    ensure!(!target_kind.is_empty(), "target_kind is required");
    ensure!(granted_quantity > 0.0, "granted_quantity must be positive");
    ensure!(payable_amount >= 0.0, "payable_amount must not be negative");

    let order_source_id = commerce_order_source_id(&order_id);
    let lot_id = commerce_order_lot_id(&order_id);
    let ledger_entry_id = commerce_order_issue_ledger_entry_id(&order_id);
    let ledger_allocation_id = commerce_order_issue_ledger_allocation_id(&order_id);

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let account = tx
                    .find_account_record(account_id)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("account {} does not exist", account_id))?;

                if let Some(existing_ledger_entry) =
                    tx.find_account_ledger_entry_record(ledger_entry_id).await?
                {
                    ensure!(
                        existing_ledger_entry.entry_type == AccountLedgerEntryType::GrantIssue,
                        "ledger entry {} is not a grant issue entry",
                        ledger_entry_id
                    );
                    ensure!(
                        existing_ledger_entry.account_id == account_id,
                        "ledger entry {} belongs to another account",
                        ledger_entry_id
                    );
                    ensure_quantity_matches(
                        existing_ledger_entry.quantity,
                        granted_quantity,
                        format!("commerce order {} granted_quantity", order_id),
                    )?;
                    ensure_quantity_matches(
                        existing_ledger_entry.amount,
                        payable_amount,
                        format!("commerce order {} payable_amount", order_id),
                    )?;

                    let lot = tx.find_account_benefit_lot(lot_id).await?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "commerce order {} lot {lot_id} is missing for replay",
                            order_id
                        )
                    })?;
                    validate_commerce_order_credit_lot(
                        &lot,
                        account_id,
                        order_source_id,
                        &order_id,
                    )?;

                    let (ledger_allocations, _) =
                        load_account_ledger_allocations_and_lots(tx, ledger_entry_id).await?;
                    return Ok(IssueCommerceOrderCreditsResult {
                        idempotent_replay: true,
                        lot,
                        ledger_entry: existing_ledger_entry,
                        ledger_allocations,
                    });
                }

                if tx.find_account_benefit_lot(lot_id).await?.is_some() {
                    anyhow::bail!(
                        "commerce order {} lot {lot_id} exists without issue ledger entry",
                        order_id
                    );
                }

                let lot = AccountBenefitLotRecord::new(
                    lot_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountBenefitType::CashCredit,
                )
                .with_source_type(AccountBenefitSourceType::Order)
                .with_source_id(Some(order_source_id))
                .with_scope_json(Some(build_commerce_order_credit_scope_json(
                    &order_id,
                    &project_id,
                    &target_kind,
                )?))
                .with_original_quantity(granted_quantity)
                .with_remaining_quantity(granted_quantity)
                .with_held_quantity(0.0)
                .with_acquired_unit_cost(Some(payable_amount / granted_quantity))
                .with_issued_at_ms(issued_at_ms)
                .with_status(AccountBenefitLotStatus::Active)
                .with_created_at_ms(issued_at_ms)
                .with_updated_at_ms(issued_at_ms);
                tx.upsert_account_benefit_lot(&lot).await?;

                let ledger_entry = AccountLedgerEntryRecord::new(
                    ledger_entry_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountLedgerEntryType::GrantIssue,
                )
                .with_benefit_type(Some("cash_credit".to_owned()))
                .with_quantity(granted_quantity)
                .with_amount(payable_amount)
                .with_created_at_ms(issued_at_ms);
                let ledger_allocations = write_account_ledger_entry(
                    tx,
                    &ledger_entry,
                    &[(ledger_allocation_id, lot.lot_id, granted_quantity)],
                    issued_at_ms,
                )
                .await?;

                Ok(IssueCommerceOrderCreditsResult {
                    idempotent_replay: false,
                    lot,
                    ledger_entry,
                    ledger_allocations,
                })
            })
        })
        .await
}

pub async fn refund_commerce_order_credits<S>(
    store: &S,
    input: RefundCommerceOrderCreditsInput<'_>,
) -> Result<RefundCommerceOrderCreditsResult>
where
    S: AccountKernelStore + AccountKernelTransactionExecutor + ?Sized,
{
    let account_id = input.account_id;
    let order_id = input.order_id.trim().to_owned();
    let refunded_quantity = input.refunded_quantity;
    let refunded_amount = input.refunded_amount;
    let refunded_at_ms = input.refunded_at_ms;
    ensure!(!order_id.is_empty(), "order_id is required");
    ensure!(
        refunded_quantity > 0.0,
        "refunded_quantity must be positive"
    );
    ensure!(
        refunded_amount >= 0.0,
        "refunded_amount must not be negative"
    );

    let order_source_id = commerce_order_source_id(&order_id);
    let lot_id = commerce_order_lot_id(&order_id);
    let issue_ledger_entry_id = commerce_order_issue_ledger_entry_id(&order_id);
    let refund_ledger_entry_id = commerce_order_refund_ledger_entry_id(&order_id);
    let refund_ledger_allocation_id = commerce_order_refund_ledger_allocation_id(&order_id);

    store
        .with_account_kernel_transaction(|tx| {
            Box::pin(async move {
                let account = tx
                    .find_account_record(account_id)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("account {} does not exist", account_id))?;

                if let Some(existing_ledger_entry) = tx
                    .find_account_ledger_entry_record(refund_ledger_entry_id)
                    .await?
                {
                    ensure!(
                        existing_ledger_entry.entry_type == AccountLedgerEntryType::Refund,
                        "ledger entry {} is not a refund entry",
                        refund_ledger_entry_id
                    );
                    ensure!(
                        existing_ledger_entry.account_id == account_id,
                        "ledger entry {} belongs to another account",
                        refund_ledger_entry_id
                    );
                    ensure_quantity_matches(
                        existing_ledger_entry.quantity,
                        refunded_quantity,
                        format!("commerce order {} refunded_quantity", order_id),
                    )?;
                    ensure_quantity_matches(
                        existing_ledger_entry.amount,
                        refunded_amount,
                        format!("commerce order {} refunded_amount", order_id),
                    )?;

                    let lot = tx.find_account_benefit_lot(lot_id).await?.ok_or_else(|| {
                        anyhow::anyhow!(
                            "commerce order {} lot {lot_id} is missing for refund replay",
                            order_id
                        )
                    })?;
                    validate_commerce_order_credit_lot(
                        &lot,
                        account_id,
                        order_source_id,
                        &order_id,
                    )?;
                    ensure!(
                        lot.status == AccountBenefitLotStatus::Disabled,
                        "commerce order {} refund replay found lot {} in status {:?}",
                        order_id,
                        lot.lot_id,
                        lot.status
                    );

                    let (ledger_allocations, _) =
                        load_account_ledger_allocations_and_lots(tx, refund_ledger_entry_id)
                            .await?;
                    return Ok(RefundCommerceOrderCreditsResult {
                        idempotent_replay: true,
                        lot,
                        ledger_entry: existing_ledger_entry,
                        ledger_allocations,
                    });
                }

                let issue_ledger_entry = tx
                    .find_account_ledger_entry_record(issue_ledger_entry_id)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "commerce order {} credits have not been issued yet",
                            order_id
                        )
                    })?;
                ensure!(
                    issue_ledger_entry.entry_type == AccountLedgerEntryType::GrantIssue,
                    "commerce order {} issue ledger entry is not a grant issue entry",
                    order_id
                );
                ensure!(
                    issue_ledger_entry.account_id == account_id,
                    "commerce order {} issue ledger entry belongs to another account",
                    order_id
                );

                let lot = tx.find_account_benefit_lot(lot_id).await?.ok_or_else(|| {
                    anyhow::anyhow!("commerce order {} issued lot does not exist", order_id)
                })?;
                validate_commerce_order_credit_lot(&lot, account_id, order_source_id, &order_id)?;
                ensure!(
                    lot.held_quantity <= ACCOUNTING_EPSILON,
                    "commerce order {} credits cannot be refunded while units are held",
                    order_id
                );
                ensure_quantity_matches(
                    lot.original_quantity,
                    refunded_quantity,
                    format!("commerce order {} original_quantity", order_id),
                )?;
                ensure_quantity_matches(
                    lot.remaining_quantity,
                    lot.original_quantity,
                    format!("commerce order {} remaining_quantity", order_id),
                )?;

                let updated_lot = lot
                    .clone()
                    .with_original_quantity(0.0)
                    .with_remaining_quantity(0.0)
                    .with_held_quantity(0.0)
                    .with_status(AccountBenefitLotStatus::Disabled)
                    .with_updated_at_ms(refunded_at_ms);
                tx.upsert_account_benefit_lot(&updated_lot).await?;

                let refund_ledger_entry = AccountLedgerEntryRecord::new(
                    refund_ledger_entry_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountLedgerEntryType::Refund,
                )
                .with_benefit_type(Some("cash_credit".to_owned()))
                .with_quantity(refunded_quantity)
                .with_amount(refunded_amount)
                .with_created_at_ms(refunded_at_ms);
                let ledger_allocations = write_account_ledger_entry(
                    tx,
                    &refund_ledger_entry,
                    &[(
                        refund_ledger_allocation_id,
                        updated_lot.lot_id,
                        -refunded_quantity,
                    )],
                    refunded_at_ms,
                )
                .await?;

                Ok(RefundCommerceOrderCreditsResult {
                    idempotent_replay: false,
                    lot: updated_lot,
                    ledger_entry: refund_ledger_entry,
                    ledger_allocations,
                })
            })
        })
        .await
}

pub async fn persist_ledger_entry(
    store: &dyn AdminStore,
    project_id: &str,
    units: u64,
    amount: f64,
) -> Result<LedgerEntry> {
    let entry = book_usage_cost(project_id, units, amount)?;
    store.insert_ledger_entry(&entry).await
}

pub async fn persist_billing_event(
    store: &dyn AdminStore,
    event: &BillingEventRecord,
) -> Result<BillingEventRecord> {
    store.insert_billing_event(event).await
}

pub async fn list_ledger_entries(store: &dyn AdminStore) -> Result<Vec<LedgerEntry>> {
    store.list_ledger_entries().await
}

pub async fn list_billing_events(store: &dyn AdminStore) -> Result<Vec<BillingEventRecord>> {
    store.list_billing_events().await
}

pub async fn persist_quota_policy(
    store: &dyn AdminStore,
    policy: &QuotaPolicy,
) -> Result<QuotaPolicy> {
    store.insert_quota_policy(policy).await
}

pub async fn list_quota_policies(store: &dyn AdminStore) -> Result<Vec<QuotaPolicy>> {
    store.list_quota_policies().await
}

#[async_trait]
pub trait BillingQuotaStore: Send + Sync {
    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>>;
    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>>;
}

#[async_trait]
impl<T> BillingQuotaStore for T
where
    T: AdminStore + ?Sized,
{
    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>> {
        AdminStore::list_ledger_entries_for_project(self, project_id).await
    }

    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>> {
        AdminStore::list_quota_policies_for_project(self, project_id).await
    }
}

#[async_trait]
pub trait GatewayCommercialBillingKernel: Send + Sync {
    async fn resolve_payable_account_for_gateway_request_context(
        &self,
        context: &IdentityGatewayRequestContext,
    ) -> Result<Option<AccountRecord>>;

    async fn plan_account_hold(
        &self,
        account_id: u64,
        requested_quantity: f64,
        now_ms: u64,
    ) -> Result<AccountHoldPlan>;

    async fn create_account_hold(
        &self,
        input: CreateAccountHoldInput,
    ) -> Result<AccountHoldMutationResult>;

    async fn release_account_hold(
        &self,
        input: ReleaseAccountHoldInput,
    ) -> Result<AccountHoldMutationResult>;

    async fn capture_account_hold(
        &self,
        input: CaptureAccountHoldInput,
    ) -> Result<CaptureAccountHoldResult>;
}

#[async_trait]
impl<T> GatewayCommercialBillingKernel for T
where
    T: AccountKernelStore + AccountKernelTransactionExecutor + Send + Sync + ?Sized,
{
    async fn resolve_payable_account_for_gateway_request_context(
        &self,
        context: &IdentityGatewayRequestContext,
    ) -> Result<Option<AccountRecord>> {
        resolve_payable_account_for_gateway_request_context(self, context).await
    }

    async fn plan_account_hold(
        &self,
        account_id: u64,
        requested_quantity: f64,
        now_ms: u64,
    ) -> Result<AccountHoldPlan> {
        plan_account_hold(self, account_id, requested_quantity, now_ms).await
    }

    async fn create_account_hold(
        &self,
        input: CreateAccountHoldInput,
    ) -> Result<AccountHoldMutationResult> {
        create_account_hold(self, input).await
    }

    async fn release_account_hold(
        &self,
        input: ReleaseAccountHoldInput,
    ) -> Result<AccountHoldMutationResult> {
        release_account_hold(self, input).await
    }

    async fn capture_account_hold(
        &self,
        input: CaptureAccountHoldInput,
    ) -> Result<CaptureAccountHoldResult> {
        capture_account_hold(self, input).await
    }
}

#[async_trait]
pub trait CommercialBillingReadKernel: Send + Sync {
    async fn resolve_payable_account_for_gateway_request_context(
        &self,
        context: &IdentityGatewayRequestContext,
    ) -> Result<Option<AccountRecord>>;

    async fn list_account_records(&self) -> Result<Vec<AccountRecord>>;

    async fn find_account_record(&self, account_id: u64) -> Result<Option<AccountRecord>>;

    async fn summarize_account_balance(
        &self,
        account_id: u64,
        now_ms: u64,
    ) -> Result<AccountBalanceSnapshot>;

    async fn list_account_ledger_history(
        &self,
        account_id: u64,
    ) -> Result<Vec<AccountLedgerHistoryEntry>>;

    async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>>;

    async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>>;

    async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>>;

    async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>>;

    async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>>;
}

#[async_trait]
pub trait CommercialBillingAdminKernel: CommercialBillingReadKernel {
    async fn insert_pricing_plan_record(
        &self,
        record: &PricingPlanRecord,
    ) -> Result<PricingPlanRecord>;

    async fn insert_pricing_rate_record(
        &self,
        record: &PricingRateRecord,
    ) -> Result<PricingRateRecord>;

    async fn issue_commerce_order_credits(
        &self,
        input: IssueCommerceOrderCreditsInput<'_>,
    ) -> Result<IssueCommerceOrderCreditsResult>;

    async fn refund_commerce_order_credits(
        &self,
        input: RefundCommerceOrderCreditsInput<'_>,
    ) -> Result<RefundCommerceOrderCreditsResult>;

    async fn insert_account_commerce_reconciliation_state(
        &self,
        record: &AccountCommerceReconciliationStateRecord,
    ) -> Result<AccountCommerceReconciliationStateRecord>;

    async fn find_account_commerce_reconciliation_state(
        &self,
        account_id: u64,
        project_id: &str,
    ) -> Result<Option<AccountCommerceReconciliationStateRecord>>;
}

#[async_trait]
impl<T> CommercialBillingReadKernel for T
where
    T: AccountKernelStore + Send + Sync + ?Sized,
{
    async fn resolve_payable_account_for_gateway_request_context(
        &self,
        context: &IdentityGatewayRequestContext,
    ) -> Result<Option<AccountRecord>> {
        resolve_payable_account_for_gateway_request_context(self, context).await
    }

    async fn list_account_records(&self) -> Result<Vec<AccountRecord>> {
        AccountKernelStore::list_account_records(self).await
    }

    async fn find_account_record(&self, account_id: u64) -> Result<Option<AccountRecord>> {
        AccountKernelStore::find_account_record(self, account_id).await
    }

    async fn summarize_account_balance(
        &self,
        account_id: u64,
        now_ms: u64,
    ) -> Result<AccountBalanceSnapshot> {
        summarize_account_balance(self, account_id, now_ms).await
    }

    async fn list_account_ledger_history(
        &self,
        account_id: u64,
    ) -> Result<Vec<AccountLedgerHistoryEntry>> {
        list_account_ledger_history(self, account_id).await
    }

    async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>> {
        AccountKernelStore::list_account_benefit_lots(self).await
    }

    async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>> {
        AccountKernelStore::list_account_holds(self).await
    }

    async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>> {
        AccountKernelStore::list_request_settlement_records(self).await
    }

    async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>> {
        AccountKernelStore::list_pricing_plan_records(self).await
    }

    async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>> {
        AccountKernelStore::list_pricing_rate_records(self).await
    }
}

#[async_trait]
impl<T> CommercialBillingAdminKernel for T
where
    T: AccountKernelStore + AccountKernelTransactionExecutor + Send + Sync + ?Sized,
{
    async fn insert_pricing_plan_record(
        &self,
        record: &PricingPlanRecord,
    ) -> Result<PricingPlanRecord> {
        AccountKernelStore::insert_pricing_plan_record(self, record).await
    }

    async fn insert_pricing_rate_record(
        &self,
        record: &PricingRateRecord,
    ) -> Result<PricingRateRecord> {
        AccountKernelStore::insert_pricing_rate_record(self, record).await
    }

    async fn issue_commerce_order_credits(
        &self,
        input: IssueCommerceOrderCreditsInput<'_>,
    ) -> Result<IssueCommerceOrderCreditsResult> {
        issue_commerce_order_credits(self, input).await
    }

    async fn refund_commerce_order_credits(
        &self,
        input: RefundCommerceOrderCreditsInput<'_>,
    ) -> Result<RefundCommerceOrderCreditsResult> {
        refund_commerce_order_credits(self, input).await
    }

    async fn insert_account_commerce_reconciliation_state(
        &self,
        record: &AccountCommerceReconciliationStateRecord,
    ) -> Result<AccountCommerceReconciliationStateRecord> {
        AccountKernelStore::insert_account_commerce_reconciliation_state(self, record).await
    }

    async fn find_account_commerce_reconciliation_state(
        &self,
        account_id: u64,
        project_id: &str,
    ) -> Result<Option<AccountCommerceReconciliationStateRecord>> {
        AccountKernelStore::find_account_commerce_reconciliation_state(self, account_id, project_id)
            .await
    }
}

fn pricing_status_is(value: &str, expected: &str) -> bool {
    value.trim().eq_ignore_ascii_case(expected)
}

fn is_pricing_plan_active(plan: &PricingPlanRecord) -> bool {
    pricing_status_is(&plan.status, "active")
}

fn is_due_planned_pricing_plan(plan: &PricingPlanRecord, now_ms: u64) -> bool {
    pricing_status_is(&plan.status, "planned")
        && plan.effective_from_ms <= now_ms
        && plan
            .effective_to_ms
            .map_or(true, |effective_to_ms| effective_to_ms >= now_ms)
}

fn compare_due_planned_pricing_candidates(
    left: &PricingPlanRecord,
    right: &PricingPlanRecord,
) -> Ordering {
    left.effective_from_ms
        .cmp(&right.effective_from_ms)
        .then(left.plan_version.cmp(&right.plan_version))
        .then(left.updated_at_ms.cmp(&right.updated_at_ms))
        .then(left.created_at_ms.cmp(&right.created_at_ms))
        .then(left.pricing_plan_id.cmp(&right.pricing_plan_id))
}

fn build_pricing_plan_with_status(
    plan: &PricingPlanRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingPlanRecord {
    PricingPlanRecord::new(
        plan.pricing_plan_id,
        plan.tenant_id,
        plan.organization_id,
        plan.plan_code.clone(),
        plan.plan_version,
    )
    .with_display_name(plan.display_name.clone())
    .with_currency_code(plan.currency_code.clone())
    .with_credit_unit_code(plan.credit_unit_code.clone())
    .with_status(status.to_owned())
    .with_effective_from_ms(plan.effective_from_ms)
    .with_effective_to_ms(plan.effective_to_ms)
    .with_created_at_ms(plan.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

fn build_pricing_rate_with_status(
    rate: &PricingRateRecord,
    status: &str,
    updated_at_ms: u64,
) -> PricingRateRecord {
    PricingRateRecord::new(
        rate.pricing_rate_id,
        rate.tenant_id,
        rate.organization_id,
        rate.pricing_plan_id,
        rate.metric_code.clone(),
    )
    .with_capability_code(rate.capability_code.clone())
    .with_model_code(rate.model_code.clone())
    .with_provider_code(rate.provider_code.clone())
    .with_charge_unit(rate.charge_unit.clone())
    .with_pricing_method(rate.pricing_method.clone())
    .with_quantity_step(rate.quantity_step)
    .with_unit_price(rate.unit_price)
    .with_display_price_unit(rate.display_price_unit.clone())
    .with_minimum_billable_quantity(rate.minimum_billable_quantity)
    .with_minimum_charge(rate.minimum_charge)
    .with_rounding_increment(rate.rounding_increment)
    .with_rounding_mode(rate.rounding_mode.clone())
    .with_included_quantity(rate.included_quantity)
    .with_priority(rate.priority)
    .with_notes(rate.notes.clone())
    .with_status(status.to_owned())
    .with_created_at_ms(rate.created_at_ms)
    .with_updated_at_ms(updated_at_ms)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PricingLifecycleSynchronizationReport {
    pub changed: bool,
    pub due_group_count: u64,
    pub activated_plan_count: u64,
    pub archived_plan_count: u64,
    pub activated_rate_count: u64,
    pub archived_rate_count: u64,
    pub skipped_plan_count: u64,
    pub synchronized_at_ms: u64,
}

pub async fn synchronize_due_pricing_plan_lifecycle_with_report<K>(
    kernel: &K,
    now_ms: u64,
) -> Result<PricingLifecycleSynchronizationReport>
where
    K: CommercialBillingAdminKernel + ?Sized,
{
    let plans = kernel.list_pricing_plan_records().await?;
    let rates = kernel.list_pricing_rate_records().await?;
    let mut due_group_keys = BTreeSet::new();

    for plan in &plans {
        if is_due_planned_pricing_plan(plan, now_ms) {
            due_group_keys.insert((plan.tenant_id, plan.organization_id, plan.plan_code.clone()));
        }
    }

    let mut report = PricingLifecycleSynchronizationReport {
        due_group_count: due_group_keys.len() as u64,
        synchronized_at_ms: now_ms,
        ..PricingLifecycleSynchronizationReport::default()
    };
    if due_group_keys.is_empty() {
        return Ok(report);
    }

    let mut changed = false;

    for (tenant_id, organization_id, plan_code) in due_group_keys {
        let mut due_candidates = plans
            .iter()
            .filter(|plan| {
                plan.tenant_id == tenant_id
                    && plan.organization_id == organization_id
                    && plan.plan_code == plan_code
                    && is_due_planned_pricing_plan(plan, now_ms)
            })
            .collect::<Vec<_>>();
        due_candidates.sort_by(|left, right| compare_due_planned_pricing_candidates(left, right));
        let Some(winner) = due_candidates.last().copied() else {
            continue;
        };

        if !rates
            .iter()
            .any(|rate| rate.pricing_plan_id == winner.pricing_plan_id)
        {
            report.skipped_plan_count += 1;
            continue;
        }

        if !is_pricing_plan_active(winner) {
            kernel
                .insert_pricing_plan_record(&build_pricing_plan_with_status(
                    winner, "active", now_ms,
                ))
                .await?;
            report.activated_plan_count += 1;
            changed = true;
        }

        let archived_plan_ids = plans
            .iter()
            .filter(|plan| {
                plan.pricing_plan_id != winner.pricing_plan_id
                    && plan.tenant_id == tenant_id
                    && plan.organization_id == organization_id
                    && plan.plan_code == plan_code
                    && (is_pricing_plan_active(plan) || is_due_planned_pricing_plan(plan, now_ms))
            })
            .map(|plan| plan.pricing_plan_id)
            .collect::<BTreeSet<_>>();

        for plan in plans
            .iter()
            .filter(|plan| archived_plan_ids.contains(&plan.pricing_plan_id))
        {
            if !pricing_status_is(&plan.status, "archived") {
                kernel
                    .insert_pricing_plan_record(&build_pricing_plan_with_status(
                        plan, "archived", now_ms,
                    ))
                    .await?;
                report.archived_plan_count += 1;
                changed = true;
            }
        }

        for rate in rates
            .iter()
            .filter(|rate| rate.pricing_plan_id == winner.pricing_plan_id)
        {
            if !pricing_status_is(&rate.status, "active") {
                kernel
                    .insert_pricing_rate_record(&build_pricing_rate_with_status(
                        rate, "active", now_ms,
                    ))
                    .await?;
                report.activated_rate_count += 1;
                changed = true;
            }
        }

        for rate in rates
            .iter()
            .filter(|rate| archived_plan_ids.contains(&rate.pricing_plan_id))
        {
            if !pricing_status_is(&rate.status, "archived") {
                kernel
                    .insert_pricing_rate_record(&build_pricing_rate_with_status(
                        rate, "archived", now_ms,
                    ))
                    .await?;
                report.archived_rate_count += 1;
                changed = true;
            }
        }
    }

    report.changed = changed;
    Ok(report)
}

pub async fn synchronize_due_pricing_plan_lifecycle<K>(kernel: &K, now_ms: u64) -> Result<bool>
where
    K: CommercialBillingAdminKernel + ?Sized,
{
    Ok(
        synchronize_due_pricing_plan_lifecycle_with_report(kernel, now_ms)
            .await?
            .changed,
    )
}

pub async fn check_quota<S>(
    store: &S,
    project_id: &str,
    requested_units: u64,
) -> Result<QuotaCheckResult>
where
    S: BillingQuotaStore + ?Sized,
{
    let used_units = store
        .list_ledger_entries_for_project(project_id)
        .await?
        .into_iter()
        .map(|entry| entry.units)
        .sum();
    let policies = store.list_quota_policies_for_project(project_id).await?;
    let registry = builtin_quota_policy_registry();
    let plugin = registry
        .resolve(STRICTEST_LIMIT_QUOTA_POLICY_ID)
        .expect("builtin strictest-limit quota policy plugin must exist");

    Ok(plugin.execute(QuotaPolicyExecutionInput {
        policies: &policies,
        used_units,
        requested_units,
    }))
}

pub fn summarize_billing_snapshot(
    entries: &[LedgerEntry],
    policies: &[QuotaPolicy],
) -> BillingSummary {
    if entries.is_empty() && policies.is_empty() {
        return BillingSummary::empty();
    }

    let mut projects = BTreeMap::<String, ProjectBillingSummary>::new();

    for entry in entries {
        let summary = projects
            .entry(entry.project_id.clone())
            .or_insert_with(|| ProjectBillingSummary::new(entry.project_id.clone()));
        summary.entry_count += 1;
        summary.used_units += entry.units;
        summary.booked_amount += entry.amount;
    }

    let active_policies = policies
        .iter()
        .filter(|policy| policy.enabled)
        .collect::<Vec<_>>();

    for policy in &active_policies {
        let summary = projects
            .entry(policy.project_id.clone())
            .or_insert_with(|| ProjectBillingSummary::new(policy.project_id.clone()));
        let replace_policy = match (
            summary.quota_limit_units,
            summary.quota_policy_id.as_deref(),
        ) {
            (None, _) => true,
            (Some(current_limit), Some(current_policy_id)) => {
                policy.max_units < current_limit
                    || (policy.max_units == current_limit
                        && policy.policy_id.as_str() < current_policy_id)
            }
            (Some(_), None) => true,
        };

        if replace_policy {
            summary.quota_policy_id = Some(policy.policy_id.clone());
            summary.quota_limit_units = Some(policy.max_units);
        }
    }

    let total_entries = entries.len() as u64;
    let total_units = entries.iter().map(|entry| entry.units).sum();
    let total_amount = entries.iter().map(|entry| entry.amount).sum();

    let mut project_summaries = projects
        .into_values()
        .map(|mut summary| {
            if let Some(limit_units) = summary.quota_limit_units {
                let remaining_units = limit_units.saturating_sub(summary.used_units);
                summary.remaining_units = Some(remaining_units);
                summary.exhausted = summary.used_units >= limit_units;
            }
            summary
        })
        .collect::<Vec<_>>();

    project_summaries.sort_by(|left, right| {
        right
            .quota_limit_units
            .is_some()
            .cmp(&left.quota_limit_units.is_some())
            .then_with(|| right.exhausted.cmp(&left.exhausted))
            .then_with(|| right.used_units.cmp(&left.used_units))
            .then_with(|| left.project_id.cmp(&right.project_id))
    });

    let exhausted_project_count = project_summaries
        .iter()
        .filter(|summary| summary.exhausted)
        .count() as u64;

    BillingSummary {
        total_entries,
        project_count: project_summaries.len() as u64,
        total_units,
        total_amount,
        active_quota_policy_count: active_policies.len() as u64,
        exhausted_project_count,
        projects: project_summaries,
    }
}

pub fn summarize_billing_events(events: &[BillingEventRecord]) -> BillingEventSummary {
    if events.is_empty() {
        return BillingEventSummary::empty();
    }

    #[derive(Default)]
    struct ProjectAccumulator {
        event_count: u64,
        request_count: u64,
        total_units: u64,
        total_input_tokens: u64,
        total_output_tokens: u64,
        total_tokens: u64,
        total_image_count: u64,
        total_audio_seconds: f64,
        total_video_seconds: f64,
        total_music_seconds: f64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct GroupAccumulator {
        project_ids: BTreeSet<String>,
        event_count: u64,
        request_count: u64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct CapabilityAccumulator {
        event_count: u64,
        request_count: u64,
        total_tokens: u64,
        image_count: u64,
        audio_seconds: f64,
        video_seconds: f64,
        music_seconds: f64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct AccountingModeAccumulator {
        event_count: u64,
        request_count: u64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    let mut projects = BTreeMap::<String, ProjectAccumulator>::new();
    let mut groups = BTreeMap::<Option<String>, GroupAccumulator>::new();
    let mut capabilities = BTreeMap::<String, CapabilityAccumulator>::new();
    let mut accounting_modes = BTreeMap::<BillingAccountingMode, AccountingModeAccumulator>::new();

    for event in events {
        let project = projects.entry(event.project_id.clone()).or_default();
        project.event_count += 1;
        project.request_count += event.request_count;
        project.total_units += event.units;
        project.total_input_tokens += event.input_tokens;
        project.total_output_tokens += event.output_tokens;
        project.total_tokens += event.total_tokens;
        project.total_image_count += event.image_count;
        project.total_audio_seconds += event.audio_seconds;
        project.total_video_seconds += event.video_seconds;
        project.total_music_seconds += event.music_seconds;
        project.total_upstream_cost += event.upstream_cost;
        project.total_customer_charge += event.customer_charge;

        let group = groups.entry(event.api_key_group_id.clone()).or_default();
        group.project_ids.insert(event.project_id.clone());
        group.event_count += 1;
        group.request_count += event.request_count;
        group.total_upstream_cost += event.upstream_cost;
        group.total_customer_charge += event.customer_charge;

        let capability = capabilities.entry(event.capability.clone()).or_default();
        capability.event_count += 1;
        capability.request_count += event.request_count;
        capability.total_tokens += event.total_tokens;
        capability.image_count += event.image_count;
        capability.audio_seconds += event.audio_seconds;
        capability.video_seconds += event.video_seconds;
        capability.music_seconds += event.music_seconds;
        capability.total_upstream_cost += event.upstream_cost;
        capability.total_customer_charge += event.customer_charge;

        let mode = accounting_modes.entry(event.accounting_mode).or_default();
        mode.event_count += 1;
        mode.request_count += event.request_count;
        mode.total_upstream_cost += event.upstream_cost;
        mode.total_customer_charge += event.customer_charge;
    }

    let mut project_summaries = projects
        .into_iter()
        .map(|(project_id, summary)| BillingEventProjectSummary {
            project_id,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_units: summary.total_units,
            total_input_tokens: summary.total_input_tokens,
            total_output_tokens: summary.total_output_tokens,
            total_tokens: summary.total_tokens,
            total_image_count: summary.total_image_count,
            total_audio_seconds: summary.total_audio_seconds,
            total_video_seconds: summary.total_video_seconds,
            total_music_seconds: summary.total_music_seconds,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    project_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.request_count.cmp(&left.request_count))
            .then_with(|| left.project_id.cmp(&right.project_id))
    });

    let mut group_summaries = groups
        .into_iter()
        .map(|(api_key_group_id, summary)| BillingEventGroupSummary {
            api_key_group_id,
            project_count: summary.project_ids.len() as u64,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    group_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.request_count.cmp(&left.request_count))
            .then_with(|| left.api_key_group_id.cmp(&right.api_key_group_id))
    });

    let mut capability_summaries = capabilities
        .into_iter()
        .map(|(capability, summary)| BillingEventCapabilitySummary {
            capability,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_tokens: summary.total_tokens,
            image_count: summary.image_count,
            audio_seconds: summary.audio_seconds,
            video_seconds: summary.video_seconds,
            music_seconds: summary.music_seconds,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    capability_summaries.sort_by(|left, right| {
        right
            .request_count
            .cmp(&left.request_count)
            .then_with(|| left.capability.cmp(&right.capability))
    });

    let mut accounting_mode_summaries = accounting_modes
        .into_iter()
        .map(
            |(accounting_mode, summary)| BillingEventAccountingModeSummary {
                accounting_mode,
                event_count: summary.event_count,
                request_count: summary.request_count,
                total_upstream_cost: summary.total_upstream_cost,
                total_customer_charge: summary.total_customer_charge,
            },
        )
        .collect::<Vec<_>>();
    accounting_mode_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.event_count.cmp(&left.event_count))
            .then_with(|| left.accounting_mode.cmp(&right.accounting_mode))
    });

    BillingEventSummary {
        total_events: events.len() as u64,
        project_count: project_summaries.len() as u64,
        group_count: group_summaries.len() as u64,
        capability_count: capability_summaries.len() as u64,
        total_request_count: events.iter().map(|event| event.request_count).sum(),
        total_units: events.iter().map(|event| event.units).sum(),
        total_input_tokens: events.iter().map(|event| event.input_tokens).sum(),
        total_output_tokens: events.iter().map(|event| event.output_tokens).sum(),
        total_tokens: events.iter().map(|event| event.total_tokens).sum(),
        total_image_count: events.iter().map(|event| event.image_count).sum(),
        total_audio_seconds: events.iter().map(|event| event.audio_seconds).sum(),
        total_video_seconds: events.iter().map(|event| event.video_seconds).sum(),
        total_music_seconds: events.iter().map(|event| event.music_seconds).sum(),
        total_upstream_cost: events.iter().map(|event| event.upstream_cost).sum(),
        total_customer_charge: events.iter().map(|event| event.customer_charge).sum(),
        projects: project_summaries,
        groups: group_summaries,
        capabilities: capability_summaries,
        accounting_modes: accounting_mode_summaries,
    }
}

pub async fn summarize_billing_from_store(store: &dyn AdminStore) -> Result<BillingSummary> {
    let entries = list_ledger_entries(store).await?;
    let policies = list_quota_policies(store).await?;
    Ok(summarize_billing_snapshot(&entries, &policies))
}

pub async fn summarize_billing_events_from_store(
    store: &dyn AdminStore,
) -> Result<BillingEventSummary> {
    let events = list_billing_events(store).await?;
    Ok(summarize_billing_events(&events))
}

fn eligible_lots_for_hold(
    lots: &[AccountBenefitLotRecord],
    now_ms: u64,
) -> Vec<&AccountBenefitLotRecord> {
    let mut eligible = lots
        .iter()
        .filter(|lot| {
            lot.status == AccountBenefitLotStatus::Active
                && lot
                    .expires_at_ms
                    .map(|expires_at_ms| expires_at_ms > now_ms)
                    .unwrap_or(true)
                && free_quantity(lot) > 0.0
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| {
        left.expires_at_ms
            .unwrap_or(u64::MAX)
            .cmp(&right.expires_at_ms.unwrap_or(u64::MAX))
            .then_with(|| right.scope_json.is_some().cmp(&left.scope_json.is_some()))
            .then_with(|| {
                benefit_cash_rank(left.benefit_type).cmp(&benefit_cash_rank(right.benefit_type))
            })
            .then_with(|| {
                left.acquired_unit_cost
                    .unwrap_or(f64::INFINITY)
                    .total_cmp(&right.acquired_unit_cost.unwrap_or(f64::INFINITY))
            })
            .then_with(|| left.lot_id.cmp(&right.lot_id))
    });
    eligible
}

fn free_quantity(lot: &AccountBenefitLotRecord) -> f64 {
    (lot.remaining_quantity - lot.held_quantity).max(0.0)
}

fn benefit_cash_rank(benefit_type: AccountBenefitType) -> u8 {
    match benefit_type {
        AccountBenefitType::CashCredit => 1,
        _ => 0,
    }
}

const ACCOUNT_LEDGER_ID_MULTIPLIER: u64 = 10;
const HOLD_CREATE_LEDGER_SUFFIX: u64 = 1;
const HOLD_RELEASE_LEDGER_SUFFIX: u64 = 2;
const SETTLEMENT_CAPTURE_LEDGER_SUFFIX: u64 = 3;
const SETTLEMENT_RELEASE_LEDGER_SUFFIX: u64 = 4;
const ACCOUNTING_EPSILON: f64 = 0.000_001;
const FNV1A_64_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A_64_PRIME: u64 = 0x100000001b3;

fn account_ledger_entry_id(base_id: u64, suffix: u64) -> u64 {
    base_id
        .saturating_mul(ACCOUNT_LEDGER_ID_MULTIPLIER)
        .saturating_add(suffix)
}

fn account_ledger_allocation_id(base_id: u64, suffix: u64) -> u64 {
    base_id
        .saturating_mul(ACCOUNT_LEDGER_ID_MULTIPLIER)
        .saturating_add(suffix)
}

fn stable_commerce_u64(namespace: &str, order_id: &str) -> u64 {
    let mut hash = FNV1A_64_OFFSET_BASIS;
    for byte in namespace
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0xff))
        .chain(order_id.as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV1A_64_PRIME);
    }

    let bounded = hash & (i64::MAX as u64);

    if bounded == 0 {
        1
    } else {
        bounded
    }
}

fn commerce_order_source_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_source", order_id)
}

fn commerce_order_lot_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_lot", order_id)
}

fn commerce_order_issue_ledger_entry_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_issue_ledger", order_id)
}

fn commerce_order_issue_ledger_allocation_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_issue_allocation", order_id)
}

fn commerce_order_refund_ledger_entry_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_refund_ledger", order_id)
}

fn commerce_order_refund_ledger_allocation_id(order_id: &str) -> u64 {
    stable_commerce_u64("commerce_refund_allocation", order_id)
}

fn ensure_quantity_matches(actual: f64, expected: f64, field_name: String) -> Result<()> {
    ensure!(
        (actual - expected).abs() <= ACCOUNTING_EPSILON,
        "{field_name} mismatch: expected {expected}, found {actual}"
    );
    Ok(())
}

fn build_commerce_order_credit_scope_json(
    order_id: &str,
    project_id: &str,
    target_kind: &str,
) -> Result<String> {
    serde_json::to_string(&serde_json::json!({
        "order_id": order_id,
        "project_id": project_id,
        "target_kind": target_kind,
    }))
    .map_err(Into::into)
}

fn validate_commerce_order_credit_lot(
    lot: &AccountBenefitLotRecord,
    account_id: u64,
    order_source_id: u64,
    order_id: &str,
) -> Result<()> {
    ensure!(
        lot.account_id == account_id,
        "commerce order {order_id} lot {} belongs to another account",
        lot.lot_id
    );
    ensure!(
        lot.source_type == AccountBenefitSourceType::Order,
        "commerce order {order_id} lot {} has unexpected source type",
        lot.lot_id
    );
    ensure!(
        lot.source_id == Some(order_source_id),
        "commerce order {order_id} lot {} has unexpected source id",
        lot.lot_id
    );
    ensure!(
        commerce_order_scope_matches(lot.scope_json.as_deref(), order_id),
        "commerce order {order_id} lot {} has unexpected scope metadata",
        lot.lot_id
    );
    Ok(())
}

fn commerce_order_scope_matches(scope_json: Option<&str>, order_id: &str) -> bool {
    scope_json
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .and_then(|value| {
            value
                .get("order_id")
                .and_then(|order_value| order_value.as_str().map(str::to_owned))
        })
        .is_some_and(|stored_order_id| stored_order_id == order_id)
}

async fn write_account_ledger_entry(
    tx: &mut dyn AccountKernelTransaction,
    entry: &AccountLedgerEntryRecord,
    allocation_deltas: &[(u64, u64, f64)],
    created_at_ms: u64,
) -> Result<Vec<AccountLedgerAllocationRecord>> {
    tx.upsert_account_ledger_entry_record(entry).await?;

    let mut allocations = Vec::with_capacity(allocation_deltas.len());
    for (ledger_allocation_id, lot_id, quantity_delta) in allocation_deltas {
        let allocation = AccountLedgerAllocationRecord::new(
            *ledger_allocation_id,
            entry.tenant_id,
            entry.organization_id,
            entry.ledger_entry_id,
            *lot_id,
        )
        .with_quantity_delta(*quantity_delta)
        .with_created_at_ms(created_at_ms);
        tx.upsert_account_ledger_allocation(&allocation).await?;
        allocations.push(allocation);
    }

    Ok(allocations)
}

async fn load_hold_allocations_and_lots(
    tx: &mut dyn AccountKernelTransaction,
    hold_id: u64,
) -> Result<(
    Vec<AccountHoldAllocationRecord>,
    Vec<AccountBenefitLotRecord>,
)> {
    let mut allocations = tx.list_account_hold_allocations_for_hold(hold_id).await?;
    allocations.sort_by_key(|allocation| allocation.hold_allocation_id);

    let mut lots = Vec::with_capacity(allocations.len());
    for allocation in &allocations {
        let lot = tx
            .find_account_benefit_lot(allocation.lot_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("lot {} does not exist", allocation.lot_id))?;
        lots.push(lot);
    }
    lots.sort_by_key(|lot| lot.lot_id);

    Ok((allocations, lots))
}

async fn load_account_ledger_allocations_and_lots(
    tx: &mut dyn AccountKernelTransaction,
    ledger_entry_id: u64,
) -> Result<(
    Vec<AccountLedgerAllocationRecord>,
    Vec<AccountBenefitLotRecord>,
)> {
    let mut allocations = tx
        .list_account_ledger_allocations_for_entry(ledger_entry_id)
        .await?;
    allocations.sort_by_key(|allocation| allocation.ledger_allocation_id);

    let mut lots = Vec::with_capacity(allocations.len());
    for allocation in &allocations {
        let lot = tx
            .find_account_benefit_lot(allocation.lot_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("lot {} does not exist", allocation.lot_id))?;
        lots.push(lot);
    }
    lots.sort_by_key(|lot| lot.lot_id);

    Ok((allocations, lots))
}
