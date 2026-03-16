# Builtin Upstream Health Probing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add active builtin upstream probing to provider health snapshot capture so persisted health evidence exists even when official HTTP providers have no runtime registry status.

**Architecture:** Extend `sdkwork-api-app-extension` with a bounded HTTP probe helper that only activates for official builtin upstream providers, prefers runtime-backed snapshots when available, and persists `runtime = "builtin"` snapshots through the existing admin store. Respect instance and installation `health_path` overrides while preventing connector or native dynamic providers from being misclassified as probe-only upstreams.

**Tech Stack:** Rust, Tokio, reqwest, Axum test fixtures

---

### Task 1: Add failing observability tests for builtin upstream probing

**Files:**
- Modify: `crates/sdkwork-api-app-extension/Cargo.toml`
- Modify: `crates/sdkwork-api-app-extension/tests/runtime_observability.rs`

**Step 1: Write the failing tests**

Add tests that prove:

- builtin providers without runtime status still produce a health snapshot through active probing
- explicit `health_path` overrides are respected
- connector-managed providers are not probed when runtime status is absent

**Step 2: Run test to verify it fails**

Run:

- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`

Expected: FAIL because builtin providers are currently skipped when no runtime status exists.

### Task 2: Implement builtin upstream probe capture

**Files:**
- Modify: `crates/sdkwork-api-app-extension/Cargo.toml`
- Modify: `crates/sdkwork-api-app-extension/src/lib.rs`

**Step 1: Add bounded HTTP probe support**

Use `reqwest` with a short timeout and probe only official builtin upstream providers after runtime-backed capture is exhausted.

**Step 2: Preserve runtime-first semantics**

Keep the existing runtime status match as the highest-priority source and only probe when no runtime signal is available.

**Step 3: Prevent false positives**

Skip probing for providers that resolve to connector or native dynamic installations.

**Step 4: Run focused test**

Run:

- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`

Expected: PASS

### Task 3: Update design-status documentation

**Files:**
- Create: `docs/plans/2026-03-15-builtin-upstream-health-probing-design.md`
- Create: `docs/plans/2026-03-15-builtin-upstream-health-probing-implementation.md`
- Modify: `docs/architecture/runtime-modes.md`
- Modify: `docs/api/compatibility-matrix.md`

**Step 1: Record the design**

Document the probe-eligibility, path-resolution, and health-interpretation rules.

**Step 2: Refresh architecture and API docs**

State that persisted provider health snapshots now include builtin upstream probe evidence in addition to live runtime status.

### Task 4: Run verification

**Files:**
- Modify: repository worktree from previous tasks

**Step 1: Focused verification**

Run:

- `cargo test -p sdkwork-api-app-extension --test runtime_observability -q`

**Step 2: Full verification**

Run:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace -q -j 1`
- `pnpm --dir console -r typecheck`

Expected: PASS
