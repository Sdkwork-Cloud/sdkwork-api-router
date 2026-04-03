# SDKWork Router Portal UI Unification Design

**Date:** 2026-04-01

## Goal

Convert `sdkwork-router-portal` into a portal application built on one shared UI framework only:

- shared UI source of truth: `@sdkwork/ui-pc-react`
- no compatibility layer kept for old portal-local UI infrastructure
- no preservation of stale primitives, shell abstractions, or duplicated theme systems
- all route pages and shared components converge onto framework-owned primitives and patterns

The target architecture favors long-term consistency over incremental compatibility.

## Product Direction

The portal should stop behaving like a standalone app with its own private design system.
It should become a normal consumer of the shared SDKWORK PC React UI framework.

That means:

1. `@sdkwork/ui-pc-react` becomes the only UI foundation.
2. Portal-local UI primitives become migration scaffolding only, then are deleted.
3. Route pages are rebuilt against framework patterns instead of carrying forward app-specific layout and visual contracts.
4. Tests stop protecting legacy implementation details and start enforcing framework adoption.

## Non-Goals

- Preserve old portal-local primitives for backwards compatibility.
- Keep the current `portalx-*` visual layer as a parallel design system.
- Maintain old shell composition if it conflicts with framework patterns.
- Introduce new app-local UI abstractions that duplicate framework-owned components.

## Current State

The portal currently ships with:

- a local Tailwind and token layer in `src/theme.css`
- portal-owned UI primitives and helpers in `packages/sdkwork-router-portal-commons`
- portal-owned shell composition in `packages/sdkwork-router-portal-core`
- page packages that already depend on portal-local `Surface`, `DataTable`, `InlineButton`, dialogs, toolbar helpers, and related wrappers
- tests that overfit to current implementation details such as component names and concrete class strings

This architecture already carries duplication:

- theme ownership is split between the portal and the shared UI framework
- common UI primitives exist both in the portal and in `@sdkwork/ui-pc-react`
- shell and workbench compositions are local even though the framework already provides them
- tests protect the old implementation instead of the desired target architecture

## Chosen Architecture

### 1. Shared framework is the only UI source of truth

`@sdkwork/ui-pc-react` becomes the sole owner of:

- theme provider and semantic design tokens
- foundational primitives
- feedback layer
- overlays and forms
- desktop shell patterns
- workbench and workspace patterns

Portal code is allowed to own:

- business data flow
- route-specific view models
- i18n strings
- formatting helpers
- page-specific business widgets that do not belong in the shared framework

Portal code is not allowed to own:

- another generic button/input/dialog/card/table system
- another shell framework
- another general-purpose theme contract
- another workbench abstraction

### 2. Direct cutover instead of compatibility-first migration

The project will not preserve old UI contracts as a permanent adapter layer.

The migration strategy is:

- wire the shared framework in
- replace portal-local UI ownership with framework ownership
- delete superseded modules after each cutover wave

This is intentionally not a soft convergence plan.
The portal will change its internal UI architecture decisively.

### 3. Framework patterns replace portal shell patterns

The framework already provides the correct abstractions for this app class:

- `DesktopShellFrame`
- `NavigationRail`
- `SectionHeader`
- `SettingsCenter`
- `ManagementWorkbench`
- `CrudWorkbench`
- `WorkspacePanel`
- `InspectorRail`
- shared feedback and modal layers

The portal should use these directly instead of preserving:

- `MainLayout` as a custom shell contract
- portal-owned shell sections where a framework pattern already exists
- per-page workbench composition when `ManagementWorkbench` or `CrudWorkbench` already fits

### 4. Portal-local UI package shrinks to a non-UI utility layer

`packages/sdkwork-router-portal-commons` should be reduced to:

- i18n provider and translation helpers
- formatting helpers
- clipboard and non-visual helpers

It should stop owning framework-level UI primitives.

All generic visual exports should be replaced by direct framework re-exports during the cutover, then the dead local implementations should be removed.

## Dependency and Workspace Strategy

The preferred dependency model is to consume `@sdkwork/ui-pc-react` as a local shared package in the workspace graph, not as vendored source and not as copied code.

Target state:

- `sdkwork-router-portal` resolves `@sdkwork/ui-pc-react` from the local repository package
- portal build and typecheck operate against the package directly
- no vendoring of framework source into portal packages

If the current `pnpm-workspace.yaml` blocks that, the workspace definition should be expanded so the portal can depend on the shared package cleanly.

This is the best architecture because it keeps one real package boundary and one real source of truth.

## Theme Strategy

### Target state

- `SdkworkThemeProvider` owns the application theme contract
- portal shell state still chooses color mode and theme flavor
- `src/theme.css` becomes a thin override file only

### Required changes

- remove the portal as the primary author of semantic visual tokens
- map portal state such as `themeMode` and `themeColor` onto framework theme configuration
- stop defining full control, surface, and table styling locally when the framework already defines them

### Deletion standard

The following category should be systematically removed once cutover is complete:

- large `portalx-*` primitive and surface styling blocks that duplicate framework visuals

Only app-specific compatibility shims that remain objectively necessary after full migration may survive, and those should be minimal.

## Shell Architecture

The current shell should be replaced with a framework-owned shell composition.

### Target shell composition

- router and providers remain portal-owned
- desktop application frame becomes `DesktopShellFrame`
- left navigation becomes framework navigation rail composition
- config center becomes framework settings-center composition
- page sections use `SectionHeader` and framework surface/panel patterns

