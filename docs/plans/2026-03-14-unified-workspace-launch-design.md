# Unified Workspace Launch Design

**Date:** 2026-03-14

**Status:** Approved by the user's standing instruction to continue autonomously without waiting for interactive checkpoints

## Context

The repository already has:

- a standalone Rust runtime split into `gateway-service`, `admin-api-service`, and `portal-api-service`
- a browser-first React console that can also run inside Tauri
- cross-platform helper scripts for backend-only startup and console-only startup
- operational README guides for Windows, Linux, and macOS

What still feels incomplete is the operator and developer entry point:

- starting the full stack still requires two commands
- the "best" startup path differs between backend and console sections
- browser and Tauri workflows are documented, but not unified under one launch surface
- Windows has native PowerShell helpers, while the cross-platform Node helpers stop short of a full workspace orchestration entry point

## Goal

Add one unified workspace launcher that starts the backend services plus the browser console or Tauri desktop shell through a single command, while keeping the current runtime topology and package boundaries unchanged.

## Approaches

### Option A: Keep separate scripts and improve README only

Pros:

- lowest implementation cost
- no new runtime process orchestration

Cons:

- keeps startup fragmented
- does not solve the "one command" expectation
- still leaves the docs to explain choreography instead of offering one default path

### Option B: Add a thin full-stack orchestrator on top of the existing scripts

Pros:

- minimal architectural risk
- reuses current `start-stack` and `start-console` flows
- provides a single copy-pasteable entry point across Windows, Linux, and macOS
- preserves the existing service-level scripts for advanced or partial startup cases

Cons:

- adds another wrapper script layer
- child process logs still share a terminal when launched from the Node entry point

### Option C: Replace the existing scripts with a heavier process manager

Pros:

- could provide richer supervision and log multiplexing

Cons:

- adds unnecessary tooling and operational complexity
- would be disproportionate to the current repository size and deployment posture
- creates churn in already-working startup flows

## Recommendation

Choose **Option B**.

The missing value is not a new runtime architecture. It is a better orchestration entry point. A thin full-stack launcher keeps the current Axum services, React console, and Tauri host exactly as they are while improving real-world usability on every supported platform.

## Target Design

### New Startup Entry Points

Add:

- `scripts/dev/start-workspace.mjs`
- `scripts/dev/start-workspace.ps1`

Behavior:

- start backend services plus console in one command
- default to browser console mode
- optionally switch to `--tauri`
- optionally switch to `--preview`
- optionally run `--install`
- support `--dry-run`
- pass database and bind overrides through to the backend stack script

### Process Model

The new launcher should not bypass the existing script boundaries.

Instead it should:

1. launch `scripts/dev/start-stack.mjs`
2. launch `scripts/dev/start-console.mjs`
3. forward relevant arguments to each child
4. terminate both child trees when the parent receives `SIGINT` or `SIGTERM`

This keeps the existing backend-only and console-only scripts as the source of truth for their own concerns.

### Configuration Surface

The new launcher should accept:

- `--database-url`
- `--gateway-bind`
- `--admin-bind`
- `--portal-bind`
- `--install`
- `--preview`
- `--tauri`
- `--dry-run`
- `--help`

Windows PowerShell should expose equivalent named parameters and forward them to the Node launcher so that all platforms share the same behavior.

### Documentation Impact

README and README.zh-CN should be updated so the new workspace launcher becomes:

- the default quick-start path
- the first recommendation for browser mode
- the first recommendation for browser + Tauri mode

The existing per-surface scripts should remain documented as lower-level fallback entry points.

## Testing Strategy

Add test-first coverage for the new launcher planning logic.

The easiest stable seam is a pure module that:

- parses workspace-launch arguments
- builds the backend and console child command arrays

Then verify:

- defaults target SQLite plus browser mode
- `--tauri`, `--preview`, `--install`, and bind overrides are forwarded correctly
- `--dry-run` is mirrored to both child processes

After implementation, run:

- script syntax checks
- dry-run verification for the new launcher
- existing Rust and frontend verification commands

## Non-Goals

This batch should not:

- replace the current backend-only or console-only scripts
- add a new external process manager
- redesign the service topology
- change runtime ports or auth behavior
- package OS-native installers
