# SDKWork Router Portal Design

**Date:** 2026-03-16

**Status:** Approved by the user's standing instruction to proceed autonomously without waiting for design checkpoints.

## Goal

Build `apps/sdkwork-router-portal` as a standalone developer self-service product that is fully
independent from `console/`, follows `ARCHITECT.md`, and upgrades the current portal from a simple
auth-and-key page into a product-grade workspace with:

- dashboard overview
- API key lifecycle management
- points and quota visibility
- usage and request history
- coupon redemption
- recharge and subscription surfaces
- guided onboarding and stronger product UX

## Context

The repository already contains:

- an existing portal browser entry inside `console/src/portal`
- reusable portal packages inside `console/packages/sdkwork-api-portal-*`
- a standalone admin app skeleton at `apps/sdkwork-router-admin`
- backend `/portal/*` routes for:
  - register
  - login
  - `me`
  - password change
  - workspace summary
  - API key list/create
- backend `/admin/*` summaries for:
  - usage records
  - usage summary
  - billing ledger
  - billing summary
  - quota policies

The repository does **not** yet contain a production commerce backend for:

- coupon catalogs
- coupon redemption persistence
- recharge orders
- subscription plans and billing provider integration
- per-request token details in the portal boundary

## Approaches

### Option A: Direct lift-and-shift from `console/`

Move current portal files into `apps/sdkwork-router-portal` with minimal structural cleanup.

Pros:

- fastest migration
- low backend impact

Cons:

- preserves current product limitations
- weak module boundaries
- does not produce a real standalone portal product

### Option B: Standalone portal monorepo with mixed live and seam-backed domains

Create `apps/sdkwork-router-portal` as a full pnpm workspace. Reuse proven auth and API key flows,
add new portal read-model endpoints for dashboard and usage, and introduce repository seams for
commerce domains that do not yet have backend support.

Pros:

- follows `ARCHITECT.md`
- produces a real independent engineering project
- upgrades portal UX substantially
- keeps missing backend domains honest through clear seams instead of fake coupling
- allows progressive backend replacement without rewriting the UI

Cons:

- more implementation work than a simple migration
- requires additive portal API work

### Option C: Full backend-first portal commerce platform in one batch

Implement coupon, balance, recharge, subscription, and detailed usage persistence end to end
before the new frontend ships.

Pros:

- most complete long-term vision

Cons:

- too broad for one reliable batch
- introduces payment-domain design decisions without enough supporting infrastructure
- high risk of shipping a large but unstable slice

## Recommendation

Choose **Option B**.

This is the best path for the current repository state. It yields an independent portal product,
lands real dashboard value on top of existing usage and billing data, and creates durable seams for
commerce domains that are not yet available server-side.

## Product Definition

### Primary product promise

`sdkwork-router-portal` is the self-service control center for developers using SDKWork Router.
Unlike the operator-focused admin application, portal optimizes for:

- quick onboarding
- clear credit and quota posture
- safe API key issuance
- self-serve account management
- transparent request history
- easy upgrade and monetization entry points

### Primary navigation

- Dashboard
- API Keys
- Usage
- Credits
- Billing
- Account

### Module responsibilities

#### Dashboard

- welcome state
- workspace identity
- points balance and quota posture
- total requests
- total token units
- recent API requests
- current plan and upgrade CTA
- onboarding checklist

#### API Keys

- list issued keys
- create environment-scoped key
- show plaintext only once
- explain environment usage and rotation posture

#### Usage

- project-scoped usage summary
- recent request list
- model/provider distribution
- per-row token-unit usage
- request count and throughput posture

#### Credits

- available points
- consumed points
- quota ceiling and remaining points
- coupon redemption entry point
- transaction-style read model backed by current billing data where possible

#### Billing

- plan catalog
- current subscription state
- recharge packs
- payment FAQ and trust signals
- repository-backed mock data for commerce until real backend APIs exist

#### Account

