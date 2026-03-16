# Admin Portal Product Differentiation Design

## Goal

Keep `admin` and `portal` as technically independent browser applications while making them
feel like different products:

- `admin` is an operator control plane for platform owners
- `portal` is a self-service developer product for workspace users

The separation must be visible in entry points, layout, navigation, session boundaries, visual
language, and product copy.

## Current Gaps

- The apps already have separate URLs and auth domains, but they still share one stylesheet entry.
- Both experiences use the same visual primitives, spacing rhythm, and card language.
- The current `admin` screen is a linear dump of backend pages rather than a clear operator
  command center.
- The current `portal` screen exposes the right features, but it does not feel like a guided
  developer product.

## Chosen Design

### 1. Separate design systems

Replace the shared `console/src/App.css` app theme with dedicated style entry points:

- `console/src/admin/admin.css`
- `console/src/portal/portal.css`
- `console/src/landing.css`

Each app gets its own typography, color tokens, surface treatment, spacing, motion, and shell.

### 2. Reshape admin into an operator command center

The admin app should emphasize:

- operational status at a glance
- dense control-plane summaries
- sectioned navigation for registry, traffic, routing, and runtime work
- a more technical visual language with stronger telemetry and operations framing

Admin will use a dashboard shell with:

- a left rail section navigator
- a compact top summary band
- grouped operator modules
- stronger metadata treatment for runtime and control-plane state

### 3. Reshape portal into a developer self-service product

The portal app should emphasize:

- onboarding and quick comprehension
- self-service account lifecycle
- workspace identity and API key lifecycle
- a friendlier, product-led visual language

Portal will use:

- a guided hero with action-focused copy
- simplified self-service modules
- onboarding steps and environment hints
- a warmer, lighter product presentation than admin

### 4. Preserve hard isolation rules

The redesign must not reintroduce coupling:

- separate HTML entry documents stay in place
- separate React roots stay in place
- separate storage keys stay in place
- separate API prefixes stay in place
- no in-app navigation from portal into admin
- no shared application theme file imported by either app

## Verification Strategy

### Structural verification

Add a zero-dependency Node test that proves:

- admin, portal, and landing each use dedicated stylesheet entry points
- admin and portal expose distinct root app classes
- admin and portal SDKs keep distinct storage keys and API prefixes
- HTML entry points remain separate

### Build verification

Run:

- `cd console && node --test tests/independent-apps.test.mjs`
- `pnpm --dir console typecheck`
- `pnpm --dir console build`

### Runtime verification

Check:

- `http://127.0.0.1:5173/admin/`
- `http://127.0.0.1:5173/portal/`

The loaded HTML must still point to different browser entry scripts and the runtime login flows
must remain functional.
