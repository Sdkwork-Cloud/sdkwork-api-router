# Managed Service Release Build Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** converge official service release builds onto a single managed repository entrypoint so Windows release builds use short-path target and temp directories consistently across scripts, workflow, and docs.

**Architecture:** keep raw `cargo build` available for ad hoc engineering use, but define `node scripts/release/run-service-release-build.mjs --target <triple>` as the only official service release runner. The runner reuses the governed release build plan, inherits managed `CARGO_TARGET_DIR` and `TEMP/TMP` policy, verifies the full release binary set, and becomes the command referenced by workflow and user docs.

**Tech Stack:** Node.js release tooling, Cargo/Rust, GitHub Actions, VitePress docs.

---

### Task 1: Repair The Managed Service Release Runner

**Files:**
- Modify: `scripts/release/run-service-release-build.mjs`
- Test: `scripts/release/run-service-release-build.test.mjs`

- [ ] **Step 1: Keep the existing failing test as the red gate**

Run:

```bash
node --test scripts/release/run-service-release-build.test.mjs
```

Expected: fail until the ESM-incompatible `require('node:fs')` usage is removed.

- [ ] **Step 2: Implement the minimal ESM-safe fix**

Replace runtime `require('node:fs').readdirSync(...)` usage with a top-level `readdirSync` import from `node:fs`.

- [ ] **Step 3: Re-run the focused runner test**

Run:

```bash
node --test scripts/release/run-service-release-build.test.mjs
```

Expected: PASS.

### Task 2: Align Public Docs With The Official Runner

**Files:**
- Modify: `docs/getting-started/build-and-packaging.md`
- Modify: `docs/reference/build-and-tooling.md`
- Modify: `docs/index.md`
- Modify: `docs/zh/getting-started/build-and-packaging.md`
- Modify: `docs/zh/reference/build-and-tooling.md`
- Modify: `docs/zh/index.md`
- Test: `scripts/release/tests/docs-product-contract.test.mjs`

- [ ] **Step 1: Keep the doc contract tests as the red gate**

Run:

```bash
node --test scripts/release/tests/docs-product-contract.test.mjs
```

Expected: fail until the public docs reference `run-service-release-build.mjs`.

- [ ] **Step 2: Update the official build guidance**

Make the docs say:

- official service release builds use `node scripts/release/run-service-release-build.mjs --target <triple>`
- the managed runner prints the resolved release directory
- Windows official builds use managed short-path target/temp directories automatically
- raw `cargo build --release` remains an engineering path, not the official governed release entrypoint

- [ ] **Step 3: Re-run the doc contract tests**

Run:

```bash
node --test scripts/release/tests/docs-product-contract.test.mjs
```

Expected: PASS.

### Task 3: Verify Workflow And Real Windows Build

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `scripts/release/release-workflow-step-contract-catalog.mjs`
- Modify: `scripts/release-flow-contract.test.mjs`

- [ ] **Step 1: Run the focused workflow and contract suite**

Run:

```bash
node --test scripts/release/run-service-release-build.test.mjs scripts/release-flow-contract.test.mjs scripts/release/tests/docs-product-contract.test.mjs scripts/release/tests/release-workflow.test.mjs
```

Expected: PASS.

- [ ] **Step 2: Run the real managed Windows service release build**

Run:

```bash
node scripts/release/run-service-release-build.mjs --target x86_64-pc-windows-msvc
```

Expected: PASS and print the managed release directory containing the full official binary set.

- [ ] **Step 3: Continue to packaging smoke if the build passes**

Run:

```bash
node scripts/release/package-release-assets.mjs native --platform windows --arch x64 --target x86_64-pc-windows-msvc --output-dir artifacts/release
node scripts/release/run-windows-installed-runtime-smoke.mjs --platform windows --arch x64 --target x86_64-pc-windows-msvc
```

Expected: packaged asset generation and installed-runtime smoke both pass.
