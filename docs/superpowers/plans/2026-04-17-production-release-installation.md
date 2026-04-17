# Production Release And Installation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** upgrade `sdkwork-api-router` from a portable-install-centric runtime into a production-grade release and installation flow with config-file-first precedence, PostgreSQL-first server defaults, OS-standard install layouts, and service-manager-based startup.

**Architecture:** preserve the existing portable install flow for local validation and release smoke tests, then add a first-class `system` install mode that generates OS-standard program/config/data/log/run layouts. Reverse the runtime config precedence to `CLI > config file > environment > defaults`, keep config discovery via env, and update service assets and docs so Docker/Helm/native installs all tell the same production story.

**Tech Stack:** Rust (`sdkwork-api-config`, `router-product-service`), Node install tooling, shell and PowerShell runtime scripts, VitePress docs, GitHub Actions and release smoke tests.

---

## Scope Notes

- This plan intentionally keeps the existing portable install path alive. Do not break current `artifacts/install/...` and installed-runtime smoke tests while introducing the `system` layout.
- Treat Windows Service support as additive. Keep `service/windows-task/` available as a compatibility path until the new Windows Service assets verify green.
- The first shippable result must satisfy:
  - config-file-first business settings
  - PostgreSQL-first server/system templates
  - `portable` and `system` install generation
  - systemd, launchd, and Windows Service assets
  - unified production docs and README entrypoints

## File Map

- Modify: `README.md`
  - replace the current config precedence text and add a single production deployment entrypoint.
- Modify: `docs/.vitepress/config.mjs`
  - add new production/install/service pages to English and Chinese navigation and sidebar sections.
- Modify: `docs/getting-started/release-builds.md`
  - narrow this page to build/package concerns and point production users to the new deployment guide.
- Modify: `docs/getting-started/quickstart.md`
  - keep this page local-development-only and remove production deployment ambiguity.
- Modify: `docs/zh/getting-started/release-builds.md`
  - Chinese parity for release-build scoping.
- Modify: `docs/zh/getting-started/quickstart.md`
  - Chinese parity for quickstart scoping.
- Modify: `deploy/README.md`
  - keep this page Docker/Helm-specific and align wording with the production guide.
- Create: `docs/getting-started/production-deployment.md`
  - canonical English production deployment, initialization, and release guide.
- Create: `docs/zh/getting-started/production-deployment.md`
  - Chinese parity for the canonical production guide.
- Create: `docs/operations/install-layout.md`
  - English OS-standard install paths, portable vs system layout, and config file locations.
- Create: `docs/zh/operations/install-layout.md`
  - Chinese parity for install layout.
- Create: `docs/operations/service-management.md`
  - English systemd/launchd/Windows Service operational guide.
- Create: `docs/zh/operations/service-management.md`
  - Chinese parity for service management.
- Modify: `crates/sdkwork-api-config/src/config_support.rs`
  - support `router.yaml` / `router.yml` / `router.json`, preserve legacy discovery, and collect `conf.d/*.yaml` overlays for watch state and loading.
- Modify: `crates/sdkwork-api-config/src/standalone_config.rs`
  - reverse precedence to `defaults -> env -> file overlays`, load primary config plus `conf.d`, and preserve relative-path normalization.
