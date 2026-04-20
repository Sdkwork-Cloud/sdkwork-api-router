use super::*;

const AI_PRICING_PLAN_CREATE_SQL: &str = "CREATE TABLE IF NOT EXISTS ai_pricing_plan (
            pricing_plan_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            plan_code TEXT NOT NULL,
            plan_version INTEGER NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            currency_code TEXT NOT NULL DEFAULT 'USD',
            credit_unit_code TEXT NOT NULL DEFAULT 'credit',
            status TEXT NOT NULL DEFAULT 'draft',
            ownership_scope TEXT NOT NULL DEFAULT 'workspace',
            effective_from_ms INTEGER NOT NULL DEFAULT 0,
            effective_to_ms INTEGER,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )";

const AI_PRICING_RATE_CREATE_SQL: &str = "CREATE TABLE IF NOT EXISTS ai_pricing_rate (
            pricing_rate_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            pricing_plan_id INTEGER NOT NULL,
            metric_code TEXT NOT NULL,
            capability_code TEXT,
            model_code TEXT,
            provider_code TEXT,
            charge_unit TEXT NOT NULL DEFAULT 'unit',
            pricing_method TEXT NOT NULL DEFAULT 'per_unit',
            quantity_step REAL NOT NULL DEFAULT 1,
            unit_price REAL NOT NULL DEFAULT 0,
            display_price_unit TEXT NOT NULL DEFAULT '',
            minimum_billable_quantity REAL NOT NULL DEFAULT 0,
            minimum_charge REAL NOT NULL DEFAULT 0,
            rounding_increment REAL NOT NULL DEFAULT 1,
            rounding_mode TEXT NOT NULL DEFAULT 'none',
            included_quantity REAL NOT NULL DEFAULT 0,
            priority INTEGER NOT NULL DEFAULT 0,
            notes TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )";

const AI_ACCOUNT_LEDGER_ENTRY_CREATE_SQL: &str =
    "CREATE TABLE IF NOT EXISTS ai_account_ledger_entry (
            ledger_entry_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            account_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            request_id INTEGER,
            hold_id INTEGER,
            entry_type TEXT NOT NULL,
            benefit_type TEXT,
            quantity REAL NOT NULL DEFAULT 0,
            amount REAL NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )";

const AI_REQUEST_METER_FACT_CREATE_SQL: &str =
    "CREATE TABLE IF NOT EXISTS ai_request_meter_fact (
            request_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            user_id INTEGER NOT NULL,
            account_id INTEGER NOT NULL,
            api_key_id INTEGER,
            api_key_hash TEXT,
            auth_type TEXT NOT NULL,
            jwt_subject TEXT,
            platform TEXT,
            owner TEXT,
            request_trace_id TEXT,
            gateway_request_ref TEXT,
            upstream_request_ref TEXT,
            protocol_family TEXT NOT NULL DEFAULT '',
            capability_code TEXT NOT NULL,
            channel_code TEXT NOT NULL,
            model_code TEXT NOT NULL,
            provider_code TEXT NOT NULL,
            request_status TEXT NOT NULL DEFAULT 'pending',
            usage_capture_status TEXT NOT NULL DEFAULT 'pending',
            cost_pricing_plan_id INTEGER,
            retail_pricing_plan_id INTEGER,
            estimated_credit_hold REAL NOT NULL DEFAULT 0,
            actual_credit_charge REAL,
            actual_provider_cost REAL,
            started_at_ms INTEGER NOT NULL DEFAULT 0,
            finished_at_ms INTEGER,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )";

