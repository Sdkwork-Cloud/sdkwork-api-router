# Crates 模块拆分实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立统一的 crate 模块边界和大文件治理机制，持续消除 `crates` 目录中的超大 `lib.rs` 与职责混杂问题。

**Architecture:** 先建立统一治理规范，再按 crate 类型分批拆分。优先处理基础设施和高复用核心模块，再处理接口层、应用层、领域层和测试层，确保结构先收敛，再继续迭代业务能力。

**Tech Stack:** Rust、TypeScript/React、Axum、SQLx、OpenAPI、workspace crates

---

### Task 1: 建立统一治理基线

**Files:**
- Create: `docs/架构/12-crates模块规划与大文件治理规范.md`
- Create: `docs/架构/13-crates模块拆分实施计划.md`

- [x] Step 1: 盘点 `crates` 目录中超过 `1000` 行的源码文件
- [x] Step 2: 归纳 crate 类型与职责失衡模式
- [x] Step 3: 固化 `lib.rs`、模块命名、文件行数、目录结构规范
- [x] Step 4: 形成后续拆分统一模板

### Task 2: 收敛基础存储抽象层

**Files:**
- Modify: `crates/sdkwork-api-storage-core/src/lib.rs`
- Create: `crates/sdkwork-api-storage-core/src/types.rs`
- Create: `crates/sdkwork-api-storage-core/src/admin_facets.rs`
- Create: `crates/sdkwork-api-storage-core/src/admin_store.rs`
- Create: `crates/sdkwork-api-storage-core/src/kernel_support.rs`
- Create: `crates/sdkwork-api-storage-core/src/identity_kernel_store.rs`
- Create: `crates/sdkwork-api-storage-core/src/account_kernel_store.rs`
- Create: `crates/sdkwork-api-storage-core/src/marketing_store.rs`
- Create: `crates/sdkwork-api-storage-core/src/account_transaction.rs`

- [x] Step 1: 按类型、facet、store trait、kernel transaction 分离模块
- [x] Step 2: 将根 `lib.rs` 收敛为导出入口
- [x] Step 3: 校验每个文件行数进入安全区间
- [x] Step 4: 把该结构作为 storage crate 样板

### Task 3: 继续拆分 Postgres 存储实现

**Files:**
- Modify: `crates/sdkwork-api-storage-postgres/src/lib.rs`
- Create: `crates/sdkwork-api-storage-postgres/src/migrations.rs`
- Create: `crates/sdkwork-api-storage-postgres/src/postgres_migration_schema.rs`
- Create: `crates/sdkwork-api-storage-postgres/src/postgres_migration_seed.rs`

- [ ] Step 1: 把 `run_migrations` 从根模块迁出
- [ ] Step 2: 按 schema / compatibility / seed 拆迁移逻辑
- [ ] Step 3: 让 `lib.rs` 只保留连接入口与导出
- [ ] Step 4: 继续清理剩余 account kernel 超大块

### Task 4: 对齐 SQLite 存储结构

**Files:**
- Modify: `crates/sdkwork-api-storage-sqlite/src/lib.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/migrations.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/catalog_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/commerce_checkout_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/commerce_finance_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/commerce_store_mappers.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/marketing_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/account_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/jobs_store.rs`
- Create: `crates/sdkwork-api-storage-sqlite/src/runtime_store.rs`

- [ ] Step 1: 以 Postgres 已成型结构为镜像模板
- [ ] Step 2: 把 SQLite 中 commerce / account / marketing / jobs / runtime 逐域拆开
- [ ] Step 3: 拆出 mapper、decoder、迁移逻辑
- [ ] Step 4: 将根文件压到 `1000` 行以下

### Task 5: 拆分 HTTP 接口总入口

**Files:**
- Modify: `crates/sdkwork-api-interface-http/src/lib.rs`
- Create: `crates/sdkwork-api-interface-http/src/routes.rs`
- Create: `crates/sdkwork-api-interface-http/src/auth.rs`
- Create: `crates/sdkwork-api-interface-http/src/chat.rs`
- Create: `crates/sdkwork-api-interface-http/src/files.rs`
- Create: `crates/sdkwork-api-interface-http/src/responses.rs`
- Create: `crates/sdkwork-api-interface-http/src/webhook.rs`
- Create: `crates/sdkwork-api-interface-http/src/openapi.rs`
- Create: `crates/sdkwork-api-interface-http/src/error.rs`

