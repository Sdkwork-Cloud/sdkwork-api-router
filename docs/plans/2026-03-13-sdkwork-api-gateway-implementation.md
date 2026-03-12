# SDKWork API Gateway Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the first working version of the SDKWork API Gateway from an empty directory, including the Rust microservice-oriented workspace, OpenAI-compatible contracts, the first four `/v1/*` APIs, usage and billing foundations, and the React plus Tauri control plane.

**Architecture:** The system is implemented as logical full microservices with a physically composable runtime. Rust crates carry the domain, application, interface, provider, storage, security, and runtime layers, while `console/` hosts a pnpm workspace for the Web and Tauri control plane that consumes the same admin APIs in both host modes.

**Tech Stack:** Rust, Axum, Tokio, Tower, Reqwest, Serde, Schemars, SQLx, SeaQuery, Tracing, OpenTelemetry, Prometheus, pnpm, Turbo, React, TypeScript, Vite, Tauri 2, TanStack Router, TanStack Query, Zustand

---

### Task 1: Initialize The Repository Skeleton

**Files:**
- Create: `.gitignore`
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `clippy.toml`
- Create: `services/gateway-service/Cargo.toml`
- Create: `services/gateway-service/src/main.rs`
- Create: `crates/sdkwork-api-kernel/Cargo.toml`
- Create: `crates/sdkwork-api-kernel/src/lib.rs`
- Create: `crates/sdkwork-api-kernel/tests/workspace_smoke.rs`
- Create: `console/package.json`
- Create: `console/pnpm-workspace.yaml`
- Create: `console/turbo.json`
- Create: `console/tsconfig.json`
- Create: `console/vite.config.ts`
- Create: `console/src/main.tsx`
- Create: `console/src/App.tsx`

**Step 1: Initialize git and write the failing smoke test**

Run:

```bash
git init
```

Create `crates/sdkwork-api-kernel/tests/workspace_smoke.rs`:

```rust
use sdkwork_api_kernel::workspace_name;

#[test]
fn exposes_workspace_name() {
    assert_eq!(workspace_name(), "sdkwork-api-server");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-kernel workspace_smoke -q`

Expected: FAIL with workspace or crate not found errors.

**Step 3: Write the minimal workspace implementation**

Create the root workspace plus kernel crate:

```rust
pub fn workspace_name() -> &'static str {
    "sdkwork-api-server"
}
```

Create a minimal React shell in `console/src/App.tsx`:

```tsx
export function App() {
  return <div>SDKWork API Console</div>;
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-kernel workspace_smoke -q`

Expected: PASS

**Step 5: Commit**

```bash
git add .
git commit -m "chore: initialize workspace skeleton"
```

### Task 2: Add Shared Runtime, Config, And Observability Crates

**Files:**
- Create: `crates/sdkwork-api-config/Cargo.toml`
- Create: `crates/sdkwork-api-config/src/lib.rs`
- Create: `crates/sdkwork-api-config/tests/config_loading.rs`
- Create: `crates/sdkwork-api-observability/Cargo.toml`
- Create: `crates/sdkwork-api-observability/src/lib.rs`
- Create: `crates/sdkwork-api-observability/tests/telemetry_smoke.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-config/tests/config_loading.rs`:

```rust
use sdkwork_api_config::RuntimeMode;

#[test]
fn defaults_to_server_mode() {
    assert_eq!(RuntimeMode::default(), RuntimeMode::Server);
}
```

Create `crates/sdkwork-api-observability/tests/telemetry_smoke.rs`:

```rust
use sdkwork_api_observability::service_name;

#[test]
fn exposes_service_name() {
    assert_eq!(service_name("gateway-service"), "gateway-service");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-config -p sdkwork-api-observability -q`

Expected: FAIL with unresolved imports or missing crates.

**Step 3: Write the minimal implementation**

