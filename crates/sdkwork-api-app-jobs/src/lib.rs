use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobCallbackRecord, AsyncJobRecord,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponReservationRecord, CouponReservationStatus, MarketingOutboxEventRecord,
};
use sdkwork_api_observability::HttpMetricsRegistry;
use sdkwork_api_storage_core::{
    AdminStore, JobStore, MarketingKernelTransaction, MarketingKernelTransactionExecutor,
    MarketingStore,
};
use serde_json::json;

pub async fn list_async_jobs(store: &dyn AdminStore) -> Result<Vec<AsyncJobRecord>> {
    JobStore::list_async_jobs(store).await
}

pub async fn find_async_job(
    store: &dyn AdminStore,
    job_id: &str,
) -> Result<Option<AsyncJobRecord>> {
    JobStore::find_async_job(store, job_id).await
}

pub async fn list_async_job_attempts(
    store: &dyn AdminStore,
    job_id: &str,
) -> Result<Vec<AsyncJobAttemptRecord>> {
    JobStore::list_async_job_attempts(store, job_id).await
}

pub async fn list_async_job_assets(
    store: &dyn AdminStore,
    job_id: &str,
) -> Result<Vec<AsyncJobAssetRecord>> {
    JobStore::list_async_job_assets(store, job_id).await
}

