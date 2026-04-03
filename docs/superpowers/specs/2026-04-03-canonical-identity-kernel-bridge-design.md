# Canonical Identity Kernel Bridge Design

**Goal:** land the first real bridge between gateway credentials and the canonical bigint identity model so request execution can resolve a `GatewayAuthSubject` from durable identity data instead of staying trapped in workspace-string request context only.

## Problem

The repository already defines the correct canonical identity direction:

- `GatewayAuthSubject`
- bigint `tenant_id`
- bigint `organization_id`
- bigint `user_id`

But the production request path still resolves only:

- `tenant_id: String`
- `project_id: String`
- `environment: String`
- `api_key_hash: String`
- optional `api_key_group_id`

That leaves three structural gaps:

1. gateway billing cannot identify the payable user subject
2. gateway hold planning cannot determine which `ai_account` to charge
3. admin, portal, and gateway do not yet share one canonical identity kernel

## Recommendation

Add the missing canonical identity kernel now, but keep it narrow:

- `ai_user`
- `ai_api_key`
- `ai_identity_binding`
- storage seam for canonical identity records
- app-identity resolver that maps a plaintext API key into `GatewayAuthSubject`

Do not cut the HTTP gateway over to the new resolver in the same slice. The right order is:

1. land canonical identity storage and resolution
2. land account lookup from canonical subject
3. land transactional hold or settlement mutation
4. only then cut the gateway request path over

## Design

### Canonical records

Add three explicit domain records under the identity domain:

- `IdentityUserRecord`
- `CanonicalApiKeyRecord`
- `IdentityBindingRecord`

These mirror the approved aggregated-cloud design and intentionally stay separate from:

- `PortalUserRecord`
- `AdminUserRecord`
- `GatewayApiKeyRecord`

Those older product records can survive temporarily as compatibility and UX surfaces, but they are not the durable billing identity kernel.

### Storage seam

Add a dedicated `IdentityKernelStore` facet instead of overloading `AdminStore` or `AccountKernelStore`.

Minimum operations for the first slice:

- insert and list users
- find user by numeric id
- insert and find canonical API keys by hash
- insert and find identity bindings by `(binding_type, issuer, subject)`

This gives enough capability to resolve both future JWT subjects and current API-key subjects without hard-wiring the logic into one storage implementation.

### Resolution flow

Add an app-identity function:

- `resolve_gateway_auth_subject_from_api_key(store, plaintext_key)`

Behavior:

1. hash plaintext key
2. load canonical API key by hash
3. reject inactive or expired keys
4. update `last_used_at_ms`
5. return `GatewayAuthSubject::for_api_key(...)`

JWT resolution can follow the same store seam later through `ai_identity_binding`.

### Database strategy

SQLite:

- real schema
- real CRUD
- real resolver tests

PostgreSQL:

- schema mirror in this slice
- CRUD parity can follow in the next storage pass if needed

This matches the repository's current reality and keeps the slice focused.

### Compatibility stance

Do not mutate or remove current `GatewayRequestContext` yet.

Instead:

- add the canonical resolver in parallel
- let existing gateway request context continue to serve current routes
- use the new resolver as the next-step seam for hold-settle cutover

That preserves momentum while keeping the architecture clean.

## Why This Order Is Best

If hold-settle mutations land before canonical identity resolution, the gateway still has no trustworthy account subject and the design stays internally inconsistent.

If the gateway is cut over before canonical storage exists, business logic gets duplicated in the HTTP layer.

This bridge slice is the minimum correct step that makes the rest of the commercial kernel actually connect together.