- Modify: `crates/sdkwork-api-config/src/loader.rs`
  - keep watch/reload behavior aligned with the expanded config file stack.
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`
  - red-green coverage for config-file-first precedence, env fallback, standard discovery names, and `conf.d` lexical ordering.
- Modify: `services/router-product-service/src/main.rs`
  - keep CLI highest priority after the config loader changes, and add validation for production/system SQLite misuse.
- Modify: `bin/router-ops.mjs`
  - add `install --mode portable|system` and `validate-config` entrypoints.
- Modify: `bin/install.sh`
  - pass through install mode and new system-layout flags.
- Modify: `bin/install.ps1`
  - pass through install mode and new system-layout flags.
- Modify: `bin/start.sh`
  - honor generated system layout metadata and stop hard-defaulting server installs to SQLite.
- Modify: `bin/start.ps1`
  - Windows parity for system layout and PostgreSQL-first server defaults.
- Modify: `bin/lib/runtime-common.sh`
  - shared path helpers needed by system install mode.
- Modify: `bin/lib/runtime-common.ps1`
  - PowerShell parity for new path/layout helpers.
- Modify: `bin/lib/router-runtime-tooling.mjs`
  - generate portable and system install plans, render config templates, and render service assets for both modes.
- Modify: `bin/tests/router-runtime-tooling.test.mjs`
  - coverage for install modes, config templates, and service asset generation.
- Modify: `scripts/release/runtime-tooling-contracts.mjs`
  - assert the new install-plan exports and service asset names.
- Modify: `scripts/release/run-unix-installed-runtime-smoke.mjs`
  - keep portable smoke green and add a system-layout validation lane if practical.
- Modify: `scripts/release/run-windows-installed-runtime-smoke.mjs`
  - keep portable smoke green and cover Windows service asset generation.
- Modify: `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`
  - regression coverage for updated install plan expectations.
- Modify: `scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`
  - regression coverage for Windows service asset expectations.
- Modify: `scripts/release/tests/deployment-assets.test.mjs`
  - keep PostgreSQL-first deployment assets aligned with new docs and install templates.

## Task 1: Lock Config Precedence And Standard Config Discovery

**Files:**
- Modify: `crates/sdkwork-api-config/src/config_support.rs`
- Modify: `crates/sdkwork-api-config/src/standalone_config.rs`
- Modify: `crates/sdkwork-api-config/src/loader.rs`
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`
- Modify: `services/router-product-service/src/main.rs`

- [ ] **Step 1: Write the failing config precedence tests**

Add focused tests in `crates/sdkwork-api-config/tests/config_loading.rs` for:

```rust
#[test]
fn config_file_wins_over_environment_for_database_and_profile() { /* ... */ }

#[test]
fn environment_still_fills_missing_fields_when_router_yaml_omits_them() { /* ... */ }

#[test]
fn loads_router_yaml_before_legacy_config_yaml() { /* ... */ }

#[test]
fn loads_conf_d_fragments_in_lexical_order() { /* ... */ }
```

- [ ] **Step 2: Add the failing service-level CLI precedence test**

Extend `services/router-product-service/src/main.rs` tests so a file-defined `database_url` is overridden by an explicit `--database-url` CLI input after the config loader change.

- [ ] **Step 3: Run the targeted Rust suites to verify RED**

Run:

```bash
cargo test -p sdkwork-api-config --test config_loading
cargo test -p router-product-service
```

Expected:

- `sdkwork-api-config` fails because the loader still lets env overwrite file values and does not load `router.yaml` + `conf.d`
- `router-product-service` fails once the new CLI-precedence assertion is added

- [ ] **Step 4: Implement config discovery and precedence changes**

Make these concrete changes:

- support primary config discovery in this order:
  - `router.yaml`
  - `router.yml`
  - `router.json`
  - legacy `config.yaml`
  - legacy `config.yml`
  - legacy `config.json`
- keep `SDKWORK_CONFIG_FILE` and `SDKWORK_CONFIG_DIR` as discovery inputs
- change load order to:
  - defaults
  - env fallbacks
  - primary config file
  - `conf.d/*.yaml` overlays in lexical order
- include discovered `conf.d` files in loader watch state

- [ ] **Step 5: Restore CLI highest precedence in `router-product-service`**

After the loader returns config-file-first values, apply explicit CLI overrides on top of the loaded config instead of routing CLI through env override behavior.

- [ ] **Step 6: Re-run the targeted Rust suites to verify GREEN**

Run:

```bash
cargo test -p sdkwork-api-config --test config_loading
cargo test -p router-product-service
```

Expected: PASS.

- [ ] **Step 7: Commit the precedence slice**