pub async fn list_async_job_callbacks(
    store: &dyn AdminStore,
    job_id: &str,
) -> Result<Vec<AsyncJobCallbackRecord>> {
    JobStore::list_async_job_callbacks(store, job_id).await
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarketingRecoveryRunReport {
    pub scanned_reservations: u64,
    pub expired_reservations: u64,
    pub released_codes: u64,
    pub released_budget_minor: u64,
    pub outbox_events_created: u64,
}

pub async fn recover_expired_coupon_reservations<S>(
    store: &S,
    metrics: Option<&HttpMetricsRegistry>,
    now_ms: u64,
) -> Result<MarketingRecoveryRunReport>
where
    S: MarketingStore + MarketingKernelTransactionExecutor + ?Sized,
{
    let result = recover_expired_coupon_reservations_inner(store, now_ms).await;
    match (&result, metrics) {
        (Ok(report), Some(metrics)) => metrics.record_marketing_recovery_success(
            report.scanned_reservations,
            report.expired_reservations,
            report.released_codes,
            report.released_budget_minor,
            report.outbox_events_created,
            now_ms,
        ),
        (Err(_), Some(metrics)) => metrics.record_marketing_recovery_failure(now_ms),
        _ => {}
    }
    result
}

async fn recover_expired_coupon_reservations_inner<S>(
    store: &S,
    now_ms: u64,
) -> Result<MarketingRecoveryRunReport>
where
    S: MarketingStore + MarketingKernelTransactionExecutor + ?Sized,
{
    let reservations = MarketingStore::list_coupon_reservation_records(store).await?;
    let mut report = MarketingRecoveryRunReport {
        scanned_reservations: reservations.len() as u64,
        ..MarketingRecoveryRunReport::default()
    };

    for reservation in reservations {
        if !should_expire_reservation(&reservation, now_ms) {
            continue;
        }

        let reservation_id = reservation.coupon_reservation_id.clone();
        if let Some(outcome) = store
            .with_marketing_kernel_transaction(|tx| {
                Box::pin(async move {
                    let Some(persisted_reservation) =
                        tx.find_coupon_reservation_record(&reservation_id).await?
                    else {
                        anyhow::bail!("coupon reservation {} missing", reservation_id);
                    };
                    if !should_expire_reservation(&persisted_reservation, now_ms) {
                        return Ok(None);
                    }

                    let Some(code) = tx
                        .find_coupon_code_record(&persisted_reservation.coupon_code_id)
                        .await?
                    else {
                        anyhow::bail!(
                            "coupon code {} missing for reservation {}",
                            persisted_reservation.coupon_code_id,
                            persisted_reservation.coupon_reservation_id
                        );
                    };

                    let recovered_code = recovered_coupon_code(&code, now_ms);
                    let (updated_budgets, released_budget_minor) = plan_budget_release_updates(
                        tx,
                        &code,
                        persisted_reservation.budget_reserved_minor,
                        now_ms,
                    )
                    .await?;
                    let outbox_event = expired_reservation_outbox_event(
                        &persisted_reservation,
                        recovered_code.as_ref().unwrap_or(&code),
                        released_budget_minor,
                        now_ms,
                    );
                    let expired_reservation = persisted_reservation
                        .clone()
                        .with_status(CouponReservationStatus::Expired)
                        .with_updated_at_ms(now_ms);

                    tx.upsert_coupon_reservation_record(&expired_reservation).await?;
                    let mut released_codes = 0;
                    if let Some(code) = recovered_code.as_ref() {
                        tx.upsert_coupon_code_record(code).await?;
                        released_codes = 1;
                    }
                    for budget in &updated_budgets {
                        tx.upsert_campaign_budget_record(budget).await?;
                    }
                    tx.upsert_marketing_outbox_event_record(&outbox_event).await?;

                    Ok(Some(MarketingRecoveryReservationOutcome {
                        released_codes,
                        released_budget_minor,
                    }))
                })
            })
            .await?
        {
            report.expired_reservations += 1;
            report.released_codes += outcome.released_codes;
            report.released_budget_minor += outcome.released_budget_minor;
            report.outbox_events_created += 1;
        }
    }

    Ok(report)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct MarketingRecoveryReservationOutcome {
    released_codes: u64,
    released_budget_minor: u64,
}

fn should_expire_reservation(reservation: &CouponReservationRecord, now_ms: u64) -> bool {
    reservation.reservation_status == CouponReservationStatus::Reserved
        && reservation.expires_at_ms < now_ms
}

fn recovered_coupon_code(code: &CouponCodeRecord, now_ms: u64) -> Option<CouponCodeRecord> {
    if code.status != CouponCodeStatus::Reserved {
        return None;
    }

    let next_status = if code.expires_at_ms.is_some_and(|expires_at_ms| expires_at_ms < now_ms) {
        CouponCodeStatus::Expired
    } else {
        CouponCodeStatus::Available
    };

    Some(code.clone().with_status(next_status).with_updated_at_ms(now_ms))
}

async fn plan_budget_release_updates(
    tx: &mut dyn MarketingKernelTransaction,
    code: &CouponCodeRecord,
    requested_release_minor: u64,
    now_ms: u64,
) -> Result<(Vec<CampaignBudgetRecord>, u64)> {
    if requested_release_minor == 0 {
        return Ok((Vec::new(), 0));
    }

    let campaigns = tx
        .list_marketing_campaign_records_for_template(&code.coupon_template_id)
        .await?;
    let mut budgets = Vec::new();
    for campaign in campaigns {
        budgets.extend(
            tx.list_campaign_budget_records_for_campaign(&campaign.marketing_campaign_id)
                .await?,
        );
    }
    budgets.sort_by(|left, right| {
        right
            .reserved_budget_minor
            .cmp(&left.reserved_budget_minor)
            .then_with(|| left.campaign_budget_id.cmp(&right.campaign_budget_id))
    });

    let mut remaining_release_minor = requested_release_minor;
    let mut updated_budgets = Vec::new();
    let mut released_budget_minor = 0;
    for mut budget in budgets {
        if remaining_release_minor == 0 {
            break;
        }
        let releasable_minor = budget.reserved_budget_minor.min(remaining_release_minor);
        if releasable_minor == 0 {
            continue;
        }

        budget.reserved_budget_minor = budget
            .reserved_budget_minor
            .saturating_sub(releasable_minor);
        if budget.status == CampaignBudgetStatus::Exhausted && budget.available_budget_minor() > 0 {
            budget.status = CampaignBudgetStatus::Active;
        }
        budget.updated_at_ms = now_ms;
        updated_budgets.push(budget);
        remaining_release_minor = remaining_release_minor.saturating_sub(releasable_minor);
        released_budget_minor += releasable_minor;
    }

    Ok((updated_budgets, released_budget_minor))
}

fn expired_reservation_outbox_event(
    reservation: &CouponReservationRecord,
    code: &CouponCodeRecord,
    released_budget_minor: u64,
    now_ms: u64,
) -> MarketingOutboxEventRecord {
    MarketingOutboxEventRecord::new(
        format!(
            "recovery_coupon_reservation_expired_{}",
            reservation.coupon_reservation_id
        ),
        "coupon_reservation",
        reservation.coupon_reservation_id.clone(),
        "coupon.reservation.expired",
        json!({
            "coupon_reservation_id": reservation.coupon_reservation_id,
            "coupon_code_id": reservation.coupon_code_id,
            "coupon_code_status": coupon_code_status_label(code.status),
            "released_budget_minor": released_budget_minor,
            "expired_at_ms": now_ms,
        })
        .to_string(),
        now_ms,
    )
    .with_updated_at_ms(now_ms)
}

fn coupon_code_status_label(status: CouponCodeStatus) -> &'static str {
    match status {
        CouponCodeStatus::Available => "available",
        CouponCodeStatus::Reserved => "reserved",
        CouponCodeStatus::Redeemed => "redeemed",
        CouponCodeStatus::Expired => "expired",
        CouponCodeStatus::Disabled => "disabled",
    }
}

#[async_trait]
pub trait AsyncJobsReadKernel: Send + Sync {
    async fn list_async_jobs(&self) -> Result<Vec<AsyncJobRecord>>;
    async fn find_async_job(&self, job_id: &str) -> Result<Option<AsyncJobRecord>>;
    async fn list_async_job_attempts(&self, job_id: &str) -> Result<Vec<AsyncJobAttemptRecord>>;
    async fn list_async_job_assets(&self, job_id: &str) -> Result<Vec<AsyncJobAssetRecord>>;
    async fn list_async_job_callbacks(&self, job_id: &str) -> Result<Vec<AsyncJobCallbackRecord>>;
}

#[async_trait]
impl<T> AsyncJobsReadKernel for T
where
    T: JobStore + Send + Sync + ?Sized,
{
    async fn list_async_jobs(&self) -> Result<Vec<AsyncJobRecord>> {
        JobStore::list_async_jobs(self).await
    }

    async fn find_async_job(&self, job_id: &str) -> Result<Option<AsyncJobRecord>> {
        JobStore::find_async_job(self, job_id).await
    }

    async fn list_async_job_attempts(&self, job_id: &str) -> Result<Vec<AsyncJobAttemptRecord>> {
        JobStore::list_async_job_attempts(self, job_id).await
    }

    async fn list_async_job_assets(&self, job_id: &str) -> Result<Vec<AsyncJobAssetRecord>> {
        JobStore::list_async_job_assets(self, job_id).await
    }

    async fn list_async_job_callbacks(&self, job_id: &str) -> Result<Vec<AsyncJobCallbackRecord>> {
        JobStore::list_async_job_callbacks(self, job_id).await
    }
}

#[async_trait]
pub trait AsyncJobsAdminKernel: AsyncJobsReadKernel {
    async fn insert_async_job(&self, record: &AsyncJobRecord) -> Result<AsyncJobRecord>;
    async fn insert_async_job_attempt(
        &self,
        record: &AsyncJobAttemptRecord,
    ) -> Result<AsyncJobAttemptRecord>;
    async fn insert_async_job_asset(
        &self,
        record: &AsyncJobAssetRecord,
    ) -> Result<AsyncJobAssetRecord>;
    async fn insert_async_job_callback(
        &self,
        record: &AsyncJobCallbackRecord,
    ) -> Result<AsyncJobCallbackRecord>;
}

#[async_trait]
impl<T> AsyncJobsAdminKernel for T
where
    T: JobStore + Send + Sync + ?Sized,
{
    async fn insert_async_job(&self, record: &AsyncJobRecord) -> Result<AsyncJobRecord> {
        JobStore::insert_async_job(self, record).await
    }

    async fn insert_async_job_attempt(
        &self,
        record: &AsyncJobAttemptRecord,
    ) -> Result<AsyncJobAttemptRecord> {
        JobStore::insert_async_job_attempt(self, record).await
    }

    async fn insert_async_job_asset(
        &self,
        record: &AsyncJobAssetRecord,
    ) -> Result<AsyncJobAssetRecord> {
        JobStore::insert_async_job_asset(self, record).await
    }

    async fn insert_async_job_callback(
        &self,
        record: &AsyncJobCallbackRecord,
    ) -> Result<AsyncJobCallbackRecord> {
        JobStore::insert_async_job_callback(self, record).await
    }
}
