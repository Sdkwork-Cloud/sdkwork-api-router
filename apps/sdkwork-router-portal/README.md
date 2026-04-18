# SDKWork Router Portal

`sdkwork-router-portal` is the standalone developer self-service workspace for SDKWork Router.

## Goals

- independent engineering project under `apps/`
- product-grade self-service portal rather than a legacy `console` sub-page
- follows `ARCHITECT.md` package ownership and dependency direction
- combines live `/portal/*` data, live commerce catalog flows, and desktop runtime evidence

## Workspace Layout

```text
apps/sdkwork-router-portal/
├── src/        # root shell composition and theme only
├── packages/   # foundation and business modules
├── tests/      # structure and architecture checks
└── dist/       # production build output
```

## Business Package Standard

Portal business packages follow the `ARCHITECT.md` shape:

```text
packages/sdkwork-router-portal-<module>/src/
├── types/       # local view contracts and page props
├── components/  # module-owned UI pieces
├── repository/  # live portal API access
├── services/    # derived product logic and recommendations
├── pages/       # route-level page entry
└── index.tsx    # thin re-export surface
```

## Package Map

### Foundation

- `sdkwork-router-portal-types`
- `sdkwork-router-portal-commons`
- `sdkwork-router-portal-core`
- `sdkwork-router-portal-portal-api`

### Business

- `sdkwork-router-portal-gateway`
- `sdkwork-router-portal-auth`
- `sdkwork-router-portal-dashboard`
- `sdkwork-router-portal-routing`
- `sdkwork-router-portal-api-keys`
- `sdkwork-router-portal-usage`
- `sdkwork-router-portal-user`
- `sdkwork-router-portal-credits`
- `sdkwork-router-portal-billing`
- `sdkwork-router-portal-account`

## Product Surface

`Gateway`
handles compatibility posture, local versus server deployment modes, role-sliced topology, and launch readiness.

`User` handles profile and password rotation.
`Account` handles cash balance, billing ledger, and runway posture.
`Redeem` handles coupon activation, invite growth, and reward visibility.

- `Gateway`
  - compatibility matrix for Codex, Claude Code, OpenCode, Gemini CLI, Gemini-compatible clients, and OpenClaw
  - live desktop runtime evidence for web, gateway, admin, and portal binds
  - desktop mode versus server mode switchboard
  - role-sliced topology playbooks
  - Commerce catalog for plans, recharge packs, and coupon inventory
  - launch readiness links into API keys, routing, and billing
- `Dashboard`
  - workspace identity
  - points and quota posture
  - total requests
  - token-unit usage
  - recent request list
  - recommended next actions derived from live posture
- `Routing`
  - active routing strategy and preset
  - provider ordering and healthy-path guardrails
  - preview route simulation
  - routing decision evidence
- `API Keys`
  - environment-scoped key issuance
  - plaintext-once handling
  - environment coverage summaries
  - copy and quickstart handoff
- `Usage`
  - request telemetry
  - model and provider distribution
  - per-call token-unit history
  - input, output, and total token detail
  - filterable workbench for models, providers, and time ranges
- `User`
  - profile and password rotation
  - personal security posture
  - recovery guidance
- `Redeem`
  - coupon redemption workflow
  - redemption-impact preview
  - invite growth tooling
  - activation reward visibility
- `Billing`
  - subscription plan catalog
  - recharge packs
  - upgrade motion
  - recommendation logic based on current usage and remaining quota
- `Account`
  - cash balance
  - billing ledger
  - runway posture

## Data Sources

- live portal API:
  - auth session
  - workspace summary
  - dashboard summary
  - commerce catalog
  - routing summary, preferences, preview, and decision logs
  - usage records and summary
  - billing summary and ledger
  - API keys
  - password rotation

## Commands

```bash
pnpm install
pnpm typecheck
pnpm build
pnpm preview
pnpm dev
pnpm product:start
pnpm product:service
pnpm server:start
pnpm server:plan
pnpm product:check
pnpm tauri:dev
pnpm tauri:dev:service
pnpm tauri:build
```

The Vite dev server proxies `/api/portal/*` to `http://127.0.0.1:8082/portal/*`.

## Product Runtime Modes

`sdkwork-router-portal` participates in two user-facing products:

- `sdkwork-router-portal-desktop`
  - Tauri shell plus a supervised `router-product-service` sidecar
- `sdkwork-api-router-product-server`
  - installed `router-product-service` runtime for server deployment

Both products expose the same public surface:

- `/portal/*`
- `/admin/*`
- `/api/portal/*`
- `/api/admin/*`
- `/api/v1/*`

The in-product `Gateway` route remains the runtime posture screen for compatibility, topology, and launch readiness.

### Desktop Mode

Desktop mode no longer boots the router product in-process. The shell supervises a bundled release-like `router-product/` payload:

- `router-product/bin/router-product-service`
- `router-product/sites/admin/dist/`
- `router-product/sites/portal/dist/`

Desktop runtime contract:

- fixed local shell base URL: `http://127.0.0.1:3001`
- access modes:
  - local-only access: `127.0.0.1:3001`
  - shared network access: `0.0.0.0:3001`
- direct internal binds stay on loopback defaults:
  - gateway: `127.0.0.1:8080`
  - admin: `127.0.0.1:8081`
  - portal: `127.0.0.1:8082`
- the shell exposes runtime controls over Tauri IPC:
  - `runtime_base_url`
  - `runtime_desktop_snapshot`
  - `restart_product_runtime`
  - `update_desktop_runtime_access_mode`

Useful commands:

```bash
pnpm product:start
pnpm product:start -- desktop
pnpm product:service
pnpm tauri:dev
pnpm tauri:dev:service
pnpm tauri:build
node ../../scripts/prepare-router-portal-desktop-runtime.mjs
```

Desktop mutable state lives outside the immutable bundle in OS-standard per-user directories:

- config root:
  - Tauri app config dir + `router-product/`
- data root:
  - Tauri app data dir + `router-product/`
- log root:
  - Tauri app log dir + `router-product/`

Desktop runtime files:

- `desktop-runtime.json`
  - persisted shell access mode
- `router.yaml`
  - canonical sidecar runtime config

The shell launches the sidecar with `--config-dir <config-dir>` and strips inherited `SDKWORK_*` environment variables so config-file values remain authoritative after discovery.

### Server Mode

`pnpm server:start` runs `router-product-service`, which starts:

- the public web host
- the gateway API
- the admin API
- the portal API

By default, server mode exposes:

- `/portal/*` for the portal web app
- `/admin/*` for the super-admin web app
- `/api/portal/*` for portal APIs
- `/api/admin/*` for admin APIs
- `/api/v1/*` for OpenAI-compatible gateway APIs
- `/api/v1/health` and `/api/v1/metrics` through the gateway proxy

Default public bind:

```bash
SDKWORK_WEB_BIND=0.0.0.0:3001
```

Server mode also supports direct CLI flags, which override environment variables:

```bash
pnpm product:start -- server
pnpm server:start -- --help
pnpm product:start -- plan
pnpm product:start -- check
pnpm server:plan
pnpm product:check
pnpm server:start -- --bind 0.0.0.0:3301 --roles web,gateway,admin,portal
pnpm server:start -- --database-url postgres://postgres:postgres@127.0.0.1:5432/sdkwork_api_router
pnpm server:start -- --dry-run --roles web --gateway-upstream 10.0.0.21:8080 --admin-upstream 10.0.0.22:8081 --portal-upstream 10.0.0.23:8082
```

Common CLI flags:

- `--config-dir`
- `--config-file`
- `--database-url`
- `--bind`
- `--roles`
- `--node-id-prefix`
- `--gateway-bind`
- `--admin-bind`
- `--portal-bind`
- `--gateway-upstream`
- `--admin-upstream`
- `--portal-upstream`
- `--admin-site-dir`
- `--portal-site-dir`
- `--dry-run`

`--dry-run` prints the resolved deployment plan and exits without binding ports. It is intended
for cluster rollout review, container entrypoint validation, and CI smoke checks.

`pnpm product:start` defaults to desktop mode and can also launch `server`, `plan`, `check`, and
`browser` modes through one product-level entrypoint:

```bash
pnpm product:start
pnpm product:start -- server --roles web --gateway-upstream 10.0.0.21:8080
pnpm product:start -- plan --roles web
pnpm product:start -- check
```

When a forwarded mode argument would conflict with the launcher itself, use a second `--`:

```bash
pnpm product:start -- server -- --help
```

`pnpm server:plan` emits the dry-run plan as JSON so deployment tooling can inspect the resolved
roles, binds, upstreams, database URL, and bundled web assets.

`pnpm product:check` runs the integrated portal/admin typechecks, rebuilds bundled desktop assets,
checks `router-product-service`, and finishes with a JSON dry-run plan.

### Cluster and Role Slicing

Server mode supports role-based topology so the product can scale beyond a single process.

Supported roles:

- `web`
- `gateway`
- `admin`
- `portal`

Use `SDKWORK_ROUTER_ROLES` to select which roles a node owns:

```bash
SDKWORK_ROUTER_ROLES=web,gateway,admin,portal
```

When `web` is enabled without a local API role, the matching upstream must be configured:

- `SDKWORK_GATEWAY_PROXY_TARGET`
- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`

Example edge-only node:

```bash
pnpm server:start -- \
  --bind 0.0.0.0:3001 \
  --roles web \
  --gateway-upstream 10.0.0.21:8080 \
  --admin-upstream 10.0.0.22:8081 \
  --portal-upstream 10.0.0.23:8082
```

Example edge-only dry-run validation:

```bash
pnpm server:start -- \
  --dry-run \
  --roles web \
  --bind 0.0.0.0:3001 \
  --gateway-upstream 10.0.0.21:8080 \
  --admin-upstream 10.0.0.22:8081 \
  --portal-upstream 10.0.0.23:8082
```

Example split control-plane node:

```bash
pnpm server:start -- \
  --roles admin,portal \
  --node-id-prefix control-a \
  --admin-bind 127.0.0.1:9081 \
  --portal-bind 127.0.0.1:9082
```

Example split data-plane node:

```bash
pnpm server:start -- \
  --roles gateway \
  --node-id-prefix gateway-a \
  --gateway-bind 127.0.0.1:9080
```

### Runtime Environment

The runtime recognizes these environment variables:

- `SDKWORK_CONFIG_DIR`
- `SDKWORK_CONFIG_FILE`
- `SDKWORK_WEB_BIND`
- `SDKWORK_DATABASE_URL`
- `SDKWORK_ROUTER_ROLES`
- `SDKWORK_ROUTER_NODE_ID_PREFIX`
- `SDKWORK_GATEWAY_PROXY_TARGET`
- `SDKWORK_ADMIN_PROXY_TARGET`
- `SDKWORK_PORTAL_PROXY_TARGET`
- `SDKWORK_ADMIN_SITE_DIR`
- `SDKWORK_PORTAL_SITE_DIR`

Config-file values are the source of truth once discovery completes. Environment variables remain discovery inputs and fallback values only for fields left unset in the active config file.
