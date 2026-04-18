# Bundle-Driven Install Packaging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make server installation consume the official `sdkwork-api-router-product-server` release bundle instead of raw workspace build outputs, and align release smoke verification to that bundle-driven install path.

**Architecture:** Runtime install planning will resolve the canonical packaged server archive plus its external manifest from `artifacts/release/native/<platform>/<arch>/bundles/`, then derive the install payload from the bundle contents and manifest metadata. Release workflow verification will run after packaging so the installed runtime smoke checks prove the same official bundle that later becomes the published product.

**Tech Stack:** Node.js tooling, GitHub Actions workflow YAML, repository install/runtime scripts, Markdown docs

---

### Task 1: Freeze The Bundle-Driven Install Contract In Tests

**Files:**
- Modify: `bin/tests/router-runtime-tooling.test.mjs`
- Modify: `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`
- Modify: `scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`
- Modify: `scripts/release/tests/release-workflow.test.mjs`

- [ ] **Step 1: Write failing install-plan tests**

Add assertions that:

- `createInstallPlan(...)` resolves the official packaged server archive and external manifest under `artifacts/release/native/<platform>/<arch>/bundles/`
- release payload entries come from bundle-relative sources instead of workspace `target/release` binaries or app `dist/` directories
- the installed release payload preserves `release-manifest.json`, `README.txt`, and `deploy/`

- [ ] **Step 2: Write failing smoke/workflow tests**

Add assertions that:

- Unix and Windows installed-runtime smoke plan creation stays parseable after bundle-driven install inputs are introduced
- the release workflow collects native release assets before the installed-runtime smoke steps

- [ ] **Step 3: Run the targeted tests and verify they fail**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/release-workflow.test.mjs
```

Expected:

- current install plan still points at raw workspace outputs and workflow still runs smoke before packaging

### Task 2: Implement Bundle-Driven Install Planning

**Files:**
- Modify: `bin/lib/router-runtime-tooling.mjs`

- [ ] **Step 1: Add official server bundle resolution helpers**

Introduce focused helpers that:

- resolve the canonical packaged server archive and external manifest for the requested platform and architecture
- parse and validate the external manifest contract
- describe bundle-derived release payload entries for install planning

- [ ] **Step 2: Replace raw release payload copy sources with bundle-derived entries**

Update `createInstallPlan(...)` so:

- versioned payload files come from the packaged server bundle contract
- current control assets still come from repository-owned scripts and generated metadata
- `installedBinaries` and payload roots align to the parsed external manifest instead of hard-coded assumptions where possible

- [ ] **Step 3: Re-run focused install-plan tests**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
```

Expected:

- new install contract tests pass

### Task 3: Teach Install Application To Materialize The Official Bundle

**Files:**
- Modify: `bin/lib/router-runtime-tooling.mjs`

- [ ] **Step 1: Add failing application tests if needed**

If the new plan entry types require it, add tests that prove `assertInstallInputsExist(...)` and `applyInstallPlan(...)` can validate and materialize bundle-derived payload inputs.

- [ ] **Step 2: Implement bundle extraction/materialization logic**

Make the installer:

- validate the packaged archive and external manifest exist
- extract the official bundle in a controlled temporary workspace
- copy bundle payload files into `releases/<version>/` while preserving the immutable release payload contract

- [ ] **Step 3: Re-run the focused runtime tooling tests**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
```

Expected:

- bundle-driven install plan and install application coverage pass together

### Task 4: Align Release Smoke And Workflow Ordering

**Files:**
- Modify: `scripts/release/run-unix-installed-runtime-smoke.mjs`
- Modify: `scripts/release/run-windows-installed-runtime-smoke.mjs`
- Modify: `.github/workflows/release.yml`
- Modify: `scripts/release/release-workflow-contracts.mjs`
- Modify: `scripts/release/tests/release-workflow.test.mjs`

- [ ] **Step 1: Keep smoke plan generation compatible with the new install input contract**

Update smoke plan creation so it resolves the official packaged server bundle path after native packaging.

- [ ] **Step 2: Reorder the release workflow**

Move native packaging ahead of installed-runtime smoke and keep all smoke lanes consuming the packaged server bundle as their install/deploy truth source.

- [ ] **Step 3: Re-run targeted workflow/smoke tests**

Run:

```bash
node --test scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/release-workflow.test.mjs
```

Expected:

- workflow and smoke contract tests pass with the new ordering

### Task 5: Update Docs And Verify End-To-End

**Files:**
- Modify: `docs/operations/install-layout.md`
- Modify: `docs/operations/service-management.md`
- Modify: `docs/getting-started/production-deployment.md`
- Modify: `docs/zh/operations/install-layout.md`
- Modify: `docs/zh/operations/service-management.md`
- Modify: `docs/zh/getting-started/production-deployment.md`
- Modify: `scripts/release/tests/docs-product-contract.test.mjs`

- [ ] **Step 1: Write doc assertions first**

Require docs to state:

- server installs are derived from the official packaged server bundle
- release smoke verifies the packaged bundle install path
- versioned release payload includes packaged runtime metadata and deploy assets

- [ ] **Step 2: Update docs**

Describe the bundle-driven install contract in both locales.

- [ ] **Step 3: Run focused and broader verification**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/docs-product-contract.test.mjs
node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs scripts/browser-runtime-smoke.test.mjs scripts/check-admin-browser-runtime.test.mjs scripts/check-portal-browser-runtime.test.mjs scripts/build-router-desktop-assets.test.mjs scripts/check-router-docs-safety.test.mjs scripts/check-router-frontend-budgets.test.mjs scripts/dev/tests/pnpm-launch-lib.test.mjs scripts/prepare-router-portal-desktop-runtime.test.mjs scripts/release-flow-contract.test.mjs scripts/release/tests/materialize-release-catalog.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release/tests/release-attestation-verify.test.mjs scripts/release/tests/docs-product-contract.test.mjs apps/sdkwork-router-portal/tests/product-entrypoint-scripts.test.mjs
```

Expected:

- targeted bundle-driven install tests pass
- broader product/release verification remains green