```bash
git add crates/sdkwork-api-config/src/config_support.rs crates/sdkwork-api-config/src/standalone_config.rs crates/sdkwork-api-config/src/loader.rs crates/sdkwork-api-config/tests/config_loading.rs services/router-product-service/src/main.rs
git commit -m "feat: prefer config files over env for server config"
```

## Task 2: Add Portable And System Install Layouts

**Files:**
- Modify: `bin/router-ops.mjs`
- Modify: `bin/install.sh`
- Modify: `bin/install.ps1`
- Modify: `bin/lib/router-runtime-tooling.mjs`
- Modify: `bin/tests/router-runtime-tooling.test.mjs`
- Modify: `scripts/release/runtime-tooling-contracts.mjs`

- [ ] **Step 1: Write the failing install-plan tests first**

Extend `bin/tests/router-runtime-tooling.test.mjs` with new expectations for:

- `createInstallPlan({ mode: 'portable' })` still targets `artifacts/install/sdkwork-api-router/current`
- `createInstallPlan({ mode: 'system', platform: 'linux' })` emits:
  - `/opt/sdkwork-api-router/current`
  - `/etc/sdkwork-api-router/router.yaml`
  - `/etc/sdkwork-api-router/conf.d/`
  - `/etc/sdkwork-api-router/router.env`
  - `/var/lib/sdkwork-api-router`
  - `/var/log/sdkwork-api-router`
  - `/run/sdkwork-api-router`
- `renderRuntimeEnvTemplate()` for `system` contains PostgreSQL placeholders and config-discovery values instead of a SQLite path

- [ ] **Step 2: Add the failing CLI parsing tests**

Add coverage in `bin/tests/router-runtime-tooling.test.mjs` or a small new test block around `bin/router-ops.mjs` for:

```js
parseArgs(['install', '--mode', 'system'])
parseArgs(['install', '--mode', 'portable', '--home', 'D:/custom/router'])
```

- [ ] **Step 3: Run the targeted Node suite to verify RED**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
```

Expected: FAIL because only the portable layout exists today.

- [ ] **Step 4: Implement install mode parsing and layout generation**

Add `install --mode portable|system` with these concrete rules:

- default mode remains `portable` for repo-local ergonomics
- `portable` keeps the current `artifacts/install/...` behavior
- `system` chooses OS-specific defaults:
  - Linux: `/opt`, `/etc`, `/var/lib`, `/var/log`, `/run`
  - macOS: `/usr/local/lib`, `/Library/Application Support`, `/Library/Logs`, `/Library/LaunchDaemons`
  - Windows: `Program Files` + `ProgramData`
- generate:
  - `router.yaml`
  - `conf.d/`
  - `router.env`
  - `router.env.example`

- [ ] **Step 5: Make the generated system templates PostgreSQL-first**

Use a placeholder like:

```env
SDKWORK_CONFIG_FILE="/etc/sdkwork-api-router/router.yaml"
SDKWORK_DATABASE_URL="postgresql://sdkwork:change-me@127.0.0.1:5432/sdkwork_api_router"
```

and ensure the actual YAML template is the canonical config source.

- [ ] **Step 6: Re-run the runtime-tooling and contract suites to verify GREEN**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/runtime-tooling-contracts.mjs
```

Expected: PASS.

- [ ] **Step 7: Commit the install-layout slice**

```bash
git add bin/router-ops.mjs bin/install.sh bin/install.ps1 bin/lib/router-runtime-tooling.mjs bin/tests/router-runtime-tooling.test.mjs scripts/release/runtime-tooling-contracts.mjs
git commit -m "feat: add system install layout generation"
```

## Task 3: Service Assets, Validate-Config, And Production Safety Gates

**Files:**
- Modify: `services/router-product-service/src/main.rs`
- Modify: `bin/start.sh`
- Modify: `bin/start.ps1`
- Modify: `bin/lib/runtime-common.sh`
- Modify: `bin/lib/runtime-common.ps1`
- Modify: `bin/lib/router-runtime-tooling.mjs`
- Modify: `bin/tests/router-runtime-tooling.test.mjs`
- Modify: `scripts/release/run-unix-installed-runtime-smoke.mjs`
- Modify: `scripts/release/run-windows-installed-runtime-smoke.mjs`
- Modify: `scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs`
- Modify: `scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs`

