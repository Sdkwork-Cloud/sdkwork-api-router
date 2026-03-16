# SDKWork Router Super-Admin Product Spec

**Date:** 2026-03-16

## Purpose

Define the next product-quality target state for the standalone super-admin control plane so it can
evolve in parallel with `sdkwork-router-portal`.

This spec is intentionally product-facing. It complements the existing engineering split by
describing what the super-admin surface should optimize for once admin and portal are fully
independent applications.

## Product Position

The super-admin product is the operator command center for the SDKWork Router control plane.

It is not a developer workspace. It is not a billing self-service console. It is the surface used
to:

- operate routing and runtime health
- govern tenants, users, and workspace posture
- monitor monetization risk and portal growth
- intervene when platform state or customer state needs manual action

## Primary Users

- platform operations
- SRE and incident response
- customer support escalation teams
- monetization and revenue operations
- solution engineering leadership

## Product Principles

- decision density over decorative simplicity
- health and risk first, drill-down second
- every page should explain current posture and next recommended operator action
- destructive actions must default to reversible or staged flows
- monetization, trust, runtime, and customer health should converge into one control surface

## Information Architecture

### 1. Command Center

Purpose:

- provide a single operating picture across runtime, customers, monetization, and incidents

Key widgets:

- global health rollup
- degraded runtime count
- failed extension or rollout count
- tenants at quota risk
- customers approaching billing or credit exhaustion
- portal funnel blockers
- portal launch-readiness distribution
- portal journey progression by milestone
- portal evidence quality and live-signal coverage
- portal workspace mode distribution
- portal route-level signal distribution
- portal operating-rhythm adherence
- portal daily-brief focus distribution
- portal risk-watch concentration
- portal mission-strip priority distribution
- portal mission-strip next-move stall concentration
- portal coupon and recharge exception queue
- portal key-governance and recovery exception queue
- approvals and task inbox

### 2. Tenants and Workspaces

Purpose:

- inspect tenant topology and intervene safely

Key capabilities:

- tenant search
- workspace ownership graph
- tenant status and monetization posture
- manual quota adjustments
- account suspension and reactivation

### 3. Traffic and Usage

Purpose:

- understand where traffic is flowing and where cost or reliability risk is increasing

Key capabilities:

- request volume trends
- model and provider mix
- routing decisions and anomalies
- cost concentration by tenant or provider
- traffic shifts after rollouts

### 4. Monetization Ops

Purpose:

- operate campaigns, credits, subscriptions, and risk exceptions

Key capabilities:

- coupon campaign lifecycle
- credit grants and reversals
- recharge review queue
- subscription overrides
- renewal and churn-risk watchlists
- portal offer-performance visibility
- suspicious redemption pattern review

### 5. Runtime and Extensions

Purpose:

- run the extension and runtime fleet with confidence

Key capabilities:

- rollout orchestration
- runtime reload controls
- node participation status
- extension trust posture
- health snapshots and failure evidence

### 6. Support Inbox

Purpose:

- make customer interventions first-class, not hidden inside scattered tables

Key capabilities:

- pending approvals
- failed onboarding cases
- low-credit customers with open tickets
- risky plan changes
- escalations requiring operator acknowledgement
- portal launch blocker interventions
- failed checkout or upgrade follow-up
- credential leak and password-recovery escalations

## Core Screens

### Command Center

Above the fold:

- one-line environment posture
- four to six high-signal metrics
- alert stack ranked by operational urgency

Below the fold:

- customer risk panels
- monetization anomaly panels
- rollout and extension posture
- operator task queue
- portal launch funnel and self-service friction

### Tenant Detail

Must answer:

- who owns this tenant
- what traffic and cost posture it has
- whether quota or payment risk exists
- whether support intervention is active

### Monetization Console

Must answer:

- what offers are live
- which tenants are consuming credits unusually
- where recharge or subscription flow is blocked
- what actions can be taken safely now

### Portal Operations Companion

Must answer:

- which portal workspaces are blocked before first launch
- where coupon, recharge, or subscription flows are failing
- which customers need human intervention versus product guidance
- whether portal self-service is scaling cleanly without operator backfill
- which workspaces have risky key posture or repeated recovery failures
- where usage anomalies should trigger support or success outreach
- where customers are getting stuck between portal modules instead of completing the self-service journey
- which milestone in the launch journey most often stalls across the portal population
- how much of portal guidance is backed by live evidence versus pending setup state
- how many workspaces are in launch, growth, or recovery mode at any given time
- which portal routes most often remain in needs-action state across the customer base
- whether portal customers are following healthy review cadence or only reacting after blockers appear
- which daily brief focus dominates across the portal population on a given day
- which risk-watch signals most often precede operator intervention
- which mission-strip priorities dominate route entry across the portal population
- which mission-strip next moves repeatedly fail to convert into user progress

## Visual and Interaction Standards

- use compact layouts with strong table hierarchy and obvious status tone
- reserve vivid color for risk, state changes, and escalation priority
- pair every alert with a recommended action
- default to side-by-side evidence and action layout on desktop
- preserve strong scan-ability on large screens and controlled stacking on mobile

## Cross-Product Relationship With Portal

The portal and super-admin products should feel related but not visually interchangeable.

Rules:

- portal emphasizes trust, onboarding, and self-service growth
- admin emphasizes control, exceptions, and platform consequences
- portal language should describe customer posture
- admin language should describe system posture and operator action
- admin must be able to see portal adoption and blockage without duplicating the portal UI

## Near-Term Delivery Priorities

### Priority A

- command center shell
- tenant monetization risk rollup
- portal health visibility from admin
- portal launch-readiness and onboarding blocker visibility
- task inbox and alert triage

### Priority B

- coupon campaign management
- manual credit actions
- subscription exception handling
- customer health drill-down
- portal commerce exception review
- portal trust-and-recovery intervention tooling

### Priority C

- support workflows
- approval chains
- richer audit evidence
- customer success coordination surfaces

## Success Criteria

The super-admin product is ready when:

- an operator can identify the highest-risk issue within seconds
- monetization and runtime signals can be reviewed from one surface
- customer intervention work is not split across unrelated pages
- portal growth and portal failure states are visible to operators without custom queries
- portal launch blockers can be triaged without asking customers for manual screenshots
