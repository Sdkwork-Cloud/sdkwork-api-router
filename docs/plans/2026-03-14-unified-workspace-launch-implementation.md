# Unified Workspace Launch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add one cross-platform command that launches backend services plus the browser or Tauri console, then make that flow the default README entry point.

**Architecture:** Keep the runtime split exactly as it is today. Add a thin launcher that composes `scripts/dev/start-stack.mjs` and `scripts/dev/start-console.mjs`, with a PowerShell wrapper for Windows convenience and shared docs that point users to the unified entry point first.

**Tech Stack:** Node.js, PowerShell, Cargo, pnpm, Vite, Tauri

---

### Task 1: Add failing tests for launcher planning

**Files:**
- Create: `scripts/dev/tests/start-workspace.test.mjs`
- Create: `scripts/dev/workspace-launch-lib.mjs`

**Step 1: Write the failing test**

Add Node test coverage for:

- default workspace settings
- forwarding of database and bind overrides
- forwarding of `--install`, `--preview`, `--tauri`, and `--dry-run`

The test should import `parseWorkspaceArgs` and `buildWorkspaceCommandPlan` from `scripts/dev/workspace-launch-lib.mjs`.

**Step 2: Run test to verify it fails**

Run:

```powershell
node --test scripts/dev/tests/start-workspace.test.mjs
```

Expected: fail because `scripts/dev/workspace-launch-lib.mjs` does not exist yet

### Task 2: Implement the shared launcher planning module

**Files:**
- Create: `scripts/dev/workspace-launch-lib.mjs`
- Modify: `scripts/dev/tests/start-workspace.test.mjs`

**Step 1: Write minimal implementation**

Implement:

- `parseWorkspaceArgs(argv)`
- `buildWorkspaceCommandPlan(settings)`
- `workspaceHelpText()`

The command plan should return:

- Node executable path
- `start-stack.mjs` argument list
- `start-console.mjs` argument list

**Step 2: Run test to verify it passes**

Run:

```powershell
node --test scripts/dev/tests/start-workspace.test.mjs
```

Expected: PASS

### Task 3: Add the executable workspace launchers

**Files:**
- Create: `scripts/dev/start-workspace.mjs`
- Create: `scripts/dev/start-workspace.ps1`

**Step 1: Implement the Node launcher**

Implement:

- argument parsing through `workspace-launch-lib.mjs`
- child process launch for backend and console
- signal forwarding and shutdown handling
- dry-run and help output

**Step 2: Implement the Windows PowerShell wrapper**

Implement:

- named PowerShell parameters that mirror the Node flags
- forwarding to `node scripts/dev/start-workspace.mjs`

**Step 3: Verify the launcher scripts**

Run:

```powershell
node --check scripts/dev/workspace-launch-lib.mjs
node --check scripts/dev/start-workspace.mjs
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1 -DryRun
node scripts/dev/start-workspace.mjs --dry-run
node scripts/dev/start-workspace.mjs --dry-run --tauri --install
```

Expected: all commands succeed and print both backend and console launch plans

### Task 4: Promote the unified launcher in both READMEs

**Files:**
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Make the unified launcher the first-class quick-start path**

Document:

- Windows command
- Linux/macOS command
- browser mode
- Tauri mode
- backend-only and console-only fallback commands

**Step 2: Keep roadmap status explicit**

Document:

- which portal and gateway capabilities are already live
- which roadmap items remain intentionally out of scope

**Step 3: Review the docs against the scripts**

Run:

```powershell
rg -n "start-workspace|start-stack|start-console|portal/register|tauri" README.md README.zh-CN.md
```

Expected: both READMEs reference the unified launcher and the fallback scripts

### Task 5: Re-run verification

**Files:**
- Review: launcher scripts, tests, and READMEs

**Step 1: Run project verification**

Run:

```powershell
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
node --test scripts/dev/tests/start-workspace.test.mjs
```

Expected: all commands exit `0`
