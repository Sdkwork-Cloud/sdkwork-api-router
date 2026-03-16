# SDKWork Router Admin Design

## Goal

Build a standalone super-admin application at `apps/sdkwork-router-admin` that serves as the
daily operations control plane for SDKWork Router.

The admin product must:

- be an independent React engineering project
- follow `ARCHITECT.md`
- support a stronger information architecture than the legacy `console` admin page
- cover users, tenants, coupons, channels, proxy providers, models, request logs, and operations
  analytics
- provide safe management workflows for operational entities

## Scope Decomposition

This request spans three separate concerns. They must be treated as one product but implemented as
bounded subsystems:

### 1. Independent frontend engineering project

Create `apps/sdkwork-router-admin` as a standalone pnpm monorepo following `ARCHITECT.md`:

- root `src/` for shell composition only
- `packages/` for reusable foundation and business modules
- package naming `sdkwork-router-admin-*`
- `Vite + React + TypeScript`

### 2. Operations product surface

The product should feel like a real super-admin control plane, not a demo page. It needs:

- a multi-zone shell with global search, alerts, and section navigation
- management workbenches for users, tenants, coupons, catalog, and request analysis
- explicit action framing, empty states, risk labels, and safe destructive actions
- a denser operator-focused UI than the public portal

### 3. Data capability layer

Not every requested management domain exists in the current backend. The product must therefore use
two data strategies:

- **live adapters** for existing admin APIs
- **workspace-scoped mock/seed adapters** for domains that are not yet exposed server-side

This keeps the app shippable now while leaving a clean seam for backend parity work.

## Recommended Product Shape

### Primary navigation

- Overview
- Users
- Tenants
- Coupons
- Catalog
- Traffic
- Operations

### Module responsibilities

#### Overview

- platform posture
- runtime status preview
- request volume and billing summary
- high-priority operational actions

#### Users

- operator accounts
- portal users
- status changes, password reset entry points, workspace linkage
- per-user request count and usage-unit summary

#### Tenants

- tenants and projects
- gateway key inventory
- quota posture and recent activity

#### Coupons

- create, edit, activate, archive, and inspect campaign coupons
- marketing-style metadata: code, type, discount, quota, expiration, notes

#### Catalog

- channels
- proxy providers
- models
- extension linkage and health visibility

#### Traffic

- raw request log list
- usage summary
- billing ledger summary
- routing decision visibility

#### Operations

- runtime health
- extension runtime reloads
- rollout visibility
- configuration rollout visibility

## CRUD Strategy

For production-grade admin systems, hard delete is often unsafe. The design therefore separates:

- **mutable catalog entities**: create and edit through upsert workflows
- **risky identities and campaigns**: safe archive/disable by default
- **logs and billing records**: read-only audit surfaces

This is a better operational standard than blindly exposing destructive delete everywhere.

## Engineering Architecture

### Foundation packages

- `sdkwork-router-admin-types`
- `sdkwork-router-admin-commons`
- `sdkwork-router-admin-core`
- `sdkwork-router-admin-admin-api`

### Business packages

- `sdkwork-router-admin-auth`
- `sdkwork-router-admin-overview`
- `sdkwork-router-admin-users`
- `sdkwork-router-admin-tenants`
- `sdkwork-router-admin-coupons`
- `sdkwork-router-admin-catalog`
- `sdkwork-router-admin-traffic`
- `sdkwork-router-admin-operations`

### App shell responsibilities

Root `src/` should only own:

- bootstrap
- route composition
- shell composition
- app theme
- shell-level providers

Business logic must remain in packages.

## Data Model Plan

### Live data from existing backend

- admin auth
- tenants and projects
- gateway API keys
- channels, providers, credentials, models
- routing summaries
- usage records and billing summary
- extension runtime and rollout views

### New server-side data to expose immediately

- list operator users
- list portal users

### Mock-first data for this phase

- coupon campaigns
- workflow alerts
- task inbox / approvals

Mock-first does not mean throwaway. These modules should sit behind repository interfaces so the
backend can replace them without rewiring the UI.

## Verification Strategy

### Structural tests

- independent app exists at `apps/sdkwork-router-admin`
- required package set exists
- shell route manifest includes the management modules
- app root imports its own theme and does not depend on `console/`

### Build verification

- `pnpm --dir apps/sdkwork-router-admin install`
- `pnpm --dir apps/sdkwork-router-admin typecheck`
- `pnpm --dir apps/sdkwork-router-admin build`

### Runtime verification

- app starts independently from legacy `console`
- management sections render
- live admin login still works

## Chosen Delivery

For this iteration, the best path is:

1. create the independent admin engineering project
2. ship the super-admin shell and management modules
3. wire all currently available live admin data
4. add user management live listing
5. ship coupon management behind a repository seam so backend persistence can land next without
   redesigning the product