- [ ] Step 1: 先按入口域拆 handlers
- [ ] Step 2: 把认证、错误映射、OpenAPI、Webhook 独立
- [ ] Step 3: 根模块只保留 router 装配和共享状态
- [ ] Step 4: 为后续真实支付回调与安全逻辑预留稳定模块边界

### Task 6: 拆分核心应用编排 crate

**Files:**
- Modify: `crates/sdkwork-api-app-gateway/src/lib.rs`
- Modify: `crates/sdkwork-api-app-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-app-identity/src/lib.rs`
- Modify: `crates/sdkwork-api-app-runtime/src/lib.rs`
- Modify: `crates/sdkwork-api-app-routing/src/lib.rs`

- [ ] Step 1: 按业务编排流程拆分 `gateway / billing / identity / runtime / routing`
- [ ] Step 2: 分离 provider 接入、策略决策、状态推进、回调处理
- [ ] Step 3: 消除应用层对接口 DTO 和存储细节的混杂引用
- [ ] Step 4: 形成 app crate 统一目录组织

### Task 7: 收敛领域与配置 crate

**Files:**
- Modify: `crates/sdkwork-api-domain-billing/src/lib.rs`
- Modify: `crates/sdkwork-api-domain-routing/src/lib.rs`
- Modify: `crates/sdkwork-api-config/src/lib.rs`

- [ ] Step 1: 将 record、enum、policy、value object 分层
- [ ] Step 2: 把配置解析、默认值、环境变量映射独立
- [ ] Step 3: 保持 domain crate 专注于模型表达

### Task 8: 收敛测试文件规模

**Files:**
- Modify: `crates/*/tests/*.rs`

- [ ] Step 1: 将超大集成测试按业务主链路拆分
- [ ] Step 2: 避免一个测试文件覆盖过多 endpoint
- [ ] Step 3: 为最终统一回归测试做准备

### Task 9: 统一复核

**Files:**
- Modify: `docs/架构/11-超大文件拆分执行清单.md`
- Modify: `docs/架构/12-crates模块规划与大文件治理规范.md`
- Modify: `docs/架构/13-crates模块拆分实施计划.md`

- [ ] Step 1: 重新盘点超限文件清单
- [ ] Step 2: 记录已完成与剩余批次
- [ ] Step 3: 在最后统一执行编译与回归测试

## 2026-04-06 本轮新增进展

- 已完成 `crates/sdkwork-api-domain-routing/src/lib.rs` 拆分
  - 新增 `decision.rs`、`policy.rs`、`profile.rs`、`routing_support.rs`
  - 根 `lib.rs` 已收敛为导出入口

- 已完成 `crates/sdkwork-api-config/src/lib.rs` 拆分
  - 新增 `env_keys.rs`、`types.rs`、`loader.rs`、`http_exposure.rs`、`standalone_config.rs`、`config_support.rs`
  - 根 `lib.rs` 已收敛为导出入口

- 已完成 `crates/sdkwork-api-app-routing/src/lib.rs` 拆分
  - 新增 `route_inputs.rs`、`route_management.rs`、`route_selection.rs`、`candidate_selection.rs`、`routing_support.rs`
  - 根 `lib.rs` 已收敛为导出入口

- 已完成 `crates/sdkwork-api-app-billing/src/lib.rs` 拆分
  - 新增 `billing_inputs.rs`、`billing_events.rs`、`account_balance.rs`、`account_mutations.rs`、`commerce_credits.rs`、`billing_kernels.rs`、`pricing_lifecycle.rs`、`billing_summary.rs`、`billing_support.rs`
  - 根 `lib.rs` 已收敛为装配与导出入口

- 已完成 `crates/sdkwork-api-app-runtime/src/lib.rs` 拆分
  - 新增 `standalone_listener.rs`、`runtime_core.rs`、`runtime_builders.rs`、`rollout_models.rs`、`rollout_execution.rs`、`runtime_reload.rs`、`tests.rs`
  - 根 `lib.rs` 已收敛为装配与导出入口

