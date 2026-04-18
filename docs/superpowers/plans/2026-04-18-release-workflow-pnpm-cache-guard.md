# Release Workflow Pnpm Cache Guard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prevent GitHub release jobs that do not install `pnpm` from failing when `actions/setup-node@v5` auto-detects the root `packageManager` field.

**Architecture:** Tighten the release workflow contract so jobs that rely on `actions/setup-node@v5` without `pnpm/action-setup@v4` must explicitly disable package-manager auto-cache. Then update `.github/workflows/release.yml` so the Rust audit, governance, and publish jobs satisfy the contract and can run on GitHub-hosted runners without `pnpm` on `PATH`.

**Tech Stack:** GitHub Actions YAML, Node.js test runner, release workflow contract helpers

---

### Task 1: Add workflow contract coverage for non-pnpm jobs

**Files:**
- Modify: `scripts/release/tests/release-workflow.test.mjs`
- Modify: `scripts/release/release-workflow-contracts.mjs`

- [ ] **Step 1: Write the failing test**

Add a repository contract assertion that requires `package-manager-cache: false` in `rust-dependency-audit`, `governance-release`, and `publish`, plus a negative fixture test that omits the flag.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test scripts/release/tests/release-workflow.test.mjs`
Expected: FAIL because `.github/workflows/release.yml` currently sets only `node-version: 22` in those jobs.

- [ ] **Step 3: Write minimal implementation**

Add contract assertions that distinguish non-pnpm jobs from pnpm-backed jobs and require `package-manager-cache: false` for the former.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test scripts/release/tests/release-workflow.test.mjs`
Expected: PASS after both the helper and workflow satisfy the new rule.

- [ ] **Step 5: Commit**

Commit message: `fix: disable setup-node auto cache in release jobs without pnpm`

### Task 2: Patch the release workflow and verify the release path

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `README.md` only if release instructions need updates after verification

- [ ] **Step 1: Patch release workflow**

Add `package-manager-cache: false` to `actions/setup-node@v5` steps in `rust-dependency-audit`, `governance-release`, and `publish`.

- [ ] **Step 2: Run focused verification**

Run: `node --test scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-sync-audit.test.mjs scripts/run-router-product.test.mjs`
Expected: PASS with zero failures.

- [ ] **Step 3: Re-verify root dev entrypoints**

Run: `pnpm tauri:dev -- --dry-run`
Run: `pnpm server:dev -- --dry-run -- --help`
Expected: both commands resolve to the intended product launch scripts.

- [ ] **Step 4: Commit and publish**

Run: `git add ...`
Run: `git commit -m "fix: disable setup-node auto cache in release jobs without pnpm"`
Run: `git push origin main`

- [ ] **Step 5: Tag and watch release**

Create the next release tag, push it, watch the workflow, inspect any failed job logs if needed, and confirm the GitHub release exists.
