# Portal Recharge Flow-State Design

**Status:** aligned to the local `sdkwork-api-router` repository on 2026-04-09

**Goal:** add a dynamic, visually explicit flow-state layer to the portal recharge page so users can see the current recharge step immediately without rereading the entire quote panel.

## Executive Decision

The recharge page now has strong purchase copy, good checkout hierarchy, and a clear post-order handoff.

The remaining weakness is state readability.

Right now the page tells the truth, but the current truth is distributed across:

- the selected option
- the quote status line
- the handoff panel
- the pending settlement callout

That means a user still has to mentally assemble the workflow.

This iteration should add one compact dynamic flow-state tracker inside the quote card that answers a single question instantly:

- where am I in the recharge-to-settlement journey right now

## Product Intent

The flow-state tracker should reduce cognitive load at the exact moment the user is deciding or handing off settlement.

It should make three states visually obvious:

- selection is ready or still missing
- order creation is the current next action or already complete
- billing settlement is pending, active, or waiting

This is not a new workflow.

It is a better state lens for the existing workflow.

## Scope

### In scope

- add a compact flow-state tracker inside the recharge quote area
- derive step statuses purely from existing page state
- reflect three stages: selection, order creation, billing settlement
- let the tracker react to:
  - active selection
  - post-order handoff mode
  - existing pending-payment queue
- lock the new contract with page and presentation-helper tests

### Out of scope

- backend changes
- new APIs
- new payment semantics
- changes to order creation behavior
- removing the existing guidance band or settlement callouts

## Interaction Model

The tracker should live near the quote note and CTA because that is where the operator decides what to do next.

It should show three ordered stages:

1. `Choose amount`
2. `Create order`
3. `Complete payment in billing`

Each stage should expose a short detail line and a visual status.

### Status behavior

#### Before selection

- step 1 is current
- step 2 is pending
- step 3 is pending

#### After a valid selection and live quote

- step 1 is complete
- step 2 becomes current
- step 3 remains pending

#### When a new order is created and handoff is active

- step 1 is complete
- step 2 is complete
- step 3 becomes current

#### When the page has pending settlement from earlier orders

- step 3 should surface attention even if the current session is not in handoff mode
- this keeps the tracker honest about settlement debt already in the queue

## Data and Logic Boundaries

This iteration should stay inside existing front-end seams and re-use the current presentation split.

Preferred implementation shape:

- keep state derivation in `src/pages/presentation.ts`
- keep rendering in focused local page components in `src/pages/index.tsx`
- use existing page state:
  - `selection`
  - `quoteSnapshot`
  - `postOrderHandoffActive`
  - `pendingPaymentSpotlight`

No new repository or service calls should be added.

## UI Contract

The page should add:

- `data-slot="portal-recharge-flow-tracker"`

The rendered copy should include:

- `Funding flow`
- `Choose amount`
- `Create order`
- `Complete payment in billing`

The detail copy can evolve if tests are updated intentionally, but the tracker must continue expressing the three-stage journey clearly.

## Visual Direction

The tracker should feel operational and premium, not decorative.

Required characteristics:

- visible current-step emphasis
- compact enough to sit above the quote hero without crowding it
- readable on mobile and desktop
- supportive of the existing right-column hierarchy
- subtle connectors or grouped cards rather than loud steppers

## Acceptance Criteria

The work is complete when:

- the quote area includes a clear dynamic flow-state tracker
- the tracker reflects selection-ready, order-ready, and billing-ready states
- pending settlement influences the payment stage state without changing backend behavior
- existing CTA and handoff behavior stays intact
- tests, typecheck, and build pass