- [ ] **Step 1: Write failing tests for the new service assets**

Extend `bin/tests/router-runtime-tooling.test.mjs` so `system` install mode expects:

- `service/systemd/sdkwork-api-router.service`
- `service/launchd/com.sdkwork.api-router.plist`
- `service/windows-service/` assets

For Windows, assert the new service assets replace or sit alongside `windows-task` without regressing existing portable expectations.

- [ ] **Step 2: Write the failing production-safety tests**

Add Rust tests in `services/router-product-service/src/main.rs` for:

- production/system posture rejects SQLite unless an explicit development override is active
- `validate-config` exits successfully for PostgreSQL placeholders and valid config discovery

- [ ] **Step 3: Run the targeted suites to verify RED**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
cargo test -p router-product-service
```

Expected: FAIL because the new service assets and production SQLite guard do not exist yet.

- [ ] **Step 4: Implement the system service assets**

Concretely:

- keep `systemd` and `launchd`, but point system-install assets at OS-standard config locations
- introduce `service/windows-service/` as the formal Windows service path
- keep `service/windows-task/` as compatibility-only until all tests are green
- prefer foreground runtime execution in service definitions

If you choose WinSW for Windows Service management, keep the wrapper contract explicit in generated files and tests.

- [ ] **Step 5: Implement `validate-config`**

Add a managed validation command, exposed through `bin/router-ops.mjs`, that:

- resolves config discovery
- runs the product-service dry-run path
- validates security posture
- rejects production/system SQLite usage

Add wrappers if needed so operators can run:

```bash
node bin/router-ops.mjs validate-config
```

- [ ] **Step 6: Update startup scripts for the new layout contract**

Make `bin/start.sh` and `bin/start.ps1` honor generated config files and system roots without silently defaulting server installs back to SQLite.

- [ ] **Step 7: Re-run the targeted suites to verify GREEN**

Run:

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs
cargo test -p router-product-service
```

Expected: PASS.

- [ ] **Step 8: Commit the service and validation slice**

```bash
git add services/router-product-service/src/main.rs bin/start.sh bin/start.ps1 bin/lib/runtime-common.sh bin/lib/runtime-common.ps1 bin/lib/router-runtime-tooling.mjs bin/tests/router-runtime-tooling.test.mjs scripts/release/run-unix-installed-runtime-smoke.mjs scripts/release/run-windows-installed-runtime-smoke.mjs scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs
git commit -m "feat: add system service assets and config validation"
```

## Task 4: Rebuild README And Operations Docs Around One Production Story

**Files:**
- Modify: `README.md`
- Modify: `docs/.vitepress/config.mjs`
- Modify: `docs/getting-started/release-builds.md`
- Modify: `docs/getting-started/quickstart.md`
- Modify: `docs/zh/getting-started/release-builds.md`
- Modify: `docs/zh/getting-started/quickstart.md`
- Modify: `deploy/README.md`
- Create: `docs/getting-started/production-deployment.md`
- Create: `docs/zh/getting-started/production-deployment.md`
- Create: `docs/operations/install-layout.md`
- Create: `docs/zh/operations/install-layout.md`
- Create: `docs/operations/service-management.md`
- Create: `docs/zh/operations/service-management.md`

- [ ] **Step 1: Write the failing docs expectations**

Add or extend docs-safety assertions so the docs set contains:

- one canonical production deployment page
- updated README precedence text matching `config file > env`
- explicit PostgreSQL-first server wording

If the current docs-safety test surface is too narrow, add direct assertions to `scripts/check-router-docs-safety.test.mjs`.

- [ ] **Step 2: Run docs-safety to verify RED**

Run:

```bash
node --test scripts/check-router-docs-safety.test.mjs
```

Expected: FAIL once the new assertions are added.

- [ ] **Step 3: Write the new production and operations pages**