- 已完成 `crates/sdkwork-api-app-identity/src/lib.rs` 拆分
  - 新增 `jwt_support.rs`、`identity_types.rs`、`admin_users.rs`、`api_key_groups.rs`、`gateway_api_keys.rs`、`portal_users.rs`、`portal_api_keys.rs`、`identity_support.rs`、`tests.rs`
  - 根 `lib.rs` 已收敛为装配与导出入口

- 已完成 `crates/sdkwork-api-storage-postgres/src/lib.rs` 二次拆分收口
  - 新增 `migrations.rs`、`postgres_migration_identity_schema.rs`、`postgres_migration_marketing_schema.rs`、`postgres_migration_routing_schema.rs`、`postgres_migration_billing_schema.rs`
  - 新增 `postgres_migration_commerce_jobs_schema.rs`、`postgres_migration_catalog_gateway_schema.rs`、`postgres_migration_runtime_schema.rs`、`postgres_migration_seed.rs`
  - 新增 `account_kernel_store.rs`、`account_kernel_transaction.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块装配与导出入口，账户内核事务与迁移编排已按职责拆离

- 已完成 `crates/sdkwork-api-app-gateway/src/lib.rs` 拆分
  - 新增 `gateway_types.rs`、`gateway_extension_host.rs`、`model_catalog.rs`、`gateway_provider_resolution.rs`、`gateway_cache.rs`、`gateway_routing.rs`
  - 新增 `gateway_execution_context.rs`、`request_context.rs`、`gateway_runtime_execution.rs`
  - 新增 `relay_chat.rs`、`relay_conversations.rs`、`relay_threads.rs`、`relay_responses.rs`、`relay_compute.rs`、`relay_containers.rs`
  - 新增 `relay_files_uploads.rs`、`relay_fine_tuning.rs`、`relay_assistants_realtime_webhooks.rs`、`relay_evals_batches.rs`、`relay_vector_stores.rs`、`relay_music_video.rs`、`tests.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块声明、有限包装转发与公开导出入口，网关执行内核、路由缓存、扩展主机与各 OpenAI 资源 relay 已按职责分离

- 已完成 `crates/sdkwork-api-provider-openai/src/lib.rs` 拆分
  - 新增 `adapter_core.rs`、`dialog_resources.rs`、`media_resources.rs`、`control_resources.rs`、`openai_transport.rs`、`trait_impl.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块声明、运输辅助内部导出与适配器公共导出入口
  - OpenAI 官方适配器已按“适配器核心 / 对话资源 / 媒体文件资源 / 控制面资源 / 传输辅助 / trait 分发”拆分，便于后续继续扩展资源端点与兼容头策略

- 已完成 `crates/sdkwork-api-extension-host/src/lib.rs` 拆分收口
  - 新增 `host_types.rs`、`extension_discovery.rs`、`extension_trust.rs`、`connector_runtime.rs`、`native_dynamic_runtime.rs`、`host_impl.rs`、`provider_invocation.rs`、`errors.rs`、`tests.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块装配、内部复用导入与公共导出入口
  - 已修正测试模块错切、错误枚举 derive 缺失等拆分残留问题

- 已完成 `crates/sdkwork-api-storage-sqlite/src/lib.rs` 交易与基础设施大文件拆分
  - 新增迁移编排与 schema 模块：
    `migrations.rs`、`sqlite_migration_identity_schema.rs`、`sqlite_migration_marketing_schema.rs`、`sqlite_migration_routing_schema.rs`、`sqlite_migration_billing_schema.rs`、`sqlite_migration_commerce_jobs_schema.rs`、`sqlite_migration_catalog_gateway_schema.rs`、`sqlite_migration_catalog_gateway_compat.rs`、`sqlite_migration_runtime_schema.rs`、`sqlite_migration_seed.rs`
  - 新增存储与支持模块：
    `sqlite_support.rs`、`catalog_support.rs`、`catalog_store.rs`、`routing_store.rs`、`usage_billing_store.rs`、`tenant_store.rs`、`coupon_store.rs`、`jobs_store.rs`、`commerce_membership_store.rs`、`identity_store.rs`、`runtime_store.rs`、`admin_store_impl.rs`、`identity_kernel_store.rs`、`account_support.rs`、`account_kernel_store.rs`、`account_kernel_transaction.rs`、`marketing_support.rs`、`marketing_store_impl.rs`、`marketing_kernel_transaction.rs`、`tests.rs`
  - 复用并纳入 `commerce_store.rs`、`commerce_checkout_store.rs`、`commerce_finance_store.rs`、`commerce_store_mappers.rs`
  - 根 `lib.rs` 已收敛为共享导入、模块装配、迁移/支持模块导出、`SqliteAdminStore` 入口定义
  - 当前 `storage-sqlite` 各源码文件已控制在 `1000` 行以内，交易、账务、营销、身份、运行时、迁移兼容逻辑按职责分区