### What gets removed

- portal-local shell composition as the long-term authority
- shell-level repeated card/surface wrappers that exist only because the framework was not yet wired in

### What stays portal-owned

- route manifest and route metadata
- Tauri integration specifics
- shell state and persisted preferences
- business navigation content

## Page Architecture

The portal pages should converge onto framework patterns by role.

### Dashboard

Use framework workspace and section patterns for:

- summary cards
- data tables
- module posture
- next-action workbench

Dashboard should stop depending on portal-local visual wrappers when the framework has equivalent surface and table patterns.

### Gateway

Use `ManagementWorkbench` as the main page composition.

The page contains:

- command posture summary
- filters
- workbench table
- optional detail rail or side panels

This matches the framework workbench model directly and should not keep a local portal-specific workbench surface.

### Routing

Use `ManagementWorkbench` plus framework dialogs and form primitives.

The page should stop owning its own general-purpose workbench grammar and instead focus only on:

- routing-specific fields
- preview logic
- provider ordering logic
- policy and evidence data mapping

### API Keys

Use `CrudWorkbench` with framework dialogs, feedback, and table surfaces.

The page should not keep a separate portal-specific management shell if the framework workbench already models the page correctly.

### Remaining pages

`usage`, `billing`, `credits`, `account`, `user`, and `auth` should each be re-authored against framework primitives and patterns once the foundation, shell, and high-reuse workbench pages are complete.

## Deletion Plan

The project should aggressively delete stale modules instead of leaving dead wrappers in place.

### Delete or collapse after cutover

- portal-local generic primitive implementations in `sdkwork-router-portal-commons`
- portal-local shell ownership that is replaced by framework shell patterns
- duplicated surface helpers such as local `Surface` components when a framework-owned panel or workspace pattern replaces them
- obsolete CSS blocks in `src/theme.css`
- stale tests that lock old implementation details

### Keep only if still justified after cutover

- business-specific widgets with clear portal-only behavior
- i18n and formatting helpers
- Tauri-specific desktop host integration

## Migration Waves

### Wave 1: Foundation replacement

- wire `@sdkwork/ui-pc-react` into the portal as a real dependency
- mount `SdkworkThemeProvider`
- import framework stylesheet
- replace portal primitive ownership in `commons` with framework usage
- update tests to assert framework adoption

**Exit criteria**

- framework dependency is active
- framework stylesheet and theme provider are mounted
- portal no longer owns primary primitive styling

### Wave 2: Shell replacement

- rebuild shell on `DesktopShellFrame`, framework navigation, and settings patterns
- remove local shell ownership that duplicates framework patterns

**Exit criteria**

- all authenticated routes render inside the framework-owned shell
- old shell composition is no longer the architectural source of truth

### Wave 3: High-reuse page replacement

- rewrite `dashboard`
- rewrite `gateway`
- rewrite `routing`
- rewrite `api-keys`

These pages should prove the architecture for:

- dashboard surfaces
- management workbenches
- CRUD workbenches
- filter bars
- dialogs
- feedback

**Exit criteria**

- those four pages use framework-owned patterns as their primary composition model
- app-local duplicated workbench grammar is removed

### Wave 4: Remaining page replacement

- rewrite `usage`
- rewrite `billing`
- rewrite `credits`
- rewrite `account`
- rewrite `user`
- rewrite `auth`

**Exit criteria**

- all route pages are visibly and structurally aligned to the shared framework

### Wave 5: Cleanup and hardening

- delete dead wrappers
- shrink `src/theme.css`
- delete obsolete tests
- replace them with framework adoption and page-pattern tests

**Exit criteria**

- old UI layer is materially gone
- framework is the only maintained UI foundation

## Test Strategy

The current test suite should be rewritten where it protects obsolete structure.

### New test contracts

- portal consumes `@sdkwork/ui-pc-react`
- framework stylesheet is mounted
- `SdkworkThemeProvider` is mounted
- shell uses framework desktop-shell patterns
- dashboard, gateway, routing, and API keys use framework workbench or workspace patterns
- portal-local stale UI modules are absent or reduced to non-visual helpers

### Remove these test smells

- assertions tied to old component names when the new architecture intentionally replaces them
- assertions tied to legacy class strings instead of framework consumption
- assertions that preserve old shell layout ownership

## Risks

### Risk: broad UI churn

This change is intentionally invasive.
The mitigation is not compatibility.
The mitigation is strict wave boundaries and verification after each wave.

### Risk: framework gaps

If a needed pattern is missing from `@sdkwork/ui-pc-react`, the correct response is:

1. add the missing pattern to the framework
2. consume it from the portal

The incorrect response is to reintroduce a portal-local duplicate.

### Risk: dead code accumulation

The migration must include deletion in the same wave that replaces ownership.
Do not leave the old path intact "temporarily" unless the next wave is blocked that same moment.

## Final Architecture Standard

The portal is complete when:

- `@sdkwork/ui-pc-react` is the sole maintained UI foundation
- the portal no longer maintains a parallel primitive system
- the shell is framework-owned in composition
- route pages are framework-first in layout and interaction patterns
- old duplicated CSS and wrappers are deleted
- tests enforce the target architecture rather than the legacy one

## Implementation Principle

Choose the architecture that minimizes future ownership, not the one that preserves the most old code.

For this project, that means:

- direct framework adoption
- deletion of stale UI layers
- no long-lived compatibility layer
- no new app-local design system drift