- profile summary
- password rotation
- workspace and tenant references
- sign-out

## Engineering Architecture

### Workspace root

Create `apps/sdkwork-router-portal` as an independent pnpm workspace with:

- root `src/` for app composition only
- `packages/` for foundation and business modules
- independent `package.json`, `pnpm-workspace.yaml`, `turbo.json`, `tsconfig.json`, and Vite
  config

### Foundation packages

- `sdkwork-router-portal-types`
- `sdkwork-router-portal-commons`
- `sdkwork-router-portal-core`
- `sdkwork-router-portal-portal-api`

### Business packages

- `sdkwork-router-portal-auth`
- `sdkwork-router-portal-dashboard`
- `sdkwork-router-portal-api-keys`
- `sdkwork-router-portal-usage`
- `sdkwork-router-portal-credits`
- `sdkwork-router-portal-billing`
- `sdkwork-router-portal-account`

### Dependency direction

Follow `ARCHITECT.md`:

- `types` -> `commons` / `core`
- `portal-api` -> `types`
- business packages -> `types`, `commons`, `core`, `portal-api`
- root `src/` -> all packages

## Data Strategy

### Live portal backend data to wire now

- auth session
- current portal user profile
- workspace summary
- API key list/create
- password change
- project-scoped usage records
- project-scoped usage summary
- project-scoped billing summary
- project-scoped ledger entries

### Additive backend work for this batch

Extend `/portal/*` with authenticated project-scoped read endpoints:

- `GET /portal/dashboard`
- `GET /portal/usage/records`
- `GET /portal/usage/summary`
- `GET /portal/billing/summary`
- `GET /portal/billing/ledger`

Also upgrade persisted usage records so the portal can show request-level token-unit details:

- `units`
- `amount`
- `created_at_ms`

This turns usage from a coarse aggregate into a product-ready request history model without
introducing a separate analytics subsystem.

### Commerce seams for this batch

These domains will ship behind repository interfaces with seeded local data:

- subscription plans
- recharge packs
- coupon catalog and redemption preview
- billing FAQs and trust copy

This keeps the portal product complete from a UX perspective while preserving a clean backend seam
for future real payment integration.

## UX and Visual Direction

### Product character

Portal should feel like a polished developer SaaS product, not an internal admin screen:

- lighter visual tone than admin
- premium but practical typography
- strong hierarchy and breathing room
- dense data where needed, but approachable onboarding

### Experience rules

- first screen must explain value in one glance
- every state must tell the user what to do next
- no raw backend dumps
- use explicit empty states and trust hints
- environment, keys, and quota must be legible without training

### Layout direction

- left navigation shell with a sticky workspace summary
- top hero that changes by section
- high-signal metric cards on dashboard
- split panels for summary vs. action flows
- cards with clear action priority and secondary help text

## Error Handling and Security

- keep portal auth fully separate from admin auth
- never expose admin-only routes or copy in portal
- keep API key plaintext write-only
- degrade commerce modules gracefully when only seeded repository data is present
- convert auth expiry into a clean return-to-login flow

## Verification Strategy

### Structural verification

- independent app exists under `apps/sdkwork-router-portal`
- package set matches the portal domain split
- root app imports its own theme and does not depend on `console/`
- route manifest includes dashboard, usage, credits, billing, API keys, and account

### Backend verification

- new portal endpoints are covered by focused route tests
- usage record persistence proves units and timestamps round-trip
- portal endpoints only expose the caller's project scope

### Frontend verification

- Node architecture test for app boundaries
- independent app `typecheck`
- independent app `build`

## Delivery Shape

This batch should ship in this order:

1. freeze architecture with design and plan documents
2. add failing structural tests for the standalone portal app
3. extend the backend portal read model for dashboard and usage
4. scaffold the independent portal workspace and foundation packages
5. migrate and improve auth and API key flows
6. build dashboard, usage, credits, billing, and account modules
7. verify the app builds independently and the new portal endpoints pass tests