- 当前剩余重点拆分目标
  - `crates/sdkwork-api-interface-http/src/lib.rs`
  - 该模块仍为当前 crates 范围内主要超限文件，下一步按 OpenAPI、HTTP 暴露与指标、鉴权与请求上下文、路由装配、OpenAI 兼容入口、各资源域 handler、商业计费与用量记录等职责继续拆分

## 2026-04-06 interface-http 拆分完成

- 已完成 `crates/sdkwork-api-interface-http/src/lib.rs` 主体瘦身，`lib.rs` 仅保留 imports 与 include 装配。
- 已完成 OpenAPI 组装与 path 定义拆分：`gateway_openapi.rs`、`gateway_openapi_paths_*.rs`。
- 已完成 HTTP 接口实现按职责拆分为 `gateway_*` 多文件，覆盖 models/chat/conversations/threads/responses/files/uploads/fine_tuning/assistants/webhooks/realtime/evals/vector_stores 等域。
- 已修复拆分过程中的边界缺口，补齐 `with_state` handler、流式 helper、multipart helper、stateless relay、commercial support 等尾段实现。
- 当前 `crates/sdkwork-api-interface-http/src` 下正式 `.rs` 文件已全部压到 `1000` 行以内，最大文件为 `gateway_containers.rs`，约 `980` 行。
- 当前拆分形态采用 `include!(...)` 保持同模块作用域，先解决大文件与职责混杂问题；后续可在此基础上逐步提升为真正 `mod` 边界。

## 2026-04-11 marketing coupon closure progress