Cover these concrete sections:

- production release flow
- Docker Compose quick deployment
- Helm deployment
- native `system` install per OS
- default install/config/data/log/run directories
- config precedence and config file names
- service registration and lifecycle
- initialization checklist
- `validate-config` usage

- [ ] **Step 4: Update README and existing docs to point at the new canonical pages**

Make these concrete wording changes:

- README:
  - update config precedence to `built-in defaults -> environment fallback -> config file -> CLI`
  - add a single production deployment entrypoint
- `quickstart`:
  - local dev only
- `release-builds`:
  - build/package only
- `deploy/README.md`:
  - Docker/Helm asset-specific only

- [ ] **Step 5: Update VitePress navigation**

Add the new pages to both English and Chinese sidebars in `docs/.vitepress/config.mjs`.

- [ ] **Step 6: Re-run docs verification to verify GREEN**

Run:

```bash
node --test scripts/check-router-docs-safety.test.mjs
pnpm --dir docs build
```

Expected: PASS.

- [ ] **Step 7: Commit the docs slice**

```bash
git add README.md docs/.vitepress/config.mjs docs/getting-started/release-builds.md docs/getting-started/quickstart.md docs/zh/getting-started/release-builds.md docs/zh/getting-started/quickstart.md deploy/README.md docs/getting-started/production-deployment.md docs/zh/getting-started/production-deployment.md docs/operations/install-layout.md docs/zh/operations/install-layout.md docs/operations/service-management.md docs/zh/operations/service-management.md
git commit -m "docs: publish production deployment and install guides"
```

## Task 5: Run End-To-End Regression Verification

**Files:**
- Modify: none unless verification exposes a real defect

- [ ] **Step 1: Run the focused Rust verification**

```bash
cargo test -p sdkwork-api-config --test config_loading
cargo test -p router-product-service
```

Expected: PASS.

- [ ] **Step 2: Run the focused Node/runtime-tooling verification**

```bash
node --test bin/tests/router-runtime-tooling.test.mjs
node --test scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs
```

Expected: PASS.

- [ ] **Step 3: Run docs verification**

```bash
node --test scripts/check-router-docs-safety.test.mjs
pnpm --dir docs build
```

Expected: PASS.

- [ ] **Step 4: Inspect the final diff**

```bash
git diff -- README.md docs/.vitepress/config.mjs docs/getting-started/release-builds.md docs/getting-started/quickstart.md docs/zh/getting-started/release-builds.md docs/zh/getting-started/quickstart.md deploy/README.md docs/getting-started/production-deployment.md docs/zh/getting-started/production-deployment.md docs/operations/install-layout.md docs/zh/operations/install-layout.md docs/operations/service-management.md docs/zh/operations/service-management.md crates/sdkwork-api-config/src/config_support.rs crates/sdkwork-api-config/src/standalone_config.rs crates/sdkwork-api-config/src/loader.rs crates/sdkwork-api-config/tests/config_loading.rs services/router-product-service/src/main.rs bin/router-ops.mjs bin/install.sh bin/install.ps1 bin/start.sh bin/start.ps1 bin/lib/runtime-common.sh bin/lib/runtime-common.ps1 bin/lib/router-runtime-tooling.mjs bin/tests/router-runtime-tooling.test.mjs scripts/release/runtime-tooling-contracts.mjs scripts/release/run-unix-installed-runtime-smoke.mjs scripts/release/run-windows-installed-runtime-smoke.mjs scripts/release/tests/run-unix-installed-runtime-smoke.test.mjs scripts/release/tests/run-windows-installed-runtime-smoke.test.mjs scripts/release/tests/deployment-assets.test.mjs docs/superpowers/specs/2026-04-17-production-release-installation-design.md docs/superpowers/plans/2026-04-17-production-release-installation.md
```

Expected: only the production release/install standardization changes appear.

- [ ] **Step 5: Commit the verification snapshot**

```bash
git status --short
```

Expected: clean working tree after the prior commits, or only intentional last-minute verification fixes.
