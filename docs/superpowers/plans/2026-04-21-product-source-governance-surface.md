# Product Source Governance Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** extend product source tracking so the official runtime control surface under `bin/` and the integrated product host under `services/router-product-service/` are governed alongside scripts, workflows, and the two product UIs.

**Architecture:** keep product source tracking narrowly scoped to product-impacting source trees, but remove blind spots by adding the release/runtime control tree and the deployable integrated service entrypoint as first-class governed roots. The audit remains distinct from release-only subtree governance and continues to filter by source file extensions.

**Tech Stack:** Node.js governance scripts, Node test runner, git-based untracked file audit.

---

### Task 1: Expand The Governance Contract

**Files:**
- Modify: `scripts/check-product-source-tracking.test.mjs`
- Modify: `scripts/check-product-source-tracking.mjs`
- Test: `scripts/check-product-source-tracking.test.mjs`

- [ ] **Step 1: Add the failing root catalog expectation**

Require `bin` and `services/router-product-service` to appear in both exported governed-root lists.

- [ ] **Step 2: Add the failing untracked-file filtering expectation**

Make the fixture git output include representative untracked files under `bin/` and `services/router-product-service/`, and assert they are returned while release-only exclusions still hold.

- [ ] **Step 3: Run the focused test and confirm the red state**

Run:

```bash
node --test scripts/check-product-source-tracking.test.mjs
```

Expected: FAIL until the governed root list is expanded in the implementation.

- [ ] **Step 4: Implement the minimal root-list change**

Add:

- `bin`
- `services/router-product-service`

to `GOVERNED_PRODUCT_SOURCE_ROOT_SPECS`, and update the failure guidance string so the operator message names the full governed surface.

- [ ] **Step 5: Re-run the focused test**

Run:

```bash
node --test scripts/check-product-source-tracking.test.mjs
```

Expected: PASS.

### Task 2: Re-verify The Governance Layer

**Files:**
- Test: `scripts/run-product-governance-node-tests.mjs`
- Test: `scripts/check-product-source-tracking.mjs`

- [ ] **Step 1: Re-run the product governance node test bundle**

Run:

```bash
node scripts/run-product-governance-node-tests.mjs
```

Expected: PASS.

- [ ] **Step 2: Re-run the live audit**

Run:

```bash
node scripts/check-product-source-tracking.mjs
```

Expected: JSON output with `ok: true` and the expanded governed root catalog.
