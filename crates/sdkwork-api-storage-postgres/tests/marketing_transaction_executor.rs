use sdkwork_api_storage_core::MarketingKernelTransactionExecutor;
use sdkwork_api_storage_postgres::PostgresAdminStore;

fn assert_marketing_transaction_executor<T: MarketingKernelTransactionExecutor>() {}

#[test]
fn postgres_store_exposes_marketing_kernel_transaction_executor() {
    assert_marketing_transaction_executor::<PostgresAdminStore>();
}
