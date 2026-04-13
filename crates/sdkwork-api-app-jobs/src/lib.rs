use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_app_marketing::{
    recover_expired_coupon_reservations as recover_expired_marketing_coupon_reservations,
    MarketingRecoveryRunReport,
};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobCallbackRecord, AsyncJobRecord,
};
use sdkwork_api_observability::HttpMetricsRegistry;
use sdkwork_api_storage_core::{
    AdminStore, JobStore, MarketingKernelTransactionExecutor, MarketingStore,
};

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

pub async fn recover_expired_coupon_reservations<S>(
    store: &S,
    metrics: Option<&HttpMetricsRegistry>,
    now_ms: u64,
) -> Result<MarketingRecoveryRunReport>
where
    S: MarketingStore + MarketingKernelTransactionExecutor + ?Sized,
{
    let result = recover_expired_marketing_coupon_reservations(store, now_ms).await;
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