- [x] Split `crates/sdkwork-api-interface-admin/src/marketing.rs` into an aggregate root plus `marketing/template.rs`, `marketing/campaign.rs`, `marketing/budget.rs`, `marketing/code.rs`, `marketing/runtime.rs`, and `marketing/support.rs`, so admin marketing routes stay aligned to governed aggregates instead of regressing to one mixed handler file.
- [x] Split admin marketing route wiring out of `crates/sdkwork-api-interface-admin/src/routes.rs` into `src/routes/marketing_routes.rs`, so handler composition and route composition share the same template/campaign/budget/code boundary and the main admin router no longer owns the full marketing surface.
- [x] Split admin commerce route wiring out of `crates/sdkwork-api-interface-admin/src/routes.rs` into `src/routes/commerce_routes.rs`, so commerce audit/payment/refund/reconciliation endpoints stop sharing the same route-root edit surface as marketing/auth/catalog composition.
- [x] Split admin billing/pricing route wiring out of `crates/sdkwork-api-interface-admin/src/routes.rs` into `src/routes/billing_routes.rs`, so usage reads, billing account/ledger endpoints, pricing lifecycle operations, and quota policy mutations stop sharing the same route-root edit surface as auth/catalog/runtime/jobs/routing composition.
- [x] Split admin catalog route wiring out of `crates/sdkwork-api-interface-admin/src/routes.rs` into `src/routes/catalog_routes.rs`, so channel/provider/credential/channel-model/provider-account/provider-model/model-price routes and tenant provider-readiness no longer share the same route-root edit surface as auth/runtime/jobs/routing composition.
- [x] Split `crates/sdkwork-api-interface-admin/src/commerce.rs` into an aggregate root plus `commerce/order.rs`, `commerce/publication.rs`, `commerce/payment.rs`, and `commerce/reconciliation.rs`, so order audit, governed catalog publication lifecycle, payment method/refund mutations, and webhook/reconciliation reads evolve independently behind one stable admin commerce surface.
- [x] Align `crates/sdkwork-api-interface-admin/src/openapi.rs` with the real marketing router surface by documenting the 4 admin marketing `/status` endpoints, so route exposure and generated docs stay consistent during the coupon-governance refactor.
- [x] Split admin marketing OpenAPI path definitions out of `crates/sdkwork-api-interface-admin/src/openapi.rs` into `src/openapi/marketing_template_paths.rs`, `marketing_campaign_paths.rs`, `marketing_budget_paths.rs`, `marketing_code_paths.rs`, and `marketing_runtime_paths.rs`, so doc ownership now matches admin handler and route aggregate boundaries.
- [x] Reduce `crates/sdkwork-api-interface-admin/src/openapi.rs` to OpenAPI assembly plus route registration for marketing paths, shrinking the file from the previous 1600+ line hotspot to roughly 1000 lines and removing another mixed marketing edit surface from the admin root.
- [x] Split the remaining non-marketing admin OpenAPI path definitions out of `crates/sdkwork-api-interface-admin/src/openapi.rs` into `src/openapi/system_paths.rs`, `auth_paths.rs`, `tenant_paths.rs`, `catalog_paths.rs`, `user_paths.rs`, `gateway_paths.rs`, `billing_paths.rs`, and `commerce_paths.rs`, so admin docs now follow the same domain boundaries as the real router and handler modules.
- [x] Split `crates/sdkwork-api-interface-admin/src/pricing.rs` into an aggregate root plus `pricing/lifecycle.rs`, `pricing/plan.rs`, `pricing/plan_lifecycle.rs`, `pricing/rate.rs`, and `pricing/quota.rs`, so lifecycle synchronization, pricing plan CRUD/clone, publish-schedule-retire transitions, pricing rate CRUD, and quota policy management evolve independently while `commerce/publication.rs` keeps reusing the exported canonical pricing status builders.
- [x] Split `crates/sdkwork-api-interface-admin/src/catalog.rs` into an aggregate root plus `catalog/channel.rs`, `catalog/provider.rs`, `catalog/credential.rs`, `catalog/provider_model.rs`, and `catalog/model.rs`, so catalog channel registry, provider integration/readiness, credential + official provider config management, provider-account/provider-model mapping, and canonical model/model-price management evolve independently behind one stable admin catalog surface.
- [x] Split admin marketing governance request DTOs out of `crates/sdkwork-api-interface-admin/src/types.rs` into `src/types/marketing.rs`, so lifecycle payloads and clone/compare requests follow the same aggregate boundary as admin handlers and OpenAPI paths.
- [x] Split the remaining admin DTOs out of `crates/sdkwork-api-interface-admin/src/types.rs` into `src/types/auth.rs`, `billing.rs`, `catalog.rs`, `commerce.rs`, `gateway.rs`, `pricing.rs`, `routing.rs`, `runtime.rs`, `tenant.rs`, and `user.rs`, so auth, user, catalog, commerce-publication, gateway, pricing, routing, runtime, and tenant payloads evolve independently behind one stable admin type export surface.
- [x] Route admin marketing campaign/budget status updates through focused `find_*` store methods and add explicit sqlite/postgres lookup implementations, so admin handlers stop depending on list-scan semantics for canonical aggregate mutation.
- [x] Centralize coupon subsidy/reserve pricing rules in `crates/sdkwork-api-app-marketing/src/pricing.rs`.
- [x] Move coupon validate/reserve/confirm/release/rollback runtime orchestration into `crates/sdkwork-api-app-marketing/src/operations.rs`.
- [x] Reduce portal and gateway coupon handlers to auth/rate-limit/DTO mapping, with atomic coupon mutations owned by `app-marketing`.
- [x] Separate coupon `budget_consumed_minor` from `subsidy_amount_minor` in redemption records so grant-unit coupons and cash-discount coupons do not share broken budget semantics.
- [x] Move order-scoped marketing context resolution into `crates/sdkwork-api-app-marketing/src/context.rs` so commerce no longer rebuilds coupon/template/campaign/budget joins locally.
- [x] Split `crates/sdkwork-api-app-marketing/src/context.rs` into context types, resolution, visibility, and recovery modules, so marketing context loading and reservation-expiration cleanup stay behind one stable API but no longer share one file.
- [x] Promote marketing finance columns to first-class storage schema fields for budget, reservation, redemption, and rollback records in both sqlite and postgres instead of hiding operational numbers only inside `record_json`.
- [x] Switch commerce order coupon reserve/confirm/release/rollback flow to shared marketing operations and remove duplicate runtime wrappers from `app-commerce`.
- [x] Split `crates/sdkwork-api-app-marketing/src/operations.rs` into validate/reserve/confirm/release/rollback modules plus shared error/types/support modules, keeping one stable app-layer runtime API while isolating idempotent replay, subject ownership, and atomic mutation orchestration by concern.
- [x] Refine `crates/sdkwork-api-app-marketing/src/operations.rs` by extracting reserve/confirm/rollback idempotent replay paths into `src/operations/replay.rs` and `src/operations/replay/{reserve,confirm,rollback}.rs`, so fresh request orchestration and replay semantics evolve independently.
- [x] Move portal coupon subject-ownership queries plus code/reward-history read-model assembly into `crates/sdkwork-api-app-marketing/src/query.rs` so `interface-portal` no longer traverses marketing storage tables directly.
- [x] Move admin canonical marketing list ordering into `crates/sdkwork-api-app-marketing/src/query.rs` so `interface-admin` no longer owns list sorting and read-model normalization.
- [x] Move admin marketing lifecycle audit list loading into `crates/sdkwork-api-app-marketing/src/query.rs` with a consistent descending requested-time sort.
- [x] Move admin commerce order-audit coupon evidence loading into `crates/sdkwork-api-app-marketing/src/query.rs` so reservation/redemption/code/template/campaign graph assembly has one shared read-side implementation.
- [x] Move gateway coupon confirm / rollback subject-ownership checks onto shared `app-marketing/query` helpers so gateway no longer carries a second copy of reservation/redemption ownership rules.
- [x] Move portal/gateway confirm and rollback rate-limit coupon code loading onto shared ownership+relation query helpers so interface handlers stop reading coupon codes directly.
- [x] Move commerce coupon rollback redemption/reservation evidence loading onto shared marketing query helpers so `app-commerce` no longer reads marketing relation records directly.
- [x] Make shared coupon context loading fall back from `effective campaign` to the latest campaign revision when no active campaign exists, preserving historical reward history, rollback, and idempotent replay context after campaign rollout ends.
- [x] Move admin coupon template / campaign / budget / coupon code governance orchestration into `crates/sdkwork-api-app-marketing/src/governance.rs` so `interface-admin` keeps only HTTP adapters, request normalization, and error/status mapping.
- [x] Split `crates/sdkwork-api-app-marketing/src/governance.rs` into `governance/template.rs`, `governance/campaign.rs`, `governance/budget.rs`, and `governance/code.rs`, keeping the public app-layer API stable while isolating lifecycle rules and audit builders per governed aggregate.
- [x] Split `crates/sdkwork-api-app-marketing/src/governance/template.rs` and `governance/campaign.rs` into aggregate-specific internals, then refine them further into `types/lookup/actionability/audit/mutation/comparison` so lineage loading, actionability/detail projection, lifecycle mutation, clone handling, and revision diff logic are independently maintainable inside each governed aggregate.
- [x] Split `crates/sdkwork-api-app-marketing/src/governance/budget.rs` and `governance/code.rs` into `types/actionability/lookup/audit/mutation` submodules so all governed aggregates share one consistent internal architecture instead of mixed leaf-module styles.
- [x] Move admin coupon template / campaign / budget / coupon code create orchestration into `crates/sdkwork-api-app-marketing/src/create.rs` so `interface-admin` no longer inserts canonical marketing records directly.
- [x] Normalize marketing create inputs in one place, including identifier trimming, coupon code normalization, create-time relation existence checks, and business-key conflict detection for template keys and coupon code values.
- [x] Split `crates/sdkwork-api-app-marketing/src/create.rs` into aggregate-specific create modules plus shared create support helpers, so template/campaign/budget/code validation and idempotent ensure behavior evolve independently behind one stable app-layer API.
- [x] Refine `crates/sdkwork-api-app-marketing/src/create/template.rs`, `create/campaign.rs`, `create/budget.rs`, and `create/code.rs` into aggregate export roots plus `prepare.rs` and `persist.rs`, so create-time normalization and idempotent conflict handling evolve independently per aggregate.
- [x] Move expired coupon reservation recovery orchestration into `crates/sdkwork-api-app-marketing/src/recovery.rs` so `app-jobs` keeps only scheduling and metrics concerns.
- [x] Reuse `app-marketing` coupon-code normalization from bootstrap-data import and coupon rate-limit hashing so data initialization, storage lookup, and rate-limit dimensions share one canonical rule.
- [x] Route bootstrap marketing stages through idempotent `ensure_*` orchestration in `app-marketing/create.rs` so repeated seed application avoids duplicate dirty data and still rejects divergent canonical records.
- [x] Collapse marketing create and bootstrap ensure flows onto one shared aggregate persist core in `crates/sdkwork-api-app-marketing/src/create.rs`, so id conflicts, business-key conflicts, relation validation, and coupon-template/code compatibility rules cannot drift.
- [x] Move canonical coupon-code normalization down to `crates/sdkwork-api-domain-marketing/src/lib.rs` and keep `app-marketing` as a re-export, so runtime, bootstrap, rate-limit, and storage adapters all depend on the same value-level rule.
- [x] Move catalog-visible coupon display projection into `crates/sdkwork-api-app-marketing/src/query.rs`, so commerce no longer formats `discount_label / audience / note / expires_on` directly from marketing entities.
- [x] Move catalog coupon resolution-by-code into `crates/sdkwork-api-app-marketing/src/query.rs`, so commerce consumes a shared catalog-visible resolution result instead of re-applying visibility filters around raw marketing context loading.
- [x] Split `crates/sdkwork-api-app-marketing/src/query.rs` into `query/subject.rs`, `query/admin.rs`, `query/order_evidence.rs`, and `query/catalog.rs`, keeping one stable export surface while isolating ownership checks, audit/list reads, evidence assembly, and catalog presentation.
- [x] Split `crates/sdkwork-api-app-marketing/src/query/admin.rs` into an export-only root plus `query/admin/lists.rs` and `query/admin/audits.rs`, so canonical admin list ordering and lifecycle audit ordering evolve independently.
- [x] Split `crates/sdkwork-api-app-marketing/src/query/subject.rs` into `query/subject/ownership.rs`, `query/subject/history.rs`, `query/subject/support.rs`, and `query/subject/types.rs`, so portal ownership validation, subject reward history assembly, and internal record loaders evolve independently.
- [x] Normalize default storage-core coupon-code lookup-by-value helpers, so fallback implementations keep the same casing and whitespace semantics as production storage adapters.
- [x] Normalize governed clone collisions as explicit `Conflict` outcomes and document `409` semantics for admin clone routes instead of treating duplicate ids/keys as generic bad requests.
- [x] Add default lineage-aware and code-bound marketing store helpers so revision sequencing and expired reservation reclamation stop depending on broad collection scans in app-layer code.
- [x] Reuse shared subject-based marketing query helpers inside runtime operations while explicitly retaining `forbidden` and `not_found` error distinctions.
- [x] Move commerce coupon catalog listing onto shared catalog-visible marketing query assembly so `app-commerce` no longer walks coupon code storage records directly.
- [x] Add empty-input short-circuiting to shared code/redemption/rollback relation helpers so batch lookup APIs never degrade into full scans for empty id sets.
- [x] Split `crates/sdkwork-api-app-marketing/src/lib.rs` into an assembly-only root plus `src/kernel.rs`, so coupon runtime kernel rules no longer live beside crate export wiring.
- [x] Refine `crates/sdkwork-api-app-marketing/src/governance/template.rs` from `types/support/mutation/comparison` into `types/lookup/actionability/audit/mutation/comparison`, removing the last mixed template helper surface.
- [x] Refine `crates/sdkwork-api-app-marketing/src/governance/campaign.rs` from `types/support/mutation/comparison` into `types/lookup/actionability/audit/mutation/comparison`, aligning campaign governance with the same high-cohesion internal layout.
- [x] Split `crates/sdkwork-api-app-marketing/src/recovery.rs` into an orchestration root plus `recovery/budget.rs`, `recovery/expiration.rs`, `recovery/outbox.rs`, and `recovery/types.rs`, so reservation expiration policy no longer evolves inside one mixed recovery file.
- [x] Refine `crates/sdkwork-api-app-marketing/src/recovery.rs` into an export-only root plus `recovery/runner.rs` and `recovery/reservation.rs`, so scan orchestration and per-reservation transactional recovery evolve independently.
- [x] Split `crates/sdkwork-api-app-marketing/src/query/subject/history.rs` into an export-only root plus `history/redemptions.rs`, `history/code_views.rs`, `history/reward_history.rs`, and `history/summary.rs`, so subject history read models no longer share one mixed leaf file.
- [x] Split `crates/sdkwork-api-app-marketing/src/governance/template/mutation.rs` into an export-only root plus `mutation/lifecycle.rs` and `mutation/clone.rs`, so lifecycle transitions and clone revision orchestration no longer share one leaf file.
- [x] Split `crates/sdkwork-api-app-marketing/src/governance/campaign/mutation.rs` into an export-only root plus `mutation/lifecycle.rs` and `mutation/clone.rs`, so campaign lifecycle transitions and clone revision orchestration no longer share one leaf file.
- [x] Split `crates/sdkwork-api-app-marketing/src/governance/template/mutation/lifecycle.rs` into an export-only root plus `mutation/lifecycle/apply.rs`, `mutation/lifecycle/validation.rs`, and `mutation/lifecycle/transition.rs`, so lifecycle orchestration, reject-audit guardrails, and transition-table rules evolve independently.
- [x] Split `crates/sdkwork-api-app-marketing/src/governance/campaign/mutation/lifecycle.rs` into an export-only root plus `mutation/lifecycle/apply.rs`, `mutation/lifecycle/validation.rs`, and `mutation/lifecycle/transition.rs`, so lifecycle orchestration, reject-audit guardrails, and transition-table rules evolve independently.
- [x] Split `crates/sdkwork-api-app-marketing/src/kernel.rs` into an export-only root plus `kernel/decision.rs`, `kernel/error.rs`, `kernel/validate.rs`, `kernel/reserve.rs`, `kernel/confirm.rs`, and `kernel/rollback.rs`, so coupon kernel rules no longer share one mixed leaf file.
- [x] Split `crates/sdkwork-api-app-marketing/src/state.rs` into an export-only root plus `state/code.rs`, `state/budget.rs`, and `state/support.rs`, so coupon-code transitions and budget transitions no longer share one mixed state file.
- [x] Split `crates/sdkwork-api-app-marketing/src/idempotency.rs` into an export-only root plus `idempotency/errors.rs`, `idempotency/keys.rs`, `idempotency/fingerprint.rs`, and `idempotency/ids.rs`, so idempotency validation, fingerprinting, and runtime business ID derivation evolve independently.
- [x] Split `crates/sdkwork-api-app-marketing/src/context/resolution.rs` into an export-only root plus `context/resolution/reference.rs`, `context/resolution/selection.rs`, and `context/resolution/loaders.rs`, so reference parsing, campaign fallback selection, and storage-backed context loading no longer share one mixed leaf file.
- [x] Split `crates/sdkwork-api-app-marketing/src/query/catalog.rs` into an export-only root plus `query/catalog/types.rs`, `query/catalog/formatting.rs`, and `query/catalog/loaders.rs`, so catalog view projection and catalog visibility loading evolve independently behind one stable query API.
- [x] Split `crates/sdkwork-api-app-marketing/src/query/subject/support.rs` into an internal export-only root plus `query/subject/support/loaders.rs` and `query/subject/support/aggregation.rs`, so subject lookup fan-out and latest-by-code reduction do not share one mixed helper file.
- [x] Split `crates/sdkwork-api-app-marketing/src/create/support.rs` into an internal export-only root plus `create/support/mode.rs`, `create/support/normalize.rs`, `create/support/errors.rs`, and `create/support/relations.rs`, so create-time idempotency semantics, normalization, and relation validation evolve independently.
- [x] Split `crates/sdkwork-api-app-marketing/src/query/order_evidence.rs` into an export-only root plus `query/order_evidence/types.rs`, `query/order_evidence/loaders.rs`, `query/order_evidence/coupon.rs`, and `query/order_evidence/campaign.rs`, so evidence assembly, coupon relation loading, and campaign lookup evolve independently.
- [x] Refine `crates/sdkwork-api-app-marketing/src/governance/template/actionability.rs` and `governance/campaign/actionability.rs` into `actionability/decision.rs` plus `actionability/detail.rs`, so action permission rules and detail projection evolve independently inside each governed aggregate.
- [ ] Run compile/tests after the remaining marketing cleanup and route verification are finished.
