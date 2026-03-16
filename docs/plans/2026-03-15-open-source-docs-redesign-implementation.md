# Open Source Documentation Redesign Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the `docs/` site into a professional open source documentation surface with stronger getting-started, architecture, API reference, build, and module documentation in both English and Chinese.

**Architecture:** Keep VitePress as the single documentation runtime, preserve existing deep references, and add a curated top-level information architecture modeled after OpenAI's official docs structure. New pages should be grounded in real workspace services, scripts, crates, and route registration.

**Tech Stack:** VitePress, TypeScript, Markdown, Rust workspace metadata

---

## Chunk 1: Planning and navigation

### Task 1: Save this plan and align the target IA

**Files:**
- Review: `docs/.vitepress/config.ts`
- Review: `docs/index.md`
- Review: `docs/zh/index.md`
- Review: `docs/plans/2026-03-15-open-source-docs-redesign-design.md`

- [ ] **Step 1: Confirm current docs runtime and existing sections**

Run: `sed -n '1,320p' docs/.vitepress/config.ts`

Expected: existing nav has `Getting Started`, `Operations`, and `Reference`, with no first-class `Architecture` or `API Reference`

- [ ] **Step 2: Map the new section model**

Target sections:

- `Getting Started`
- `Architecture`
- `API Reference`
- `Operations`
- `Reference`

- [ ] **Step 3: Keep old deep links as supporting reference**

Retain:

- `docs/api/compatibility-matrix.md`
- `docs/architecture/runtime-modes.md`
- `docs/reference/api-compatibility.md`
- `docs/reference/repository-layout.md`

## Chunk 2: English docs redesign

### Task 2: Rework the English nav, sidebar, and homepage

**Files:**
- Modify: `docs/.vitepress/config.ts`
- Modify: `docs/index.md`

- [ ] **Step 1: Update the English navigation**

Add top-level entries for:

- `Getting Started`
- `Architecture`
- `API Reference`
- `Operations`
- `Reference`

- [ ] **Step 2: Expand the English sidebar**

Ensure the sidebar includes:

- build and packaging
- software architecture
- functional modules
- gateway, admin, and portal API reference
- build and tooling reference

- [ ] **Step 3: Rewrite the homepage**

Cover:

- product positioning
- primary runtime surfaces
- fast path links
- architecture and API reference links
- operational entry points

### Task 3: Add or rewrite English getting-started pages

**Files:**
- Modify: `docs/getting-started/installation.md`
- Modify: `docs/getting-started/source-development.md`
- Create: `docs/getting-started/build-and-packaging.md`
- Modify: `docs/getting-started/release-builds.md`
- Modify: `docs/getting-started/runtime-modes.md`
- Modify: `docs/getting-started/public-portal.md`

- [ ] **Step 1: Tighten installation around prerequisites and repo bootstrapping**
- [ ] **Step 2: Clarify source development workflows and verification commands**
- [ ] **Step 3: Add a dedicated build-and-packaging page**

Cover:

- service compilation
- console build
- docs build
- Tauri packaging
- recommended compile commands per target

- [ ] **Step 4: Reframe release builds to focus on deployment artifacts**
- [ ] **Step 5: Keep runtime-modes and portal pages aligned with current implementation**

### Task 4: Add English architecture pages

**Files:**
- Create: `docs/architecture/software-architecture.md`
- Create: `docs/architecture/functional-modules.md`
- Modify: `docs/architecture/runtime-modes.md`

- [ ] **Step 1: Add software architecture**

Cover:

- standalone services
- console surfaces
- interface/app/domain/storage layering
- extension runtime and provider flow
- config and secret boundaries

- [ ] **Step 2: Add functional modules**

Cover:

- gateway
- admin control plane
- portal
- routing
- billing and usage
- extension host
- storage
- console
- docs and developer scripts

- [ ] **Step 3: Tighten runtime-modes as a deep-dive architecture page**

### Task 5: Add English API reference pages

**Files:**
- Create: `docs/api-reference/overview.md`
- Create: `docs/api-reference/gateway-api.md`
- Create: `docs/api-reference/admin-api.md`
- Create: `docs/api-reference/portal-api.md`
- Modify: `docs/reference/api-compatibility.md`

- [ ] **Step 1: Add API reference overview**

Cover:

- service base URLs
- auth boundaries
- when to use gateway vs admin vs portal
- cross-links to compatibility truth

- [ ] **Step 2: Add gateway API reference**

Group routes by family:

- models
- chat and responses
- completions and embeddings
- moderations, images, audio
- files, uploads, containers
- assistants, threads, conversations
- vector stores, batches, fine-tuning, webhooks, evals, videos, realtime

- [ ] **Step 3: Add admin API reference**

Group routes by family:

- auth
- tenancy and API keys
- channels, providers, credentials, models
- extensions and rollouts
- usage, billing, routing

- [ ] **Step 4: Add portal API reference**

Group routes by family:

- auth
- workspace
- self-service API keys

- [ ] **Step 5: Reframe API compatibility as supporting reference**

### Task 6: Add English supporting reference pages

**Files:**
- Create: `docs/reference/build-and-tooling.md`
- Modify: `docs/reference/repository-layout.md`

- [ ] **Step 1: Add build and tooling reference**

Cover:

- required toolchains
- docs runtime
- console runtime
- key scripts
- recommended verification commands

- [ ] **Step 2: Upgrade repository layout with clearer module ownership**

## Chunk 3: Chinese docs parity

### Task 7: Mirror the primary docs structure in Chinese

**Files:**
- Modify: `docs/zh/index.md`
- Create: `docs/zh/getting-started/build-and-packaging.md`
- Create: `docs/zh/architecture/software-architecture.md`
- Create: `docs/zh/architecture/functional-modules.md`
- Create: `docs/zh/api-reference/overview.md`
- Create: `docs/zh/api-reference/gateway-api.md`
- Create: `docs/zh/api-reference/admin-api.md`
- Create: `docs/zh/api-reference/portal-api.md`
- Create: `docs/zh/reference/build-and-tooling.md`
- Modify: existing mirrored `docs/zh/*` pages where needed
- Modify: `docs/.vitepress/config.ts`

- [ ] **Step 1: Add Chinese nav and sidebar parity**
- [ ] **Step 2: Mirror the homepage messaging**
- [ ] **Step 3: Mirror getting-started, architecture, API reference, and reference pages**
- [ ] **Step 4: Keep terminology consistent across gateway, admin, portal, runtime, extension, and billing sections**

## Chunk 4: Verification and polish

### Task 8: Validate the docs runtime

**Files:**
- Review: all modified docs files

- [ ] **Step 1: Run docs typecheck**

Run: `pnpm --dir docs typecheck`

Expected: exit `0`

- [ ] **Step 2: Run docs build**

Run: `pnpm --dir docs build`

Expected: exit `0`

- [ ] **Step 3: Check the new files for whitespace issues**

Run: `git diff --check -- docs/.vitepress/config.ts docs/index.md docs/getting-started docs/architecture docs/api-reference docs/reference docs/zh`

Expected: exit `0`

- [ ] **Step 4: Summarize the new documentation entry points**

Report:

- new homepage structure
- new architecture pages
- new API reference pages
- new build-and-packaging coverage