pub(crate) async fn apply_sqlite_billing_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account (
            account_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            user_id INTEGER NOT NULL,
            account_type TEXT NOT NULL,
            currency_code TEXT NOT NULL DEFAULT 'USD',
            credit_unit_code TEXT NOT NULL DEFAULT 'credit',
            status TEXT NOT NULL DEFAULT 'active',
            allow_overdraft INTEGER NOT NULL DEFAULT 0,
            overdraft_limit REAL NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_account_user_type
         ON ai_account (tenant_id, organization_id, user_id, account_type)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_benefit_lot (
            lot_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            account_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            benefit_type TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_id INTEGER,
            scope_json TEXT,
            original_quantity REAL NOT NULL DEFAULT 0,
            remaining_quantity REAL NOT NULL DEFAULT 0,
            held_quantity REAL NOT NULL DEFAULT 0,
            priority INTEGER NOT NULL DEFAULT 0,
            acquired_unit_cost REAL,
            issued_at_ms INTEGER NOT NULL DEFAULT 0,
            expires_at_ms INTEGER,
            status TEXT NOT NULL DEFAULT 'active',
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_benefit_lot_account_status_expiry
         ON ai_account_benefit_lot (tenant_id, organization_id, account_id, status, expires_at_ms)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_benefit_lot_account_lot
         ON ai_account_benefit_lot (account_id, lot_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_hold (
            hold_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            account_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            request_id INTEGER NOT NULL,
            hold_status TEXT NOT NULL DEFAULT 'held',
            estimated_quantity REAL NOT NULL DEFAULT 0,
            captured_quantity REAL NOT NULL DEFAULT 0,
            released_quantity REAL NOT NULL DEFAULT 0,
            expires_at_ms INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_account_hold_request
         ON ai_account_hold (tenant_id, organization_id, request_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_hold_allocation (
            hold_allocation_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            hold_id INTEGER NOT NULL,
            lot_id INTEGER NOT NULL,
            allocated_quantity REAL NOT NULL DEFAULT 0,
            captured_quantity REAL NOT NULL DEFAULT 0,
            released_quantity REAL NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_hold_allocation_hold_lot
         ON ai_account_hold_allocation (tenant_id, organization_id, hold_id, lot_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(AI_ACCOUNT_LEDGER_ENTRY_CREATE_SQL)
        .execute(pool)
        .await?;
    rebuild_legacy_ai_account_ledger_entry_table(pool).await?;
    ensure_sqlite_column(
        pool,
        "ai_account_ledger_entry",
        "quantity",
        "quantity REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_account_ledger_entry",
        "amount",
        "amount REAL NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_ledger_entry_account_created_at
         ON ai_account_ledger_entry (tenant_id, organization_id, account_id, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_ledger_allocation (
            ledger_allocation_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            ledger_entry_id INTEGER NOT NULL,
            lot_id INTEGER NOT NULL,
            quantity_delta REAL NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_ledger_allocation_ledger_lot
         ON ai_account_ledger_allocation (tenant_id, organization_id, ledger_entry_id, lot_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_account_commerce_reconciliation_state (
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            account_id INTEGER NOT NULL,
            project_id TEXT NOT NULL,
            last_order_updated_at_ms INTEGER NOT NULL DEFAULT 0,
            last_order_created_at_ms INTEGER NOT NULL DEFAULT 0,
            last_order_id TEXT NOT NULL,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (account_id, project_id)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_account_commerce_reconciliation_state_account_updated
         ON ai_account_commerce_reconciliation_state (
            tenant_id, organization_id, account_id, updated_at_ms DESC
         )",
    )
    .execute(pool)
    .await?;
    sqlx::query(AI_REQUEST_METER_FACT_CREATE_SQL)
        .execute(pool)
        .await?;
    rebuild_legacy_ai_request_meter_fact_table(pool).await?;
    ensure_sqlite_column(
        pool,
        "ai_request_meter_fact",
        "cost_pricing_plan_id",
        "cost_pricing_plan_id INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_request_meter_fact",
        "retail_pricing_plan_id",
        "retail_pricing_plan_id INTEGER",
    )
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_user_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, user_id, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_api_key_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, api_key_id, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_fact_provider_model_created_at
         ON ai_request_meter_fact (tenant_id, organization_id, provider_code, model_code, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_request_meter_metric (
            request_metric_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            request_id INTEGER NOT NULL,
            metric_code TEXT NOT NULL,
            quantity REAL NOT NULL DEFAULT 0,
            provider_field TEXT,
            source_kind TEXT NOT NULL DEFAULT 'provider',
            capture_stage TEXT NOT NULL DEFAULT 'final',
            is_billable INTEGER NOT NULL DEFAULT 1,
            captured_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_request_meter_metric_request_metric
         ON ai_request_meter_metric (tenant_id, organization_id, request_id, metric_code)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_request_settlement (
            request_settlement_id INTEGER PRIMARY KEY NOT NULL,
            tenant_id INTEGER NOT NULL,
            organization_id INTEGER NOT NULL DEFAULT 0,
            request_id INTEGER NOT NULL,
            account_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            hold_id INTEGER,
            settlement_status TEXT NOT NULL DEFAULT 'pending',
            estimated_credit_hold REAL NOT NULL DEFAULT 0,
            released_credit_amount REAL NOT NULL DEFAULT 0,
            captured_credit_amount REAL NOT NULL DEFAULT 0,
            provider_cost_amount REAL NOT NULL DEFAULT 0,
            retail_charge_amount REAL NOT NULL DEFAULT 0,
            shortfall_amount REAL NOT NULL DEFAULT 0,
            refunded_amount REAL NOT NULL DEFAULT 0,
            settled_at_ms INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_request_settlement_request
         ON ai_request_settlement (tenant_id, organization_id, request_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(AI_PRICING_PLAN_CREATE_SQL).execute(pool).await?;
    rebuild_legacy_ai_pricing_plan_table(pool).await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "pricing_plan_id",
        "pricing_plan_id INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "plan_version",
        "plan_version INTEGER NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "display_name",
        "display_name TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "currency_code",
        "currency_code TEXT NOT NULL DEFAULT 'USD'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "credit_unit_code",
        "credit_unit_code TEXT NOT NULL DEFAULT 'credit'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "status",
        "status TEXT NOT NULL DEFAULT 'draft'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "ownership_scope",
        "ownership_scope TEXT NOT NULL DEFAULT 'workspace'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "effective_from_ms",
        "effective_from_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "effective_to_ms",
        "effective_to_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_plan",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_pricing_plan_code_version
         ON ai_pricing_plan (tenant_id, organization_id, plan_code, plan_version)",
    )
    .execute(pool)
    .await?;
    sqlx::query(AI_PRICING_RATE_CREATE_SQL).execute(pool).await?;
    rebuild_legacy_ai_pricing_rate_table(pool).await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "pricing_rate_id",
        "pricing_rate_id INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "pricing_plan_id",
        "pricing_plan_id INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_pricing_rate", "model_code", "model_code TEXT").await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "provider_code",
        "provider_code TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "quantity_step",
        "quantity_step REAL NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "unit_price",
        "unit_price REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "capability_code",
        "capability_code TEXT",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "charge_unit",
        "charge_unit TEXT NOT NULL DEFAULT 'unit'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "pricing_method",
        "pricing_method TEXT NOT NULL DEFAULT 'per_unit'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "display_price_unit",
        "display_price_unit TEXT NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "minimum_billable_quantity",
        "minimum_billable_quantity REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "minimum_charge",
        "minimum_charge REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "rounding_increment",
        "rounding_increment REAL NOT NULL DEFAULT 1",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "rounding_mode",
        "rounding_mode TEXT NOT NULL DEFAULT 'none'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "included_quantity",
        "included_quantity REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "priority",
        "priority INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_pricing_rate", "notes", "notes TEXT").await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "status",
        "status TEXT NOT NULL DEFAULT 'draft'",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_pricing_rate",
        "updated_at_ms",
        "updated_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_pricing_rate_plan_metric
         ON ai_pricing_rate (tenant_id, organization_id, pricing_plan_id, metric_code)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_usage_records (
            project_id TEXT NOT NULL,
            model TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            units INTEGER NOT NULL DEFAULT 0,
            amount REAL NOT NULL DEFAULT 0,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            api_key_hash TEXT,
            channel_id TEXT,
            latency_ms INTEGER,
            reference_amount REAL,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_project_created_at
         ON ai_usage_records (project_id, created_at_ms DESC, provider_id, model)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_created_at
         ON ai_usage_records (created_at_ms DESC, project_id, provider_id, model)",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "units",
        "units INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "amount",
        "amount REAL NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "input_tokens",
        "input_tokens INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "output_tokens",
        "output_tokens INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "total_tokens",
        "total_tokens INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "api_key_hash",
        "api_key_hash TEXT",
    )
    .await?;
    ensure_sqlite_column(pool, "ai_usage_records", "channel_id", "channel_id TEXT").await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "latency_ms",
        "latency_ms INTEGER",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "reference_amount",
        "reference_amount REAL",
    )
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_usage_records",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_usage_records_project_fact_filters
         ON ai_usage_records (project_id, created_at_ms DESC, api_key_hash, channel_id, model)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_events (
            event_id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            api_key_group_id TEXT,
            capability TEXT NOT NULL,
            route_key TEXT NOT NULL,
            usage_model TEXT NOT NULL,
            provider_id TEXT NOT NULL,
            accounting_mode TEXT NOT NULL,
            operation_kind TEXT NOT NULL,
            modality TEXT NOT NULL,
            api_key_hash TEXT,
            channel_id TEXT,
            reference_id TEXT,
            latency_ms INTEGER,
            units INTEGER NOT NULL DEFAULT 0,
            request_count INTEGER NOT NULL DEFAULT 1,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read_tokens INTEGER NOT NULL DEFAULT 0,
            cache_write_tokens INTEGER NOT NULL DEFAULT 0,
            image_count INTEGER NOT NULL DEFAULT 0,
            audio_seconds REAL NOT NULL DEFAULT 0,
            video_seconds REAL NOT NULL DEFAULT 0,
            music_seconds REAL NOT NULL DEFAULT 0,
            upstream_cost REAL NOT NULL DEFAULT 0,
            customer_charge REAL NOT NULL DEFAULT 0,
            applied_routing_profile_id TEXT,
            compiled_routing_snapshot_id TEXT,
            fallback_reason TEXT,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_project_created_at
         ON ai_billing_events (project_id, created_at_ms DESC, capability, provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_group_created_at
         ON ai_billing_events (api_key_group_id, created_at_ms DESC, project_id, capability)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_events_capability_created_at
         ON ai_billing_events (capability, created_at_ms DESC, project_id, provider_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_ledger_entries (
            project_id TEXT NOT NULL,
            units INTEGER NOT NULL,
            amount REAL NOT NULL,
            created_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    ensure_sqlite_column(
        pool,
        "ai_billing_ledger_entries",
        "created_at_ms",
        "created_at_ms INTEGER NOT NULL DEFAULT 0",
    )
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_ledger_entries_project_created_at
         ON ai_billing_ledger_entries (project_id, created_at_ms DESC)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_ledger_entries_created_at
         ON ai_billing_ledger_entries (created_at_ms DESC, project_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_billing_quota_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL,
            max_units INTEGER NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_billing_quota_policies_project_enabled
         ON ai_billing_quota_policies (project_id, enabled, policy_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_gateway_rate_limit_policies (
            policy_id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL,
            api_key_hash TEXT,
            route_key TEXT,
            model_name TEXT,
            requests_per_window INTEGER NOT NULL,
            window_seconds INTEGER NOT NULL DEFAULT 60,
            burst_requests INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            notes TEXT,
            created_at_ms INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_gateway_rate_limit_policies_project_enabled
         ON ai_gateway_rate_limit_policies (project_id, enabled, policy_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ai_gateway_rate_limit_policies_project_scope
         ON ai_gateway_rate_limit_policies (project_id, api_key_hash, route_key, model_name, enabled, policy_id)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_gateway_rate_limit_windows (
            policy_id TEXT NOT NULL,
            window_start_ms INTEGER NOT NULL,
            request_count INTEGER NOT NULL DEFAULT 0,
            updated_at_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (policy_id, window_start_ms)
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn rebuild_legacy_ai_account_ledger_entry_table(pool: &SqlitePool) -> Result<()> {
    let columns = sqlite_table_columns(pool, "ai_account_ledger_entry").await?;
    let needs_rebuild = !sqlite_has_column(&columns, "quantity")
        || !sqlite_has_column(&columns, "amount")
        || sqlite_has_column(&columns, "quantity_delta")
        || sqlite_has_column(&columns, "balance_after")
        || sqlite_has_column(&columns, "source_type")
        || sqlite_has_column(&columns, "source_id")
        || sqlite_has_column(&columns, "notes");
    if !needs_rebuild {
        return Ok(());
    }

    let ledger_entry_id_expr =
        sqlite_first_available_expr(&columns, &["ledger_entry_id"], "0");
    let tenant_id_expr = sqlite_first_available_expr(&columns, &["tenant_id"], "0");
    let organization_id_expr = sqlite_first_available_expr(&columns, &["organization_id"], "0");
    let account_id_expr = sqlite_first_available_expr(&columns, &["account_id"], "0");
    let user_id_expr = sqlite_first_available_expr(&columns, &["user_id"], "0");
    let request_id_expr = sqlite_first_available_expr(&columns, &["request_id"], "NULL");
    let hold_id_expr = sqlite_first_available_expr(&columns, &["hold_id"], "NULL");
    let entry_type_expr = sqlite_first_text_expr(&columns, &["entry_type"], "''");
    let benefit_type_expr = sqlite_first_nullable_text_expr(&columns, &["benefit_type"]);
    let quantity_expr =
        sqlite_first_available_expr(&columns, &["quantity", "quantity_delta"], "0");
    let amount_expr = sqlite_first_available_expr(&columns, &["amount"], "0");
    let created_at_expr = sqlite_first_available_expr(&columns, &["created_at_ms"], "0");

    let insert_sql = format!(
        "INSERT INTO ai_account_ledger_entry__sdkwork_rebuild (
            ledger_entry_id, tenant_id, organization_id, account_id, user_id, request_id,
            hold_id, entry_type, benefit_type, quantity, amount, created_at_ms
        )
        SELECT
            {ledger_entry_id_expr},
            {tenant_id_expr},
            {organization_id_expr},
            {account_id_expr},
            {user_id_expr},
            {request_id_expr},
            {hold_id_expr},
            {entry_type_expr},
            {benefit_type_expr},
            {quantity_expr},
            {amount_expr},
            {created_at_expr}
        FROM ai_account_ledger_entry"
    );

    rebuild_sqlite_table_with_projection(
        pool,
        "ai_account_ledger_entry",
        AI_ACCOUNT_LEDGER_ENTRY_CREATE_SQL,
        &insert_sql,
    )
    .await
}

async fn rebuild_legacy_ai_request_meter_fact_table(pool: &SqlitePool) -> Result<()> {
    let columns = sqlite_table_columns(pool, "ai_request_meter_fact").await?;
    let needs_rebuild = !sqlite_has_column(&columns, "cost_pricing_plan_id")
        || !sqlite_has_column(&columns, "retail_pricing_plan_id")
        || sqlite_has_column(&columns, "cost_pricing_plan_version_id")
        || sqlite_has_column(&columns, "retail_pricing_plan_version_id");
    if !needs_rebuild {
        return Ok(());
    }

    let request_id_expr = sqlite_first_available_expr(&columns, &["request_id"], "0");
    let tenant_id_expr = sqlite_first_available_expr(&columns, &["tenant_id"], "0");
    let organization_id_expr = sqlite_first_available_expr(&columns, &["organization_id"], "0");
    let user_id_expr = sqlite_first_available_expr(&columns, &["user_id"], "0");
    let account_id_expr = sqlite_first_available_expr(&columns, &["account_id"], "0");
    let api_key_id_expr = sqlite_first_available_expr(&columns, &["api_key_id"], "NULL");
    let api_key_hash_expr = sqlite_first_nullable_text_expr(&columns, &["api_key_hash"]);
    let auth_type_expr = sqlite_first_text_expr(&columns, &["auth_type"], "''");
    let jwt_subject_expr = sqlite_first_nullable_text_expr(&columns, &["jwt_subject"]);
    let platform_expr = sqlite_first_nullable_text_expr(&columns, &["platform"]);
    let owner_expr = sqlite_first_nullable_text_expr(&columns, &["owner"]);
    let request_trace_id_expr =
        sqlite_first_nullable_text_expr(&columns, &["request_trace_id"]);
    let gateway_request_ref_expr =
        sqlite_first_nullable_text_expr(&columns, &["gateway_request_ref"]);
    let upstream_request_ref_expr =
        sqlite_first_nullable_text_expr(&columns, &["upstream_request_ref"]);
    let protocol_family_expr = sqlite_first_text_expr(&columns, &["protocol_family"], "''");
    let capability_code_expr = sqlite_first_text_expr(&columns, &["capability_code"], "''");
    let channel_code_expr = sqlite_first_text_expr(&columns, &["channel_code"], "''");
    let model_code_expr = sqlite_first_text_expr(&columns, &["model_code"], "''");
    let provider_code_expr = sqlite_first_text_expr(&columns, &["provider_code"], "''");
    let request_status_expr = sqlite_first_text_expr(&columns, &["request_status"], "'pending'");
    let usage_capture_status_expr =
        sqlite_first_text_expr(&columns, &["usage_capture_status"], "'pending'");
    let cost_pricing_plan_id_expr = sqlite_non_zero_or_fallback_expr(
        &columns,
        "cost_pricing_plan_id",
        "cost_pricing_plan_version_id",
        "NULL",
    );
    let retail_pricing_plan_id_expr = sqlite_non_zero_or_fallback_expr(
        &columns,
        "retail_pricing_plan_id",
        "retail_pricing_plan_version_id",
        "NULL",
    );
    let estimated_credit_hold_expr =
        sqlite_first_available_expr(&columns, &["estimated_credit_hold"], "0");
    let actual_credit_charge_expr =
        sqlite_first_available_expr(&columns, &["actual_credit_charge"], "NULL");
    let actual_provider_cost_expr =
        sqlite_first_available_expr(&columns, &["actual_provider_cost"], "NULL");
    let started_at_expr = sqlite_first_available_expr(&columns, &["started_at_ms"], "0");
    let finished_at_expr = sqlite_first_available_expr(&columns, &["finished_at_ms"], "NULL");
    let created_at_expr = sqlite_first_available_expr(&columns, &["created_at_ms"], "0");
    let updated_at_expr = sqlite_first_available_expr(&columns, &["updated_at_ms"], "0");

    let insert_sql = format!(
        "INSERT INTO ai_request_meter_fact__sdkwork_rebuild (
            request_id, tenant_id, organization_id, user_id, account_id, api_key_id,
            api_key_hash, auth_type, jwt_subject, platform, owner, request_trace_id,
            gateway_request_ref, upstream_request_ref, protocol_family, capability_code,
            channel_code, model_code, provider_code, request_status, usage_capture_status,
            cost_pricing_plan_id, retail_pricing_plan_id, estimated_credit_hold,
            actual_credit_charge, actual_provider_cost, started_at_ms, finished_at_ms,
            created_at_ms, updated_at_ms
        )
        SELECT
            {request_id_expr},
            {tenant_id_expr},
            {organization_id_expr},
            {user_id_expr},
            {account_id_expr},
            {api_key_id_expr},
            {api_key_hash_expr},
            {auth_type_expr},
            {jwt_subject_expr},
            {platform_expr},
            {owner_expr},
            {request_trace_id_expr},
            {gateway_request_ref_expr},
            {upstream_request_ref_expr},
            {protocol_family_expr},
            {capability_code_expr},
            {channel_code_expr},
            {model_code_expr},
            {provider_code_expr},
            {request_status_expr},
            {usage_capture_status_expr},
            {cost_pricing_plan_id_expr},
            {retail_pricing_plan_id_expr},
            {estimated_credit_hold_expr},
            {actual_credit_charge_expr},
            {actual_provider_cost_expr},
            {started_at_expr},
            {finished_at_expr},
            {created_at_expr},
            {updated_at_expr}
        FROM ai_request_meter_fact"
    );

    rebuild_sqlite_table_with_projection(
        pool,
        "ai_request_meter_fact",
        AI_REQUEST_METER_FACT_CREATE_SQL,
        &insert_sql,
    )
    .await
}

async fn rebuild_legacy_ai_pricing_plan_table(pool: &SqlitePool) -> Result<()> {
    let columns = sqlite_table_columns(pool, "ai_pricing_plan").await?;
    let needs_rebuild = !sqlite_has_column(&columns, "pricing_plan_id")
        || sqlite_has_column(&columns, "pricing_plan_version_id")
        || sqlite_has_column(&columns, "plan_type")
        || sqlite_has_column(&columns, "scope_kind")
        || sqlite_has_column(&columns, "scope_ref_id")
        || sqlite_has_column(&columns, "created_by");
    if !needs_rebuild {
        return Ok(());
    }

    let pricing_plan_id_expr = sqlite_non_zero_or_fallback_expr(
        &columns,
        "pricing_plan_id",
        "pricing_plan_version_id",
        "0",
    );
    let tenant_id_expr = sqlite_first_available_expr(&columns, &["tenant_id"], "0");
    let organization_id_expr = sqlite_first_available_expr(&columns, &["organization_id"], "0");
    let plan_code_expr = sqlite_first_text_expr(&columns, &["plan_code"], "''");
    let plan_version_expr = sqlite_first_available_expr(&columns, &["plan_version"], "1");
    let display_name_expr = sqlite_first_text_expr(&columns, &["display_name"], "''");
    let currency_code_expr = sqlite_first_text_expr(&columns, &["currency_code"], "'USD'");
    let credit_unit_code_expr =
        sqlite_first_text_expr(&columns, &["credit_unit_code"], "'credit'");
    let status_expr = sqlite_first_text_expr(&columns, &["status"], "'draft'");
    let ownership_scope_expr =
        sqlite_first_text_expr(&columns, &["ownership_scope", "scope_kind"], "'workspace'");
    let effective_from_expr = sqlite_first_available_expr(&columns, &["effective_from_ms"], "0");
    let effective_to_expr = sqlite_first_available_expr(&columns, &["effective_to_ms"], "NULL");
    let created_at_expr = sqlite_first_available_expr(&columns, &["created_at_ms"], "0");
    let updated_at_expr = sqlite_first_available_expr(&columns, &["updated_at_ms"], "0");

    let insert_sql = format!(
        "INSERT INTO ai_pricing_plan__sdkwork_rebuild (
            pricing_plan_id, tenant_id, organization_id, plan_code, plan_version,
            display_name, currency_code, credit_unit_code, status, ownership_scope,
            effective_from_ms, effective_to_ms, created_at_ms, updated_at_ms
        )
        SELECT
            {pricing_plan_id_expr},
            {tenant_id_expr},
            {organization_id_expr},
            {plan_code_expr},
            {plan_version_expr},
            {display_name_expr},
            {currency_code_expr},
            {credit_unit_code_expr},
            {status_expr},
            {ownership_scope_expr},
            {effective_from_expr},
            {effective_to_expr},
            {created_at_expr},
            {updated_at_expr}
        FROM ai_pricing_plan"
    );

    rebuild_sqlite_table_with_projection(
        pool,
        "ai_pricing_plan",
        AI_PRICING_PLAN_CREATE_SQL,
        &insert_sql,
    )
    .await
}

async fn rebuild_legacy_ai_pricing_rate_table(pool: &SqlitePool) -> Result<()> {
    let columns = sqlite_table_columns(pool, "ai_pricing_rate").await?;
    let needs_rebuild = !sqlite_has_column(&columns, "pricing_rate_id")
        || sqlite_has_column(&columns, "pricing_plan_version_id")
        || sqlite_has_column(&columns, "match_channel_code")
        || sqlite_has_column(&columns, "match_model_code")
        || sqlite_has_column(&columns, "match_provider_code")
        || sqlite_has_column(&columns, "match_capability_code")
        || sqlite_has_column(&columns, "unit_size")
        || sqlite_has_column(&columns, "price_value")
        || sqlite_has_column(&columns, "sort_order");
    if !needs_rebuild {
        return Ok(());
    }

    let pricing_rate_id_expr = sqlite_first_available_expr(&columns, &["pricing_rate_id"], "0");
    let tenant_id_expr = sqlite_first_available_expr(&columns, &["tenant_id"], "0");
    let organization_id_expr = sqlite_first_available_expr(&columns, &["organization_id"], "0");
    let pricing_plan_id_expr = sqlite_non_zero_or_fallback_expr(
        &columns,
        "pricing_plan_id",
        "pricing_plan_version_id",
        "0",
    );
    let metric_code_expr = sqlite_first_text_expr(&columns, &["metric_code"], "''");
    let capability_code_expr = sqlite_first_nullable_text_expr(
        &columns,
        &["capability_code", "match_capability_code"],
    );
    let model_code_expr =
        sqlite_first_nullable_text_expr(&columns, &["model_code", "match_model_code"]);
    let provider_code_expr =
        sqlite_first_nullable_text_expr(&columns, &["provider_code", "match_provider_code"]);
    let charge_unit_expr = sqlite_first_text_expr(&columns, &["charge_unit"], "'unit'");
    let pricing_method_expr =
        sqlite_first_text_expr(&columns, &["pricing_method"], "'per_unit'");
    let quantity_step_expr =
        sqlite_first_available_expr(&columns, &["quantity_step", "unit_size"], "1");
    let unit_price_expr =
        sqlite_first_available_expr(&columns, &["unit_price", "price_value"], "0");
    let display_price_unit_expr =
        sqlite_first_text_expr(&columns, &["display_price_unit"], "''");
    let minimum_billable_quantity_expr =
        sqlite_first_available_expr(&columns, &["minimum_billable_quantity"], "0");
    let minimum_charge_expr = sqlite_first_available_expr(&columns, &["minimum_charge"], "0");
    let rounding_increment_expr =
        sqlite_first_available_expr(&columns, &["rounding_increment"], "1");
    let rounding_mode_expr = sqlite_first_text_expr(&columns, &["rounding_mode"], "'none'");
    let included_quantity_expr =
        sqlite_first_available_expr(&columns, &["included_quantity"], "0");
    let priority_expr = sqlite_first_available_expr(&columns, &["priority", "sort_order"], "0");
    let notes_expr = sqlite_first_nullable_text_expr(&columns, &["notes"]);
    let status_expr = sqlite_first_text_expr(&columns, &["status"], "'draft'");
    let created_at_expr = sqlite_first_available_expr(&columns, &["created_at_ms"], "0");
    let updated_at_expr = sqlite_first_available_expr(&columns, &["updated_at_ms"], "0");

    let insert_sql = format!(
        "INSERT INTO ai_pricing_rate__sdkwork_rebuild (
            pricing_rate_id, tenant_id, organization_id, pricing_plan_id, metric_code,
            capability_code, model_code, provider_code, charge_unit, pricing_method,
            quantity_step, unit_price, display_price_unit, minimum_billable_quantity,
            minimum_charge, rounding_increment, rounding_mode, included_quantity, priority,
            notes, status, created_at_ms, updated_at_ms
        )
        SELECT
            {pricing_rate_id_expr},
            {tenant_id_expr},
            {organization_id_expr},
            {pricing_plan_id_expr},
            {metric_code_expr},
            {capability_code_expr},
            {model_code_expr},
            {provider_code_expr},
            {charge_unit_expr},
            {pricing_method_expr},
            {quantity_step_expr},
            {unit_price_expr},
            {display_price_unit_expr},
            {minimum_billable_quantity_expr},
            {minimum_charge_expr},
            {rounding_increment_expr},
            {rounding_mode_expr},
            {included_quantity_expr},
            {priority_expr},
            {notes_expr},
            {status_expr},
            {created_at_expr},
            {updated_at_expr}
        FROM ai_pricing_rate"
    );

    rebuild_sqlite_table_with_projection(
        pool,
        "ai_pricing_rate",
        AI_PRICING_RATE_CREATE_SQL,
        &insert_sql,
    )
    .await
}

async fn rebuild_sqlite_table_with_projection(
    pool: &SqlitePool,
    table_name: &str,
    create_sql: &str,
    insert_sql: &str,
) -> Result<()> {
    let rebuild_table_name = format!("{table_name}__sdkwork_rebuild");
    let drop_rebuild_view = format!("DROP VIEW IF EXISTS {rebuild_table_name}");
    sqlx::query(&drop_rebuild_view).execute(pool).await?;
    let drop_rebuild_table = format!("DROP TABLE IF EXISTS {rebuild_table_name}");
    sqlx::query(&drop_rebuild_table).execute(pool).await?;

    let rebuild_create_sql = create_sql.replacen(table_name, &rebuild_table_name, 1);
    sqlx::query(&rebuild_create_sql).execute(pool).await?;
    sqlx::query(insert_sql).execute(pool).await?;

    let drop_source_table = format!("DROP TABLE {table_name}");
    sqlx::query(&drop_source_table).execute(pool).await?;
    let rename_sql = format!("ALTER TABLE {rebuild_table_name} RENAME TO {table_name}");
    sqlx::query(&rename_sql).execute(pool).await?;
    Ok(())
}

fn sqlite_has_column(columns: &[String], column_name: &str) -> bool {
    columns.iter().any(|candidate| candidate == column_name)
}

fn sqlite_first_available_expr(columns: &[String], candidates: &[&str], fallback: &str) -> String {
    candidates
        .iter()
        .find(|candidate| sqlite_has_column(columns, candidate))
        .map(|candidate| (*candidate).to_owned())
        .unwrap_or_else(|| fallback.to_owned())
}

fn sqlite_first_text_expr(columns: &[String], candidates: &[&str], fallback: &str) -> String {
    let expressions: Vec<String> = candidates
        .iter()
        .filter(|candidate| sqlite_has_column(columns, candidate))
        .map(|candidate| format!("NULLIF({candidate}, '')"))
        .collect();

    if expressions.is_empty() {
        fallback.to_owned()
    } else {
        format!("COALESCE({}, {fallback})", expressions.join(", "))
    }
}

fn sqlite_first_nullable_text_expr(columns: &[String], candidates: &[&str]) -> String {
    let expressions: Vec<String> = candidates
        .iter()
        .filter(|candidate| sqlite_has_column(columns, candidate))
        .map(|candidate| format!("NULLIF({candidate}, '')"))
        .collect();

    if expressions.is_empty() {
        "NULL".to_owned()
    } else if expressions.len() == 1 {
        expressions[0].clone()
    } else {
        format!("COALESCE({})", expressions.join(", "))
    }
}

fn sqlite_non_zero_or_fallback_expr(
    columns: &[String],
    primary: &str,
    secondary: &str,
    fallback: &str,
) -> String {
    match (
        sqlite_has_column(columns, primary),
        sqlite_has_column(columns, secondary),
    ) {
        (true, true) => format!(
            "CASE WHEN {primary} IS NOT NULL AND {primary} <> 0 THEN {primary} ELSE {secondary} END"
        ),
        (true, false) => primary.to_owned(),
        (false, true) => secondary.to_owned(),
        (false, false) => fallback.to_owned(),
    }
}
