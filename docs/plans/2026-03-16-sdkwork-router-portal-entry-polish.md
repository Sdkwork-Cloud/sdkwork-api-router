# SDKWork Router Portal Entry Polish

**Date:** 2026-03-16

## Goal

Take `sdkwork-router-portal` from a technically independent frontend to a product-grade user
entry that can stand on its own as the primary self-service surface for end users.

This polish cycle focuses on product quality, not new backend scope. It assumes the current live
portal reads remain the source of truth for auth, dashboard, usage, billing summary, ledger, and
API keys, while commerce stays behind explicit repository seams until the backend catches up.

## Product Problem

The portal already exposes the right core capabilities:

- dashboard
- API key lifecycle
- usage visibility
- credits and coupon redemption seam
- billing and subscription seam
- account management

But the current experience still behaves like a set of good internal pages rather than a fully
finished customer-facing product entry. The remaining gaps are:

- the unauthenticated entry does not feel like a designed launch path
- the authenticated shell does not keep workspace pulse visible across pages
- the dashboard is informative but not yet a real command center for the next user action
- billing and credits expose data, but they can do more to convert uncertainty into confident decisions

## Product Direction

### 1. Portal entry should feel like a launch cockpit

The auth experience should explain:

- what the portal is for
- how quickly a workspace becomes usable
- why the user should trust this boundary with production traffic
- what the first four moves are after sign-up

The visual goal is a product entrance, not a plain login form.

### 2. The shell should keep workspace pulse visible

After sign-in, users should not need to return to the dashboard to remember:

- current project identity
- whether keys exist
- whether traffic has started
- whether remaining points are healthy
- what the recommended next step is

This requires a global pulse layer in the shell, derived from live dashboard and billing reads.

### 3. Dashboard should become the command center

The dashboard should answer:

- what is healthy
- what is blocked
- what should happen next
- how close the workspace is to production readiness

This means adding:

- prioritized action queue
- explicit launch checklist
- stronger guidance on first key, first request, and quota posture

### 4. Billing and credits should reduce buyer uncertainty

The commerce-facing routes should translate posture into confidence:

- estimated runway from current usage
- recommended recharge or plan path
- guardrails around coupon redemption and quota exhaustion
- clear explanation of what happens next

### 5. Super-admin planning must evolve in parallel

As the portal becomes more productized, the super-admin product should explicitly plan for:

- portal growth visibility
- onboarding blockers
- monetization exception handling
- support workflows tied to portal user state

### 6. Operational trust polish should complete the user journey

After the entry and dashboard become product-grade, the remaining user-facing modules must also
feel like first-class operating surfaces:

- API key management should explain environment strategy, rotation order, and secret-handling guardrails
- usage should translate raw telemetry into traffic profile, spend watch, and request diagnostics
- account should behave like a trust center with security checklist and recovery guidance

### 7. Route-to-route handoff must become explicit

Even strong standalone pages still fail as a product if users have to guess the next step after
finishing a task. The portal should therefore make route handoff explicit:

- API keys should hand off into usage validation, dashboard posture review, or quota confirmation
- usage should hand off into credits, billing, or credential review depending on the traffic signal
- credits should hand off into recharge or subscription decisions
- billing should hand off back into activation and launch validation
- account should hand off back into the live workspace command flow instead of becoming a dead-end settings page

### 8. Journey guidance must persist outside the dashboard

Once route handoff exists, the shell itself should keep the launch path visible from anywhere in
the product:

- a persistent launch journey summary should show progress across identity, keys, traffic, and runway
- the shell should keep the current blocker visible without forcing a return to dashboard
- the shell should surface the next milestone and its direct destination route
- the dashboard should restate the same logic in a richer milestone map so page-level detail and shell-level guidance stay aligned

### 9. Guidance must be backed by evidence, not only recommendations

The portal should not only tell users what to do next. It should also explain why the product is
recommending that path, using visible evidence:

- the shell should keep recent activity and latest evidence visible from any route
- the dashboard should expose an evidence timeline for credentials, traffic, and quota posture
- confidence signals should explain whether current launch posture is backed by live data or still depends on pending setup
- recommendation layers should remain readable and auditable instead of feeling arbitrary

### 10. The portal should make the current operating mode obvious