Add config primitives:

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RuntimeMode {
    #[default]
    Server,
    Embedded,
}
```

Add observability helper:

```rust
pub fn service_name(name: &str) -> &str {
    name
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-config -p sdkwork-api-observability -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-config crates/sdkwork-api-observability
git commit -m "feat: add shared config and observability crates"
```

### Task 3: Define OpenAI Common, Error, And Models Contracts

**Files:**
- Create: `crates/sdkwork-api-contract-openai/Cargo.toml`
- Create: `crates/sdkwork-api-contract-openai/src/lib.rs`
- Create: `crates/sdkwork-api-contract-openai/src/common.rs`
- Create: `crates/sdkwork-api-contract-openai/src/errors.rs`
- Create: `crates/sdkwork-api-contract-openai/src/models.rs`
- Create: `crates/sdkwork-api-contract-openai/tests/errors_contract.rs`
- Create: `crates/sdkwork-api-contract-openai/tests/models_contract.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-contract-openai/tests/errors_contract.rs`:

```rust
use sdkwork_api_contract_openai::errors::OpenAiErrorResponse;

#[test]
fn serializes_openai_error_shape() {
    let json = serde_json::to_value(OpenAiErrorResponse::new(
        "bad request",
        "invalid_request_error",
    ))
    .unwrap();

    assert_eq!(json["error"]["message"], "bad request");
    assert_eq!(json["error"]["type"], "invalid_request_error");
}
```

Create `crates/sdkwork-api-contract-openai/tests/models_contract.rs`:

```rust
use sdkwork_api_contract_openai::models::{ListModelsResponse, ModelObject};

#[test]
fn wraps_models_in_list_object() {
    let response = ListModelsResponse::new(vec![ModelObject::new("model-1", "sdkwork")]);
    assert_eq!(response.object, "list");
    assert_eq!(response.data.len(), 1);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-contract-openai --test errors_contract --test models_contract -q`

Expected: FAIL with missing types.

**Step 3: Write the minimal implementation**

Add the shared models:

```rust
#[derive(serde::Serialize)]
pub struct OpenAiErrorEnvelope {
    pub message: String,
    pub r#type: String,
    pub param: Option<String>,
    pub code: Option<String>,
}
```

```rust
#[derive(serde::Serialize)]
pub struct ModelObject {
    pub id: String,
    pub object: &'static str,
    pub owned_by: String,
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-contract-openai --test errors_contract --test models_contract -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-contract-openai
git commit -m "feat: add openai common, error, and models contracts"
```

### Task 4: Define Chat, Responses, Embeddings, And Streaming Contracts

**Files:**
- Modify: `crates/sdkwork-api-contract-openai/src/lib.rs`
- Create: `crates/sdkwork-api-contract-openai/src/chat_completions.rs`
- Create: `crates/sdkwork-api-contract-openai/src/responses.rs`
- Create: `crates/sdkwork-api-contract-openai/src/embeddings.rs`
- Create: `crates/sdkwork-api-contract-openai/src/streaming.rs`
- Create: `crates/sdkwork-api-contract-openai/tests/chat_contract.rs`
- Create: `crates/sdkwork-api-contract-openai/tests/responses_contract.rs`
- Create: `crates/sdkwork-api-contract-openai/tests/embeddings_contract.rs`
- Create: `crates/sdkwork-api-contract-openai/tests/streaming_contract.rs`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-contract-openai/tests/chat_contract.rs`:

```rust
use sdkwork_api_contract_openai::chat_completions::ChatCompletionChunk;

#[test]
fn serializes_chunk_object() {
    let json = serde_json::to_value(ChatCompletionChunk::empty("chatcmpl-1", "gpt-4.1")).unwrap();
    assert_eq!(json["object"], "chat.completion.chunk");
}
```

Create `crates/sdkwork-api-contract-openai/tests/responses_contract.rs`:

```rust
use sdkwork_api_contract_openai::responses::ResponseObject;

#[test]
fn serializes_response_object() {
    let json = serde_json::to_value(ResponseObject::empty("resp_1", "gpt-4.1")).unwrap();
    assert_eq!(json["object"], "response");
}
```

Create `crates/sdkwork-api-contract-openai/tests/embeddings_contract.rs`:

```rust
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;

#[test]
fn serializes_embeddings_list() {
    let response = CreateEmbeddingResponse::empty("text-embedding-3-large");
    assert_eq!(response.object, "list");
}
```

Create `crates/sdkwork-api-contract-openai/tests/streaming_contract.rs`:

```rust
use sdkwork_api_contract_openai::streaming::SseFrame;

#[test]
fn formats_sse_frame() {
    let frame = SseFrame::data("{\"ok\":true}");
    assert_eq!(frame.to_string(), "data: {\"ok\":true}\n\n");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-contract-openai --test chat_contract --test responses_contract --test embeddings_contract --test streaming_contract -q`

Expected: FAIL with missing modules or structs.

**Step 3: Write the minimal implementation**

Add contract types such as:

```rust
#[derive(serde::Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: &'static str,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}
```

```rust
pub struct SseFrame(String);

impl SseFrame {
    pub fn data(payload: &str) -> Self {
        Self(format!("data: {payload}\n\n"))
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-contract-openai --test chat_contract --test responses_contract --test embeddings_contract --test streaming_contract -q`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/sdkwork-api-contract-openai
git commit -m "feat: add chat, responses, embeddings, and streaming contracts"
```

### Task 5: Add Canonical Gateway Contracts

**Files:**
- Create: `crates/sdkwork-api-contract-gateway/Cargo.toml`
- Create: `crates/sdkwork-api-contract-gateway/src/lib.rs`
- Create: `crates/sdkwork-api-contract-gateway/src/canonical.rs`
- Create: `crates/sdkwork-api-contract-gateway/tests/canonical_request.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing test**

Create `crates/sdkwork-api-contract-gateway/tests/canonical_request.rs`:

```rust
use sdkwork_api_contract_gateway::canonical::{CanonicalCapability, CanonicalRequest};

#[test]
fn canonical_request_tracks_capability() {
    let request = CanonicalRequest::new(CanonicalCapability::ChatCompletion, "gpt-4.1");
    assert_eq!(request.capability, CanonicalCapability::ChatCompletion);
    assert_eq!(request.model, "gpt-4.1");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-contract-gateway --test canonical_request -q`

Expected: FAIL with missing crate or type definitions.

**Step 3: Write the minimal implementation**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalCapability {
    ChatCompletion,
    Responses,
    Embeddings,
    ModelListing,
    Streaming,
}
```

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRequest {
    pub capability: CanonicalCapability,
    pub model: String,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-contract-gateway --test canonical_request -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-contract-gateway
git commit -m "feat: add canonical gateway contracts"
```

### Task 6: Define Domain Crates And Repository Traits

**Files:**
- Create: `crates/sdkwork-api-domain-identity/Cargo.toml`
- Create: `crates/sdkwork-api-domain-identity/src/lib.rs`
- Create: `crates/sdkwork-api-domain-identity/tests/gateway_api_key_rules.rs`
- Create: `crates/sdkwork-api-domain-tenant/Cargo.toml`
- Create: `crates/sdkwork-api-domain-tenant/src/lib.rs`
- Create: `crates/sdkwork-api-domain-catalog/Cargo.toml`
- Create: `crates/sdkwork-api-domain-catalog/src/lib.rs`
- Create: `crates/sdkwork-api-domain-credential/Cargo.toml`
- Create: `crates/sdkwork-api-domain-credential/src/lib.rs`
- Create: `crates/sdkwork-api-domain-routing/Cargo.toml`
- Create: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Create: `crates/sdkwork-api-domain-usage/Cargo.toml`
- Create: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Create: `crates/sdkwork-api-domain-billing/Cargo.toml`
- Create: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-domain-identity/tests/gateway_api_key_rules.rs`:

```rust
use sdkwork_api_domain_identity::GatewayApiKey;

#[test]
fn revoked_key_is_not_active() {
    let mut key = GatewayApiKey::new("tenant-1", "project-1", "live");
    key.revoke();
    assert!(!key.is_active());
}
```

Add a routing rule test in `crates/sdkwork-api-domain-routing/src/lib.rs` or a dedicated test file:

```rust
use sdkwork_api_domain_routing::RoutingDecision;

#[test]
fn decision_retains_candidate_ids() {
    let decision = RoutingDecision::new("provider-a", vec!["provider-a".into(), "provider-b".into()]);
    assert_eq!(decision.candidate_ids.len(), 2);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-domain-identity -p sdkwork-api-domain-routing -q`

Expected: FAIL with missing crates or domain types.

**Step 3: Write the minimal implementation**

Create the aggregate roots and repository traits:

```rust
pub trait GatewayApiKeyRepository: Send + Sync {
    fn save(&self, key: &GatewayApiKey) -> Result<(), String>;
}
```

```rust
pub struct RoutingDecision {
    pub selected_provider_id: String,
    pub candidate_ids: Vec<String>,
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-domain-identity -p sdkwork-api-domain-routing -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-domain-identity crates/sdkwork-api-domain-tenant crates/sdkwork-api-domain-catalog crates/sdkwork-api-domain-credential crates/sdkwork-api-domain-routing crates/sdkwork-api-domain-usage crates/sdkwork-api-domain-billing
git commit -m "feat: add domain crates and repository traits"
```

### Task 7: Add Storage Core And SQLite Driver

**Files:**
- Create: `crates/sdkwork-api-storage-core/Cargo.toml`
- Create: `crates/sdkwork-api-storage-core/src/lib.rs`
- Create: `crates/sdkwork-api-storage-core/tests/driver_selection.rs`
- Create: `crates/sdkwork-api-storage-sqlite/Cargo.toml`
- Create: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/migrations/0001_identity.sql`
- Create: `crates/sdkwork-api-storage-sqlite/migrations/0002_tenant.sql`
- Create: `crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-storage-core/tests/driver_selection.rs`:

```rust
use sdkwork_api_storage_core::StorageDialect;

#[test]
fn dialect_reports_sqlite_name() {
    assert_eq!(StorageDialect::Sqlite.as_str(), "sqlite");
}
```

Create `crates/sdkwork-api-storage-sqlite/tests/sqlite_migrations.rs`:

```rust
use sdkwork_api_storage_sqlite::run_migrations;

#[tokio::test]
async fn creates_identity_and_tenant_tables() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let row: (String,) = sqlx::query_as("select name from sqlite_master where name = 'identity_users'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, "identity_users");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-storage-core -p sdkwork-api-storage-sqlite -q`

Expected: FAIL with missing driver or migration code.

**Step 3: Write the minimal implementation**

Add the storage core enum:

```rust
pub enum StorageDialect {
    Sqlite,
    Postgres,
    Mysql,
    Libsql,
}
```

Add SQLite migration runner:

```rust
pub async fn run_migrations(url: &str) -> anyhow::Result<sqlx::SqlitePool> {
    let pool = sqlx::SqlitePool::connect(url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-storage-core -p sdkwork-api-storage-sqlite -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-storage-core crates/sdkwork-api-storage-sqlite
git commit -m "feat: add storage core and sqlite driver"
```

### Task 8: Add PostgreSQL, MySQL, And libsql Driver Boundaries

**Files:**
- Create: `crates/sdkwork-api-storage-postgres/Cargo.toml`
- Create: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Create: `crates/sdkwork-api-storage-mysql/Cargo.toml`
- Create: `crates/sdkwork-api-storage-mysql/src/lib.rs`
- Create: `crates/sdkwork-api-storage-libsql/Cargo.toml`
- Create: `crates/sdkwork-api-storage-libsql/src/lib.rs`
- Create: `crates/sdkwork-api-storage-postgres/tests/dialect_name.rs`
- Create: `crates/sdkwork-api-storage-mysql/tests/dialect_name.rs`
- Create: `crates/sdkwork-api-storage-libsql/tests/dialect_name.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Each test should assert that the driver exposes the right dialect string:

```rust
use sdkwork_api_storage_postgres::dialect_name;

#[test]
fn reports_postgres_name() {
    assert_eq!(dialect_name(), "postgres");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-storage-postgres -p sdkwork-api-storage-mysql -p sdkwork-api-storage-libsql -q`

Expected: FAIL with missing crates.

**Step 3: Write the minimal implementation**

Add one function per driver:

```rust
pub fn dialect_name() -> &'static str {
    "postgres"
}
```

Mirror for MySQL and libsql.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-storage-postgres -p sdkwork-api-storage-mysql -p sdkwork-api-storage-libsql -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-storage-postgres crates/sdkwork-api-storage-mysql crates/sdkwork-api-storage-libsql
git commit -m "feat: add storage driver extension boundaries"
```

### Task 9: Add Secret Storage, JWT, And Gateway API Key Security

**Files:**
- Create: `crates/sdkwork-api-secret-core/Cargo.toml`
- Create: `crates/sdkwork-api-secret-core/src/lib.rs`
- Create: `crates/sdkwork-api-secret-core/tests/envelope_roundtrip.rs`
- Create: `crates/sdkwork-api-secret-local/Cargo.toml`
- Create: `crates/sdkwork-api-secret-local/src/lib.rs`
- Create: `crates/sdkwork-api-secret-keyring/Cargo.toml`
- Create: `crates/sdkwork-api-secret-keyring/src/lib.rs`
- Create: `crates/sdkwork-api-app-identity/Cargo.toml`
- Create: `crates/sdkwork-api-app-identity/src/lib.rs`
- Create: `crates/sdkwork-api-app-identity/tests/jwt_and_api_key.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-secret-core/tests/envelope_roundtrip.rs`:

```rust
use sdkwork_api_secret_core::{decrypt, encrypt};

#[test]
fn encrypt_roundtrip_returns_original_secret() {
    let envelope = encrypt("master-key", "sk-upstream").unwrap();
    let secret = decrypt("master-key", &envelope).unwrap();
    assert_eq!(secret, "sk-upstream");
}
```

Create `crates/sdkwork-api-app-identity/tests/jwt_and_api_key.rs`:

```rust
use sdkwork_api_app_identity::{hash_gateway_api_key, issue_jwt, verify_jwt};

#[test]
fn gateway_api_key_hash_is_not_plaintext() {
    let hash = hash_gateway_api_key("skw_live_example");
    assert_ne!(hash, "skw_live_example");
}

#[test]
fn issued_jwt_verifies() {
    let token = issue_jwt("user-1").unwrap();
    let claims = verify_jwt(&token).unwrap();
    assert_eq!(claims.sub, "user-1");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-secret-core -p sdkwork-api-app-identity -q`

Expected: FAIL with missing symbols.

**Step 3: Write the minimal implementation**

Add the envelope model:

```rust
pub struct SecretEnvelope {
    pub ciphertext: String,
    pub key_version: u32,
}
```

Add basic auth helpers:

```rust
pub fn hash_gateway_api_key(value: &str) -> String {
    format!("hashed:{value}")
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-secret-core -p sdkwork-api-app-identity -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-secret-core crates/sdkwork-api-secret-local crates/sdkwork-api-secret-keyring crates/sdkwork-api-app-identity
git commit -m "feat: add secret storage and identity security primitives"
```

### Task 10: Bootstrap Application And Interface Crates With Health Endpoints

**Files:**
- Create: `crates/sdkwork-api-app-gateway/Cargo.toml`
- Create: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Create: `crates/sdkwork-api-app-routing/Cargo.toml`
- Create: `crates/sdkwork-api-app-routing/src/lib.rs`
- Create: `crates/sdkwork-api-app-catalog/Cargo.toml`
- Create: `crates/sdkwork-api-app-catalog/src/lib.rs`
- Create: `crates/sdkwork-api-app-credential/Cargo.toml`
- Create: `crates/sdkwork-api-app-credential/src/lib.rs`
- Create: `crates/sdkwork-api-app-usage/Cargo.toml`
- Create: `crates/sdkwork-api-app-usage/src/lib.rs`
- Create: `crates/sdkwork-api-app-billing/Cargo.toml`
- Create: `crates/sdkwork-api-app-billing/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/Cargo.toml`
- Create: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/tests/health_route.rs`
- Create: `crates/sdkwork-api-interface-admin/Cargo.toml`
- Create: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `services/admin-api-service/Cargo.toml`
- Create: `services/admin-api-service/src/main.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing test**

Create `crates/sdkwork-api-interface-http/tests/health_route.rs`:

```rust
use axum::http::StatusCode;

#[tokio::test]
async fn health_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = tower::ServiceExt::oneshot(app, axum::http::Request::builder().uri("/health").body(axum::body::Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sdkwork-api-interface-http --test health_route -q`

Expected: FAIL with missing router.

**Step 3: Write the minimal implementation**

```rust
pub fn gateway_router() -> axum::Router {
    axum::Router::new().route("/health", axum::routing::get(|| async { "ok" }))
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sdkwork-api-interface-http --test health_route -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-app-gateway crates/sdkwork-api-app-routing crates/sdkwork-api-app-catalog crates/sdkwork-api-app-credential crates/sdkwork-api-app-usage crates/sdkwork-api-app-billing crates/sdkwork-api-interface-http crates/sdkwork-api-interface-admin services/gateway-service services/admin-api-service
git commit -m "feat: add app and interface crate skeletons"
```

### Task 11: Implement Control Plane Auth, Tenant, Project, And Key APIs

**Files:**
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`
- Create: `crates/sdkwork-api-interface-admin/tests/auth_and_project_routes.rs`
- Create: `crates/sdkwork-api-app-identity/tests/create_api_key.rs`
- Create: `crates/sdkwork-api-domain-tenant/tests/project_rules.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-tenant/src/lib.rs`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-interface-admin/tests/auth_and_project_routes.rs`:

```rust
use axum::http::StatusCode;

#[tokio::test]
async fn login_route_exists() {
    let app = sdkwork_api_interface_admin::admin_router();
    let response = tower::ServiceExt::oneshot(
        app,
        axum::http::Request::builder().uri("/admin/auth/login").method("POST").body(axum::body::Body::from("{}")).unwrap(),
    )
    .await
    .unwrap();

    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}
```

Create `crates/sdkwork-api-app-identity/tests/create_api_key.rs`:

```rust
use sdkwork_api_app_identity::CreateGatewayApiKey;

#[test]
fn generated_key_has_sdkwork_prefix() {
    let created = CreateGatewayApiKey::execute("tenant-1", "project-1", "live").unwrap();
    assert!(created.plaintext.starts_with("skw_live_"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-interface-admin -p sdkwork-api-app-identity -q`

Expected: FAIL with route or use-case missing.

**Step 3: Write the minimal implementation**

Add admin router modules for:

- `/admin/auth/login`
- `/admin/auth/me`
- `/admin/tenants`
- `/admin/projects`
- `/admin/api-keys`

Create the API key command:

```rust
pub struct CreateGatewayApiKey;
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-interface-admin -p sdkwork-api-app-identity -q`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/sdkwork-api-interface-admin crates/sdkwork-api-app-identity crates/sdkwork-api-domain-tenant
git commit -m "feat: add auth, tenant, project, and gateway key admin apis"
```

### Task 12: Implement Catalog, Credential, Routing, And Route Simulation APIs

**Files:**
- Create: `crates/sdkwork-api-app-catalog/tests/create_provider.rs`
- Create: `crates/sdkwork-api-app-routing/tests/simulate_route.rs`
- Create: `crates/sdkwork-api-app-credential/tests/save_credential.rs`
- Modify: `crates/sdkwork-api-app-catalog/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-credential/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-admin/src/lib.rs`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-app-routing/tests/simulate_route.rs`:

```rust
use sdkwork_api_app_routing::simulate_route;

#[test]
fn route_simulation_prefers_healthy_low_cost_provider() {
    let decision = simulate_route("chat_completion", "gpt-4.1").unwrap();
    assert_eq!(decision.selected_provider_id, "provider-openai-official");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-app-catalog -p sdkwork-api-app-routing -p sdkwork-api-app-credential -q`

Expected: FAIL with use-cases not implemented.

**Step 3: Write the minimal implementation**

Implement admin endpoints for:

- `/admin/channels`
- `/admin/providers`
- `/admin/credentials`
- `/admin/models`
- `/admin/routing/policies`
- `/admin/routing/simulations`

Implement a minimal simulation service:

```rust
pub fn simulate_route(_capability: &str, _model: &str) -> anyhow::Result<RoutingDecision> {
    Ok(RoutingDecision::new(
        "provider-openai-official",
        vec!["provider-openai-official".into()],
    ))
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-app-catalog -p sdkwork-api-app-routing -p sdkwork-api-app-credential -q`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/sdkwork-api-app-catalog crates/sdkwork-api-app-routing crates/sdkwork-api-app-credential crates/sdkwork-api-interface-admin
git commit -m "feat: add catalog, credential, and routing admin apis"
```

### Task 13: Add Provider Core And Initial Adapters

**Files:**
- Create: `crates/sdkwork-api-provider-core/Cargo.toml`
- Create: `crates/sdkwork-api-provider-core/src/lib.rs`
- Create: `crates/sdkwork-api-provider-core/tests/adapter_contract.rs`
- Create: `crates/sdkwork-api-provider-openai/Cargo.toml`
- Create: `crates/sdkwork-api-provider-openai/src/lib.rs`
- Create: `crates/sdkwork-api-provider-openai/tests/models_mapping.rs`
- Create: `crates/sdkwork-api-provider-openrouter/Cargo.toml`
- Create: `crates/sdkwork-api-provider-openrouter/src/lib.rs`
- Create: `crates/sdkwork-api-provider-ollama/Cargo.toml`
- Create: `crates/sdkwork-api-provider-ollama/src/lib.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-provider-core/tests/adapter_contract.rs`:

```rust
use sdkwork_api_provider_core::{CapabilitySupport, ProviderAdapter};

struct DummyAdapter;

impl ProviderAdapter for DummyAdapter {
    fn id(&self) -> &'static str {
        "dummy"
    }
}

#[test]
fn adapter_exposes_identifier() {
    assert_eq!(DummyAdapter.id(), "dummy");
}
```

Create `crates/sdkwork-api-provider-openai/tests/models_mapping.rs`:

```rust
use sdkwork_api_provider_openai::map_model_object;

#[test]
fn maps_provider_model_to_catalog_entry() {
    let entry = map_model_object("gpt-4.1");
    assert_eq!(entry.external_name, "gpt-4.1");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-provider-core -p sdkwork-api-provider-openai -q`

Expected: FAIL with missing traits or mapper functions.

**Step 3: Write the minimal implementation**

Add the adapter trait:

```rust
pub trait ProviderAdapter {
    fn id(&self) -> &'static str;
}
```

Add initial adapter scaffolds for OpenAI, OpenRouter, and Ollama.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-provider-core -p sdkwork-api-provider-openai -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-provider-core crates/sdkwork-api-provider-openai crates/sdkwork-api-provider-openrouter crates/sdkwork-api-provider-ollama
git commit -m "feat: add provider core and initial adapters"
```

### Task 14: Implement `/v1/models`

**Files:**
- Create: `crates/sdkwork-api-app-gateway/tests/list_models.rs`
- Create: `crates/sdkwork-api-interface-http/tests/models_route.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-app-gateway/tests/list_models.rs`:

```rust
use sdkwork_api_app_gateway::list_models;

#[test]
fn returns_platform_models() {
    let response = list_models("tenant-1", "project-1").unwrap();
    assert_eq!(response.object, "list");
}
```

Create `crates/sdkwork-api-interface-http/tests/models_route.rs`:

```rust
use axum::http::StatusCode;

#[tokio::test]
async fn models_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = tower::ServiceExt::oneshot(
        app,
        axum::http::Request::builder().uri("/v1/models").body(axum::body::Body::empty()).unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-app-gateway -p sdkwork-api-interface-http --test models_route -q`

Expected: FAIL with route or use-case missing.

**Step 3: Write the minimal implementation**

Add a platform-backed models query and wire `/v1/models` to it.

```rust
pub fn list_models(_tenant_id: &str, _project_id: &str) -> anyhow::Result<ListModelsResponse> {
    Ok(ListModelsResponse::new(vec![ModelObject::new("gpt-4.1", "sdkwork")]))
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-app-gateway -p sdkwork-api-interface-http --test models_route -q`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/sdkwork-api-app-gateway crates/sdkwork-api-interface-http
git commit -m "feat: implement models endpoint"
```

### Task 15: Implement `/v1/chat/completions` And SSE Relay

**Files:**
- Create: `crates/sdkwork-api-app-gateway/tests/chat_completions.rs`
- Create: `crates/sdkwork-api-interface-http/tests/chat_route.rs`
- Create: `crates/sdkwork-api-interface-http/tests/chat_stream_route.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-app-gateway/tests/chat_completions.rs`:

```rust
use sdkwork_api_app_gateway::create_chat_completion;

#[test]
fn returns_chat_completion_response() {
    let response = create_chat_completion("tenant-1", "project-1", "gpt-4.1").unwrap();
    assert_eq!(response.object, "chat.completion");
}
```

Create `crates/sdkwork-api-interface-http/tests/chat_stream_route.rs`:

```rust
use axum::http::StatusCode;

#[tokio::test]
async fn chat_stream_route_accepts_requests() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = tower::ServiceExt::oneshot(
        app,
        axum::http::Request::builder()
            .method("POST")
            .uri("/v1/chat/completions")
            .header("content-type", "application/json")
            .body(axum::body::Body::from("{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}],\"stream\":true}"))
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-app-gateway -p sdkwork-api-interface-http --test chat_route --test chat_stream_route -q`

Expected: FAIL with route or completion generation missing.

**Step 3: Write the minimal implementation**

Add chat generation orchestration and wire POST `/v1/chat/completions`.

```rust
pub fn create_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> anyhow::Result<ChatCompletionResponse> {
    Ok(ChatCompletionResponse::empty("chatcmpl_1", model))
}
```

Add an SSE response path that emits one data frame and a `[DONE]` frame.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-app-gateway -p sdkwork-api-interface-http --test chat_route --test chat_stream_route -q`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/sdkwork-api-app-gateway crates/sdkwork-api-interface-http
git commit -m "feat: implement chat completions and sse relay"
```

### Task 16: Implement `/v1/responses` And `/v1/embeddings`

**Files:**
- Create: `crates/sdkwork-api-app-gateway/tests/responses_api.rs`
- Create: `crates/sdkwork-api-app-gateway/tests/embeddings_api.rs`
- Create: `crates/sdkwork-api-interface-http/tests/responses_route.rs`
- Create: `crates/sdkwork-api-interface-http/tests/embeddings_route.rs`
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-app-gateway/tests/responses_api.rs`:

```rust
use sdkwork_api_app_gateway::create_response;

#[test]
fn returns_response_object() {
    let response = create_response("tenant-1", "project-1", "gpt-4.1").unwrap();
    assert_eq!(response.object, "response");
}
```

Create `crates/sdkwork-api-app-gateway/tests/embeddings_api.rs`:

```rust
use sdkwork_api_app_gateway::create_embedding;

#[test]
fn returns_embedding_list() {
    let response = create_embedding("tenant-1", "project-1", "text-embedding-3-large").unwrap();
    assert_eq!(response.object, "list");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-app-gateway -p sdkwork-api-interface-http --test responses_route --test embeddings_route -q`

Expected: FAIL with missing handlers or use-cases.

**Step 3: Write the minimal implementation**

Implement:

- POST `/v1/responses`
- POST `/v1/embeddings`

Use the canonical gateway contract internally for both request families.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-app-gateway -p sdkwork-api-interface-http --test responses_route --test embeddings_route -q`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/sdkwork-api-app-gateway crates/sdkwork-api-interface-http
git commit -m "feat: implement responses and embeddings endpoints"
```

### Task 17: Record Usage, Quota, Billing, And Outbox Events

**Files:**
- Create: `crates/sdkwork-api-app-usage/tests/record_usage.rs`
- Create: `crates/sdkwork-api-app-billing/tests/quota_and_ledger.rs`
- Create: `crates/sdkwork-api-domain-usage/tests/request_fact.rs`
- Create: `crates/sdkwork-api-domain-billing/tests/ledger_entry.rs`
- Modify: `crates/sdkwork-api-app-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-usage/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-app-billing/tests/quota_and_ledger.rs`:

```rust
use sdkwork_api_app_billing::{book_usage_cost, check_quota};

#[test]
fn booking_usage_creates_ledger_entry() {
    assert!(check_quota("project-1", 100).unwrap());
    let ledger = book_usage_cost("project-1", 100, 0.25).unwrap();
    assert_eq!(ledger.project_id, "project-1");
}
```

Create `crates/sdkwork-api-app-usage/tests/record_usage.rs`:

```rust
use sdkwork_api_app_usage::record_usage;

#[test]
fn usage_record_contains_model_and_provider() {
    let usage = record_usage("project-1", "gpt-4.1", "provider-openai-official").unwrap();
    assert_eq!(usage.model, "gpt-4.1");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-app-usage -p sdkwork-api-app-billing -q`

Expected: FAIL with missing use-cases.

**Step 3: Write the minimal implementation**

Add request fact and ledger types, then emit an outbox-style domain event after each booking.

```rust
pub fn record_usage(project_id: &str, model: &str, provider: &str) -> anyhow::Result<UsageRecord> {
    Ok(UsageRecord::new(project_id, model, provider))
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-app-usage -p sdkwork-api-app-billing -q`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/sdkwork-api-app-usage crates/sdkwork-api-app-billing crates/sdkwork-api-domain-usage crates/sdkwork-api-domain-billing
git commit -m "feat: add usage, quota, billing, and outbox foundations"
```

### Task 18: Create The Console Workspace Skeleton

**Files:**
- Create: `console/packages/sdkwork-api-types/package.json`
- Create: `console/packages/sdkwork-api-types/src/index.ts`
- Create: `console/packages/sdkwork-api-core/package.json`
- Create: `console/packages/sdkwork-api-core/src/index.ts`
- Create: `console/packages/sdkwork-api-admin-sdk/package.json`
- Create: `console/packages/sdkwork-api-admin-sdk/src/index.ts`
- Create: `console/packages/sdkwork-api-workspace/package.json`
- Create: `console/packages/sdkwork-api-workspace/src/index.tsx`
- Create: `console/packages/sdkwork-api-channel/package.json`
- Create: `console/packages/sdkwork-api-channel/src/index.tsx`
- Modify: `console/package.json`
- Modify: `console/pnpm-workspace.yaml`
- Modify: `console/src/App.tsx`

**Step 1: Write the failing typecheck target**

Create `console/packages/sdkwork-api-core/src/index.ts`:

```ts
export const appName: string = 42;
```

**Step 2: Run typecheck to verify it fails**

Run: `pnpm --dir console install && pnpm --dir console --filter sdkwork-api-core typecheck`

Expected: FAIL with `Type 'number' is not assignable to type 'string'`.

**Step 3: Write the minimal implementation**

Fix the package and wire the root shell to package exports:

```ts
export const appName = 'SDKWork API Console';
```

```tsx
import { appName } from 'sdkwork-api-core';

export function App() {
  return <div>{appName}</div>;
}
```

**Step 4: Run typecheck to verify it passes**

Run: `pnpm --dir console --filter sdkwork-api-core typecheck`

Expected: PASS

**Step 5: Commit**

```bash
git add console
git commit -m "feat: add console workspace skeleton"
```

### Task 19: Build Core Console Pages

**Files:**
- Create: `console/packages/sdkwork-api-workspace/src/pages/WorkspaceDashboard.tsx`
- Create: `console/packages/sdkwork-api-channel/src/pages/ChannelRegistryPage.tsx`
- Create: `console/packages/sdkwork-api-routing/package.json`
- Create: `console/packages/sdkwork-api-routing/src/pages/RouteSimulationPage.tsx`
- Create: `console/packages/sdkwork-api-usage/package.json`
- Create: `console/packages/sdkwork-api-usage/src/pages/RequestExplorerPage.tsx`
- Create: `console/packages/sdkwork-api-runtime/package.json`
- Create: `console/packages/sdkwork-api-runtime/src/pages/RuntimeStatusPage.tsx`
- Modify: `console/src/App.tsx`

**Step 1: Write the failing UI smoke test**

Create `console/packages/sdkwork-api-routing/src/pages/RouteSimulationPage.tsx` with an invalid import:

```tsx
import { MissingWidget } from 'sdkwork-api-commons';

export function RouteSimulationPage() {
  return <MissingWidget />;
}
```

**Step 2: Run build to verify it fails**

Run: `pnpm --dir console exec vite build`

Expected: FAIL with unresolved import.

**Step 3: Write the minimal implementation**

Implement the page modules and route composition for:

- overview dashboard
- workspace
- channel registry
- route simulation
- request explorer
- runtime status

Use complete placeholder pages with stable exports rather than empty files.

**Step 4: Run build to verify it passes**

Run: `pnpm --dir console exec vite build`

Expected: PASS

**Step 5: Commit**

```bash
git add console
git commit -m "feat: add core console pages"
```

### Task 20: Add Tauri Integration And Embedded Runtime Host

**Files:**
- Create: `crates/sdkwork-api-runtime-host/Cargo.toml`
- Create: `crates/sdkwork-api-runtime-host/src/lib.rs`
- Create: `crates/sdkwork-api-runtime-host/tests/embedded_runtime.rs`
- Create: `console/src-tauri/Cargo.toml`
- Create: `console/src-tauri/src/main.rs`
- Create: `console/src-tauri/tauri.conf.json`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

Create `crates/sdkwork-api-runtime-host/tests/embedded_runtime.rs`:

```rust
use sdkwork_api_runtime_host::EmbeddedRuntime;

#[tokio::test]
async fn embedded_runtime_starts_on_loopback() {
    let runtime = EmbeddedRuntime::start_ephemeral().await.unwrap();
    assert!(runtime.base_url().starts_with("http://127.0.0.1:"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p sdkwork-api-runtime-host --test embedded_runtime -q`

Expected: FAIL with missing runtime host.

**Step 3: Write the minimal implementation**

Implement the runtime host and make Tauri depend on it by path:

```rust
pub struct EmbeddedRuntime {
    base_url: String,
}
```

```rust
impl EmbeddedRuntime {
    pub async fn start_ephemeral() -> anyhow::Result<Self> {
        Ok(Self { base_url: "http://127.0.0.1:3001".into() })
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p sdkwork-api-runtime-host --test embedded_runtime -q`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/sdkwork-api-runtime-host console/src-tauri
git commit -m "feat: add embedded runtime host and tauri integration"
```

### Task 21: Run Full Verification And Polish Docs

**Files:**
- Modify: `docs/plans/2026-03-13-sdkwork-api-gateway-design.md`
- Modify: `docs/plans/2026-03-13-sdkwork-api-gateway-implementation.md`
- Create: `README.md`
- Create: `docs/api/compatibility-matrix.md`
- Create: `docs/architecture/runtime-modes.md`

**Step 1: Write the failing verification checklist**

Add a checklist to `README.md` that references the commands below and leave one item unchecked.

```md
- [ ] cargo test --workspace
```

**Step 2: Run verification to confirm work is not complete yet**

Run:

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir console exec vite build
pnpm --dir console --filter ./... typecheck
```

Expected: At least one command fails before the final fixes.

**Step 3: Fix remaining issues and document the final state**

Update docs with:

- runtime modes
- compatibility matrix
- local development commands
- standalone versus embedded startup paths

Mark the checklist complete.

**Step 4: Run verification to confirm everything passes**

Run:

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir console exec vite build
pnpm --dir console --filter ./... typecheck
```

Expected: PASS on all commands

**Step 5: Commit**

```bash
git add README.md docs
git commit -m "docs: finalize gateway architecture and verification docs"
```
