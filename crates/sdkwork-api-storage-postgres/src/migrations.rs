use super::*;

pub async fn run_migrations(url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    apply_postgres_identity_schema(&pool).await?;
    apply_postgres_marketing_schema(&pool).await?;
    apply_postgres_routing_schema(&pool).await?;
    apply_postgres_billing_schema(&pool).await?;
    apply_postgres_commerce_jobs_schema(&pool).await?;
    apply_postgres_catalog_gateway_schema(&pool).await?;
    apply_postgres_runtime_schema(&pool).await?;
    seed_postgres_builtin_channels(&pool).await?;
    Ok(pool)
}
