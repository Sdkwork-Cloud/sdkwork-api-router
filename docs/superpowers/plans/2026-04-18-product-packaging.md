# Product Packaging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Converge SDKWork API Router packaging and release behavior onto two official products: the server bundle and the portal desktop bundle.

**Architecture:** Release and build tooling will treat Rust binaries, browser builds, and Tauri bundle directories as intermediate outputs, then materialize only the server product archive and portal desktop installers as official release assets. Workflow contracts, local build plans, and operator docs will all align to that product taxonomy.

**Tech Stack:** Node.js tooling, GitHub Actions, Rust/Tauri, pnpm workspace apps, Markdown docs

---

### Task 1: Freeze The Product-Line Contract In Tests

**Files:**
- Modify: `bin/tests/router-runtime-tooling.test.mjs`
- Modify: `scripts/release/tests/release-workflow.test.mjs`
- Modify: `scripts/release-flow-contract.test.mjs`
- Modify: `scripts/release/release-workflow-contracts.mjs`

- [ ] **Step 1: Write failing tests for official SKU expectations**

Add assertions that:

- local release build plans only schedule the portal desktop product, not the admin desktop product
- local release build plans no longer schedule public `web` release packaging
- release workflow publishes only official server and desktop products
- release workflow does not keep a public `web-release` asset job
- official desktop product identity is the portal app

- [ ] **Step 2: Run the targeted Node tests and confirm they fail for the expected reasons**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release-flow-contract.test.mjs
```

Expected:

- assertions fail because the current code still references admin desktop release packaging and web release assets

- [ ] **Step 3: Update workflow contract helpers to match the new product taxonomy**

Change the helper expectations so they validate:

- a single official desktop release path
- server packaging and governance artifacts
- no public web release asset path

- [ ] **Step 4: Re-run the targeted tests**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release-flow-contract.test.mjs
```

Expected:

- workflow contract tests still fail until the implementation files are updated

### Task 2: Converge Local Build And Packaging Tooling

**Files:**
- Modify: `bin/lib/router-runtime-tooling.mjs`
- Modify: `bin/router-ops.mjs`
- Modify: `scripts/release/package-release-assets.mjs`
- Modify: `scripts/release/run-desktop-release-build.mjs`

- [ ] **Step 1: Write failing tests for local build-plan convergence**

Extend the build-plan tests to require:

- only the portal desktop release build step
- no final `web` package step
- official packaging language centered on server and desktop products

- [ ] **Step 2: Run the focused tooling tests and confirm failure**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
```

Expected:

- failures on old admin desktop and web packaging expectations

- [ ] **Step 3: Implement the minimal build-plan and packager changes**

Change the production build plan so it:

- still builds admin and portal browser assets for the server product
- only builds the portal desktop installer as an official desktop product
- packages only official server and desktop release assets

- [ ] **Step 4: Re-run the focused tooling tests**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs scripts/release-flow-contract.test.mjs
```

Expected:

- local build and packager contract tests pass

### Task 3: Converge GitHub Release Workflow

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `scripts/release/tests/release-workflow.test.mjs`
- Modify: `scripts/release/release-workflow-contracts.mjs`

- [ ] **Step 1: Write or update failing workflow assertions first**

Require the release workflow to:

- publish only official server and desktop assets
- keep governance evidence as artifacts
- avoid public web asset publication

- [ ] **Step 2: Run workflow tests and verify failure**

Run:

```bash
node --test scripts/release/tests/release-workflow.test.mjs
```

Expected:

- failures because the workflow still has the old asset topology

- [ ] **Step 3: Implement the workflow changes**

Update `release.yml` so that:

- native matrix lanes build server payloads and portal desktop installers
- governance jobs upload evidence artifacts without publishing them as user-facing release assets
- the publish job only attaches official server and desktop assets

- [ ] **Step 4: Re-run workflow tests**

Run:

```bash
node --test scripts/release/tests/release-workflow.test.mjs scripts/release-flow-contract.test.mjs
```

Expected:

- workflow contract tests pass

### Task 4: Update Operator And Release Documentation

**Files:**
- Modify: `docs/getting-started/release-builds.md`
- Modify: `docs/getting-started/production-deployment.md`
- Modify: `docs/operations/install-layout.md`
- Modify: `docs/operations/service-management.md`
- Modify: `docs/operations/configuration.md`
- Modify: `docs/zh/getting-started/release-builds.md`
- Modify: `docs/zh/getting-started/production-deployment.md`
- Modify: `docs/zh/operations/install-layout.md`
- Modify: `docs/zh/operations/service-management.md`
- Modify: `docs/zh/operations/configuration.md`

- [ ] **Step 1: Write doc assertions or review checklist items before editing**

Confirm the docs will explicitly state:

- two official products only
- server default database is PostgreSQL
- config files override environment fallback values
- public release assets exclude the old admin desktop and web release concepts

- [ ] **Step 2: Update the docs**

Rewrite release and deployment pages so the user-facing story is:

- build server
- build desktop
- publish official release
- deploy server online
- initialize config and PostgreSQL

- [ ] **Step 3: Run markdown or contract checks that cover these docs**

Run:

```bash
node --test scripts/check-router-product.test.mjs scripts/product-verification-workflow.test.mjs
```

Expected:

- relevant product and workflow doc-adjacent checks pass

### Task 5: Full Verification

**Files:**
- No new files

- [ ] **Step 1: Run the targeted packaging and workflow suite**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs scripts/release/tests/release-workflow.test.mjs scripts/release-flow-contract.test.mjs
```

Expected:

- all targeted packaging and release workflow tests pass

- [ ] **Step 2: Run the broader product verification subset**

Run:

```bash
node --test scripts/product-verification-workflow.test.mjs scripts/check-router-product.test.mjs
```

Expected:

- product verification contracts remain green after the packaging convergence

- [ ] **Step 3: Do a final requirement pass**

Check the implementation against the design spec:

- exactly two official release products
- server bundle remains deployable
- desktop bundle remains portal-first
- docs match actual build and publish behavior
