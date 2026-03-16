# SDKWork Router Control Plane Product Roadmap

**Date:** 2026-03-16

## Goal

Define the product system for the two independent frontend control surfaces in SDKWork Router:

- `sdkwork-router-admin`
- `sdkwork-router-portal`

The goal is not just technical separation. The goal is a coherent product architecture where each
surface has:

- a clear audience
- strong information architecture
- distinct interaction patterns
- durable backend seams
- credible expansion paths for enterprise use

## Product System

### 1. Super-admin control plane

**Audience:**

- platform operators
- internal SRE and support teams
- solution engineers
- operations leadership

**Job to be done:**

- operate the routing mesh
- manage tenants and projects
- audit traffic and billing posture
- supervise extensions and runtime health
- run promotions and support workflows safely

**Product posture:**

- dense
- operational
- high-signal
- action-first
- risk-aware

### 2. Developer self-service portal

**Audience:**

- external developers
- customer engineering teams
- internal product teams consuming the gateway

**Job to be done:**

- onboard a workspace
- inspect quota and points posture
- create API keys
- understand request usage
- recharge or upgrade before production events

**Product posture:**

- guided
- polished
- trust-building
- commercially legible
- simpler than admin without being shallow

## Portfolio Rules

- admin and portal remain independent engineering projects
- admin and portal never share a root application shell
- auth boundaries stay separate
- design systems can share principles, but not collapse into the same visual product
- live backend capabilities should be used where available
- missing business domains must sit behind explicit repository seams, not hidden mock logic

## Information Architecture Standards

### Admin IA

- `Overview`
- `Users`
- `Tenants`
- `Coupons`
- `Catalog`
- `Traffic`
- `Operations`

### Portal IA

- `Dashboard`
- `API Keys`
- `Usage`
- `Credits`
- `Billing`
- `Account`

## UX Standards

### Shared standards

- first paint must answer "where am I and what can I do next"
- empty states must be instructional, not apologetic
- status and trust messaging must be explicit
- route-level pages must have a hero, summary, and actionable surface
- data tables must prioritize scan-ability over ornamental UI

### Admin standards

- operator pages should emphasize consequences and operational posture
- destructive actions should default to safe archive or disable semantics
- logs and billing should be audit-first, not marketing-first
- routing and runtime states should foreground system health
- admin dashboards should consolidate alerts, tenant monetization risk, and system health in a single command posture
- support and campaign operations should be visible as workflows, not scattered tables

### Portal standards

- commercial value should be understandable without a salesperson
- quota, points, and token usage must be visible together
- API key lifecycle must feel safe and intentional
- billing surfaces should increase confidence, not anxiety
- recommendation logic should turn raw telemetry into next-step guidance
- onboarding, first request, and first recharge should all feel like designed journeys
- each portal session should open with a clear daily brief, explicit focus board, and live risk watchlist
- every authenticated portal route should restate the current mission through a shared shell-level mission strip

## Product Maturity Roadmap

### Phase 1: Independent surfaces

- complete admin and portal project extraction from `console/`
- lock package architecture by domain
- ensure each project builds independently

### Phase 2: Product-grade read models

- admin: expand users, runtime, and audit visibility
- portal: expand dashboard, usage, billing, and request history
- unify product language around quota, token units, and billing posture
- add product-owned recommendation layers on top of read models so users can act without interpretation overhead
- expose portal launch-readiness, onboarding friction, and commerce exceptions inside admin read models

### Phase 3: Commerce and growth

- real coupon persistence
- redemption history
- recharge orders
- subscription lifecycle
- entitlement changes reflected in both portal and admin
- admin-side exception tooling for failed portal redemption, recharge, and upgrade journeys

### Phase 4: Enterprise operations

- support workflows
- approvals and task inbox
- customer success intervention tooling
- multi-workspace memberships
- role-based policy surfaces

## Immediate Next Backlog

### Portal

- add real coupon redemption persistence behind `/portal/*`
- add subscription and recharge backend contracts
- add date ranges in usage workbench
- add API key rotation and revocation flows
- add project-level request export and spend forecasting
- keep the global workspace pulse current across all portal routes
- deepen dashboard command-center guidance with launch readiness history
- deepen dashboard guidance into a daily brief, focus board, and risk watchlist derived from live portal evidence
- add a shell-level mission strip so route entry remains action-first even outside dashboard
- deepen API key posture into full rotation and revocation governance
- extend usage diagnostics toward anomaly timelines and budget watchlists
- extend account trust center into session, recovery, and secret-hygiene guidance

### Admin

- expand managed user list endpoints and support actions
- connect coupon workbench to persisted campaign storage
- add productized task inbox and alert triage
- add tenant-level monetization and portal health views
- add a unified super-admin command center for incidents, revenue risk, and customer health
- add portal onboarding blocker, failed checkout, and launch-readiness intervention views
- add portal key-governance, recovery-escalation, and usage-anomaly intervention views
- add operator visibility into portal daily-brief focus concentration and risk-watch escalations
- add operator visibility into portal mission-strip priority concentration and repeated next-move stalls

## Target Product Workstreams

### 1. Super-admin command center

- cross-tenant health rollup
- incident and alert inbox
- monetization exceptions
- customer intervention queue
- detailed product spec: `docs/plans/2026-03-16-sdkwork-router-super-admin-product-spec.md`

### 2. Monetization operations

- coupon campaign lifecycle
- tenant credit adjustments
- subscription overrides and approvals
- portal growth diagnostics

### 3. Developer portal growth loop

- onboarding to first key
- first request to usage insight
- low-quota to recharge recommendation
- plan-change to entitlement confirmation

### 4. Portal operations and intervention loop

- portal workspace pulse aggregation for operators
- portal launch blocker triage
- portal commerce exception handling
- portal support handoff with workspace context
- portal daily-brief focus concentration and risk-watch pattern visibility
- portal mission-strip next-move stall visibility

## Success Criteria

The control plane suite is in a good state when:

- admin feels like operator software
- portal feels like a customer-facing SaaS product
- product boundaries map cleanly to code boundaries
- backend seams are honest
- roadmap expansion does not require re-architecting either frontend
