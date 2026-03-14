# Local Config Runtime Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add local JSON and YAML config loading under `~/.sdkwork/router/`, preserve environment overrides, and normalize local defaults for cross-platform standalone service startup.

**Architecture:** Keep `sdkwork-api-config` as the single config boundary. Add local path resolution, config-file discovery, parsed file overlays, and export of final resolved values back into `SDKWORK_*` environment pairs so legacy env-based consumers continue to observe the same config.

**Tech Stack:** Rust, serde, serde_json, serde_yaml, anyhow

---

### Task 1: Add failing config-loading tests

**Files:**
- Modify: `crates/sdkwork-api-config/tests/config_loading.rs`

**Step 1: Write failing tests**

Add tests for:

- default local config paths from a provided home directory
- config file discovery precedence
- YAML loading before env overrides
- JSON loading when YAML is absent
- relative path normalization

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test -p sdkwork-api-config config_loading -q
```

Expected: fail because the new path and file-loading APIs do not exist yet

### Task 2: Implement the local config runtime in `sdkwork-api-config`

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/sdkwork-api-config/Cargo.toml`
- Modify: `crates/sdkwork-api-config/src/lib.rs`

**Step 1: Add parsing and path dependencies**

Add:

- `serde`
- `serde_json`
- `serde_yaml`

**Step 2: Add path and file-loading support**

Implement:

- local config root resolution
- config file search order
- JSON and YAML parsing
- local default path generation
- file overlay merge
- env override merge
- resolved env export pairs

**Step 3: Run tests to verify they pass**

Run:

```powershell
cargo test -p sdkwork-api-config config_loading -q
```

Expected: PASS

### Task 3: Make standalone services export resolved config

**Files:**
- Modify: `services/admin-api-service/src/main.rs`
- Modify: `services/gateway-service/src/main.rs`
- Modify: `services/portal-api-service/src/main.rs`

**Step 1: Export resolved config into process env on startup**

After `StandaloneConfig::from_env()` resolves:

- call a helper that writes the final `SDKWORK_*` values into the process environment

**Step 2: Verify the workspace still compiles and tests**

Run:

```powershell
cargo test --workspace -q -j 1
```

Expected: PASS

### Task 4: Document the config file contract

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`
- Modify: `docs/operations/configuration.md`
- Modify: `docs/zh/operations/configuration.md`

**Step 1: Add documentation**

Document:

- default config root
- file search order
- JSON and YAML examples
- env override precedence
- config directory overrides

**Step 2: Verify the docs references**

Run:

```powershell
rg -n "SDKWORK_CONFIG_DIR|SDKWORK_CONFIG_FILE|~/.sdkwork/router|config.yaml|config.json" README.md README.zh-CN.md docs/operations/configuration.md docs/zh/operations/configuration.md
```

Expected: all files reference the config directory and file contract

### Task 5: Re-run verification

**Files:**
- Review: config crate, services, docs

**Step 1: Run fresh verification**

Run:

```powershell
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir docs typecheck
pnpm --dir docs build
pnpm --dir console -r typecheck
pnpm --dir console build
```

Expected: all commands exit `0`