Even with journey and evidence, users still need a simple answer to "what kind of work am I doing
here right now?" The portal should therefore surface an operating mode:

- launch mode when the workspace is still completing first key and first traffic setup
- growth mode when credentials, traffic, and runway are healthy and the focus shifts to optimization
- recovery mode when quota or other hard blockers become the primary concern
- both the shell and dashboard should explain why the current mode is active and what decision path follows from it

### 11. Navigation itself should carry status, not only destinations

Industrial-grade self-service products do not treat navigation as a passive menu. The portal
sidebar should also behave like a status surface:

- route-level signals should show which modules are healthy, live, stable, or still need action
- users should be able to scan the navigation and spot the most relevant route without reading every page
- dashboard should mirror the same route signal map so page-level and shell-level status language stay aligned

### 12. The portal should teach users an operating rhythm

A mature user portal does not merely expose tools. It teaches the operating cadence for using
those tools:

- the shell should explain what to check before traffic, during traffic, and if risk appears
- dashboard should convert the current workspace mode into a review cadence and playbook lane
- the same portal should therefore work for first launch, steady growth, and recovery scenarios without feeling like three different products

### 13. The portal should open each day with a concise operating brief

Even with route signals, journey guidance, and cadence, users still need one compact answer to
"what should I lead with today?" The portal should therefore expose a briefing layer:

- the shell should show a daily brief that names the current operating priority, top focus, and primary risk watch
- dashboard should expand the same logic into a focus board so the next few moves are explicit
- dashboard should also expose a risk watchlist so users can see whether runway, credentials, traffic, or trust are the current constraint
- the briefing must be derived from the existing live dashboard snapshot instead of introducing a second interpretation system

### 14. The shell should behave like a route-agnostic mission layer

Even a strong sidebar still leaves too much context on the left edge of the product. The main
content area should also restate the current mission at the moment a route opens:

- the shell should expose a mission strip at the top of every authenticated route
- the strip should keep primary mission, immediate next move, lead risk, and operating mode visible in one scan
- the strip must reuse the same dashboard-derived semantics as daily brief, journey guidance, and mode framing
- users should therefore be able to enter any route and still understand the current operating posture within seconds

## Delivery Scope

### Portal code

- auth shell product polish
- global workspace pulse shell
- dashboard command-center upgrade
- billing decision-support upgrade
- credits trust and guardrail upgrade
- account trust and boundary reinforcement
- API key environment-strategy upgrade
- usage diagnostics and spend-interpretation upgrade
- account recovery and password-policy guidance
- cross-route action handoff and continuous self-service journey
- persistent shell-level launch journey guidance
- shell-level recent-activity and evidence guidance
- explicit workspace mode framing
- route-level signal density in navigation
- shell-level operating rhythm and dashboard playbook cadence
- shell-level daily brief plus dashboard focus board and risk watchlist
- shell-level mission strip above every authenticated route

### Product docs

- add a dedicated portal polish record
- extend super-admin planning with portal-observability and intervention milestones

## Constraints

- keep `sdkwork-router-portal` as an independent app under `apps/`
- preserve `ARCHITECT.md` ownership direction
- keep admin content out of portal
- keep missing commerce backend capabilities behind explicit seams
- avoid fake platform complexity that is not backed by local data

## Success Criteria

The polish cycle is successful when:

- the portal entrance feels like a deliberate user-facing product
- authenticated users can see workspace pulse from anywhere in the product
- the dashboard behaves like an action-oriented command center
- billing and credits guide the user toward the next decision instead of only restating numbers
- portal and super-admin planning remain clearly differentiated but operationally coordinated
- API key, usage, and account pages no longer feel like utility pages and instead support safe self-service operation
- users can move across portal modules through clear recommended next steps instead of inferring the journey themselves
- users can understand launch progress and the current blocker from any route, not just from the dashboard
- users can see the evidence behind recommendations instead of treating the portal as a black box
- users can immediately understand whether the workspace is in launch, growth, or recovery mode
- users can infer which route needs attention directly from navigation state
- users can understand not only where to go next, but when and why to revisit each module
- users can start each day with an explicit top focus and risk watch instead of inferring priority from scattered surfaces
- users can open any route and immediately see the primary mission, next move, and lead risk without returning to dashboard
